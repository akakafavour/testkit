//! Signed, tamper-evident execution receipts.
//!
//! A [`Receipt`] is the durable record of a test run: its outcome, the
//! hardware envelope it ran in, the invariants that were checked, and any
//! violations observed. The signature is a SHA-256 over a canonical JSON
//! payload (the receipt serialized with an empty `signature`), so any change
//! to the recorded data invalidates verification.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Overall outcome of a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiptStatus {
    /// Every required invariant held.
    #[serde(rename = "PASSED")]
    Passed,
    /// At least one required invariant failed.
    #[serde(rename = "FAILED")]
    Failed,
    /// The run completed without proving the required behavior.
    #[serde(rename = "NOT_PROVEN")]
    NotProven,
    /// The run could not execute because its environment was unavailable.
    #[serde(rename = "BLOCKED")]
    Blocked,
}

/// How much a violation matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Release blocker with no workaround.
    #[serde(rename = "P0")]
    P0,
    /// Serious regression that must block a merge.
    #[serde(rename = "HIGH")]
    High,
    /// Noticeable problem that should be fixed soon.
    #[serde(rename = "MEDIUM")]
    Medium,
    /// Cosmetic or informational problem.
    #[serde(rename = "LOW")]
    Low,
}

impl Severity {
    /// Returns `true` when this severity must block a release or merge.
    pub fn is_blocking(&self) -> bool {
        matches!(self, Severity::P0 | Severity::High)
    }
}

/// Resource envelope observed during a run.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct HardwareMetrics {
    /// Highest CPU utilization observed, in percent.
    pub peak_cpu_percent: f32,
    /// Highest resident memory observed, in mebibytes.
    pub peak_ram_mb: u64,
}

/// Result of checking one invariant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantCheck {
    /// Human-readable invariant name.
    pub name: String,
    /// Whether the invariant held.
    pub passed: bool,
    /// Supporting detail, such as the observed value.
    pub detail: String,
}

/// A single rule that was broken during a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Violation {
    /// Name of the invariant that was violated.
    pub invariant: String,
    /// How serious the violation is.
    pub severity: Severity,
    /// Description of what went wrong.
    pub message: String,
    /// Optional pointer to supporting evidence.
    pub evidence_ref: Option<String>,
}

/// A signed record of a test run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    /// Stable identifier, normally `receipt-YYYY-MM-DD-NNNN`.
    pub id: String,
    /// Overall outcome of the run.
    pub status: ReceiptStatus,
    /// Highest severity among the run's violations.
    pub severity: Severity,
    /// When the run finished.
    pub timestamp: DateTime<Utc>,
    /// Wall-clock duration of the run in milliseconds.
    pub duration_ms: u64,
    /// Resource envelope observed during the run.
    pub hardware_metrics: HardwareMetrics,
    /// Invariants that were evaluated.
    pub invariants_checked: Vec<InvariantCheck>,
    /// Violations that were found.
    pub violations: Vec<Violation>,
    /// Hash of the evidence bundle backing this receipt.
    pub evidence_hash: String,
    /// Hex-encoded SHA-256 of [`Receipt::canonical_payload`].
    pub signature: String,
}

impl Receipt {
    /// Serializes the receipt with an empty signature field.
    ///
    /// The payload is deterministic: struct fields serialize in declaration
    /// order and no unordered collections are involved.
    pub fn canonical_payload(&self) -> Vec<u8> {
        let mut unsigned = self.clone();
        unsigned.signature.clear();
        serde_json::to_vec(&unsigned).unwrap_or_default()
    }

    fn computed_signature(&self) -> String {
        hex::encode(Sha256::digest(self.canonical_payload()))
    }

    /// Signs the receipt in place, replacing any previous signature.
    pub fn sign(&mut self) {
        self.signature.clear();
        self.signature = self.computed_signature();
    }

    /// Returns `true` when the stored signature matches the current payload.
    pub fn verify(&self) -> bool {
        let expected = self.computed_signature();
        constant_time_eq(expected.as_bytes(), self.signature.as_bytes())
    }

    /// Returns `true` when the status and severity together block a release.
    pub fn is_blocking(&self) -> bool {
        matches!(self.status, ReceiptStatus::Failed | ReceiptStatus::Blocked)
            && self.severity.is_blocking()
    }

    /// Writes pretty-printed JSON to `run_{id}.json` inside `dir`.
    ///
    /// Creates `dir` when missing. The id is validated first so a receipt can
    /// never escape the target directory.
    pub fn write_to_dir(&self, dir: &Path) -> anyhow::Result<PathBuf> {
        validate_id(&self.id)?;
        std::fs::create_dir_all(dir)
            .with_context(|| format!("failed to create receipt directory '{}'", dir.display()))?;
        let path = dir.join(format!("run_{}.json", self.id));
        let json = serde_json::to_string_pretty(self).context("failed to serialize receipt")?;
        std::fs::write(&path, json)
            .with_context(|| format!("failed to write receipt '{}'", path.display()))?;
        Ok(path)
    }
}

