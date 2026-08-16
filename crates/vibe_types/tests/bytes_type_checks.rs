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
    // Assert the exact diagnostic set, not just that E2265 is present
    // somewhere in it. Before `Bytes` was taught to every resolution
    // ladder, `Bytes` fell through to `parse_type_ref`'s final rule and
    // became a phantom `UserType("Bytes")`: that also produces E2265
    // (since a phantom user type is never compatible with `Str`), but
    // alongside two spurious `E2005: unknown type` errors. A test that
    // only checks E2265's presence cannot tell that coincidental
    // rejection apart from a real `Bytes` type genuinely rejecting `Str`.
    // `assert_eq!` on the whole vector fails the moment E2005 rides
    // along, which is exactly the bug this task fixes.
    assert_eq!(
        diags,
        vec![
            "E2265: argument 1 type mismatch in call to `take`: expected `Bytes`, got `Str`"
                .to_string()
        ],
        "passing a Str where Bytes is declared must be rejected with exactly \
         one diagnostic (E2265), with no accompanying E2005; got {diags:?}"
    );
}
