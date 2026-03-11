// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use vibe_diagnostics::Diagnostic;
use vibe_fmt::format_source;
use vibe_indexer::extract::build_file_index;
use vibe_indexer::incremental::IncrementalTelemetry;
use vibe_indexer::model::{IndexSpan, IndexedDiagnostic, SymbolKind};
use vibe_indexer::queries::{
    definition_for_position, find_references, references_for_position, symbol_at_position,
};
use vibe_indexer::{IncrementalIndexer, IndexStore};
use vibe_parser::parse_source;
use vibe_types::check_and_lower;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LspLocation {
    pub file: String,
    pub span: IndexSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub function_name: String,
    pub file: String,
    pub intent_text: Option<String>,
    pub effects_declared: Vec<String>,
    pub effects_observed: Vec<String>,
    pub has_examples: bool,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionEntry {
    pub label: String,
    pub kind: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSymbolEntry {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub span: IndexSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSymbolEntry {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub span: IndexSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenameEdit {
    pub file: String,
    pub span: IndexSpan,
    pub new_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenameResult {
    pub edits: Vec<RenameEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeActionSuggestion {
    pub title: String,
    pub kind: String,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OpenDocument {
    text: String,
    version: i64,
}

#[derive(Debug)]
pub struct LspSession {
    indexer: IncrementalIndexer,
    open_docs: BTreeMap<String, OpenDocument>,
}

impl LspSession {
    pub fn open_or_create(index_root: impl Into<PathBuf>) -> Result<Self, String> {
        let store = IndexStore::open_or_create(index_root)?;
        Ok(Self {
            indexer: IncrementalIndexer::new(store),
            open_docs: BTreeMap::new(),
        })
    }

    pub fn open_document(
        &mut self,
        file_path: &Path,
        source: &str,
        version: Option<i64>,
    ) -> Result<Vec<IndexedDiagnostic>, String> {
        let normalized = normalize_path(file_path);
        let path_buf = PathBuf::from(&normalized);
        let parsed = parse_source(source);
        let checked = check_and_lower(&parsed.ast);

        let mut all_diagnostics: Vec<Diagnostic> = parsed.diagnostics.into_sorted();
        all_diagnostics.extend(checked.diagnostics.into_sorted());
        let file_index = build_file_index(
            &path_buf,
            source,
            &parsed.ast,
            &checked.hir,
            &all_diagnostics,
        );
        let diagnostics = file_index.diagnostics.clone();
        let mut telemetry = IncrementalTelemetry::default();
        self.indexer.record_file_index(file_index, &mut telemetry);
        self.indexer.store().save()?;
        self.open_docs.insert(
            normalized,
            OpenDocument {
                text: source.to_string(),
                version: version.unwrap_or(0),
            },
        );
        Ok(diagnostics)
    }

    pub fn change_document(
        &mut self,
        file_path: &Path,
        source: &str,
        version: Option<i64>,
    ) -> Result<Vec<IndexedDiagnostic>, String> {
        self.open_document(file_path, source, version)
    }

    pub fn close_document(&mut self, file_path: &Path) {
        let normalized = normalize_path(file_path);
        self.open_docs.remove(&normalized);
    }

    pub fn definition(&self, file: &str, line: usize, col: usize) -> Option<LspLocation> {
        let normalized = normalize_path(Path::new(file));
        let symbol =
            definition_for_position(self.indexer.store().snapshot(), &normalized, line, col)?;
        Some(LspLocation {
            file: symbol.file,
            span: symbol.span,
        })
    }

    pub fn references(&self, file: &str, line: usize, col: usize) -> Vec<LspLocation> {
        let normalized = normalize_path(Path::new(file));
        references_for_position(self.indexer.store().snapshot(), &normalized, line, col)
            .into_iter()
            .map(|reference| LspLocation {
                file: reference.file,
                span: reference.span,
            })
            .collect()
    }

    pub fn diagnostics_for_file(&self, file: &str) -> Vec<IndexedDiagnostic> {
        let normalized = normalize_path(Path::new(file));
        self.indexer
            .store()
            .snapshot()
            .files
            .get(&normalized)
            .map(|entry| entry.diagnostics.clone())
            .unwrap_or_default()
    }

    pub fn hover_contract_metadata(
        &self,
        file: &str,
        line: usize,
        col: usize,
    ) -> Option<ContractMetadata> {
        let normalized = normalize_path(Path::new(file));
        let symbol_id =
            symbol_at_position(self.indexer.store().snapshot(), &normalized, line, col)?;
        self.indexer
            .store()
            .snapshot()
            .files
            .values()
            .flat_map(|entry| entry.function_meta.iter())
            .find(|meta| meta.symbol_id == symbol_id)
            .map(|meta| ContractMetadata {
                function_name: meta.function_name.clone(),
                file: meta.file.clone(),
                intent_text: meta.intent_text.clone(),
                effects_declared: meta.effects_declared.clone(),
                effects_observed: meta.effects_observed.clone(),
                has_examples: meta.has_examples,
                is_public: meta.is_public,
            })
    }

    pub fn completion(&self, file: &str, line: usize, col: usize) -> Vec<CompletionEntry> {
        let normalized = normalize_path(Path::new(file));
        let prefix = self
            .open_docs
            .get(&normalized)
            .and_then(|doc| identifier_prefix_at_position(&doc.text, line, col))
            .unwrap_or_default();

        let mut entries = BTreeMap::<String, CompletionEntry>::new();
        for keyword in [
            "module", "import", "type", "pub", "async", "mut", "const", "return", "if", "else",
            "for", "in", "while", "repeat", "match", "case", "default", "select", "after",
            "closed", "go", "thread", "break", "continue", "await", "none", "true", "false",
        ] {
            maybe_insert_completion(
                &mut entries,
                &prefix,
                keyword,
                "keyword",
                Some("language keyword"),
            );
        }
        for ty in [
            "Int", "Float", "Bool", "Str", "Char", "List", "Map", "Chan", "Result", "Option",
        ] {
            maybe_insert_completion(&mut entries, &prefix, ty, "type", Some("builtin type"));
        }
        for symbol in self
            .indexer
            .store()
            .snapshot()
            .files
            .values()
            .flat_map(|entry| entry.symbols.iter())
        {
            let kind = match symbol.kind {
                SymbolKind::Function => "function",
                SymbolKind::Param | SymbolKind::Local => "variable",
                SymbolKind::Contract => "contract",
                SymbolKind::Effect => "effect",
            };
            maybe_insert_completion(
                &mut entries,
                &prefix,
                &symbol.name,
                kind,
                Some("indexed symbol"),
            );
        }
        entries.into_values().collect()
    }

    pub fn document_symbols(&self, file: &str) -> Vec<DocumentSymbolEntry> {
        let normalized = normalize_path(Path::new(file));
        let Some(file_index) = self.indexer.store().snapshot().files.get(&normalized) else {
            return Vec::new();
        };
        let mut out = file_index
            .symbols
            .iter()
            .map(|symbol| DocumentSymbolEntry {
                name: symbol.name.clone(),
                kind: symbol_kind_name(symbol.kind).to_string(),
                file: symbol.file.clone(),
                span: symbol.span,
            })
            .collect::<Vec<_>>();
        out.sort_by(|a, b| {
            (a.span.line_start, a.span.col_start, a.name.as_str()).cmp(&(
                b.span.line_start,
                b.span.col_start,
                b.name.as_str(),
            ))
        });
        out
    }

    pub fn workspace_symbols(&self, query: &str) -> Vec<WorkspaceSymbolEntry> {
        let query_lower = query.to_lowercase();
        let mut out = self
            .indexer
            .store()
            .snapshot()
            .files
            .values()
            .flat_map(|entry| entry.symbols.iter())
            .filter(|symbol| {
                query_lower.is_empty() || symbol.name.to_lowercase().contains(&query_lower)
            })
            .map(|symbol| WorkspaceSymbolEntry {
                name: symbol.name.clone(),
                kind: symbol_kind_name(symbol.kind).to_string(),
                file: symbol.file.clone(),
                span: symbol.span,
            })
            .collect::<Vec<_>>();
        out.sort_by(|a, b| {
            (
                a.name.as_str(),
                a.file.as_str(),
                a.span.line_start,
                a.span.col_start,
            )
                .cmp(&(
                    b.name.as_str(),
                    b.file.as_str(),
                    b.span.line_start,
                    b.span.col_start,
                ))
        });
        out
    }

    pub fn rename(
        &self,
        file: &str,
        line: usize,
        col: usize,
        new_name: &str,
    ) -> Option<RenameResult> {
        if !is_valid_identifier(new_name) {
            return None;
        }
        let normalized = normalize_path(Path::new(file));
        let snapshot = self.indexer.store().snapshot();
        let symbol_id = symbol_at_position(snapshot, &normalized, line, col)?;

        let mut edits = Vec::<RenameEdit>::new();
        let mut seen = BTreeSet::<(String, usize, usize, usize, usize)>::new();

        if let Some(definition) = snapshot
            .files
            .values()
            .flat_map(|entry| entry.symbols.iter())
            .find(|symbol| symbol.id == symbol_id)
        {
            let key = (
                definition.file.clone(),
                definition.span.line_start,
                definition.span.col_start,
                definition.span.line_end,
                definition.span.col_end,
            );
            if seen.insert(key) {
                edits.push(RenameEdit {
                    file: definition.file.clone(),
                    span: definition.span,
                    new_text: new_name.to_string(),
                });
            }
        }

        for reference in find_references(snapshot, symbol_id) {
            let key = (
                reference.file.clone(),
                reference.span.line_start,
                reference.span.col_start,
                reference.span.line_end,
                reference.span.col_end,
            );
            if seen.insert(key) {
                edits.push(RenameEdit {
                    file: reference.file,
                    span: reference.span,
                    new_text: new_name.to_string(),
                });
            }
        }

        if edits.is_empty() {
            return None;
        }
        edits.sort_by(|a, b| {
            (
                a.file.as_str(),
                a.span.line_start,
                a.span.col_start,
                a.span.line_end,
                a.span.col_end,
            )
                .cmp(&(
                    b.file.as_str(),
                    b.span.line_start,
                    b.span.col_start,
                    b.span.line_end,
                    b.span.col_end,
                ))
        });
        Some(RenameResult { edits })
    }

    pub fn code_actions(&self, file: &str) -> Vec<CodeActionSuggestion> {
        let diagnostics = self.diagnostics_for_file(file);
        let mut out = Vec::<CodeActionSuggestion>::new();
        let mut seen = BTreeSet::<(String, String)>::new();
        for diagnostic in diagnostics {
            let title = match diagnostic.code.as_str() {
                "E2001" => "Create local binding for unknown identifier",
                "E1001" => "Check syntax and close unmatched delimiters",
                _ => "Review diagnostic and apply deterministic fix",
            };
            let key = (title.to_string(), diagnostic.code.clone());
            if seen.insert(key.clone()) {
                out.push(CodeActionSuggestion {
                    title: key.0,
                    kind: "quickfix".to_string(),
                    code: key.1,
                });
            }
        }
        out
    }

    pub fn format_document(&self, file: &str) -> Option<String> {
        let normalized = normalize_path(Path::new(file));
        self.open_docs
            .get(&normalized)
            .map(|doc| format_source(&doc.text))
    }

    pub fn save(&self) -> Result<(), String> {
        self.indexer.store().save()
    }
}

fn maybe_insert_completion(
    out: &mut BTreeMap<String, CompletionEntry>,
    prefix: &str,
    label: &str,
    kind: &str,
    detail: Option<&str>,
) {
    if !prefix.is_empty() && !label.starts_with(prefix) {
        return;
    }
    out.entry(label.to_string())
        .or_insert_with(|| CompletionEntry {
            label: label.to_string(),
            kind: kind.to_string(),
            detail: detail.map(|value| value.to_string()),
        });
}

fn identifier_prefix_at_position(source: &str, line: usize, col: usize) -> Option<String> {
    let index = line_col_to_index(source, line, col)?;
    let mut start = index;
    let bytes = source.as_bytes();
    while start > 0 {
        let c = bytes[start - 1] as char;
        if c.is_ascii_alphanumeric() || c == '_' {
            start -= 1;
        } else {
            break;
        }
    }
    Some(source[start..index].to_string())
}

fn line_col_to_index(source: &str, line: usize, col: usize) -> Option<usize> {
    if line == 0 || col == 0 {
        return None;
    }
    let mut current_line = 1usize;
    let mut line_start_idx = 0usize;
    for segment in source.split_inclusive('\n') {
        if current_line == line {
            let line_no_newline = segment.strip_suffix('\n').unwrap_or(segment);
            let col_idx = col.saturating_sub(1).min(line_no_newline.chars().count());
            return Some(line_start_idx + col_idx);
        }
        line_start_idx += segment.len();
        current_line += 1;
    }
    if current_line == line {
        let remaining = &source[line_start_idx..];
        let col_idx = col.saturating_sub(1).min(remaining.chars().count());
        return Some(line_start_idx + col_idx);
    }
    None
}

fn symbol_kind_name(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Function => "function",
        SymbolKind::Param => "param",
        SymbolKind::Local => "local",
        SymbolKind::Contract => "contract",
        SymbolKind::Effect => "effect",
    }
}

fn is_valid_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn normalize_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::LspSession;

    #[test]
    fn open_document_publishes_diagnostics() {
        let dir = tempdir().expect("temp dir");
        let root = dir.path().join(".yb/index");
        let file = dir.path().join("broken.yb");
        let src = "broken() -> Int { unknown_name }";
        fs::write(&file, src).expect("write source");

        let mut session = LspSession::open_or_create(root).expect("open lsp session");
        let diagnostics = session
            .open_document(&file, src, Some(1))
            .expect("open document with diagnostics");
        assert!(
            diagnostics.iter().any(|d| d.code == "E2001"),
            "expected unknown identifier diagnostic"
        );
    }

    #[test]
    fn definition_and_references_are_available() {
        let dir = tempdir().expect("temp dir");
        let root = dir.path().join(".yb/index");
        let file = dir.path().join("nav.yb");
        let src = r#"foo() -> Int { 1 }
bar() -> Int { foo() }
"#;
        fs::write(&file, src).expect("write source");

        let mut session = LspSession::open_or_create(root).expect("open lsp session");
        let _ = session
            .open_document(&file, src, Some(1))
            .expect("open document");
        let definition = session
            .definition(&file.to_string_lossy(), 2, 17)
            .expect("definition should resolve");
        assert_eq!(definition.span.line_start, 1);

        let refs = session.references(&file.to_string_lossy(), 2, 17);
        assert!(!refs.is_empty(), "references should resolve from callsite");
    }

    #[test]
    fn hover_surfaces_intent_and_effects() {
        let dir = tempdir().expect("temp dir");
        let root = dir.path().join(".yb/index");
        let file = dir.path().join("hover.yb");
        let src = r#"foo() -> Int {
  @intent "compute a deterministic value"
  @effect alloc
  1
}
bar() -> Int { foo() }
"#;
        fs::write(&file, src).expect("write source");

        let mut session = LspSession::open_or_create(root).expect("open lsp session");
        let _ = session
            .open_document(&file, src, Some(1))
            .expect("open document");
        let meta = session
            .hover_contract_metadata(&file.to_string_lossy(), 6, 17)
            .expect("hover metadata should resolve");
        assert_eq!(meta.function_name, "foo");
        assert!(
            meta.intent_text
                .as_ref()
                .is_some_and(|text| text.contains("deterministic")),
            "intent metadata should be surfaced"
        );
        assert!(
            meta.effects_declared.iter().any(|e| e == "alloc"),
            "declared effects should be surfaced"
        );
    }

    #[test]
    fn completion_includes_keywords_and_symbols() {
        let dir = tempdir().expect("temp dir");
        let root = dir.path().join(".yb/index");
        let file = dir.path().join("complete.yb");
        let src = r#"foo() -> Int { 1 }
bar() -> Int { fo }
"#;
        fs::write(&file, src).expect("write source");

        let mut session = LspSession::open_or_create(root).expect("open lsp session");
        let _ = session
            .open_document(&file, src, Some(1))
            .expect("open document");
        let completion = session.completion(&file.to_string_lossy(), 2, 17);
        assert!(
            completion.iter().any(|item| item.label == "foo"),
            "completion should include indexed function name"
        );
        assert!(
            completion.iter().any(|item| item.label == "for"),
            "completion should include language keywords"
        );
    }

    #[test]
    fn format_document_matches_vibe_fmt_behavior() {
        let dir = tempdir().expect("temp dir");
        let root = dir.path().join(".yb/index");
        let file = dir.path().join("format.yb");
        let src = "main() -> Int {\n\t1    \n\n\n  0\n}\n";
        fs::write(&file, src).expect("write source");

        let mut session = LspSession::open_or_create(root).expect("open lsp session");
        let _ = session
            .open_document(&file, src, Some(1))
            .expect("open document");
        let formatted = session
            .format_document(&file.to_string_lossy())
            .expect("formatted output");
        assert_eq!(formatted, "main() -> Int {\n  1\n\n  0\n}\n");
    }
}