/// Compares two byte slices without exiting early on the first mismatch.
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        difference |= a ^ b;
    }
    difference == 0
}

/// Allocates the next `receipt-YYYY-MM-DD-NNNN` id for today (UTC).
///
/// Existing `run_*.json` files in `dir` are scanned for ids that match today's
/// date; the largest sequence number wins and the next one is returned. A
/// missing directory or an unparseable file is treated as if it held no
/// receipts. Returns `0001` when nothing matches.
pub fn next_receipt_id(dir: &Path) -> anyhow::Result<String> {
    let prefix = format!("receipt-{}-", Utc::now().format("%Y-%m-%d"));
    let mut highest = 0u32;
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(format!("{prefix}0001"));
        }
        Err(error) => {
            return Err(anyhow::Error::new(error)
                .context(format!("failed to read receipt directory '{}'", dir.display())));
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let file_name = entry.file_name();
        let file_name = match file_name.to_str() {
            Some(file_name) => file_name,
            None => continue,
        };
        if let Some(sequence) = parse_sequence(file_name, &prefix) {
            highest = highest.max(sequence);
        }
    }
    let next = highest + 1;
    if next > 9_999 {
        bail!("receipt id sequence exhausted for {}", prefix.trim_end_matches('-'));
    }
    Ok(format!("{prefix}{next:04}"))
}

/// Extracts `NNNN` from a `run_receipt-YYYY-MM-DD-NNNN.json` file name.
fn parse_sequence(file_name: &str, prefix: &str) -> Option<u32> {
    let stem = file_name.strip_prefix("run_")?.strip_suffix(".json")?;
    let digits = stem.strip_prefix(prefix)?;
    if digits.len() != 4 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse::<u32>().ok()
}

/// Rejects ids that are empty or unsafe to embed in a file name.
fn validate_id(id: &str) -> anyhow::Result<()> {
    if id.is_empty() {
        bail!("receipt id must not be empty");
    }
    if id.len() > 128 {
        bail!("receipt id must be at most 128 bytes");
    }
    if id.contains("..") {
        bail!("receipt id must not contain '..'");
    }
    for character in id.chars() {
        let forbidden = matches!(
            character,
            '/' | '\\' | '<' | '>' | ':' | '"' | '|' | '?' | '*'
        ) || character.is_control();
        if forbidden {
            bail!("receipt id contains a character that is unsafe in a file name: {character:?}");
        }
    }
    Ok(())
}

/// Creates a `receipt-YYYY-MM-DD-NNNN` id from the current time.
///
/// The numeric suffix is the current second-of-day modulo 10,000, zero-padded
/// to four digits. It is deterministic for a given timestamp and fits the four
/// digit contract, but it is not a directory-wide sequence. Callers that
/// persist receipts should allocate an id with [`next_receipt_id`] and pass it
/// to [`ReceiptBuilder::id`] to avoid collisions.
fn default_receipt_id(now: DateTime<Utc>) -> String {
    let sequence = now.time().num_seconds_from_midnight() % 10_000;
    format!("receipt-{}-{sequence:04}", now.format("%Y-%m-%d"))
}

/// Fluent constructor for signed [`Receipt`] values.
#[derive(Debug, Clone)]
pub struct ReceiptBuilder {
    id: Option<String>,
    status: ReceiptStatus,
    severity: Severity,
    duration_ms: u64,
    hardware_metrics: HardwareMetrics,
    invariants_checked: Vec<InvariantCheck>,
    violations: Vec<Violation>,
    evidence_hash: String,
}

impl ReceiptBuilder {
    /// Starts a builder for a run whose highest severity is `severity`.
    ///
    /// Status defaults to [`ReceiptStatus::NotProven`]; the id is generated at
    /// build time unless one is set explicitly.
    pub fn new(severity: Severity) -> Self {
        Self {
            id: None,
            status: ReceiptStatus::NotProven,
            severity,
            duration_ms: 0,
            hardware_metrics: HardwareMetrics::default(),
            invariants_checked: Vec::new(),
            violations: Vec::new(),
            evidence_hash: String::new(),
        }
    }

    /// Overrides the generated id.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the overall run status.
    pub fn status(mut self, s: ReceiptStatus) -> Self {
        self.status = s;
        self
    }

    /// Sets the run duration in milliseconds.
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Sets the observed hardware envelope.
    pub fn hardware_metrics(mut self, m: HardwareMetrics) -> Self {
        self.hardware_metrics = m;
        self
    }

