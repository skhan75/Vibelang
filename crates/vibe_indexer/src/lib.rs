// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod extract;
pub mod incremental;
pub mod layout;
pub mod model;
pub mod queries;
pub mod store;
pub mod watcher;

use std::fs;
use std::path::{Path, PathBuf};

pub use extract::build_file_index;
pub use incremental::{IncrementalIndexer, IncrementalTelemetry};
pub use layout::{
    is_supported_source_ext, is_supported_source_file, legacy_metadata_root_for, metadata_root_for,
    LEGACY_METADATA_DIR, LEGACY_SOURCE_EXT, PRIMARY_METADATA_DIR, PRIMARY_SOURCE_EXT,
    SUPPORTED_SOURCE_EXTS,
};
pub use model::{
    EffectMismatch, FileIndex, FunctionMeta, IndexSnapshot, IndexSpan, IndexStats,
    IndexedDiagnostic, IndexedSeverity, Reference, Symbol, SymbolId, SymbolKind, INDEX_FILENAME,
    INDEX_SCHEMA_VERSION,
};
pub use queries::{
    definition_for_position, effect_mismatches, find_by_intent, find_references, find_symbol,
    list_missing_examples, references_for_position, symbol_at_position,
};
pub use store::IndexStore;
pub use watcher::{FileChange, FileChangeKind, FileWatcher};

pub fn default_metadata_root(target_path: &Path) -> PathBuf {
    metadata_root_for(&project_root_for_target(target_path))
}

pub fn legacy_metadata_root(target_path: &Path) -> PathBuf {
    legacy_metadata_root_for(&project_root_for_target(target_path))
}

pub fn default_index_root(target_path: &Path) -> PathBuf {
    default_metadata_root(target_path).join("index")
}

pub fn legacy_index_root(target_path: &Path) -> PathBuf {
    legacy_metadata_root(target_path).join("index")
}

pub fn prepare_index_root(target_path: &Path) -> Result<PathBuf, String> {
    let primary = default_index_root(target_path);
    let primary_index = primary.join(INDEX_FILENAME);
    if primary_index.exists() {
        return Ok(primary);
    }

    let legacy_index = legacy_index_root(target_path).join(INDEX_FILENAME);
    if !legacy_index.exists() {
        return Ok(primary);
    }

    fs::create_dir_all(&primary).map_err(|e| {
        format!(
            "failed to create new index root `{}`: {e}",
            primary.display()
        )
    })?;
    if !primary_index.exists() {
        fs::copy(&legacy_index, &primary_index).map_err(|e| {
            format!(
                "failed to migrate legacy index `{}` to `{}`: {e}",
                legacy_index.display(),
                primary_index.display()
            )
        })?;
    }
    Ok(primary)
}

