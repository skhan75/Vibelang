use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{json, Value};
use vibe_indexer::model::{IndexSpan, IndexedDiagnostic, IndexedSeverity};

use crate::capabilities::initialize_result;
use crate::session::{
    CodeActionSuggestion, CompletionEntry, DocumentSymbolEntry, LspLocation, LspSession,
    WorkspaceSymbolEntry,
};

#[derive(Debug, Default)]
pub struct ServerState {
    pub initialized: bool,
    pub shutdown_requested: bool,
    pub should_exit: bool,
}

#[derive(Debug)]
pub struct RequestOutcome {
    pub result: Value,
    pub notifications: Vec<Value>,
}

pub fn handle_request(
    method: &str,
    params: &Value,
    session: &mut LspSession,
    state: &mut ServerState,
) -> Result<RequestOutcome, String> {
    let mut notifications = Vec::<Value>::new();
    let result = match method {
        "initialize" => initialize_result(),
        "shutdown" => {
            state.shutdown_requested = true;
            json!(null)
        }
        "textDocument/definition" => {
            let (file, line, col) = extract_file_and_position(params)?;
            let location = session.definition(&file, line, col);
            match location {
                Some(value) => lsp_location_value(&value),
                None => json!(null),
            }
        }
        "textDocument/references" => {
            let (file, line, col) = extract_file_and_position(params)?;
            let values = session.references(&file, line, col);
            Value::Array(values.iter().map(lsp_location_value).collect())
        }
        "textDocument/hover" => {
            let (file, line, col) = extract_file_and_position(params)?;
            if let Some(meta) = session.hover_contract_metadata(&file, line, col) {
                let mut markdown = format!("### `{}`\n\n", meta.function_name);
                if let Some(intent) = meta.intent_text {
                    markdown.push_str(&format!("**Intent:** {}\n\n", intent));
                }
                markdown.push_str(&format!(
                    "**Effects declared:** {}\n\n",
                    if meta.effects_declared.is_empty() {
                        "none".to_string()
                    } else {
                        meta.effects_declared.join(", ")
                    }
                ));
                markdown.push_str(&format!(
                    "**Effects observed:** {}\n\n",
                    if meta.effects_observed.is_empty() {
                        "none".to_string()
                    } else {
                        meta.effects_observed.join(", ")
                    }
                ));
                markdown.push_str(&format!(
                    "**Examples:** {}\n",
                    if meta.has_examples { "yes" } else { "no" }
                ));
                json!({
                    "contents": {
                        "kind": "markdown",
                        "value": markdown
                    }
                })
            } else {
                json!(null)
            }
        }
        "textDocument/completion" => {
            let (file, line, col) = extract_file_and_position(params)?;
            let items = session.completion(&file, line, col);
            json!({
                "isIncomplete": false,
                "items": items.into_iter().map(completion_entry_value).collect::<Vec<_>>()
            })
        }
        "textDocument/documentSymbol" => {
            let file = extract_file_from_text_document(params)?;
            let symbols = session.document_symbols(&file);
            Value::Array(symbols.iter().map(document_symbol_value).collect())
        }
        "workspace/symbol" => {
            let query = params
                .get("query")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let symbols = session.workspace_symbols(query);
            Value::Array(symbols.iter().map(workspace_symbol_value).collect())
        }
        "textDocument/rename" => {
            let (file, line, col) = extract_file_and_position(params)?;
            let new_name = params
                .get("newName")
                .and_then(Value::as_str)
                .ok_or_else(|| "rename request missing newName".to_string())?;
            match session.rename(&file, line, col, new_name) {
                Some(result) => {
                    let mut changes = BTreeMap::<String, Vec<Value>>::new();
                    for edit in result.edits {
                        let uri = path_to_uri(&edit.file);
                        changes.entry(uri).or_default().push(json!({
                            "range": index_span_to_lsp_range(edit.span),
                            "newText": edit.new_text
                        }));
                    }
                    json!({ "changes": changes })
                }
                None => json!(null),
            }
        }
        "textDocument/codeAction" => {
            let file = extract_file_from_text_document(params)?;
            let actions = session.code_actions(&file);
            Value::Array(actions.iter().map(code_action_value).collect())
        }
        "textDocument/formatting" | "textDocument/rangeFormatting" => {
            let file = extract_file_from_text_document(params)?;
            if let Some(formatted) = session.format_document(&file) {
                let range = full_document_range(&formatted);
                json!([
                    {
                        "range": range,
                        "newText": formatted
                    }
                ])
            } else {
                json!([])
            }
        }
        other => {
            return Err(format!("unsupported LSP request method `{other}`"));
        }
    };

    if method == "initialize" {
        state.initialized = true;
    }

    if method == "textDocument/definition"
        || method == "textDocument/references"
        || method == "textDocument/hover"
        || method == "textDocument/completion"
        || method == "textDocument/documentSymbol"
        || method == "workspace/symbol"
        || method == "textDocument/rename"
        || method == "textDocument/codeAction"
        || method == "textDocument/formatting"
        || method == "textDocument/rangeFormatting"
    {
        let file = if method == "workspace/symbol" {
            None
        } else if method == "textDocument/definition"
            || method == "textDocument/references"
            || method == "textDocument/hover"
            || method == "textDocument/completion"
            || method == "textDocument/rename"
        {
            Some(extract_file_and_position(params)?.0)
        } else {
            Some(extract_file_from_text_document(params)?)
        };
        if let Some(file) = file {
            let diagnostics = session.diagnostics_for_file(&file);
            notifications.push(publish_diagnostics_notification(&file, diagnostics));
        }
    }

    Ok(RequestOutcome {
        result,
        notifications,
    })
}

