//! `testkit gate` — evaluate the receipt ledger and decide pass/block.

use std::path::{Path, PathBuf};
use std::process::exit;

use serde_json::{json, Value};

use testkit_core::receipt::{Receipt, ReceiptStatus, Severity};

/// One blocking entry in the gate result.
enum Blocker<'a> {
    /// A parsed receipt that is blocking or carries an invalid signature.
    Receipt { receipt: &'a Receipt, tampered: bool },
    /// A file that could not be parsed at all.
    Malformed { file: String },
}

/// Run the gate check against a project root.
///
/// Prints a report in JSON or human form. Returns `Ok(())` on graceful paths
/// and terminates with `exit(1)` when at least one blocking receipt is found.
pub fn run(
    path: Option<PathBuf>,
    json: bool,
    quiet: bool,
    warn_only: bool,
) -> anyhow::Result<()> {
    let root = resolve_root(path);
    let testkit_dir = root.join(".testkit");

    if !testkit_dir.is_dir() {
        eprintln!("error: TestKit not initialized here. Run `testkit init` first.");
        exit(2);
    }

    let receipts_dir = testkit_dir.join("receipts");
    let (receipts, malformed) = load_receipts(&receipts_dir);
    let total = receipts.len() + malformed.len();

    let blockers = collect_blockers(&receipts, &malformed);
    let blocked = !blockers.is_empty();

    if json {
        emit_json(&blockers, total, blocked);
    } else if blocked {
        emit_human(&blockers);
    } else if !quiet {
        if total == 0 {
            println!("GATE PASSED — no receipts found (nothing verified yet).");
        } else {
            println!("GATE PASSED — {} receipt(s) verified.", total);
        }
    }

    if blocked {
        if warn_only {
            if !json && !quiet {
                println!(
                    "WARN (warn-only mode): gate would fail — {total} receipt(s) require attention."
                );
            }
            return Ok(());
        }
        exit(1);
    }

    Ok(())
}

/// Resolve the project root, preferring an explicit path over the cwd.
fn resolve_root(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// Build the blocker list from parsed receipts plus malformed filenames.
///
/// A receipt blocks when it is inherently blocking or when its signature does
/// not verify; signature failures are always reported, regardless of status.
fn collect_blockers<'a>(receipts: &'a [Receipt], malformed: &[String]) -> Vec<Blocker<'a>> {
    let mut blockers: Vec<Blocker<'a>> = Vec::new();

    for receipt in receipts {
        let tampered = !receipt.verify();
        if tampered || receipt.is_blocking() {
            blockers.push(Blocker::Receipt { receipt, tampered });
        }
    }

    for file in malformed {
        blockers.push(Blocker::Malformed { file: file.clone() });
    }

    blockers
}

/// Report the effective status/severity pair for a blocking receipt.
fn blocker_labels(receipt: &Receipt, tampered: bool) -> (String, String) {
    if tampered {
        return ("BLOCKED".to_string(), "P0".to_string());
    }
    (status_label(&receipt.status), severity_label(&receipt.severity))
}

/// Render the gate outcome as pretty JSON.
fn emit_json(blockers: &[Blocker<'_>], total: usize, blocked: bool) {
    let mut blocking: Vec<Value> = Vec::with_capacity(blockers.len());

    for blocker in blockers {
        match blocker {
            Blocker::Receipt { receipt, tampered } => {
                let mut violations: Vec<Value> = receipt
                    .violations
                    .iter()
                    .map(|violation| {
                        json!({
                            "invariant": violation.invariant,
                            "message": violation.message,
                            "evidence_ref": violation.evidence_ref,
                        })
                    })
                    .collect();

                if *tampered {
                    violations.push(json!({
                        "invariant": "signature",
                        "message": "signature invalid — receipt modified",
                        "evidence_ref": Value::Null,
                    }));
                }

                let (status, severity) = blocker_labels(receipt, *tampered);
                blocking.push(json!({
                    "id": receipt.id,
                    "status": status,
                    "severity": severity,
                    "violations": violations,
                }));
            }
            Blocker::Malformed { file } => {
                blocking.push(json!({
                    "id": file,
                    "status": "BLOCKED",
                    "severity": "MEDIUM",
                    "violations": [{
                        "invariant": "receipt",
                        "message": "malformed receipt file",
                        "evidence_ref": Value::Null,
                    }],
                }));
            }
        }
    }

    let payload = json!({
        "blocked": blocked,
        "total": total,
        "blocking": blocking,
    });

    match serde_json::to_string_pretty(&payload) {
        Ok(text) => println!("{}", text),
        Err(error) => eprintln!("error: failed to render gate report: {}", error),
    }
}

/// Render the gate outcome for terminals.
fn emit_human(blockers: &[Blocker<'_>]) {
    for blocker in blockers {
        match blocker {
            Blocker::Receipt { receipt, tampered } => {
                let (status, severity) = blocker_labels(receipt, *tampered);
                println!("BLOCKED {} [{}/{}]", receipt.id, status, severity);

                for violation in &receipt.violations {
                    println!("  - {}: {}", violation.invariant, violation.message);
                }

                if *tampered {
                    println!("  - signature: signature invalid — receipt modified");
                }

                println!(
                    "  replay: testkit verify --scenario {}",
                    scenario_hint(receipt)
                );
            }
            Blocker::Malformed { file } => {
                println!("BLOCKED {} [BLOCKED/MEDIUM]", file);
                println!("  - receipt: malformed receipt file");
                println!("  replay: testkit verify --scenario {}", file);
            }
        }
    }

    println!(
        "Gate failed: {} blocking receipt(s). Run: testkit verify",
        blockers.len()
    );
}

/// Pick a scenario identifier for the replay suggestion.
fn scenario_hint(receipt: &Receipt) -> String {
    for check in &receipt.invariants_checked {
        if !check.passed {
            return check.name.clone();
        }
    }

    receipt
        .violations
        .first()
        .map(|violation| violation.invariant.clone())
        .unwrap_or_else(|| receipt.id.clone())
}

/// Load and parse every `run_*.json` receipt in `dir`.
///
/// Returns the parsed receipts and the filenames that failed to parse.
fn load_receipts(dir: &Path) -> (Vec<Receipt>, Vec<String>) {
    let mut receipts: Vec<Receipt> = Vec::new();
    let mut malformed: Vec<String> = Vec::new();

    if !dir.is_dir() {
        return (receipts, malformed);
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return (receipts, malformed),
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !file_name.starts_with("run_") || !file_name.ends_with(".json") {
            continue;
        }

        let contents = match std::fs::read_to_string(entry.path()) {
            Ok(contents) => contents,
            Err(_) => {
                malformed.push(file_name);
                continue;
            }
        };

        match serde_json::from_str::<Receipt>(&contents) {
            Ok(receipt) => receipts.push(receipt),
            Err(_) => malformed.push(file_name),
        }
    }

    receipts.sort_by(|a, b| a.id.cmp(&b.id));
    malformed.sort();
    (receipts, malformed)
}

/// Render a receipt status as an upper-case label.
fn status_label(status: &ReceiptStatus) -> String {
    format!("{:?}", status).to_uppercase()
}

/// Render a severity as an upper-case label.
fn severity_label(severity: &Severity) -> String {
    format!("{:?}", severity).to_uppercase()
}
