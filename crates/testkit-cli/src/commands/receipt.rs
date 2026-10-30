//! `testkit receipt` — inspect the receipt ledger and verify individual files.

use std::path::{Path, PathBuf};
use std::process::exit;

use anyhow::{Context, Result};

use testkit_core::receipt::Receipt;

/// A single row in the receipt listing.
struct Row {
    id: String,
    status: String,
    severity: String,
    signature: &'static str,
    violations: usize,
}

/// List every receipt in the project ledger with signature status.
///
/// Informational only: always returns `Ok(())`, even when some signatures do
/// not verify, because the footer reports the invalid count.
pub fn list(path: Option<PathBuf>) -> Result<()> {
    let root = resolve_root(path);
    let dir = root.join(".testkit").join("receipts");

    let mut rows: Vec<Row> = Vec::new();
    let mut invalid = 0usize;

    if dir.is_dir() {
        let mut files = collect_receipt_files(&dir);
        files.sort();

        for file in files {
            let contents = match std::fs::read_to_string(&file) {
                Ok(contents) => contents,
                Err(error) => {
                    eprintln!("warning: could not read {}: {}", file.display(), error);
                    continue;
                }
            };

            let receipt: Receipt = match serde_json::from_str(&contents) {
                Ok(receipt) => receipt,
                Err(error) => {
                    eprintln!("warning: could not parse {}: {}", file.display(), error);
                    continue;
                }
            };

            let valid = receipt.verify();
            if !valid {
                invalid += 1;
            }

            rows.push(Row {
                id: receipt.id.clone(),
                status: status_label(&receipt),
                severity: severity_label(&receipt),
                signature: if valid { "valid" } else { "INVALID" },
                violations: receipt.violations.len(),
            });
        }
    }

    if rows.is_empty() {
        println!("No receipts found.");
        return Ok(());
    }

    println!(
        "{:<24}  {:<10}  {:<8}  {:<7}  {}",
        "ID", "STATUS", "SEVERITY", "SIGNATURE", "VIOLATIONS"
    );

    for row in &rows {
        println!(
            "{:<24}  {:<10}  {:<8}  {:<7}  {} violation(s)",
            row.id, row.status, row.severity, row.signature, row.violations
        );
    }

    println!(
        "{} receipt(s), {} invalid signature(s)",
        rows.len(),
        invalid
    );

    Ok(())
}

/// Show the most recent receipt in the ledger and verify its signature.
///
/// Receipt file names sort chronologically (`run_receipt-YYYY-MM-DD-NNNN.json`),
/// so the lexicographically last file is the newest. Delegates to
/// [`verify_file`], which terminates with `exit(1)` when the signature does
/// not match.
pub fn latest(path: Option<PathBuf>) -> Result<()> {
    let root = resolve_root(path);
    let dir = root.join(".testkit").join("receipts");

    let mut files = collect_receipt_files(&dir);
    files.sort();

    let file = match files.pop() {
        Some(file) => file,
        None => {
            println!("No receipts found.");
            return Ok(());
        }
    };

    println!("Latest receipt: {}", file.display());
    verify_file(file)
}

/// Verify a single receipt file and print its details or the mismatch.
///
/// Returns `Ok(())` for a valid receipt; terminates with `exit(1)` when the
/// signature does not match the signed payload.
pub fn verify_file(file: PathBuf) -> Result<()> {
    let contents = std::fs::read_to_string(&file)
        .with_context(|| format!("reading receipt file {}", file.display()))?;

    let receipt: Receipt = serde_json::from_str(&contents)
        .with_context(|| format!("parsing receipt file {}", file.display()))?;

    if !receipt.verify() {
        println!(
            "INVALID SIGNATURE {} — receipt was modified after signing",
            receipt.id
        );
        exit(1);
    }

    println!(
        "VERIFIED {} [{}/{}]",
        receipt.id,
        status_label(&receipt),
        severity_label(&receipt)
    );
    println!("timestamp: {}", receipt.timestamp.to_rfc3339());
    println!("duration_ms: {}", receipt.duration_ms);
    println!("hardware_metrics: {:?}", receipt.hardware_metrics);
    println!("invariants_checked: {}", receipt.invariants_checked.len());
    println!("evidence_hash: {}", evidence_prefix(&receipt.evidence_hash));

    Ok(())
}

/// Resolve the project root, preferring an explicit path over the cwd.
fn resolve_root(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// Collect candidate `.json` files from the ledger directory.
fn collect_receipt_files(dir: &Path) -> Vec<PathBuf> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect()
}

/// Return the first 16 characters of a hash, or the whole value if shorter.
fn evidence_prefix(hash: &str) -> &str {
    match hash.char_indices().nth(16) {
        Some((index, _)) => &hash[..index],
        None => hash,
    }
}

/// Render a receipt status as an upper-case label.
fn status_label(receipt: &Receipt) -> String {
    format!("{:?}", receipt.status).to_uppercase()
}

/// Render a receipt severity as an upper-case label.
fn severity_label(receipt: &Receipt) -> String {
    format!("{:?}", receipt.severity).to_uppercase()
}
