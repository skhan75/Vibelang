// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use vibe_indexer::{
    effect_mismatches, list_missing_examples, FunctionMeta, IndexSnapshot, IndexStore,
};

#[derive(Debug, Clone)]
pub struct ReadOnlyIndex {
    store: IndexStore,
}

impl ReadOnlyIndex {
    pub fn open(index_root: &Path) -> Result<Self, String> {
        let store = IndexStore::open_or_create(index_root)?;
        Ok(Self { store })
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
}
