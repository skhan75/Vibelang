use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::{FileIndex, IndexSnapshot, IndexStats, INDEX_FILENAME, INDEX_SCHEMA_VERSION};

#[derive(Debug, Clone)]
pub struct IndexStore {
    root: PathBuf,
    snapshot: IndexSnapshot,
    recovered_from_corruption: bool,
}

impl IndexStore {
    pub fn open_or_create(root: impl Into<PathBuf>) -> Result<Self, String> {
        let root = root.into();
        fs::create_dir_all(&root)
            .map_err(|e| format!("failed to create index directory `{}`: {e}", root.display()))?;
        let index_path = root.join(INDEX_FILENAME);
        if !index_path.exists() {
            return Ok(Self {
                root,
                snapshot: IndexSnapshot::default(),
                recovered_from_corruption: false,
            });
        }

        let text = fs::read_to_string(&index_path)
            .map_err(|e| format!("failed to read index file `{}`: {e}", index_path.display()))?;
        let mut recovered_from_corruption = false;
        let snapshot = match serde_json::from_str::<IndexSnapshot>(&text) {
            Ok(mut snapshot) => {
                if snapshot.schema_version != INDEX_SCHEMA_VERSION {
                    move_to_corrupt_backup(&index_path)?;
                    recovered_from_corruption = true;
                    IndexSnapshot::default()
                } else {
                    snapshot.normalize();
                    snapshot
                }
            }
            Err(_) => {
                move_to_corrupt_backup(&index_path)?;
                recovered_from_corruption = true;
                IndexSnapshot::default()
            }
        };
        Ok(Self {
            root,
            snapshot,
            recovered_from_corruption,
        })
    }

    pub fn save(&self) -> Result<(), String> {
        fs::create_dir_all(&self.root)
            .map_err(|e| format!("failed to create index directory `{}`: {e}", self.root.display()))?;
        let mut snapshot = self.snapshot.clone();
        snapshot.normalize();
        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| format!("failed to serialize index snapshot: {e}"))?;
        let target = self.root.join(INDEX_FILENAME);
        let temp = self.root.join(format!("{INDEX_FILENAME}.tmp"));
        fs::write(&temp, json)
            .map_err(|e| format!("failed to write temp index file `{}`: {e}", temp.display()))?;
        fs::rename(&temp, &target).map_err(|e| {
            format!(
                "failed to atomically replace index file `{}` with `{}`: {e}",
                target.display(),
                temp.display()
            )
        })?;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.snapshot = IndexSnapshot::default();
    }

    pub fn upsert_file(&mut self, file_index: FileIndex) {
        self.snapshot
            .files
            .insert(file_index.file.clone(), file_index);
        self.snapshot.normalize();
    }

    pub fn remove_file(&mut self, file_path: &str) {
        self.snapshot.files.remove(file_path);
    }

    pub fn snapshot(&self) -> &IndexSnapshot {
        &self.snapshot
    }

    pub fn snapshot_mut(&mut self) -> &mut IndexSnapshot {
        &mut self.snapshot
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn recovered_from_corruption(&self) -> bool {
        self.recovered_from_corruption
    }

    pub fn stats(&self) -> IndexStats {
        let files = self.snapshot.files.len();
        let mut symbols = 0usize;
        let mut references = 0usize;
        let mut function_meta = 0usize;
        let mut diagnostics = 0usize;
        let mut bytes = 0usize;

        for (file, file_index) in &self.snapshot.files {
            bytes += file.len();
            bytes += file_index.file_hash.len();
            symbols += file_index.symbols.len();
            references += file_index.references.len();
            function_meta += file_index.function_meta.len();
            diagnostics += file_index.diagnostics.len();

            for dep in &file_index.dependencies {
                bytes += dep.len();
            }
            for symbol in &file_index.symbols {
                bytes += symbol.name.len();
                bytes += symbol.file.len();
                bytes += symbol.module.as_ref().map(|m| m.len()).unwrap_or(0);
            }
            for reference in &file_index.references {
                bytes += reference.file.len();
            }
            for meta in &file_index.function_meta {
                bytes += meta.function_name.len();
                bytes += meta.file.len();
                bytes += meta.signature_hash.len();
                bytes += meta.intent_text.as_ref().map(|t| t.len()).unwrap_or(0);
                bytes += meta.effects_declared.iter().map(String::len).sum::<usize>();
                bytes += meta.effects_observed.iter().map(String::len).sum::<usize>();
            }
            for diag in &file_index.diagnostics {
                bytes += diag.code.len() + diag.message.len();
            }
            for mismatch in &file_index.effect_mismatches {
                bytes += mismatch.function_name.len() + mismatch.file.len();
                bytes += mismatch.declared_only.iter().map(String::len).sum::<usize>();
                bytes += mismatch.observed_only.iter().map(String::len).sum::<usize>();
            }
        }

        IndexStats {
            files,
            symbols,
            references,
            function_meta,
            diagnostics,
            memory_estimate_bytes: bytes,
        }
    }
}

fn move_to_corrupt_backup(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("system time error: {e}"))?
        .as_secs();
    let backup = path.with_extension(format!("corrupt.{ts}"));
    fs::rename(path, &backup).map_err(|e| {
        format!(
            "failed to move corrupted index `{}` to backup `{}`: {e}",
            path.display(),
            backup.display()
        )
    })?;
    Ok(())
}
