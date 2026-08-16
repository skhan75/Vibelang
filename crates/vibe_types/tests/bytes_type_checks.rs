// Copyright 2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! `Bytes` must resolve identically in every position a type can appear:
//! parameter, return, and inside a function type. There are three separate
//! type-name resolution ladders in `vibe_types` and a variant added to only
//! some of them resolves in some positions and silently becomes `Unknown` in
//! the others.

use vibe_parser::parse_source;
use vibe_types::check_and_lower;

fn errors(src: &str) -> Vec<String> {
    let parsed = parse_source(src);
    assert!(!parsed.diagnostics.has_errors(), "unexpected parse errors");
    check_and_lower(&parsed.ast)
        .diagnostics
        .sorted()
        .iter()
        .filter(|d| d.severity == vibe_diagnostics::Severity::Error)
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect()
}

#[test]
fn bytes_resolves_as_a_parameter_and_return_type() {
    let diags = errors(
        r#"take(b: Bytes) -> Bytes {
  b
}

pub main() -> Int {
  0
}
"#,
    );
    assert!(diags.is_empty(), "diagnostics: {diags:?}");
}

#[test]
fn bytes_resolves_inside_a_function_type() {
    let diags = errors(
        r#"apply(f: fn(Bytes) -> Bytes, b: Bytes) -> Bytes {
  f(b)
}

pub main() -> Int {
  0
}
"#,
    );
    assert!(diags.is_empty(), "diagnostics: {diags:?}");
}

#[test]
fn bytes_is_not_interchangeable_with_str() {
    let diags = errors(
        r#"take(b: Bytes) -> Bytes {
  b
}

pub main() -> Int {
  take("hello")
  0
}
"#,
    );
    assert!(
        diags.iter().any(|d| d.starts_with("E2265")),
        "passing a Str where Bytes is declared must be rejected; got {diags:?}"
    );
}
