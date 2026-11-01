//! Core runtime for TestKit: project bootstrap, durable evidence, telemetry,
//! virtual time, and signed receipts.

pub mod telemetry;
pub mod time_travel;
pub mod receipt;
pub mod evidence;
pub mod semantic_map;

pub use evidence::EvidenceStore;
pub use receipt::{
    HardwareMetrics, InvariantCheck, Receipt, ReceiptBuilder, ReceiptStatus, Severity, Violation,
};
pub use semantic_map::{
    MapAuthor, MapCoverage, MapEdge, MapNode, Reconciliation, SemanticMap, UnresolvedPath,
    MAP_VERSION,
};
pub use telemetry::{
    ResourceEnvelope, RunawayFinding, RunawayKind, TelemetrySample, TelemetryWatcher,
};
pub use time_travel::{TimeJump, VirtualClock};

use std::path::{Path, PathBuf};

use anyhow::Context;

/// Crate version, populated by Cargo at build time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const TESTKIT_TEMPLATE: &str = include_str!("../../../templates/TESTKIT.md");

const GITIGNORE_ENTRIES: [&str; 2] = [".testkit/evidence/", ".testkit/scratch/"];

const WORK_DIRS: [&str; 4] = ["brain", "evidence", "receipts", "scenarios"];

const EVIDENCE_DIRS: [&str; 4] = ["screenshots", "traces", "logs", "telemetry"];

/// Summary of what [`init_project`] created or changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitReport {
    pub project_root: PathBuf,
    pub created: Vec<PathBuf>,
    pub gitignore_updated: bool,
    pub testkit_md_written: bool,
}

/// Prepare the `.testkit/` workspace for a project. Idempotent: existing
/// directories and files are left untouched.
pub fn init_project(project_root: &Path) -> anyhow::Result<InitReport> {
    let base = project_root.join(".testkit");
    let mut created = Vec::new();

    create_dir(&base, &mut created)?;
    for dir in WORK_DIRS {
        create_dir(&base.join(dir), &mut created)?;
    }
    for dir in EVIDENCE_DIRS {
        create_dir(&base.join("evidence").join(dir), &mut created)?;
    }

    let testkit_md = project_root.join("TESTKIT.md");
    let testkit_md_written = if testkit_md.exists() {
        false
    } else {
        std::fs::write(&testkit_md, TESTKIT_TEMPLATE)
            .with_context(|| format!("writing {}", testkit_md.display()))?;
        true
    };

    let gitignore_updated = evidence::ensure_gitignore(project_root, &GITIGNORE_ENTRIES)?;

    Ok(InitReport {
        project_root: project_root.to_path_buf(),
        created,
        gitignore_updated,
        testkit_md_written,
    })
}

fn create_dir(path: &Path, created: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if path.is_dir() {
        return Ok(());
    }
    std::fs::create_dir_all(path).with_context(|| format!("creating {}", path.display()))?;
    created.push(path.to_path_buf());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_project() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "testkit-lib-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn init_creates_workspace_and_is_idempotent() {
        let root = temp_project();
        let report = init_project(&root).expect("first init");
        assert!(root.join(".testkit/brain").is_dir());
        assert!(root.join(".testkit/evidence/screenshots").is_dir());
        assert!(root.join(".testkit/evidence/telemetry").is_dir());
        assert!(report.testkit_md_written);
        assert!(report.gitignore_updated);

        let second = init_project(&root).expect("second init");
        assert!(second.created.is_empty());
        assert!(!second.testkit_md_written);
        assert!(!second.gitignore_updated);
    }
}
