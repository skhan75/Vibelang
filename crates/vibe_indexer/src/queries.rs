// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::cmp::Ordering;

use crate::model::{EffectMismatch, FunctionMeta, IndexSnapshot, Reference, Symbol, SymbolId};

pub fn find_symbol(snapshot: &IndexSnapshot, name: &str) -> Vec<Symbol> {
    let mut out = snapshot
        .files
        .values()
        .flat_map(|entry| entry.symbols.iter())
        .filter(|symbol| symbol.name == name)
        .cloned()
        .collect::<Vec<_>>();
    sort_symbols(&mut out);
    out
}

pub fn find_references(snapshot: &IndexSnapshot, symbol_id: SymbolId) -> Vec<Reference> {
    let mut out = snapshot
        .files
        .values()
        .flat_map(|entry| entry.references.iter())
        .filter(|reference| reference.symbol_id == symbol_id)
        .cloned()
        .collect::<Vec<_>>();
    out.sort_by(|a, b| {
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
    out
}

pub fn find_by_intent(snapshot: &IndexSnapshot, text_query: &str) -> Vec<FunctionMeta> {
    let query_lower = text_query.to_lowercase();
    let mut out = snapshot
        .files
        .values()
        .flat_map(|entry| entry.function_meta.iter())
        .filter(|meta| {
            meta.intent_text
                .as_ref()
                .is_some_and(|text| text.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect::<Vec<_>>();
    out.sort_by(|a, b| {
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
    out
}

pub fn list_missing_examples(snapshot: &IndexSnapshot, public_only: bool) -> Vec<FunctionMeta> {
    let mut out = snapshot
        .files
        .values()
        .flat_map(|entry| entry.function_meta.iter())
        .filter(|meta| !meta.has_examples)
        .filter(|meta| !public_only || meta.is_public)
        .cloned()
        .collect::<Vec<_>>();
    out.sort_by(|a, b| {
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
    out
}

pub fn effect_mismatches(snapshot: &IndexSnapshot) -> Vec<EffectMismatch> {
    let mut out = snapshot
        .files
        .values()
        .flat_map(|entry| entry.effect_mismatches.iter())
        .cloned()
        .collect::<Vec<_>>();
    out.sort_by(|a, b| {
        (a.file.as_str(), a.function_name.as_str())
            .cmp(&(b.file.as_str(), b.function_name.as_str()))
    });
    out
}

pub fn symbol_at_position(
    snapshot: &IndexSnapshot,
    file: &str,
    line: usize,
    col: usize,
) -> Option<SymbolId> {
    let references = snapshot
        .files
        .get(file)
        .map(|entry| {
            entry
                .references
                .iter()
                .filter(|r| r.span.contains(line, col))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !references.is_empty() {
        return references
            .into_iter()
            .min_by_key(|r| r.span.area())
            .map(|r| r.symbol_id);
    }

    let mut candidates = snapshot
        .files
        .get(file)
        .map(|entry| {
            entry
                .symbols
                .iter()
                .filter(|s| s.span.contains(line, col))
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    sort_symbols(&mut candidates);
    candidates
        .into_iter()
        .min_by(|a, b| match a.span.area().cmp(&b.span.area()) {
            Ordering::Equal => a.id.0.cmp(&b.id.0),
            other => other,
        })
        .map(|s| s.id)
}

pub fn definition_for_position(
    snapshot: &IndexSnapshot,
    file: &str,
    line: usize,
    col: usize,
) -> Option<Symbol> {
    let symbol_id = symbol_at_position(snapshot, file, line, col)?;
    snapshot
        .files
        .values()
        .flat_map(|entry| entry.symbols.iter())
        .find(|symbol| symbol.id == symbol_id)
        .cloned()
}

pub fn references_for_position(
    snapshot: &IndexSnapshot,
    file: &str,
    line: usize,
    col: usize,
) -> Vec<Reference> {
    let Some(symbol_id) = symbol_at_position(snapshot, file, line, col) else {
        return Vec::new();
    };
    find_references(snapshot, symbol_id)
}

fn sort_symbols(symbols: &mut [Symbol]) {
    symbols.sort_by(|a, b| {
        (
            a.file.as_str(),
            a.span.line_start,
            a.span.col_start,
            a.span.line_end,
            a.span.col_end,
            a.name.as_str(),
            a.id.0,
        )
            .cmp(&(
                b.file.as_str(),
                b.span.line_start,
                b.span.col_start,
                b.span.line_end,
                b.span.col_end,
                b.name.as_str(),
                b.id.0,
            ))
    });
}
