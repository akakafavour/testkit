//! Ambient background watcher for a TestKit project.
//!
//! Polls `.testkit/{brain,scenarios,receipts}` modification times and appends
//! change events to `.testkit/evidence/logs/daemon.log`; a held loopback
//! socket doubles as a single-instance lock.

use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::signal;

/// Watched directories, paired with the event label emitted on change.
const WATCHED: [(&str, &str); 3] = [
    ("brain", "brain-changed"),
    ("scenarios", "scenarios-changed"),
    ("receipts", "receipts-changed"),
];

/// Run the watcher until Ctrl+C, adopting an instance that already owns `port`.
pub async fn run(project: PathBuf, port: u16, interval_ms: u64) -> Result<()> {
    let (root, testkit) = resolve_project(&project).await?;
    let Some(_lock) = bind_instance_lock(port).await? else {
        println!(
            "testkitd: another instance is already running on this machine (port {port}) — adopting it."
        );
        return Ok(());
    };

    let logs = testkit.join("evidence").join("logs");
    let pid = std::process::id();
    append_line(&logs, &format!("daemon-started pid={pid} project={}", root.display())).await?;
    let _pid_file = PidFile::write(testkit.join("daemon.pid"), pid).await?;

    watch(&testkit, &logs, interval_ms).await;
    if let Err(error) = append_line(&logs, "daemon-stopped").await {
        eprintln!("testkitd: failed to write daemon-stopped: {error:#}");
    }
    Ok(())
}

/// One-shot hook mode: record a remap-requested event and exit without
/// acquiring the instance lock, so it is safe beside a running daemon.
pub async fn once_remap(project: PathBuf) -> Result<()> {
    let (root, testkit) = resolve_project(&project).await?;
    let logs = testkit.join("evidence").join("logs");
    append_line(
        &logs,
        &format!(
            "remap-requested project={} watched_dirs={}",
            root.display(),
            WATCHED.len()
        ),
    )
    .await?;
    Ok(())
}

/// Canonicalize the project root and check that TestKit initialized it.
async fn resolve_project(project: &Path) -> Result<(PathBuf, PathBuf)> {
    let root = tokio::fs::canonicalize(project)
        .await
        .with_context(|| format!("failed to resolve project root {}", project.display()))?;
    let testkit = root.join(".testkit");
    if !is_directory(&testkit) {
        eprintln!("testkitd: project not initialized. Run `testkit init` first.");
        std::process::exit(2);
    }
    Ok((root, testkit))
}

/// Poll for file changes until Ctrl+C.
async fn watch(testkit: &Path, logs: &Path, interval_ms: u64) {
    let mut state = WatchState::capture(testkit);
    let mut ticker = tokio::time::interval(Duration::from_millis(interval_ms.max(1)));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let shutdown = signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            result = shutdown.as_mut() => {
                if let Err(error) = result {
                    eprintln!("testkitd: failed to listen for shutdown signal: {error}");
                }
                break;
            }
            _ = ticker.tick() => {
                for event in state.refresh(testkit) {
                    if let Err(error) = append_line(logs, &event).await {
                        eprintln!("testkitd: failed to write daemon log: {error:#}");
                    }
                }
            }
        }
    }
}

/// Bind the loopback instance lock; `None` means another daemon owns the port.
async fn bind_instance_lock(port: u16) -> Result<Option<TcpListener>> {
    match TcpListener::bind(("127.0.0.1", port)).await {
        Ok(listener) => Ok(Some(listener)),
        Err(error) if error.kind() == ErrorKind::AddrInUse => Ok(None),
        Err(error) => Err(error)
            .with_context(|| format!("failed to bind instance lock on 127.0.0.1:{port}")),
    }
}

