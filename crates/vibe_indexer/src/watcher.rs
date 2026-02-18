use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::extract::stable_hash_hex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChangeKind {
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub file: String,
    pub kind: FileChangeKind,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FileWatcher {
    known_hashes: BTreeMap<String, String>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prime_from_paths(&mut self, paths: &[PathBuf]) -> Result<(), String> {
        let snapshot = compute_hashes(paths)?;
        self.known_hashes = snapshot;
        Ok(())
    }

    pub fn scan(&mut self, paths: &[PathBuf]) -> Result<Vec<FileChange>, String> {
        let next_hashes = compute_hashes(paths)?;
        let mut out = Vec::new();

        let old_files = self.known_hashes.keys().cloned().collect::<BTreeSet<_>>();
        let new_files = next_hashes.keys().cloned().collect::<BTreeSet<_>>();

        for file in new_files.difference(&old_files) {
            out.push(FileChange {
                file: file.clone(),
                kind: FileChangeKind::Added,
                hash: next_hashes.get(file).cloned(),
            });
        }

        for file in old_files.difference(&new_files) {
            out.push(FileChange {
                file: file.clone(),
                kind: FileChangeKind::Removed,
                hash: None,
            });
        }

        for file in old_files.intersection(&new_files) {
            let old_hash = self.known_hashes.get(file);
            let new_hash = next_hashes.get(file);
            if old_hash != new_hash {
                out.push(FileChange {
                    file: file.clone(),
                    kind: FileChangeKind::Modified,
                    hash: new_hash.cloned(),
                });
            }
        }

        out.sort_by(|a, b| a.file.cmp(&b.file));
        self.known_hashes = next_hashes;
        Ok(out)
    }
}

fn compute_hashes(paths: &[PathBuf]) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    for path in paths {
        if !path.exists() {
            continue;
        }
        if path.is_file() {
            if path.extension().and_then(|e| e.to_str()) == Some("vibe") {
                let key = normalize_path(path);
                let source = fs::read_to_string(path)
                    .map_err(|e| format!("failed to read `{}`: {e}", path.display()))?;
                out.insert(key, stable_hash_hex(&source));
            }
            continue;
        }
        collect_file_hashes(path, &mut out)?;
    }
    Ok(out)
}

fn collect_file_hashes(dir: &Path, out: &mut BTreeMap<String, String>) -> Result<(), String> {
    let mut entries = fs::read_dir(dir)
        .map_err(|e| format!("failed to read directory `{}`: {e}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_file_hashes(&path, out)?;
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("vibe") {
            continue;
        }
        let source = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read `{}`: {e}", path.display()))?;
        out.insert(normalize_path(&path), stable_hash_hex(&source));
    }
    Ok(())
}

fn normalize_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}
