//! `testkit init` — mechanical project grounding.
//!
//! Initialization is deterministic and non-conversational: no prompts, no
//! interviews, no model calls. Re-running it is safe and prints the same four
//! guarantees, matching the idempotent behavior of `git init`.

use std::env;
use std::path::PathBuf;

use anyhow::Context;

use testkit_core::init_project;

/// U+2714 HEAVY CHECK MARK.
const CHECK: &str = "\u{2714}";

/// Ground `path` (or the current directory) as a TestKit project.
///
/// The returned [`testkit_core::InitReport`] is the authority for the resolved
/// project root; the four success lines are printed unconditionally because a
/// re-run that changes nothing is still a successful initialization.
pub fn run(path: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    let root = resolve_root(path)?;
    let report = init_project(&root)
        .with_context(|| format!("failed to initialize TestKit at {}", root.display()))?;

    println!("{CHECK} Created .testkit/ structure (brain/, evidence/, receipts/, scenarios/)");
    println!("{CHECK} Added .testkit/evidence/ and .testkit/scratch/ to .gitignore");
    println!("{CHECK} Dropped TESTKIT.md (Agent Operating Directive)");
    println!("{CHECK} TestKit initialized. Scaffolding and agent rules in place.");
    println!("Project root: {}", report.project_root.display());

    Ok(())
}

/// Prefer the explicit `--path`, otherwise use the current working directory.
fn resolve_root(path: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    match path {
        Some(path) => {
            anyhow::ensure!(!path.as_os_str().is_empty(), "--path must not be empty");
            Ok(path)
        }
        None => env::current_dir().context("could not determine the current working directory"),
    }
}
