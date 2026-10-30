//! TestKit command-line interface.
//!
//! One binary, four verbs: `init` grounds a workspace, `verify` executes
//! scenarios, `gate` evaluates the receipt ledger, and `receipt` inspects the
//! signed artifacts. Every command is deterministic and non-interactive so it
//! can run identically from a terminal, a hook, or CI.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands {
    pub mod gate;
    pub mod init;
    pub mod receipt;
    pub mod verify;
}

/// Top-level command-line surface.
#[derive(Debug, Parser)]
#[command(
    name = "testkit",
    version,
    about = "Deterministic verification, evidence, and signed receipts for agent-built software",
    long_about = None,
    propagate_version = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// The four TestKit verbs.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Ground a project: create .testkit/, update .gitignore, and drop TESTKIT.md
    Init {
        /// Project directory (defaults to the current directory)
        #[arg(long, value_name = "dir")]
        path: Option<PathBuf>,
    },

    /// Run deterministic scenarios and record evidence
    Verify {
        /// Project directory (defaults to the current directory)
        #[arg(long, value_name = "dir")]
        path: Option<PathBuf>,

        /// Run only the scenario with this id
        #[arg(long, value_name = "id")]
        scenario: Option<String>,

        /// Run only scenarios whose invariant tag contains this string
        #[arg(long, value_name = "tag")]
        tag: Option<String>,

        /// Override the CPU envelope ceiling (percent) for this run
        #[arg(long, value_name = "percent")]
        max_cpu: Option<f32>,

        /// Override the RAM envelope ceiling (MiB) for this run
        #[arg(long, value_name = "mb")]
        max_rss_mb: Option<u64>,

        /// Stop at the first failing scenario
        #[arg(long)]
        fail_fast: bool,
    },

    /// Evaluate signed receipts and block when invariants are not proven
    Gate {
        /// Project directory (defaults to the current directory)
        #[arg(long, value_name = "dir")]
        path: Option<PathBuf>,

        /// Emit the verdict as machine-readable JSON
        #[arg(long)]
        json: bool,

        /// Suppress human-readable output
        #[arg(long)]
        quiet: bool,

        /// Report blocking receipts but always exit 0 (advisory/hook mode)
        #[arg(long)]
        warn_only: bool,
    },

    /// Inspect and verify cryptographic receipts
    Receipt {
        #[command(subcommand)]
        command: ReceiptCommands,
    },
}

/// Subcommands of `testkit receipt`.
#[derive(Debug, Subcommand)]
enum ReceiptCommands {
    /// List every receipt in the project ledger
    List {
        /// Project directory (defaults to the current directory)
        #[arg(long, value_name = "dir")]
        path: Option<PathBuf>,
    },

    /// Show the most recent receipt in the ledger
    Latest {
        /// Project directory (defaults to the current directory)
        #[arg(long, value_name = "dir")]
        path: Option<PathBuf>,
    },

    /// Verify one receipt file's signature
    Verify {
        /// Path to a run_XXXX.json receipt
        #[arg(value_name = "file")]
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    match run() {
        Ok(()) => Ok(()),
        Err(error) => {
            eprintln!("error: {error:#}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => commands::init::run(path),
        Commands::Verify {
            path,
            scenario,
            tag,
            max_cpu,
            max_rss_mb,
            fail_fast,
        } => commands::verify::run(path, scenario, tag, max_cpu, max_rss_mb, fail_fast),
        Commands::Gate {
            path,
            json,
            quiet,
            warn_only,
        } => commands::gate::run(path, json, quiet, warn_only),
        Commands::Receipt { command } => match command {
            ReceiptCommands::List { path } => commands::receipt::list(path),
            ReceiptCommands::Latest { path } => commands::receipt::latest(path),
            ReceiptCommands::Verify { file } => commands::receipt::verify_file(file),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_accepts_an_optional_path() {
        let cli = Cli::try_parse_from(["testkit", "init"]).expect("init without path");
        assert!(matches!(cli.command, Commands::Init { path: None }));

        let cli = Cli::try_parse_from(["testkit", "init", "--path", "demo"])
            .expect("init with path");
        match cli.command {
            Commands::Init { path } => assert_eq!(path, Some(PathBuf::from("demo"))),
            other => panic!("expected init, got {other:?}"),
        }
    }

    #[test]
    fn verify_flags_are_optional() {
        let cli = Cli::try_parse_from(["testkit", "verify", "--fail-fast"])
            .expect("verify with fail-fast");
        match cli.command {
            Commands::Verify {
                path,
                scenario,
                tag,
                max_cpu,
                max_rss_mb,
                fail_fast,
            } => {
                assert_eq!(path, None);
                assert_eq!(scenario, None);
                assert_eq!(tag, None);
                assert_eq!(max_cpu, None);
                assert_eq!(max_rss_mb, None);
                assert!(fail_fast);
            }
            other => panic!("expected verify, got {other:?}"),
        }
    }

    #[test]
    fn verify_accepts_tag_and_envelope_overrides() {
        let cli = Cli::try_parse_from([
            "testkit",
            "verify",
            "--tag",
            "INV-012",
            "--max-cpu",
            "80",
            "--max-rss-mb",
            "256",
        ])
        .expect("verify with tag and envelope overrides");
        match cli.command {
            Commands::Verify {
                tag,
                max_cpu,
                max_rss_mb,
                ..
            } => {
                assert_eq!(tag.as_deref(), Some("INV-012"));
                assert_eq!(max_cpu, Some(80.0));
                assert_eq!(max_rss_mb, Some(256));
            }
            other => panic!("expected verify, got {other:?}"),
        }
    }

    #[test]
    fn receipt_latest_takes_an_optional_path() {
        let cli = Cli::try_parse_from(["testkit", "receipt", "latest"]).expect("receipt latest");
        match cli.command {
            Commands::Receipt {
                command: ReceiptCommands::Latest { path },
            } => assert_eq!(path, None),
            other => panic!("expected receipt latest, got {other:?}"),
        }
    }

    #[test]
    fn receipt_verify_requires_a_file() {
        assert!(Cli::try_parse_from(["testkit", "receipt", "verify"]).is_err());

        let cli = Cli::try_parse_from(["testkit", "receipt", "verify", "run_1.json"])
            .expect("receipt verify with file");
        match cli.command {
            Commands::Receipt {
                command: ReceiptCommands::Verify { file },
            } => assert_eq!(file, PathBuf::from("run_1.json")),
            other => panic!("expected receipt verify, got {other:?}"),
        }
    }
}