pub fn handle_notification(
    method: &str,
    params: &Value,
    session: &mut LspSession,
    state: &mut ServerState,
) -> Result<Vec<Value>, String> {
    match method {
        "initialized" => {
            state.initialized = true;
            Ok(Vec::new())
        }
        "exit" => {
            state.should_exit = true;
            Ok(Vec::new())
        }
        "textDocument/didOpen" => {
            let text_document = params
                .get("textDocument")
                .ok_or_else(|| "didOpen missing textDocument".to_string())?;
            let uri = text_document
                .get("uri")
                .and_then(Value::as_str)
                .ok_or_else(|| "didOpen missing textDocument.uri".to_string())?;
            let text = text_document
                .get("text")
                .and_then(Value::as_str)
                .ok_or_else(|| "didOpen missing textDocument.text".to_string())?;
            let version = text_document.get("version").and_then(Value::as_i64);
            let file = uri_to_path(uri);
            let diagnostics = session.open_document(Path::new(&file), text, version)?;
            Ok(vec![publish_diagnostics_notification(&file, diagnostics)])
        }
        "textDocument/didChange" => {
            let text_document = params
                .get("textDocument")
                .ok_or_else(|| "didChange missing textDocument".to_string())?;
            let uri = text_document
                .get("uri")
                .and_then(Value::as_str)
                .ok_or_else(|| "didChange missing textDocument.uri".to_string())?;
            let version = text_document.get("version").and_then(Value::as_i64);
            let content_changes = params
                .get("contentChanges")
                .and_then(Value::as_array)
                .ok_or_else(|| "didChange missing contentChanges".to_string())?;
            let text = content_changes
                .last()
                .and_then(|entry| entry.get("text"))
                .and_then(Value::as_str)
                .ok_or_else(|| "didChange missing full text payload".to_string())?;
            let file = uri_to_path(uri);
            let diagnostics = session.change_document(Path::new(&file), text, version)?;
            Ok(vec![publish_diagnostics_notification(&file, diagnostics)])
        }
        "textDocument/didClose" => {
            let file = extract_file_from_text_document(params)?;
            session.close_document(Path::new(&file));
            Ok(vec![publish_diagnostics_notification(&file, Vec::new())])
        }
        _ => Ok(Vec::new()),
    }
}

pub fn lsp_location_value(location: &LspLocation) -> Value {
    json!({
        "uri": path_to_uri(&location.file),
        "range": index_span_to_lsp_range(location.span)
    })
}

pub fn index_span_to_lsp_range(span: IndexSpan) -> Value {
    json!({
        "start": {
            "line": span.line_start.saturating_sub(1),
            "character": span.col_start.saturating_sub(1)
        },
        "end": {
            "line": span.line_end.saturating_sub(1),
            "character": span.col_end
        }
    })
}

fn publish_diagnostics_notification(file: &str, diagnostics: Vec<IndexedDiagnostic>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": path_to_uri(file),
            "diagnostics": diagnostics
                .into_iter()
                .map(indexed_diagnostic_value)
                .collect::<Vec<_>>()
        }
    })
}

fn indexed_diagnostic_value(diagnostic: IndexedDiagnostic) -> Value {
    let severity = match diagnostic.severity {
        IndexedSeverity::Error => 1,
        IndexedSeverity::Warning => 2,
        IndexedSeverity::Info => 3,
    };
    json!({
        "range": index_span_to_lsp_range(diagnostic.span),
        "severity": severity,
        "code": diagnostic.code,
        "source": "vibelang",
        "message": diagnostic.message
    })
}

