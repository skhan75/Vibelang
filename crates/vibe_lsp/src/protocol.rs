use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use serde_json::{json, Value};

use crate::handlers::{handle_notification, handle_request, ServerState};
use crate::session::LspSession;

pub fn run_jsonrpc_stdio(index_root: impl Into<PathBuf>) -> Result<(), String> {
    let mut session = LspSession::open_or_create(index_root)?;
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());
    let mut state = ServerState::default();

    while let Some(payload) = read_message(&mut reader)? {
        let message: Value = serde_json::from_str(&payload)
            .map_err(|e| format!("invalid JSON-RPC payload: {e}; payload={payload}"))?;
        dispatch_message(message, &mut session, &mut state, &mut writer)?;
        if state.should_exit {
            break;
        }
    }

    let _ = session.save();
    Ok(())
}

fn dispatch_message<W: Write>(
    message: Value,
    session: &mut LspSession,
    state: &mut ServerState,
    writer: &mut W,
) -> Result<(), String> {
    let Some(object) = message.as_object() else {
        return Err("jsonrpc message must be an object".to_string());
    };

    let method = object
        .get("method")
        .and_then(Value::as_str)
        .ok_or_else(|| "jsonrpc message missing method".to_string())?;
    let params = object.get("params").cloned().unwrap_or_else(|| json!({}));

    if let Some(id) = object.get("id").cloned() {
        match handle_request(method, &params, session, state) {
            Ok(outcome) => {
                for notification in outcome.notifications {
                    write_message(writer, &notification)?;
                }
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": outcome.result
                });
                write_message(writer, &response)?;
            }
            Err(error) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": error
                    }
                });
                write_message(writer, &response)?;
            }
        }
        return Ok(());
    }

    let notifications = handle_notification(method, &params, session, state)?;
    for notification in notifications {
        write_message(writer, &notification)?;
    }
    Ok(())
}

fn read_message<R: BufRead>(reader: &mut R) -> Result<Option<String>, String> {
    let mut content_length = None::<usize>;
    loop {
        let mut line = String::new();
        let bytes_read = reader
            .read_line(&mut line)
            .map_err(|e| format!("failed to read jsonrpc header line: {e}"))?;
        if bytes_read == 0 {
            return Ok(None);
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        let lower = line.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("content-length:") {
            let parsed = value
                .trim()
                .parse::<usize>()
                .map_err(|e| format!("invalid Content-Length header `{value}`: {e}"))?;
            content_length = Some(parsed);
        }
    }

    let body_len = content_length.ok_or_else(|| "missing Content-Length header".to_string())?;
    let mut body = vec![0u8; body_len];
    reader
        .read_exact(&mut body)
        .map_err(|e| format!("failed to read jsonrpc body: {e}"))?;
    String::from_utf8(body)
        .map(Some)
        .map_err(|e| format!("jsonrpc body is not valid UTF-8: {e}"))
}

fn write_message<W: Write>(writer: &mut W, message: &Value) -> Result<(), String> {
    let payload = message.to_string();
    write!(
        writer,
        "Content-Length: {}\r\n\r\n{}",
        payload.len(),
        payload
    )
    .map_err(|e| format!("failed to write jsonrpc payload: {e}"))?;
    writer
        .flush()
        .map_err(|e| format!("failed to flush jsonrpc payload: {e}"))?;
    Ok(())
}
