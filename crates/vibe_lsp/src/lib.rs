use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::json;
use vibe_diagnostics::Diagnostic;
use vibe_indexer::extract::build_file_index;
use vibe_indexer::incremental::IncrementalTelemetry;
use vibe_indexer::model::{IndexSpan, IndexedDiagnostic};
use vibe_indexer::queries::{definition_for_position, references_for_position, symbol_at_position};
use vibe_indexer::{IndexStore, IncrementalIndexer};
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

#[derive(Debug)]
pub struct LspSession {
    indexer: IncrementalIndexer,
}

impl LspSession {
    pub fn open_or_create(index_root: impl Into<PathBuf>) -> Result<Self, String> {
        let store = IndexStore::open_or_create(index_root)?;
        Ok(Self {
            indexer: IncrementalIndexer::new(store),
        })
    }

    pub fn open_document(
        &mut self,
        file_path: &Path,
        source: &str,
    ) -> Result<Vec<IndexedDiagnostic>, String> {
        let normalized = normalize_path(file_path);
        let path_buf = PathBuf::from(&normalized);
        let parsed = parse_source(source);
        let checked = check_and_lower(&parsed.ast);

        let mut all_diagnostics: Vec<Diagnostic> = parsed.diagnostics.into_sorted();
        all_diagnostics.extend(checked.diagnostics.into_sorted());
        let file_index = build_file_index(&path_buf, source, &parsed.ast, &checked.hir, &all_diagnostics);
        let diagnostics = file_index.diagnostics.clone();
        let mut telemetry = IncrementalTelemetry::default();
        self.indexer.record_file_index(file_index, &mut telemetry);
        self.indexer.store().save()?;
        Ok(diagnostics)
    }

    pub fn change_document(
        &mut self,
        file_path: &Path,
        source: &str,
    ) -> Result<Vec<IndexedDiagnostic>, String> {
        self.open_document(file_path, source)
    }

    pub fn definition(&self, file: &str, line: usize, col: usize) -> Option<LspLocation> {
        let normalized = normalize_path(Path::new(file));
        let symbol = definition_for_position(self.indexer.store().snapshot(), &normalized, line, col)?;
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
        let symbol_id = symbol_at_position(self.indexer.store().snapshot(), &normalized, line, col)?;
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

    pub fn save(&self) -> Result<(), String> {
        self.indexer.store().save()
    }
}

pub fn run_line_stdio(index_root: impl Into<PathBuf>) -> Result<(), String> {
    let mut session = LspSession::open_or_create(index_root)?;
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| format!("failed to read stdin line: {e}"))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let command = serde_json::from_str::<StdioCommand>(trimmed)
            .map_err(|e| format!("invalid lsp command json: {e}"))?;
        let response = match command {
            StdioCommand::Open { file, text } => match session.open_document(Path::new(&file), &text) {
                Ok(diags) => json!({ "ok": true, "diagnostics": diags }),
                Err(err) => json!({ "ok": false, "error": err }),
            },
            StdioCommand::Change { file, text } => {
                match session.change_document(Path::new(&file), &text) {
                    Ok(diags) => json!({ "ok": true, "diagnostics": diags }),
                    Err(err) => json!({ "ok": false, "error": err }),
                }
            }
            StdioCommand::Definition { file, line, col } => {
                let result = session.definition(&file, line, col);
                json!({ "ok": true, "result": result })
            }
            StdioCommand::References { file, line, col } => {
                let result = session.references(&file, line, col);
                json!({ "ok": true, "result": result })
            }
            StdioCommand::Hover { file, line, col } => {
                let result = session.hover_contract_metadata(&file, line, col);
                json!({ "ok": true, "result": result })
            }
            StdioCommand::Diagnostics { file } => {
                let result = session.diagnostics_for_file(&file);
                json!({ "ok": true, "result": result })
            }
            StdioCommand::Shutdown => {
                let _ = session.save();
                let response = json!({ "ok": true, "result": "shutdown" });
                writeln!(stdout, "{response}")
                    .map_err(|e| format!("failed to write lsp response: {e}"))?;
                stdout
                    .flush()
                    .map_err(|e| format!("failed to flush lsp response: {e}"))?;
                break;
            }
        };
        writeln!(stdout, "{response}").map_err(|e| format!("failed to write lsp response: {e}"))?;
        stdout
            .flush()
            .map_err(|e| format!("failed to flush lsp response: {e}"))?;
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
enum StdioCommand {
    Open { file: String, text: String },
    Change { file: String, text: String },
    Definition { file: String, line: usize, col: usize },
    References { file: String, line: usize, col: usize },
    Hover { file: String, line: usize, col: usize },
    Diagnostics { file: String },
    Shutdown,
}

fn normalize_path(path: &Path) -> String {
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
        let root = dir.path().join(".vibe/index");
        let file = dir.path().join("broken.vibe");
        let src = "broken() -> Int { unknown_name }";
        fs::write(&file, src).expect("write source");

        let mut session = LspSession::open_or_create(root).expect("open lsp session");
        let diagnostics = session
            .open_document(&file, src)
            .expect("open document with diagnostics");
        assert!(
            diagnostics.iter().any(|d| d.code == "E2001"),
            "expected unknown identifier diagnostic"
        );
    }

    #[test]
    fn definition_and_references_are_available() {
        let dir = tempdir().expect("temp dir");
        let root = dir.path().join(".vibe/index");
        let file = dir.path().join("nav.vibe");
        let src = r#"foo() -> Int { 1 }
bar() -> Int { foo() }
"#;
        fs::write(&file, src).expect("write source");

        let mut session = LspSession::open_or_create(root).expect("open lsp session");
        let _ = session.open_document(&file, src).expect("open document");
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
        let root = dir.path().join(".vibe/index");
        let file = dir.path().join("hover.vibe");
        let src = r#"foo() -> Int {
  @intent "compute a deterministic value"
  @effect alloc
  1
}
bar() -> Int { foo() }
"#;
        fs::write(&file, src).expect("write source");

        let mut session = LspSession::open_or_create(root).expect("open lsp session");
        let _ = session.open_document(&file, src).expect("open document");
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
}