fn completion_entry_value(item: CompletionEntry) -> Value {
    let kind = match item.kind.as_str() {
        "function" => 3,
        "variable" => 6,
        "type" => 7,
        "keyword" => 14,
        "effect" => 21,
        _ => 1,
    };
    json!({
        "label": item.label,
        "kind": kind,
        "detail": item.detail
    })
}

fn document_symbol_value(item: &DocumentSymbolEntry) -> Value {
    json!({
        "name": item.name,
        "kind": symbol_kind_number(&item.kind),
        "range": index_span_to_lsp_range(item.span),
        "selectionRange": index_span_to_lsp_range(item.span)
    })
}

fn workspace_symbol_value(item: &WorkspaceSymbolEntry) -> Value {
    json!({
        "name": item.name,
        "kind": symbol_kind_number(&item.kind),
        "location": {
            "uri": path_to_uri(&item.file),
            "range": index_span_to_lsp_range(item.span)
        }
    })
}

fn code_action_value(item: &CodeActionSuggestion) -> Value {
    json!({
        "title": item.title,
        "kind": item.kind,
        "diagnostics": [
            {
                "code": item.code
            }
        ],
        "isPreferred": false
    })
}

fn symbol_kind_number(kind: &str) -> u32 {
    match kind {
        "function" => 12,
        "type" => 5,
        "param" | "local" => 13,
        "effect" => 14,
        "contract" => 14,
        _ => 13,
    }
}

fn extract_file_and_position(params: &Value) -> Result<(String, usize, usize), String> {
    let file = extract_file_from_text_document(params)?;
    let position = params
        .get("position")
        .ok_or_else(|| "request missing position".to_string())?;
    let line = position.get("line").and_then(Value::as_u64).unwrap_or(0) as usize + 1;
    let col = position
        .get("character")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize
        + 1;
    Ok((file, line, col))
}

fn extract_file_from_text_document(params: &Value) -> Result<String, String> {
    let uri = params
        .get("textDocument")
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
        .ok_or_else(|| "request missing textDocument.uri".to_string())?;
    Ok(uri_to_path(uri))
}

fn full_document_range(text: &str) -> Value {
    if text.is_empty() {
        return json!({
            "start": {"line": 0, "character": 0},
            "end": {"line": 0, "character": 0}
        });
    }
    let mut lines = text.split('\n').collect::<Vec<_>>();
    let ends_with_newline = text.ends_with('\n');
    if ends_with_newline {
        lines.pop();
    }
    let last_line = lines.last().copied().unwrap_or_default();
    let end_line = if ends_with_newline {
        lines.len()
    } else {
        lines.len().saturating_sub(1)
    };
    let end_character = if ends_with_newline {
        0
    } else {
        last_line.chars().count()
    };
    json!({
        "start": {"line": 0, "character": 0},
        "end": {"line": end_line, "character": end_character}
    })
}

pub fn path_to_uri(path: &str) -> String {
    if path.starts_with("file://") {
        return path.to_string();
    }
    format!("file://{}", percent_encode(path))
}

pub fn uri_to_path(uri: &str) -> String {
    if let Some(raw) = uri.strip_prefix("file://") {
        return percent_decode(raw);
    }
    uri.to_string()
}

fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for byte in input.bytes() {
        let ch = byte as char;
        let keep = ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '.' | '~' | ':');
        if keep {
            out.push(ch);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut idx = 0usize;
    let mut out = Vec::<u8>::new();
    while idx < bytes.len() {
        if bytes[idx] == b'%' && idx + 2 < bytes.len() {
            let h1 = bytes[idx + 1] as char;
            let h2 = bytes[idx + 2] as char;
            if let (Some(v1), Some(v2)) = (h1.to_digit(16), h2.to_digit(16)) {
                out.push(((v1 << 4) + v2) as u8);
                idx += 3;
                continue;
            }
        }
        out.push(bytes[idx]);
        idx += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::json;
    use tempfile::tempdir;

    use crate::session::LspSession;

    use super::{handle_notification, handle_request, path_to_uri, ServerState};

    #[test]
    fn jsonrpc_handlers_support_definition_and_references_parity() {
        let dir = tempdir().expect("temp dir");
        let index_root = dir.path().join(".yb/index");
        let file = dir.path().join("nav.yb");
        let src = r#"foo() -> Int { 1 }
bar() -> Int { foo() }
"#;
        fs::write(&file, src).expect("write fixture source");

        let mut session = LspSession::open_or_create(index_root).expect("open lsp session");
        let mut state = ServerState::default();
        let open_params = json!({
            "textDocument": {
                "uri": path_to_uri(&file.to_string_lossy()),
                "version": 1,
                "text": src
            }
        });
        let open_notifications = handle_notification(
            "textDocument/didOpen",
            &open_params,
            &mut session,
            &mut state,
        )
        .expect("didOpen should succeed");
        assert!(
            !open_notifications.is_empty(),
            "didOpen should publish diagnostics notification"
        );

        let req_params = json!({
            "textDocument": { "uri": path_to_uri(&file.to_string_lossy()) },
            "position": { "line": 1, "character": 17 }
        });
        let definition = handle_request(
            "textDocument/definition",
            &req_params,
            &mut session,
            &mut state,
        )
        .expect("definition request should succeed");
        assert!(
            definition.result.get("uri").is_some(),
            "definition result should include a location"
        );

        let references = handle_request(
            "textDocument/references",
            &req_params,
            &mut session,
            &mut state,
        )
        .expect("references request should succeed");
        let count = references
            .result
            .as_array()
            .map(|items| items.len())
            .unwrap_or_default();
        assert!(count >= 1, "references should include callsite(s)");
    }

    #[test]
    fn jsonrpc_handlers_support_hover_parity() {
        let dir = tempdir().expect("temp dir");
        let index_root = dir.path().join(".yb/index");
        let file = dir.path().join("hover.yb");
        let src = r#"foo() -> Int {
  @intent "compute deterministic value"
  @effect alloc
  1
}
bar() -> Int { foo() }
"#;
        fs::write(&file, src).expect("write fixture source");

        let mut session = LspSession::open_or_create(index_root).expect("open lsp session");
        let mut state = ServerState::default();
        let open_params = json!({
            "textDocument": {
                "uri": path_to_uri(&file.to_string_lossy()),
                "version": 1,
                "text": src
            }
        });
        let _ = handle_notification(
            "textDocument/didOpen",
            &open_params,
            &mut session,
            &mut state,
        )
        .expect("didOpen should succeed");

        let req_params = json!({
            "textDocument": { "uri": path_to_uri(&file.to_string_lossy()) },
            "position": { "line": 5, "character": 17 }
        });
        let hover = handle_request("textDocument/hover", &req_params, &mut session, &mut state)
            .expect("hover request should succeed");
        let content = hover
            .result
            .get("contents")
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        assert!(
            content.contains("deterministic"),
            "hover markdown should include contract metadata"
        );
    }

    #[test]
    fn jsonrpc_handlers_support_completion_and_symbol_search() {
        let dir = tempdir().expect("temp dir");
        let index_root = dir.path().join(".yb/index");
        let file = dir.path().join("symbols.yb");
        let src = r#"foo() -> Int { 1 }
bar() -> Int { fo }
"#;
        fs::write(&file, src).expect("write fixture source");

        let mut session = LspSession::open_or_create(index_root).expect("open lsp session");
        let mut state = ServerState::default();
        let open_params = json!({
            "textDocument": {
                "uri": path_to_uri(&file.to_string_lossy()),
                "version": 1,
                "text": src
            }
        });
        let _ = handle_notification(
            "textDocument/didOpen",
            &open_params,
            &mut session,
            &mut state,
        )
        .expect("didOpen should succeed");

        let completion_params = json!({
            "textDocument": { "uri": path_to_uri(&file.to_string_lossy()) },
            "position": { "line": 1, "character": 17 }
        });
        let completion = handle_request(
            "textDocument/completion",
            &completion_params,
            &mut session,
            &mut state,
        )
        .expect("completion request should succeed");
        let has_foo = completion
            .result
            .get("items")
            .and_then(|v| v.as_array())
            .is_some_and(|items| {
                items.iter().any(|item| {
                    item.get("label")
                        .and_then(|value| value.as_str())
                        .is_some_and(|label| label == "foo")
                })
            });
        assert!(
            has_foo,
            "completion items should include indexed symbol `foo`"
        );

        let doc_symbol = handle_request(
            "textDocument/documentSymbol",
            &json!({ "textDocument": { "uri": path_to_uri(&file.to_string_lossy()) } }),
            &mut session,
            &mut state,
        )
        .expect("document symbol request should succeed");
        assert!(
            doc_symbol
                .result
                .as_array()
                .is_some_and(|items| !items.is_empty()),
            "documentSymbol should return indexed declarations"
        );

        let workspace_symbol = handle_request(
            "workspace/symbol",
            &json!({ "query": "foo" }),
            &mut session,
            &mut state,
        )
        .expect("workspace symbol request should succeed");
        assert!(
            workspace_symbol
                .result
                .as_array()
                .is_some_and(|items| !items.is_empty()),
            "workspace/symbol should return matching entries"
        );
    }

    #[test]
    fn jsonrpc_handlers_support_deterministic_rename() {
        let dir = tempdir().expect("temp dir");
        let index_root = dir.path().join(".yb/index");
        let file = dir.path().join("rename.yb");
        let src = r#"foo() -> Int { 1 }
bar() -> Int { foo() + foo() }
"#;
        fs::write(&file, src).expect("write fixture source");

        let mut session = LspSession::open_or_create(index_root).expect("open lsp session");
        let mut state = ServerState::default();
        let open_params = json!({
            "textDocument": {
                "uri": path_to_uri(&file.to_string_lossy()),
                "version": 1,
                "text": src
            }
        });
        let _ = handle_notification(
            "textDocument/didOpen",
            &open_params,
            &mut session,
            &mut state,
        )
        .expect("didOpen should succeed");

        let rename = handle_request(
            "textDocument/rename",
            &json!({
                "textDocument": { "uri": path_to_uri(&file.to_string_lossy()) },
                "position": { "line": 1, "character": 17 },
                "newName": "fooRenamed"
            }),
            &mut session,
            &mut state,
        )
        .expect("rename request should succeed");
        let changes = rename
            .result
            .get("changes")
            .and_then(|v| v.as_object())
            .expect("rename should return workspace changes");
        let edit_count = changes
            .values()
            .flat_map(|v| v.as_array().cloned().unwrap_or_default())
            .count();
        assert!(
            edit_count >= 3,
            "rename should include declaration and callsite edits"
        );
    }

    #[test]
    fn jsonrpc_handlers_surface_code_actions_from_diagnostics() {
        let dir = tempdir().expect("temp dir");
        let index_root = dir.path().join(".yb/index");
        let file = dir.path().join("actions.yb");
        let src = "main() -> Int { unknown_symbol }";
        fs::write(&file, src).expect("write fixture source");

        let mut session = LspSession::open_or_create(index_root).expect("open lsp session");
        let mut state = ServerState::default();
        let open_params = json!({
            "textDocument": {
                "uri": path_to_uri(&file.to_string_lossy()),
                "version": 1,
                "text": src
            }
        });
        let _ = handle_notification(
            "textDocument/didOpen",
            &open_params,
            &mut session,
            &mut state,
        )
        .expect("didOpen should succeed");

        let actions = handle_request(
            "textDocument/codeAction",
            &json!({
                "textDocument": { "uri": path_to_uri(&file.to_string_lossy()) },
                "range": {
                    "start": { "line": 0, "character": 0 },
                    "end": { "line": 0, "character": 10 }
                },
                "context": { "diagnostics": [] }
            }),
            &mut session,
            &mut state,
        )
        .expect("codeAction request should succeed");
        assert!(
            actions
                .result
                .as_array()
                .is_some_and(|items| !items.is_empty()),
            "codeAction should return deterministic quickfix suggestions"
        );
    }

    #[test]
    fn jsonrpc_handlers_use_formatter_for_document_formatting() {
        let dir = tempdir().expect("temp dir");
        let index_root = dir.path().join(".yb/index");
        let file = dir.path().join("formatting.yb");
        let src = "main() -> Int {\n\t1    \n\n\n  0\n}\n";
        fs::write(&file, src).expect("write fixture source");

        let mut session = LspSession::open_or_create(index_root).expect("open lsp session");
        let mut state = ServerState::default();
        let open_params = json!({
            "textDocument": {
                "uri": path_to_uri(&file.to_string_lossy()),
                "version": 1,
                "text": src
            }
        });
        let _ = handle_notification(
            "textDocument/didOpen",
            &open_params,
            &mut session,
            &mut state,
        )
        .expect("didOpen should succeed");

        let formatted = handle_request(
            "textDocument/formatting",
            &json!({
                "textDocument": { "uri": path_to_uri(&file.to_string_lossy()) },
                "options": { "tabSize": 2, "insertSpaces": true }
            }),
            &mut session,
            &mut state,
        )
        .expect("formatting request should succeed");
        let new_text = formatted
            .result
            .as_array()
            .and_then(|items| items.first())
            .and_then(|item| item.get("newText"))
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        assert_eq!(new_text, "main() -> Int {\n  1\n\n  0\n}\n");
    }
}
