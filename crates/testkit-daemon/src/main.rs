//! `testkitd` — TestKit's ambient background watcher.

use std::path::PathBuf;

use clap::Parser;

mod daemon;

/// Ambient background watcher for TestKit projects.
#[derive(Debug, Parser)]
#[command(
    name = "testkitd",
    version,
    about = "Watch TestKit project state and record change events"
)]
struct Cli {
    /// Project root containing a `.testkit` directory.
    #[arg(long, value_name = "DIR", default_value = ".")]
    project: PathBuf,

    /// Loopback port used as the single-instance lock.
    #[arg(
        long,
        value_name = "PORT",
        default_value_t = 4949,
        value_parser = clap::value_parser!(u16).range(1..)
    )]
    port: u16,

    /// Poll interval in milliseconds.
    #[arg(
        long,
        value_name = "MS",
        default_value_t = 1000,
        value_parser = clap::value_parser!(u64).range(1..)
    )]
    interval_ms: u64,

    /// Perform one remap/staleness pass, log it, and exit (hook mode).
    #[arg(long)]
    once_remap: bool,

    /// Suppress informational output (accepted for hook compatibility).
    #[arg(long)]
    quiet: bool,
}

/// Keep the runtime small: two workers are ample for a polling watcher.
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let cli = Cli::parse();
    let _ = cli.quiet;

    if cli.once_remap {
        if let Err(error) = daemon::once_remap(cli.project).await {
            eprintln!("testkitd: {error:#}");
            std::process::exit(1);
        }
        return;
    }

    if let Err(error) = daemon::run(cli.project, cli.port, cli.interval_ms).await {
        eprintln!("testkitd: {error:#}");
        std::process::exit(1);
    }
}