    /// Sets the evidence bundle hash.
    pub fn evidence_hash(mut self, h: impl Into<String>) -> Self {
        self.evidence_hash = h.into();
        self
    }

    /// Records the result of one invariant check.
    pub fn add_invariant_check(mut self, name: &str, passed: bool, detail: &str) -> Self {
        self.invariants_checked.push(InvariantCheck {
            name: name.to_string(),
            passed,
            detail: detail.to_string(),
        });
        self
    }

    /// Records a violation.
    pub fn add_violation(mut self, v: Violation) -> Self {
        self.violations.push(v);
        self
    }

    /// Finalizes the receipt, timestamps it, and signs it.
    ///
    /// When no non-empty id was provided, an id derived from the current
    /// second-of-day is generated; see `default_receipt_id` for the scheme.
    /// Fails only when a supplied id is unsafe to use as a file name.
    pub fn build(self) -> anyhow::Result<Receipt> {
        let now = Utc::now();
        let id = self
            .id
            .filter(|id| !id.trim().is_empty())
            .unwrap_or_else(|| default_receipt_id(now));
        validate_id(&id)?;
        let mut receipt = Receipt {
            id,
            status: self.status,
            severity: self.severity,
            timestamp: now,
            duration_ms: self.duration_ms,
            hardware_metrics: self.hardware_metrics,
            invariants_checked: self.invariants_checked,
            violations: self.violations,
            evidence_hash: self.evidence_hash,
            signature: String::new(),
        };
        receipt.sign();
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(status: ReceiptStatus, severity: Severity) -> Receipt {
        ReceiptBuilder::new(severity)
            .status(status)
            .duration_ms(42)
            .hardware_metrics(HardwareMetrics {
                peak_cpu_percent: 12.5,
                peak_ram_mb: 64,
            })
            .evidence_hash("evidence-sha256")
            .add_invariant_check("no regressions", true, "baseline matched")
            .add_violation(Violation {
                invariant: "latency budget".to_string(),
                severity: Severity::Medium,
                message: "p95 exceeded budget".to_string(),
                evidence_ref: Some("evidence/latency.txt".to_string()),
            })
            .build()
            .expect("receipt should build")
    }

    #[test]
    fn sign_then_verify_roundtrips() {
        let mut receipt = sample(ReceiptStatus::Passed, Severity::Low);
        assert_eq!(receipt.signature.len(), 64);
        assert!(receipt.verify());

        let payload = receipt.canonical_payload();
        receipt.sign();
        assert_eq!(payload, receipt.canonical_payload());
        assert!(receipt.verify());
    }

    #[test]
    fn tampering_with_payload_breaks_verification() {
        let mut receipt = sample(ReceiptStatus::Failed, Severity::High);
        assert!(receipt.verify());

        receipt.duration_ms += 1;
        assert!(!receipt.verify());

        receipt.duration_ms -= 1;
        assert!(receipt.verify());

        receipt.violations[0].message.push('!');
        assert!(!receipt.verify());
    }

    #[test]
    fn is_blocking_matrix() {
        let cases = [
            (ReceiptStatus::Passed, Severity::P0, false),
            (ReceiptStatus::Passed, Severity::Low, false),
            (ReceiptStatus::NotProven, Severity::P0, false),
            (ReceiptStatus::Failed, Severity::P0, true),
            (ReceiptStatus::Failed, Severity::High, true),
            (ReceiptStatus::Failed, Severity::Medium, false),
            (ReceiptStatus::Failed, Severity::Low, false),
            (ReceiptStatus::Blocked, Severity::High, true),
            (ReceiptStatus::Blocked, Severity::P0, true),
            (ReceiptStatus::Blocked, Severity::Low, false),
        ];
        for (status, severity, expected) in cases {
            let receipt = sample(status, severity);
            assert_eq!(receipt.is_blocking(), expected, "{status:?} / {severity:?}");
        }
    }

    #[test]
    fn wire_format_is_stable() {
        assert_eq!(
            serde_json::to_string(&ReceiptStatus::Passed).unwrap(),
            "\"PASSED\""
        );
        assert_eq!(
            serde_json::to_string(&ReceiptStatus::NotProven).unwrap(),
            "\"NOT_PROVEN\""
        );
        assert_eq!(serde_json::to_string(&Severity::P0).unwrap(), "\"P0\"");
        assert_eq!(serde_json::to_string(&Severity::High).unwrap(), "\"HIGH\"");
        assert_eq!(serde_json::to_string(&Severity::Medium).unwrap(), "\"MEDIUM\"");
        assert_eq!(serde_json::to_string(&Severity::Low).unwrap(), "\"LOW\"");
    }
}
