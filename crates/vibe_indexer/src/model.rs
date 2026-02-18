use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use vibe_diagnostics::{Severity, Span};

pub const INDEX_SCHEMA_VERSION: u32 = 1;
pub const INDEX_FILENAME: &str = "index_v1.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SymbolId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Param,
    Local,
    Contract,
    Effect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IndexSpan {
    pub line_start: usize,
    pub col_start: usize,
    pub line_end: usize,
    pub col_end: usize,
}

impl IndexSpan {
    pub fn contains(&self, line: usize, col: usize) -> bool {
        let start_ok =
            (line > self.line_start) || (line == self.line_start && col >= self.col_start);
        let end_ok = (line < self.line_end) || (line == self.line_end && col <= self.col_end);
        start_ok && end_ok
    }

    pub fn area(&self) -> usize {
        let lines = self.line_end.saturating_sub(self.line_start) + 1;
        let cols = self.col_end.saturating_sub(self.col_start) + 1;
        lines.saturating_mul(cols)
    }
}

impl From<Span> for IndexSpan {
    fn from(value: Span) -> Self {
        Self {
            line_start: value.line_start,
            col_start: value.col_start,
            line_end: value.line_end,
            col_end: value.col_end,
        }
    }
}

impl From<IndexSpan> for Span {
    fn from(value: IndexSpan) -> Self {
        Self {
            line_start: value.line_start,
            col_start: value.col_start,
            line_end: value.line_end,
            col_end: value.col_end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub module: Option<String>,
    pub file: String,
    pub span: IndexSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    pub symbol_id: SymbolId,
    pub file: String,
    pub span: IndexSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionMeta {
    pub symbol_id: SymbolId,
    pub function_name: String,
    pub file: String,
    pub signature_hash: String,
    pub effects_declared: Vec<String>,
    pub effects_observed: Vec<String>,
    pub intent_text: Option<String>,
    pub has_examples: bool,
    pub is_public: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectMismatch {
    pub function_name: String,
    pub file: String,
    pub declared_only: Vec<String>,
    pub observed_only: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IndexedSeverity {
    Error,
    Warning,
    Info,
}

impl From<Severity> for IndexedSeverity {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Error => Self::Error,
            Severity::Warning => Self::Warning,
            Severity::Info => Self::Info,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedDiagnostic {
    pub code: String,
    pub severity: IndexedSeverity,
    pub message: String,
    pub span: IndexSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileIndex {
    pub file: String,
    pub file_hash: String,
    pub symbols: Vec<Symbol>,
    pub references: Vec<Reference>,
    pub function_meta: Vec<FunctionMeta>,
    pub effect_mismatches: Vec<EffectMismatch>,
    pub diagnostics: Vec<IndexedDiagnostic>,
    pub dependencies: Vec<String>,
}

impl FileIndex {
    pub fn normalize(&mut self) {
        self.symbols.sort_by(|a, b| {
            (
                a.file.as_str(),
                a.span.line_start,
                a.span.col_start,
                a.span.line_end,
                a.span.col_end,
                a.kind,
                a.name.as_str(),
                a.id.0,
            )
                .cmp(&(
                    b.file.as_str(),
                    b.span.line_start,
                    b.span.col_start,
                    b.span.line_end,
                    b.span.col_end,
                    b.kind,
                    b.name.as_str(),
                    b.id.0,
                ))
        });
        self.references.sort_by(|a, b| {
            (
                a.file.as_str(),
                a.span.line_start,
                a.span.col_start,
                a.span.line_end,
                a.span.col_end,
                a.symbol_id.0,
            )
                .cmp(&(
                    b.file.as_str(),
                    b.span.line_start,
                    b.span.col_start,
                    b.span.line_end,
                    b.span.col_end,
                    b.symbol_id.0,
                ))
        });
        self.function_meta.sort_by(|a, b| {
            (
                a.file.as_str(),
                a.function_name.as_str(),
                a.symbol_id.0,
                a.signature_hash.as_str(),
            )
                .cmp(&(
                    b.file.as_str(),
                    b.function_name.as_str(),
                    b.symbol_id.0,
                    b.signature_hash.as_str(),
                ))
        });
        self.effect_mismatches.sort_by(|a, b| {
            (a.file.as_str(), a.function_name.as_str())
                .cmp(&(b.file.as_str(), b.function_name.as_str()))
        });
        self.diagnostics.sort_by(|a, b| {
            (
                a.span.line_start,
                a.span.col_start,
                a.span.line_end,
                a.span.col_end,
                a.code.as_str(),
                a.severity,
            )
                .cmp(&(
                    b.span.line_start,
                    b.span.col_start,
                    b.span.line_end,
                    b.span.col_end,
                    b.code.as_str(),
                    b.severity,
                ))
        });
        self.dependencies.sort();
        self.dependencies.dedup();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexSnapshot {
    pub schema_version: u32,
    pub files: BTreeMap<String, FileIndex>,
}

impl Default for IndexSnapshot {
    fn default() -> Self {
        Self {
            schema_version: INDEX_SCHEMA_VERSION,
            files: BTreeMap::new(),
        }
    }
}

impl IndexSnapshot {
    pub fn normalize(&mut self) {
        for file_index in self.files.values_mut() {
            file_index.normalize();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IndexStats {
    pub files: usize,
    pub symbols: usize,
    pub references: usize,
    pub function_meta: usize,
    pub diagnostics: usize,
    pub memory_estimate_bytes: usize,
}
