//! Durable evidence storage rooted at `<project>/.testkit/evidence`.

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

const EVIDENCE_SUBDIRS: [&str; 4] = ["screenshots", "traces", "logs", "telemetry"];

/// Writes artefacts beneath a project's evidence directory.
#[derive(Debug, Clone)]
pub struct EvidenceStore {
    root: PathBuf,
}

impl EvidenceStore {
    /// Point a store at `<project_root>/.testkit/evidence`.
    pub fn new(project_root: &Path) -> Self {
        Self {
            root: project_root.join(".testkit").join("evidence"),
        }
    }

    /// The evidence root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Create the root and its standard subdirectories.
    pub fn ensure_dirs(&self) -> io::Result<()> {
        fs::create_dir_all(&self.root)?;
        for sub in EVIDENCE_SUBDIRS {
            fs::create_dir_all(self.root.join(sub))?;
        }
        Ok(())
    }

    /// Write raw bytes to `subdir/name`, creating parents as needed.
    pub fn write_bytes(&self, subdir: &str, name: &str, bytes: &[u8]) -> Result<PathBuf> {
        let path = self.resolve(subdir, name)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
        Ok(path)
    }

    /// Write UTF-8 text to `subdir/name`.
    pub fn write_text(&self, subdir: &str, name: &str, content: &str) -> Result<PathBuf> {
        self.write_bytes(subdir, name, content.as_bytes())
    }

    /// Write pretty-printed JSON to `subdir/name`.
    pub fn write_json<T: serde::Serialize>(
        &self,
        subdir: &str,
        name: &str,
        value: &T,
    ) -> Result<PathBuf> {
        let bytes = serde_json::to_vec_pretty(value)
            .with_context(|| format!("serializing {subdir}/{name}"))?;
        self.write_bytes(subdir, name, &bytes)
    }

    /// Hex-encoded SHA-256 of `bytes`.
    pub fn hash_bytes(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    /// Hex-encoded SHA-256 of a file's contents.
    pub fn hash_file(path: &Path) -> Result<String> {
        let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        Ok(Self::hash_bytes(&bytes))
    }

    /// Hash the whole evidence tree: sorted `path:hash` lines, hashed once.
    pub fn hash_tree(&self) -> Result<String> {
        if !self.root.exists() {
            return Ok(Self::hash_bytes(b""));
        }
        let mut entries = Vec::new();
        collect_files(&self.root, &self.root, &mut entries)?;
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let mut hasher = Sha256::new();
        for (relative, hash) in entries {
            hasher.update(format!("{relative}:{hash}\n").as_bytes());
        }
        Ok(hex::encode(hasher.finalize()))
    }

    fn resolve(&self, subdir: &str, name: &str) -> Result<PathBuf> {
        if !is_safe_relative(subdir) {
            anyhow::bail!("invalid evidence subdirectory: {subdir:?}");
        }
        if !is_safe_relative(name) {
            anyhow::bail!("invalid evidence file name: {name:?}");
        }
        Ok(self.root.join(subdir).join(name))
    }
}

/// Append any missing entries to `<project>/.gitignore`. Returns whether the
/// file changed. Existing lines are never duplicated.
pub fn ensure_gitignore(project_root: &Path, entries: &[&str]) -> Result<bool> {
    let path = project_root.join(".gitignore");
    let existing = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err).with_context(|| format!("reading {}", path.display())),
    };

    let known: Vec<String> = existing
        .lines()
        .map(|line| line.trim().to_string())
        .collect();
    let mut additions: Vec<&str> = Vec::new();
    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() || known.iter().any(|line| line.as_str() == entry) {
            continue;
        }
        if !additions.contains(&entry) {
            additions.push(entry);
        }
    }

    if additions.is_empty() {
        return Ok(false);
    }

    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    for entry in additions {
        updated.push_str(entry);
        updated.push('\n');
    }
    fs::write(&path, updated).with_context(|| format!("writing {}", path.display()))?;
    Ok(true)
}

fn is_safe_relative(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) -> Result<()> {
    let reader = fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))?;
    for entry in reader {
        let entry = entry.with_context(|| format!("reading entry in {}", dir.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("inspecting {}", path.display()))?;
        if file_type.is_dir() {
            collect_files(root, &path, out)?;
        } else if file_type.is_file() {
            let relative = relative_key(root, &path);
            let hash = EvidenceStore::hash_file(&path)?;
            out.push((relative, hash));
        }
    }
    Ok(())
}

fn relative_key(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "testkit-evidence-{label}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn ensure_gitignore_is_idempotent() {
        let root = temp_dir("gitignore");
        assert!(ensure_gitignore(&root, &[".testkit/evidence/"]).unwrap());
        assert!(!ensure_gitignore(&root, &[".testkit/evidence/"]).unwrap());
        assert!(ensure_gitignore(&root, &[".testkit/scratch/"]).unwrap());

        let content = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert_eq!(content.matches(".testkit/evidence/").count(), 1);
        assert_eq!(content.matches(".testkit/scratch/").count(), 1);
    }

    #[test]
    fn writes_files_and_hashes_tree() {
        let root = temp_dir("store");
        let store = EvidenceStore::new(&root);
        store.ensure_dirs().unwrap();

        let log = store.write_text("logs", "run.txt", "hello").unwrap();
        assert!(log.is_file());
        let json = store
            .write_json("telemetry", "sample.json", &serde_json::json!({"ok": true}))
            .unwrap();
        assert!(json.is_file());

        assert_eq!(EvidenceStore::hash_bytes(b"abc").len(), 64);
        assert_eq!(
            EvidenceStore::hash_file(&log).unwrap(),
            EvidenceStore::hash_bytes(b"hello")
        );
        let tree = store.hash_tree().unwrap();
        assert_eq!(tree.len(), 64);
        assert_eq!(tree, store.hash_tree().unwrap());
    }

    #[test]
    fn rejects_paths_that_escape_the_root() {
        let root = temp_dir("escape");
        let store = EvidenceStore::new(&root);
        assert!(store.write_text("..", "evil.txt", "x").is_err());
        assert!(store.write_text("logs", "../evil.txt", "x").is_err());
    }
}