fn project_root_for_target(target_path: &Path) -> PathBuf {
    if target_path.is_file() {
        target_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        target_path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use vibe_diagnostics::Diagnostic;
    use vibe_parser::parse_source;
    use vibe_types::check_and_lower;

    use crate::extract::build_file_index;
    use crate::incremental::{IncrementalIndexer, IncrementalTelemetry};
    use crate::model::INDEX_FILENAME;
    use crate::queries::{
        definition_for_position, effect_mismatches, find_by_intent, find_references, find_symbol,
        list_missing_examples, references_for_position,
    };
    use crate::store::IndexStore;
    use crate::watcher::{FileChangeKind, FileWatcher};
    use crate::{default_index_root, legacy_index_root, prepare_index_root};

    fn parse_index(path: &std::path::Path, source: &str) -> crate::FileIndex {
        let parsed = parse_source(source);
        let checked = check_and_lower(&parsed.ast);
        let mut diagnostics: Vec<Diagnostic> = parsed.diagnostics.into_sorted();
        diagnostics.extend(checked.diagnostics.into_sorted());
        build_file_index(path, source, &parsed.ast, &checked.hir, &diagnostics)
    }

    #[test]
    fn extraction_and_queries_are_deterministic() {
        let dir = tempdir().expect("temp dir");
        let file = dir.path().join("sample.vibe");
        let src = r#"pub alpha(x: Int) -> Int {
  @intent "alpha math"
  @effect alloc
  x
}

beta() -> Int {
  alpha(1)
}
"#;
        fs::write(&file, src).expect("write source");
        let file_index = parse_index(&file, src);

        let mut store =
            IndexStore::open_or_create(dir.path().join(".yb/index")).expect("open store");
        store.upsert_file(file_index.clone());
        let snapshot = store.snapshot();

        let symbols = find_symbol(snapshot, "alpha");
        assert_eq!(symbols.len(), 1);
        let intent = find_by_intent(snapshot, "alpha");
        assert_eq!(intent.len(), 1);
        let missing_examples = list_missing_examples(snapshot, true);
        assert_eq!(missing_examples.len(), 1);

        let symbols_again = find_symbol(snapshot, "alpha");
        assert_eq!(symbols, symbols_again);
    }

    #[test]
    fn references_and_definition_queries_work_by_position() {
        let dir = tempdir().expect("temp dir");
        let file = dir.path().join("nav.vibe");
        let src = r#"foo() -> Int { 1 }
bar() -> Int {
  foo()
}
"#;
        fs::write(&file, src).expect("write source");
        let file_index = parse_index(&file, src);
        let mut store =
            IndexStore::open_or_create(dir.path().join(".yb/index")).expect("open store");
        store.upsert_file(file_index.clone());
        let snapshot = store.snapshot();

        let foo_symbol = find_symbol(snapshot, "foo")
            .into_iter()
            .next()
            .expect("foo symbol should exist");
        let foo_refs = find_references(snapshot, foo_symbol.id);
        let call_ref = foo_refs
            .iter()
            .find(|r| r.span.line_start >= 3)
            .expect("foo call reference should exist");

        let definition = definition_for_position(
            snapshot,
            &file.to_string_lossy(),
            call_ref.span.line_start,
            call_ref.span.col_start,
        )
        .expect("definition for foo call");
        assert_eq!(definition.name, "foo");

        let refs = references_for_position(
            snapshot,
            &file.to_string_lossy(),
            call_ref.span.line_start,
            call_ref.span.col_start,
        );
        assert!(
            !refs.is_empty(),
            "references should include call-site usage"
        );
        let direct = find_references(snapshot, definition.id);
        assert_eq!(refs, direct);
    }

    #[test]
    fn effect_mismatch_query_reports_transparency_data() {
        let dir = tempdir().expect("temp dir");
        let file = dir.path().join("effects.vibe");
        let src = r#"noisy() -> Int {
  println("x")
  0
}
"#;
        fs::write(&file, src).expect("write source");
        let file_index = parse_index(&file, src);
        let mut store =
            IndexStore::open_or_create(dir.path().join(".yb/index")).expect("open store");
        store.upsert_file(file_index);
        let mismatches = effect_mismatches(store.snapshot());
        assert_eq!(mismatches.len(), 1, "expected one effect mismatch entry");
        assert_eq!(mismatches[0].function_name, "noisy");
        assert!(
            mismatches[0].observed_only.iter().any(|e| e == "io"),
            "observed-only effects should include io"
        );
    }

    #[test]
    fn store_roundtrip_and_schema_file_written() {
        let dir = tempdir().expect("temp dir");
        let file = dir.path().join("sample.vibe");
        let src = "main() -> Int { 0 }";
        fs::write(&file, src).expect("write source");
        let file_index = parse_index(&file, src);

        let root = dir.path().join(".yb/index");
        let mut store = IndexStore::open_or_create(&root).expect("open store");
        store.upsert_file(file_index);
        store.save().expect("save store");
        assert!(
            root.join(INDEX_FILENAME).exists(),
            "index snapshot should be persisted"
        );

        let reopened = IndexStore::open_or_create(&root).expect("reopen store");
        assert_eq!(reopened.snapshot().files.len(), 1);
    }

    #[test]
    fn corrupted_index_recovers_to_empty_snapshot() {
        let dir = tempdir().expect("temp dir");
        let root = dir.path().join(".yb/index");
        fs::create_dir_all(&root).expect("create index dir");
        fs::write(root.join(INDEX_FILENAME), "{not-valid-json").expect("write invalid json");

        let store = IndexStore::open_or_create(&root).expect("open recovered store");
        assert!(store.recovered_from_corruption());
        assert!(store.snapshot().files.is_empty());
    }

    #[test]
    fn prepare_index_root_bootstraps_from_legacy_snapshot() {
        let dir = tempdir().expect("temp dir");
        let legacy_root = legacy_index_root(dir.path());
        fs::create_dir_all(&legacy_root).expect("create legacy root");
        let legacy_snapshot = legacy_root.join(INDEX_FILENAME);
        fs::write(&legacy_snapshot, "legacy-index-data").expect("write legacy index");

        let primary_root = prepare_index_root(dir.path()).expect("prepare root");
        let primary_snapshot = primary_root.join(INDEX_FILENAME);
        assert_eq!(primary_root, default_index_root(dir.path()));
        assert!(
            primary_snapshot.exists(),
            "primary index should be bootstrapped"
        );
        assert_eq!(
            fs::read_to_string(&primary_snapshot).expect("read primary index"),
            "legacy-index-data"
        );

        fs::write(&primary_snapshot, "primary-wins").expect("write primary index");
        let prepared_again = prepare_index_root(dir.path()).expect("prepare root again");
        assert_eq!(prepared_again, primary_root);
        assert_eq!(
            fs::read_to_string(&primary_snapshot).expect("read primary index"),
            "primary-wins"
        );
    }

    #[test]
    fn incremental_reverse_dependency_fanout() {
        let dir = tempdir().expect("temp dir");
        let file_a = dir.path().join("a.vibe");
        let src_a = r#"foo() -> Int { 1 }"#;
        fs::write(&file_a, src_a).expect("write a");
        let file_b = dir.path().join("b.vibe");
        let src_b = r#"bar() -> Int { foo() }"#;
        fs::write(&file_b, src_b).expect("write b");

        let root = dir.path().join(".yb/index");
        let store = IndexStore::open_or_create(root).expect("open store");
        let mut incremental = IncrementalIndexer::new(store);
        let mut telemetry = IncrementalTelemetry::default();
        incremental.record_file_index(parse_index(&file_a, src_a), &mut telemetry);
        incremental.record_file_index(parse_index(&file_b, src_b), &mut telemetry);

        let affected = incremental.affected_files_for_change(&file_a.to_string_lossy());
        assert!(affected
            .iter()
            .any(|f| f == &file_a.to_string_lossy().to_string()));
        assert!(affected
            .iter()
            .any(|f| f == &file_b.to_string_lossy().to_string()));
    }

    #[test]
    fn incremental_loader_updates_changed_and_dependents_only() {
        let dir = tempdir().expect("temp dir");
        let file_a = dir.path().join("a.vibe");
        let src_a = r#"foo() -> Int { 1 }"#;
        fs::write(&file_a, src_a).expect("write a");
        let file_b = dir.path().join("b.vibe");
        let src_b = r#"bar() -> Int { foo() }"#;
        fs::write(&file_b, src_b).expect("write b");
        let file_c = dir.path().join("c.vibe");
        let src_c = r#"baz() -> Int { 7 }"#;
        fs::write(&file_c, src_c).expect("write c");

        let root = dir.path().join(".yb/index");
        let store = IndexStore::open_or_create(root).expect("open store");
        let mut incremental = IncrementalIndexer::new(store);
        let mut telemetry = IncrementalTelemetry::default();
        incremental.record_file_index(parse_index(&file_a, src_a), &mut telemetry);
        incremental.record_file_index(parse_index(&file_b, src_b), &mut telemetry);
        incremental.record_file_index(parse_index(&file_c, src_c), &mut telemetry);

        let changed = file_a.to_string_lossy().to_string();
        let mut loaded = Vec::<String>::new();
        let report = incremental
            .update_changed_files_with_loader(&changed, |file_path| {
                loaded.push(file_path.to_string());
                let path = std::path::PathBuf::from(file_path);
                let source = fs::read_to_string(&path)
                    .map_err(|e| format!("failed to read `{}`: {e}", path.display()))?;
                Ok(Some(parse_index(&path, &source)))
            })
            .expect("incremental update with loader");

        assert!(
            loaded
                .iter()
                .any(|p| p == &file_a.to_string_lossy().to_string()),
            "changed file should be reloaded"
        );
        assert!(
            loaded
                .iter()
                .any(|p| p == &file_b.to_string_lossy().to_string()),
            "reverse dependent file should be reloaded"
        );
        assert!(
            !loaded
                .iter()
                .any(|p| p == &file_c.to_string_lossy().to_string()),
            "unrelated file should not be reloaded"
        );
        assert!(report.invalidation_fanout >= 2);
    }

    #[test]
    fn file_watcher_detects_added_modified_and_removed_files() {
        let dir = tempdir().expect("temp dir");
        let root = dir.path();
        let file = root.join("watch.yb");
        fs::write(&file, "main() -> Int { 0 }").expect("write initial file");

        let mut watcher = FileWatcher::new();
        watcher
            .prime_from_paths(&[root.to_path_buf()])
            .expect("prime watcher");

        fs::write(&file, "main() -> Int { 1 }").expect("modify file");
        let modified = watcher.scan(&[root.to_path_buf()]).expect("scan modified");
        assert!(modified
            .iter()
            .any(|change| change.kind == FileChangeKind::Modified));

        fs::remove_file(&file).expect("remove watched file");
        let removed = watcher.scan(&[root.to_path_buf()]).expect("scan removed");
        assert!(removed
            .iter()
            .any(|change| change.kind == FileChangeKind::Removed));

        fs::write(&file, "main() -> Int { 2 }").expect("add file back");
        let added = watcher.scan(&[root.to_path_buf()]).expect("scan added");
        assert!(added
            .iter()
            .any(|change| change.kind == FileChangeKind::Added));
    }
}
