//! `testkit verify` — replay scenarios, capture evidence, and write signed receipts.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use chrono::Utc;
use reqwest::blocking::Client;
use reqwest::header::{HeaderName, HeaderValue};
use serde::Deserialize;
use serde_json::{json, Value};

use testkit_core::receipt::{
    next_receipt_id, HardwareMetrics, ReceiptBuilder, ReceiptStatus, Severity, Violation,
};
use testkit_core::telemetry::{ResourceEnvelope, RunawayKind, TelemetryWatcher};
use testkit_core::time_travel::{parse_jump_spec, VirtualClock};
use testkit_core::EvidenceStore;

const SAMPLE_INTERVAL: Duration = Duration::from_millis(50);
const HARD_TIMEOUT: Duration = Duration::from_secs(300);
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
const TRACE_BODY_LIMIT: usize = 4_000;
const LOG_BODY_LIMIT: usize = 100_000;

// ---------------------------------------------------------------------------
// Scenario schema
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ScenarioKind {
    Process,
    Http,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Scenario {
    id: String,
    invariant: String,
    kind: ScenarioKind,
    #[serde(default)] severity: Option<String>,
    #[serde(default)] description: String,
    #[serde(default)] time_jump: Option<String>,
    #[serde(default)] process: Option<ProcessSpec>,
    #[serde(default)] http: Option<HttpSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ProcessSpec {
    program: String,
    #[serde(default)] args: Vec<String>,
    #[serde(default)] cwd: Option<PathBuf>,
    #[serde(default)] env: BTreeMap<String, String>,
    #[serde(default)] expected_exit_code: Option<i32>,
    #[serde(default)] stdout_contains: Vec<String>,
    #[serde(default)] stderr_contains: Vec<String>,
    #[serde(default)] max_duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
struct HttpSpec {
    #[serde(default = "default_http_method")] method: String,
    url: String,
    #[serde(default)] headers: BTreeMap<String, String>,
    #[serde(default)] body: Option<String>,
    #[serde(default)] expected_status: Vec<u16>,
    #[serde(default)] expected_body_contains: Vec<String>,
}

fn default_http_method() -> String {
    "GET".to_string()
}

// ---------------------------------------------------------------------------
// Execution types
// ---------------------------------------------------------------------------

enum Outcome {
    Process(ProcessObservation),
    Http(HttpObservation),
}

struct ProcessObservation {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    duration: Duration,
}

struct HttpObservation {
    status: u16,
    body: String,
    duration: Duration,
}

type WorkerMessage = std::result::Result<Outcome, String>;

struct Execution {
    outcome: WorkerMessage,
    telemetry: TelemetryReport,
    duration_ms: u64,
    fake_now: String,
}

impl Execution {
    fn blocked(reason: impl Into<String>, fake_now: &str) -> Self {
        Self {
            outcome: Err(reason.into()),
            telemetry: TelemetryReport::default(),
            duration_ms: 0,
            fake_now: fake_now.to_string(),
        }
    }
}

#[derive(Default)]
struct TelemetryReport {
    peak_cpu_percent: f32,
    peak_ram_mb: u64,
    samples: Vec<Value>,
    runaway_cpu_loop: Option<String>,
}

struct Evaluation {
    passed: bool,
    detail: String,
    violations: Vec<Violation>,
    status: ReceiptStatus,
}

struct Verdict {
    status: ReceiptStatus,
    duration_ms: u64,
}

#[derive(Default)]
struct Tally {
    passed: u32,
    failed: u32,
    blocked: u32,
}

impl Tally {
    fn record(&mut self, status: ReceiptStatus) {
        match status {
            ReceiptStatus::Passed => self.passed += 1,
            ReceiptStatus::Failed => self.failed += 1,
            ReceiptStatus::Blocked | ReceiptStatus::NotProven => self.blocked += 1,
        }
    }
}

struct VerificationReport {
    tally: Tally,
    matched: usize,
}

struct LoadFailure {
    code: &'static str,
    message: String,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Verify scenarios matching the optional id/tag filters, writing evidence and
/// receipts. Exits with code 2 when the project is not initialized and code 1
/// when any receipt is failed or blocked.
///
/// `max_cpu_percent` / `max_rss_mb` override the resource envelope for this
/// run; passing neither uses the default $12-server envelope.
pub fn run(
    path: Option<PathBuf>,
    scenario_filter: Option<String>,
    tag_filter: Option<String>,
    max_cpu_percent: Option<f32>,
    max_rss_mb: Option<u64>,
    fail_fast: bool,
) -> Result<()> {
    let root = resolve_root(path);
    if !root.join(".testkit").is_dir() {
        eprintln!("TestKit not initialized here. Run `testkit init` first.");
        std::process::exit(2);
    }

    let envelope = if max_cpu_percent.is_none() && max_rss_mb.is_none() {
        ResourceEnvelope::twelve_dollar_server()
    } else {
        let defaults = ResourceEnvelope::default();
        ResourceEnvelope::custom(
            max_cpu_percent.unwrap_or(defaults.max_cpu_percent),
            max_rss_mb.unwrap_or(defaults.max_rss_mb),
        )
    };

    let store = EvidenceStore::new(&root);
    store
        .ensure_dirs()
        .context("preparing the TestKit evidence store")?;

    let receipts_dir = root.join(".testkit").join("receipts");
    fs::create_dir_all(&receipts_dir)
        .with_context(|| format!("creating receipt directory {}", receipts_dir.display()))?;

    let scenarios_dir = root.join(".testkit").join("scenarios");
    let files = collect_scenario_files(&scenarios_dir)?;
    if files.is_empty() {
        println!("No scenarios found in {}.", scenarios_dir.display());
        return Ok(());
    }

    let report = verify_scenarios(
        &root,
        &store,
        &receipts_dir,
        &files,
        scenario_filter.as_deref(),
        tag_filter.as_deref(),
        envelope,
        fail_fast,
    )?;

    if report.matched == 0 {
        if let Some(filter) = scenario_filter.as_deref() {
            eprintln!("No scenarios matched id '{filter}'.");
        }
        if let Some(tag) = tag_filter.as_deref() {
            eprintln!("No scenarios matched invariant tag '{tag}'.");
        }
        return Ok(());
    }

    println!(
        "Receipts: {} passed, {} failed, {} blocked",
        report.tally.passed, report.tally.failed, report.tally.blocked
    );

    if report.tally.failed + report.tally.blocked > 0 {
        std::process::exit(1);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Scenario loop
// ---------------------------------------------------------------------------

fn verify_scenarios(
    root: &Path,
    store: &EvidenceStore,
    receipts_dir: &Path,
    files: &[PathBuf],
    filter: Option<&str>,
    tag_filter: Option<&str>,
    envelope: ResourceEnvelope,
    fail_fast: bool,
) -> Result<VerificationReport> {
    let mut tally = Tally::default();
    let mut matched = 0usize;

    for file in files {
        match load_scenario(file) {
            Ok(scenario) => {
                if filter.is_some_and(|id| id != scenario.id.as_str()) {
                    continue;
                }
                if tag_filter.is_some_and(|tag| !scenario.invariant.contains(tag)) {
                    continue;
                }
                matched += 1;
                let verdict = verify_scenario(root, store, receipts_dir, &scenario, envelope)?;
                print_verdict(
                    verdict.status,
                    &scenario.id,
                    &scenario.invariant,
                    verdict.duration_ms,
                );
                tally.record(verdict.status);
                if fail_fast && is_failure(verdict.status) {
                    break;
                }
            }
            Err(failure) => {
                // Broken JSON cannot reveal its id, so it is always reported.
                matched += 1;
                let stem = evidence_stem(file_stem(file));
                write_parse_error_receipt(store, receipts_dir, &stem, &failure)?;
                print_verdict(ReceiptStatus::Blocked, file_stem(file), "scenario", 0);
                tally.record(ReceiptStatus::Blocked);
                if fail_fast {
                    break;
                }
            }
        }
    }

    Ok(VerificationReport { tally, matched })
}

fn verify_scenario(
    root: &Path,
    store: &EvidenceStore,
    receipts_dir: &Path,
    scenario: &Scenario,
    envelope: ResourceEnvelope,
) -> Result<Verdict> {
    let stem = evidence_stem(&scenario.id);
    let fake_now = compute_fake_now(scenario.time_jump.as_deref());
    let execution = execute_scenario(root, scenario, &fake_now, envelope);

    record_evidence(store, scenario, &stem, &execution)?;
    let evaluation = evaluate_scenario(scenario, &execution, &stem, envelope);
    let evidence_hash = store
        .hash_tree()
        .context("hashing the evidence bundle")?;
    write_receipt(
        receipts_dir,
        scenario,
        &evaluation,
        execution.duration_ms,
        &execution.telemetry,
        &evidence_hash,
    )?;

    Ok(Verdict {
        status: evaluation.status,
        duration_ms: execution.duration_ms,
    })
}

fn execute_scenario(
    root: &Path,
    scenario: &Scenario,
    fake_now: &str,
    envelope: ResourceEnvelope,
) -> Execution {
    match scenario.kind {
        ScenarioKind::Process => match &scenario.process {
            Some(spec) => execute_process(root, spec, fake_now, envelope),
            None => Execution::blocked("scenario has kind 'process' but no process block", fake_now),
        },
        ScenarioKind::Http => match &scenario.http {
            Some(spec) => execute_http(spec, fake_now, envelope),
            None => Execution::blocked("scenario has kind 'http' but no http block", fake_now),
        },
    }
}

fn compute_fake_now(jump: Option<&str>) -> String {
    let mut clock = VirtualClock::new(Utc::now());
    if let Some(spec) = jump {
        if let Some(parsed) = parse_jump_spec(spec) {
            clock.jump(parsed);
        }
        // Unrecognised specs intentionally leave the anchor untouched; the
        // raw spec still lands in the trace for auditing.
    }
    clock.now().to_rfc3339()
}

// ---------------------------------------------------------------------------
// Process execution
// ---------------------------------------------------------------------------

fn execute_process(root: &Path, spec: &ProcessSpec, fake_now: &str, envelope: ResourceEnvelope) -> Execution {
    let started = Instant::now();
    let child = match spawn_process(spec, root, fake_now) {
        Ok(child) => child,
        Err(error) => {
            return Execution {
                outcome: Err(format!("failed to start '{}': {error}", spec.program)),
                telemetry: TelemetryReport::default(),
                duration_ms: millis(started.elapsed()),
                fake_now: fake_now.to_string(),
            };
        }
    };

    let pid = child.id();
    let (tx, rx) = mpsc::channel::<WorkerMessage>();
    thread::spawn(move || {
        let message = match child.wait_with_output() {
            Ok(output) => Ok(Outcome::Process(ProcessObservation {
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                duration: started.elapsed(),
            })),
            Err(error) => Err(format!("failed to wait for the process: {error}")),
        };
        let _ = tx.send(message);
    });

    let watcher = TelemetryWatcher::new(pid, envelope);
    let (outcome, telemetry) = await_worker(rx, watcher);

    Execution {
        outcome,
        telemetry,
        duration_ms: millis(started.elapsed()),
        fake_now: fake_now.to_string(),
    }
}

fn spawn_process(spec: &ProcessSpec, root: &Path, fake_now: &str) -> io::Result<Child> {
    let cwd = match &spec.cwd {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => root.join(path),
        None => root.to_path_buf(),
    };

    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .current_dir(cwd)
        .env("TESTKIT_FAKE_NOW", fake_now)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in &spec.env {
        command.env(key, value);
    }
    command.spawn()
}

// ---------------------------------------------------------------------------
// HTTP execution
// ---------------------------------------------------------------------------

fn execute_http(spec: &HttpSpec, fake_now: &str, envelope: ResourceEnvelope) -> Execution {
    let started = Instant::now();
    let (tx, rx) = mpsc::channel::<WorkerMessage>();
    let spec = spec.clone();
    let worker_fake_now = fake_now.to_string();

    thread::spawn(move || {
        let message = perform_http(&spec, &worker_fake_now);
        let _ = tx.send(message);
    });

    // An HTTP probe has no child process, so the CLI samples itself. That is a
    // host-level proxy for memory health, not the target server's footprint.
    let watcher = TelemetryWatcher::new(std::process::id(), envelope);
    let (outcome, telemetry) = await_worker(rx, watcher);

    Execution {
        outcome,
        telemetry,
        duration_ms: millis(started.elapsed()),
        fake_now: fake_now.to_string(),
    }
}

fn perform_http(spec: &HttpSpec, fake_now: &str) -> WorkerMessage {
    let started = Instant::now();
    let method = reqwest::Method::from_bytes(spec.method.as_bytes())
        .map_err(|error| format!("invalid HTTP method '{}': {error}", spec.method))?;
    let client = Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .map_err(|error| format!("failed to build the HTTP client: {error}"))?;

    let mut request = client
        .request(method, &spec.url)
        .header("X-Testkit-Fake-Now", fake_now);

    for (name, value) in &spec.headers {
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|error| format!("invalid header name '{name}': {error}"))?;
        let header_value = HeaderValue::from_str(value).map_err(|error| {
            format!(
                "invalid value for header '{}': {error}",
                header_name.as_str()
            )
        })?;
        request = request.header(header_name, header_value);
    }

    if let Some(body) = &spec.body {
        request = request.body(body.clone());
    }

    let response = request
        .send()
        .map_err(|error| format!("HTTP request failed: {error}"))?;
    let status = response.status().as_u16();
    let body = response
        .text()
        .map_err(|error| format!("failed to read the HTTP response body: {error}"))?;

    Ok(Outcome::Http(HttpObservation {
        status,
        body,
        duration: started.elapsed(),
    }))
}

// ---------------------------------------------------------------------------
// Telemetry
// ---------------------------------------------------------------------------

fn await_worker(
    receiver: Receiver<WorkerMessage>,
    mut watcher: TelemetryWatcher,
) -> (WorkerMessage, TelemetryReport) {
    let deadline = Instant::now() + HARD_TIMEOUT;

    let message = loop {
        // Sample for the whole run: a leak that creeps in at minute two must
        // still reach the receipt. The watcher bounds what it retains by
        // decimating, so memory stays flat no matter how long the run takes.
        let _ = watcher.sample_now();

        match receiver.recv_timeout(SAMPLE_INTERVAL) {
            Ok(message) => break message,
            Err(RecvTimeoutError::Timeout) => {
                if Instant::now() >= deadline {
                    break Err(format!(
                        "scenario did not finish within {}s; the worker may still be running",
                        HARD_TIMEOUT.as_secs()
                    ));
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                break Err("scenario worker stopped without reporting a result".to_string());
            }
        }
    };

    (message, telemetry_report(&watcher))
}

fn telemetry_report(watcher: &TelemetryWatcher) -> TelemetryReport {
    let (peak_cpu_percent, peak_ram_mb) = watcher.peak();
    let samples = watcher
        .samples()
        .iter()
        .map(|sample| {
            json!({
                "timestamp": sample.timestamp.to_rfc3339(),
                "cpu_percent": sample.cpu_percent,
                "rss_mb": sample.rss_mb,
            })
        })
        .collect();
    let runaway_cpu_loop = match watcher.detect_runaway() {
        Some(finding) if finding.kind == RunawayKind::CpuLoop => Some(finding.detail),
        _ => None,
    };
    TelemetryReport {
        peak_cpu_percent,
        peak_ram_mb,
        samples,
        runaway_cpu_loop,
    }
}

// ---------------------------------------------------------------------------
// Evidence
// ---------------------------------------------------------------------------

fn record_evidence(
    store: &EvidenceStore,
    scenario: &Scenario,
    stem: &str,
    execution: &Execution,
) -> Result<()> {
    let fake_now = execution.fake_now.as_str();

    match &execution.outcome {
        Ok(Outcome::Process(observation)) => {
            store.write_text("logs", &format!("{stem}.stdout.log"), &observation.stdout)
                .context("writing stdout evidence")?;
            store.write_text("logs", &format!("{stem}.stderr.log"), &observation.stderr)
                .context("writing stderr evidence")?;
            let spec = scenario.process.as_ref();
            let trace = json!({
                "scenario": &scenario.id,
                "description": &scenario.description,
                "kind": "process",
                "program": spec.map(|spec| spec.program.as_str()).unwrap_or(""),
                "args": spec.map(|spec| spec.args.clone()).unwrap_or_default(),
                "exit_code": observation.exit_code,
                "duration_ms": millis(observation.duration),
                "fake_now": fake_now,
                "time_jump": &scenario.time_jump,
            });
            store.write_json("traces", &format!("{stem}.json"), &trace)
                .context("writing the process trace")?;
        }
        Ok(Outcome::Http(observation)) => {
            let spec = scenario.http.as_ref();
            let trace = json!({
                "scenario": &scenario.id,
                "description": &scenario.description,
                "kind": "http",
                "method": spec.map(|spec| spec.method.as_str()).unwrap_or("GET"),
                "url": spec.map(|spec| spec.url.as_str()).unwrap_or(""),
                "status": observation.status,
                "duration_ms": millis(observation.duration),
                "fake_now": fake_now,
                "time_jump": &scenario.time_jump,
                "response_body": truncate(&observation.body, TRACE_BODY_LIMIT),
            });
            store.write_json("traces", &format!("{stem}.json"), &trace)
                .context("writing the HTTP trace")?;
            store.write_text(
                "logs",
                &format!("{stem}.response.log"),
                &truncate(&observation.body, LOG_BODY_LIMIT),
            )
            .context("writing the response log")?;
        }
        Err(reason) => {
            let log_name = match scenario.kind {
                ScenarioKind::Process => format!("{stem}.stderr.log"),
                ScenarioKind::Http => format!("{stem}.response.log"),
            };
            store.write_text("logs", &log_name, &format!("execution error: {reason}\n"))
                .context("writing the failure log")?;
            let process = scenario.process.as_ref();
            let http = scenario.http.as_ref();
            let trace = json!({
                "scenario": &scenario.id,
                "description": &scenario.description,
                "kind": match scenario.kind {
                    ScenarioKind::Process => "process",
                    ScenarioKind::Http => "http",
                },
                "program": process.map(|spec| spec.program.as_str()),
                "args": process.map(|spec| spec.args.clone()),
                "method": http.map(|spec| spec.method.as_str()),
                "url": http.map(|spec| spec.url.as_str()),
                "error": reason,
                "fake_now": fake_now,
                "time_jump": &scenario.time_jump,
            });
            store.write_json("traces", &format!("{stem}.json"), &trace)
                .context("writing the failure trace")?;
        }
    }

    store.write_json(
        "telemetry",
        &format!("{stem}.json"),
        &execution.telemetry.samples,
    )
    .context("writing telemetry evidence")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

fn evaluate_scenario(
    scenario: &Scenario,
    execution: &Execution,
    stem: &str,
    envelope: ResourceEnvelope,
) -> Evaluation {
    let mut violations = hardware_violations(&execution.telemetry, stem, envelope);

    match &execution.outcome {
        Ok(Outcome::Process(observation)) => match &scenario.process {
            Some(spec) => {
                let (passed, detail) = evaluate_process(spec, observation);
                finish_evaluation(passed, detail, violations)
            }
            None => blocked_evaluation(
                "process output arrived without a process block".to_string(),
                violations,
            ),
        },
        Ok(Outcome::Http(observation)) => match &scenario.http {
            Some(spec) => {
                let (passed, detail) = evaluate_http(spec, observation);
                finish_evaluation(passed, detail, violations)
            }
            None => blocked_evaluation(
                "http output arrived without an http block".to_string(),
                violations,
            ),
        },
        Err(reason) => {
            violations.push(Violation {
                invariant: "EXECUTION_ERROR".to_string(),
                severity: Severity::Medium,
                message: reason.clone(),
                evidence_ref: Some(format!("traces/{stem}.json")),
            });
            blocked_evaluation(format!("could not execute: {reason}"), violations)
        }
    }
}

fn evaluate_process(spec: &ProcessSpec, observation: &ProcessObservation) -> (bool, String) {
    let expected_exit = spec.expected_exit_code.unwrap_or(0);
    let observed_exit = observation
        .exit_code
        .map(|code| code.to_string())
        .unwrap_or_else(|| "signal/unknown".to_string());
    let elapsed_ms = millis(observation.duration);
    let mut problems = Vec::new();

    if observation.exit_code != Some(expected_exit) {
        problems.push(format!(
            "exit expected {expected_exit}, observed {observed_exit}"
        ));
    }
    for needle in &spec.stdout_contains {
        if !observation.stdout.contains(needle) {
            problems.push(format!("stdout missing {}", short_quote(needle)));
        }
    }
    for needle in &spec.stderr_contains {
        if !observation.stderr.contains(needle) {
            problems.push(format!("stderr missing {}", short_quote(needle)));
        }
    }
    if let Some(max) = spec.max_duration_ms {
        if elapsed_ms > max {
            problems.push(format!(
                "duration expected <= {max}ms, observed {elapsed_ms}ms"
            ));
        }
    }

    if problems.is_empty() {
        (
            true,
            format!("exit {observed_exit}, {elapsed_ms}ms, stdout/stderr markers matched"),
        )
    } else {
        (false, problems.join("; "))
    }
}

fn evaluate_http(spec: &HttpSpec, observation: &HttpObservation) -> (bool, String) {
    let expected = expected_status_label(&spec.expected_status);
    let status_ok = if spec.expected_status.is_empty() {
        (200..=299).contains(&observation.status)
    } else {
        spec.expected_status.contains(&observation.status)
    };
    let mut problems = Vec::new();

    if !status_ok {
        problems.push(format!(
            "status expected {expected}, observed {}",
            observation.status
        ));
    }
    for needle in &spec.expected_body_contains {
        if !observation.body.contains(needle) {
            problems.push(format!("body missing {}", short_quote(needle)));
        }
    }

    if problems.is_empty() {
        (
            true,
            format!(
                "status {}, body markers matched in {}ms",
                observation.status,
                millis(observation.duration)
            ),
        )
    } else {
        (false, problems.join("; "))
    }
}

fn hardware_violations(
    telemetry: &TelemetryReport,
    stem: &str,
    envelope: ResourceEnvelope,
) -> Vec<Violation> {
    let mut violations = Vec::new();
    let evidence_ref = Some(format!("telemetry/{stem}.json"));

    if telemetry.peak_ram_mb > envelope.max_rss_mb {
        violations.push(Violation {
            invariant: "RAM_CEILING".to_string(),
            severity: Severity::High,
            message: format!(
                "peak resident memory {} MB exceeded the {} MB envelope",
                telemetry.peak_ram_mb, envelope.max_rss_mb
            ),
            evidence_ref: evidence_ref.clone(),
        });
    }
    if let Some(detail) = &telemetry.runaway_cpu_loop {
        violations.push(Violation {
            invariant: "CPU_RUNAWAY".to_string(),
            severity: Severity::High,
            message: detail.clone(),
            evidence_ref,
        });
    }

    violations
}

fn finish_evaluation(passed: bool, detail: String, violations: Vec<Violation>) -> Evaluation {
    let status = if passed && violations.is_empty() {
        ReceiptStatus::Passed
    } else {
        ReceiptStatus::Failed
    };
    Evaluation {
        passed,
        detail,
        violations,
        status,
    }
}

fn blocked_evaluation(detail: String, violations: Vec<Violation>) -> Evaluation {
    Evaluation {
        passed: false,
        detail,
        violations,
        status: ReceiptStatus::Blocked,
    }
}

// ---------------------------------------------------------------------------
// Receipts
// ---------------------------------------------------------------------------

fn write_receipt(
    receipts_dir: &Path,
    scenario: &Scenario,
    evaluation: &Evaluation,
    duration_ms: u64,
    telemetry: &TelemetryReport,
    evidence_hash: &str,
) -> Result<()> {
    let receipt_id = next_receipt_id(receipts_dir).context("allocating the next receipt id")?;
    let hardware_metrics = HardwareMetrics {
        peak_cpu_percent: telemetry.peak_cpu_percent,
        peak_ram_mb: telemetry.peak_ram_mb,
    };
    let mut builder = ReceiptBuilder::new(severity_for(scenario))
        .id(receipt_id)
        .status(evaluation.status)
        .duration_ms(duration_ms)
        .hardware_metrics(hardware_metrics)
        .evidence_hash(evidence_hash)
        .add_invariant_check(&scenario.invariant, evaluation.passed, &evaluation.detail);
    for violation in &evaluation.violations {
        builder = builder.add_violation(violation.clone());
    }
    let receipt = builder.build().context("building the signed receipt")?;
    receipt
        .write_to_dir(receipts_dir)
        .context("writing the signed receipt")?;
    Ok(())
}

fn write_parse_error_receipt(
    store: &EvidenceStore,
    receipts_dir: &Path,
    stem: &str,
    failure: &LoadFailure,
) -> Result<()> {
    let log_name = format!("{stem}.parse-error.log");
    store
        .write_text("logs", &log_name, &failure.message)
        .context("writing the parse error log")?;
    let evidence_hash = store.hash_tree().context("hashing the evidence bundle")?;
    let receipt_id = next_receipt_id(receipts_dir).context("allocating the next receipt id")?;

    let receipt = ReceiptBuilder::new(Severity::Medium)
        .id(receipt_id)
        .status(ReceiptStatus::Blocked)
        .duration_ms(0)
        .hardware_metrics(HardwareMetrics::default())
        .evidence_hash(evidence_hash)
        .add_invariant_check("scenario", false, &failure.message)
        .add_violation(Violation {
            invariant: failure.code.to_string(),
            severity: Severity::Medium,
            message: failure.message.clone(),
            evidence_ref: Some(format!("logs/{log_name}")),
        })
        .build()
        .context("building the parse error receipt")?;
    receipt
        .write_to_dir(receipts_dir)
        .context("writing the parse error receipt")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Loading and helpers
// ---------------------------------------------------------------------------

fn load_scenario(path: &Path) -> std::result::Result<Scenario, LoadFailure> {
    let display = path.display();
    let contents = fs::read_to_string(path).map_err(|error| LoadFailure {
        code: "SCENARIO_LOAD_ERROR",
        message: format!("failed to read scenario file '{display}': {error}"),
    })?;
    serde_json::from_str(&contents).map_err(|error| LoadFailure {
        code: "SCENARIO_PARSE_ERROR",
        message: format!("failed to parse scenario file '{display}': {error}"),
    })
}

fn collect_scenario_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(anyhow::Error::new(error))
                .with_context(|| format!("reading scenario directory {}", dir.display()))
        }
    };

    let mut files = Vec::new();
    for entry in entries {
        let entry =
            entry.with_context(|| format!("reading an entry in {}", dir.display()))?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn resolve_root(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|error| {
            eprintln!("warning: could not resolve the current directory ({error}); using '.'");
            PathBuf::from(".")
        })
    })
}

fn file_stem(path: &Path) -> &str {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("scenario")
}

/// Sanitize a scenario id so it is safe as an evidence file name.
fn evidence_stem(id: &str) -> String {
    let stem: String = id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect();
    if stem.is_empty() || stem == "." || stem == ".." {
        "scenario".to_string()
    } else {
        stem
    }
}

fn severity_for(scenario: &Scenario) -> Severity {
    match scenario.severity.as_deref().map(|raw| raw.trim().to_ascii_uppercase()) {
        Some(raw) if raw == "P0" => Severity::P0,
        Some(raw) if raw == "HIGH" => Severity::High,
        Some(raw) if raw == "LOW" => Severity::Low,
        _ => Severity::Medium,
    }
}

fn expected_status_label(expected: &[u16]) -> String {
    if expected.is_empty() {
        "2xx".to_string()
    } else {
        expected
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join("/")
    }
}

fn short_quote(text: &str) -> String {
    let limit = 60;
    let mut quoted: String = text.chars().take(limit).collect();
    if text.chars().count() > limit {
        quoted.push_str("...");
    }
    format!("\"{quoted}\"")
}

fn truncate(text: &str, limit: usize) -> String {
    let mut shortened: String = text.chars().take(limit).collect();
    if text.chars().count() > limit {
        shortened.push_str("\n[truncated]");
    }
    shortened
}

fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn status_label(status: ReceiptStatus) -> &'static str {
    match status {
        ReceiptStatus::Passed => "PASSED",
        ReceiptStatus::Failed => "FAILED",
        ReceiptStatus::Blocked => "BLOCKED",
        ReceiptStatus::NotProven => "NOT_PROVEN",
    }
}

fn is_failure(status: ReceiptStatus) -> bool {
    matches!(status, ReceiptStatus::Failed | ReceiptStatus::Blocked)
}

fn print_verdict(status: ReceiptStatus, id: &str, invariant: &str, duration_ms: u64) {
    println!(
        "{}  {}  {}  {}ms",
        status_label(status),
        id,
        invariant,
        duration_ms
    );
}
