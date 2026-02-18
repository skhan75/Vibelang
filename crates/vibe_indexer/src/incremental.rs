use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::Instant;

use crate::model::FileIndex;
use crate::store::IndexStore;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IncrementalTelemetry {
    pub files_reindexed: usize,
    pub invalidation_fanout: usize,
    pub incremental_update_latency_ms: u128,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

#[derive(Debug)]
pub struct IncrementalIndexer {
    store: IndexStore,
    reverse_deps: BTreeMap<String, BTreeSet<String>>,
    function_owner_file: BTreeMap<String, String>,
}

impl IncrementalIndexer {
    pub fn new(store: IndexStore) -> Self {
        let mut this = Self {
            store,
            reverse_deps: BTreeMap::new(),
            function_owner_file: BTreeMap::new(),
        };
        this.rebuild_dependency_graphs();
        this
    }

    pub fn store(&self) -> &IndexStore {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut IndexStore {
        &mut self.store
    }

    pub fn into_store(self) -> IndexStore {
        self.store
    }

    pub fn record_file_index(
        &mut self,
        file_index: FileIndex,
        telemetry: &mut IncrementalTelemetry,
    ) {
        let old_hash = self
            .store
            .snapshot()
            .files
            .get(&file_index.file)
            .map(|existing| existing.file_hash.clone());
        match old_hash {
            Some(hash) if hash == file_index.file_hash => {
                telemetry.cache_hits += 1;
            }
            _ => {
                telemetry.cache_misses += 1;
                self.store.upsert_file(file_index);
                telemetry.files_reindexed += 1;
                self.rebuild_dependency_graphs();
            }
        }
    }

    pub fn remove_file(&mut self, file_path: &str) {
        self.store.remove_file(file_path);
        self.rebuild_dependency_graphs();
    }

    pub fn affected_files_for_change(&self, changed_file: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(changed_file.to_string());
        while let Some(file) = queue.pop_front() {
            if !seen.insert(file.clone()) {
                continue;
            }
            out.push(file.clone());
            if let Some(dependents) = self.reverse_deps.get(&file) {
                for dependent in dependents {
                    queue.push_back(dependent.clone());
                }
            }
        }
        out.sort();
        out
    }

    pub fn update_changed_files_with_loader<F>(
        &mut self,
        changed_file: &str,
        mut loader: F,
    ) -> Result<IncrementalTelemetry, String>
    where
        F: FnMut(&str) -> Result<Option<FileIndex>, String>,
    {
        let start = Instant::now();
        let mut telemetry = IncrementalTelemetry::default();
        let affected = self.affected_files_for_change(changed_file);
        telemetry.invalidation_fanout = affected.len();
        for file in affected {
            match loader(&file)? {
                Some(file_index) => self.record_file_index(file_index, &mut telemetry),
                None => self.remove_file(&file),
            }
        }
        telemetry.incremental_update_latency_ms = start.elapsed().as_millis();
        Ok(telemetry)
    }

    fn rebuild_dependency_graphs(&mut self) {
        self.function_owner_file.clear();
        self.reverse_deps.clear();

        for (file, file_index) in &self.store.snapshot().files {
            for symbol in &file_index.symbols {
                if matches!(symbol.kind, crate::model::SymbolKind::Function) {
                    self.function_owner_file
                        .entry(symbol.name.clone())
                        .or_insert_with(|| file.clone());
                }
            }
        }

        for (file, file_index) in &self.store.snapshot().files {
            for dep_symbol in &file_index.dependencies {
                if let Some(owner_file) = self.function_owner_file.get(dep_symbol) {
                    if owner_file == file {
                        continue;
                    }
                    self.reverse_deps
                        .entry(owner_file.clone())
                        .or_default()
                        .insert(file.clone());
                }
            }
        }
    }
}
