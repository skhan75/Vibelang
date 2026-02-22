use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::json;

use crate::session::LspSession;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
enum StdioCommand {
    Open {
        file: String,
        text: String,
        #[serde(default)]
        version: Option<i64>,
    },
    Change {
        file: String,
        text: String,
        #[serde(default)]
        version: Option<i64>,
    },
    Definition {
        file: String,
        line: usize,
        col: usize,
    },
    References {
        file: String,
        line: usize,
        col: usize,
    },
    Hover {
        file: String,
        line: usize,
        col: usize,
    },
    Diagnostics {
        file: String,
    },
    Completion {
        file: String,
        line: usize,
        col: usize,
    },
    DocumentSymbols {
        file: String,
    },
    WorkspaceSymbols {
        #[serde(default)]
        query: String,
    },
    Rename {
        file: String,
        line: usize,
        col: usize,
        new_name: String,
    },
    CodeActions {
        file: String,
    },
    Format {
        file: String,
    },
    Shutdown,
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
            StdioCommand::Open {
                file,
                text,
                version,
            } => match session.open_document(Path::new(&file), &text, version) {
                Ok(diags) => json!({ "ok": true, "diagnostics": diags }),
                Err(err) => json!({ "ok": false, "error": err }),
            },
            StdioCommand::Change {
                file,
                text,
                version,
            } => match session.change_document(Path::new(&file), &text, version) {
                Ok(diags) => json!({ "ok": true, "diagnostics": diags }),
                Err(err) => json!({ "ok": false, "error": err }),
            },
            StdioCommand::Definition { file, line, col } => {
                json!({ "ok": true, "result": session.definition(&file, line, col) })
            }
            StdioCommand::References { file, line, col } => {
                json!({ "ok": true, "result": session.references(&file, line, col) })
            }
            StdioCommand::Hover { file, line, col } => {
                json!({ "ok": true, "result": session.hover_contract_metadata(&file, line, col) })
            }
            StdioCommand::Diagnostics { file } => {
                json!({ "ok": true, "result": session.diagnostics_for_file(&file) })
            }
            StdioCommand::Completion { file, line, col } => {
                json!({ "ok": true, "result": session.completion(&file, line, col) })
            }
            StdioCommand::DocumentSymbols { file } => {
                json!({ "ok": true, "result": session.document_symbols(&file) })
            }
            StdioCommand::WorkspaceSymbols { query } => {
                json!({ "ok": true, "result": session.workspace_symbols(&query) })
            }
            StdioCommand::Rename {
                file,
                line,
                col,
                new_name,
            } => {
                json!({ "ok": true, "result": session.rename(&file, line, col, &new_name) })
            }
            StdioCommand::CodeActions { file } => {
                json!({ "ok": true, "result": session.code_actions(&file) })
            }
            StdioCommand::Format { file } => {
                json!({ "ok": true, "result": session.format_document(&file) })
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
