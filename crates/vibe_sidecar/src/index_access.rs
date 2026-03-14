// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use vibe_indexer::{
    effect_mismatches, list_missing_examples, FunctionMeta, IndexSnapshot, IndexStore,
};

#[derive(Debug, Clone)]
pub struct ReadOnlyIndex {
    store: IndexStore,
    index_root: std::path::PathBuf,
}

impl ReadOnlyIndex {
    pub fn open(index_root: &Path) -> Result<Self, String> {
        let store = IndexStore::open_or_create(index_root)?;
        Ok(Self {
            store,
            index_root: index_root.to_path_buf(),
        })
    }

    pub fn index_root(&self) -> &Path {
        &self.index_root
    }

    pub fn snapshot(&self) -> &IndexSnapshot {
        self.store.snapshot()
    }

    pub fn public_functions_missing_examples(&self) -> Vec<FunctionMeta> {
        list_missing_examples(self.snapshot(), true)
    }

    pub fn all_functions(&self) -> Vec<FunctionMeta> {
        let mut out = self
            .snapshot()
            .files
            .values()
            .flat_map(|f| f.function_meta.iter())
            .cloned()
            .collect::<Vec<_>>();
        out.sort_by(|a, b| {
            (a.file.as_str(), a.function_name.as_str())
                .cmp(&(b.file.as_str(), b.function_name.as_str()))
        });
        out
    }

    pub fn effect_mismatch_findings(&self) -> Vec<vibe_indexer::EffectMismatch> {
        effect_mismatches(self.snapshot())
    }

    /// Read the source code for a given file path from disk.
    /// Returns `None` if the file cannot be read.
    pub fn read_source_file(&self, file_path: &str) -> Option<String> {
        std::fs::read_to_string(file_path).ok()
    }
}