/// Append one RFC3339-stamped line to `daemon.log`, creating its directory.
async fn append_line(logs_dir: &Path, message: &str) -> Result<()> {
    tokio::fs::create_dir_all(logs_dir)
        .await
        .with_context(|| format!("failed to create {}", logs_dir.display()))?;
    let path = logs_dir.join("daemon.log");
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .with_context(|| format!("failed to open {}", path.display()))?;
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    file.write_all(format!("{timestamp} {message}\n").as_bytes())
        .await
        .with_context(|| format!("failed to append to {}", path.display()))?;
    file.flush()
        .await
        .with_context(|| format!("failed to flush {}", path.display()))
}

/// Owns `daemon.pid`; removes it whenever the daemon exits.
struct PidFile {
    path: PathBuf,
}

impl PidFile {
    async fn write(path: PathBuf, pid: u32) -> Result<Self> {
        tokio::fs::write(&path, pid.to_string())
            .await
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(Self { path })
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_file(&self.path) {
            if error.kind() != ErrorKind::NotFound {
                eprintln!("testkitd: failed to remove {}: {error}", self.path.display());
            }
        }
    }
}

/// Last observed modification time for each watched file.
struct WatchState {
    snapshot: HashMap<PathBuf, SystemTime>,
}

impl WatchState {
    /// Take an initial snapshot without emitting events.
    fn capture(testkit: &Path) -> Self {
        Self {
            snapshot: scan(testkit),
        }
    }

    /// Diff against the previous snapshot, store the new one, and return events.
    fn refresh(&mut self, testkit: &Path) -> Vec<String> {
        let current = scan(testkit);
        let mut events = Vec::new();
        for (path, modified) in &current {
            if self.snapshot.get(path) != Some(modified) {
                push_event(&mut events, testkit, path);
            }
        }
        for path in self.snapshot.keys() {
            if !current.contains_key(path) {
                push_event(&mut events, testkit, path);
            }
        }
        self.snapshot = current;
        events.sort();
        events
    }
}

/// Push the event label for `path` when it lives in a watched directory.
fn push_event(events: &mut Vec<String>, testkit: &Path, path: &Path) {
    if let Some(label) = event_label(testkit, path) {
        events.push(format!("{label} {}", path.display()));
    }
}

/// Map a watched file to its event label using its top-level directory.
fn event_label(testkit: &Path, path: &Path) -> Option<&'static str> {
    let relative = path.strip_prefix(testkit).ok()?;
    let Some(Component::Normal(name)) = relative.components().next() else {
        return None;
    };
    let name = name.to_str()?;
    WATCHED
        .iter()
        .find(|(directory, _)| *directory == name)
        .map(|(_, label)| *label)
}

/// Collect modification times for files directly inside each watched directory.
fn scan(testkit: &Path) -> HashMap<PathBuf, SystemTime> {
    let mut snapshot = HashMap::new();
    for (directory, _) in WATCHED {
        let Ok(entries) = std::fs::read_dir(testkit.join(directory)) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                continue;
            }
            if let Ok(modified) = metadata.modified() {
                snapshot.insert(entry.path(), modified);
            }
        }
    }
    snapshot
}

fn is_directory(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|metadata| metadata.is_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_additions_and_removals() {
        let testkit = std::env::temp_dir().join(format!("testkitd-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&testkit);
        std::fs::create_dir_all(testkit.join("brain")).expect("create brain");
        let mut state = WatchState::capture(&testkit);
        let file = testkit.join("brain").join("map.json");
        std::fs::write(&file, "{}").expect("write map");
        assert_eq!(state.refresh(&testkit).len(), 1);
        std::fs::remove_file(&file).expect("remove map");
        assert!(state.refresh(&testkit)[0].starts_with("brain-changed "));
        std::fs::remove_dir_all(&testkit).expect("clean scratch");
    }

    #[test]
    fn labels_paths_by_watched_directory() {
        let testkit = PathBuf::from(".testkit");
        assert_eq!(
            event_label(&testkit, &testkit.join("receipts").join("receipt.json")),
            Some("receipts-changed")
        );
        assert_eq!(event_label(&testkit, &testkit.join("logs").join("x")), None);
    }
}
