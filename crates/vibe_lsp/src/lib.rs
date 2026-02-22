mod capabilities;
mod handlers;
mod legacy;
mod protocol;
mod session;

use std::path::PathBuf;

pub use session::{
    CodeActionSuggestion, CompletionEntry, ContractMetadata, DocumentSymbolEntry, LspLocation,
    LspSession, RenameEdit, RenameResult, WorkspaceSymbolEntry,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    Legacy,
    JsonRpc,
}

pub fn run_lsp_stdio(index_root: impl Into<PathBuf>, mode: TransportMode) -> Result<(), String> {
    match mode {
        TransportMode::Legacy => legacy::run_line_stdio(index_root),
        TransportMode::JsonRpc => protocol::run_jsonrpc_stdio(index_root),
    }
}

pub fn run_line_stdio(index_root: impl Into<PathBuf>) -> Result<(), String> {
    legacy::run_line_stdio(index_root)
}

pub fn run_jsonrpc_stdio(index_root: impl Into<PathBuf>) -> Result<(), String> {
    protocol::run_jsonrpc_stdio(index_root)
}
