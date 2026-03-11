// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_json::{json, Value};

pub fn initialize_result() -> Value {
    json!({
        "capabilities": {
            "textDocumentSync": {
                "openClose": true,
                "change": 1
            },
            "definitionProvider": true,
            "referencesProvider": true,
            "hoverProvider": true,
            "completionProvider": {
                "resolveProvider": false,
                "triggerCharacters": [".", "@", ":"]
            },
            "documentSymbolProvider": true,
            "workspaceSymbolProvider": true,
            "renameProvider": true,
            "codeActionProvider": true,
            "documentFormattingProvider": true,
            "documentRangeFormattingProvider": true
        },
        "serverInfo": {
            "name": "vibelang-lsp",
            "version": "0.13.1"
        }
    })
}
