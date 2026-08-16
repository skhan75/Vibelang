// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Operand type checking for comparison, equality, and logical binary
//! operators (audit P0-03): incompatible concrete operands must be rejected
//! at check time, while `Unknown` operands stay permissive.

use vibe_diagnostics::{Diagnostic, Severity};
use vibe_parser::parse_source;
use vibe_types::check_and_lower;

fn check(src: &str) -> Vec<Diagnostic> {
    let parsed = parse_source(src);
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors:\n{}",
        parsed.diagnostics.to_golden()
    );
    check_and_lower(&parsed.ast).diagnostics.sorted()
}

fn codes(diags: &[Diagnostic]) -> Vec<&str> {
    diags.iter().map(|d| d.code.as_str()).collect()
}

#[test]
fn eq_int_vs_str_is_rejected() {
    let diags = check(
        r#"check(x: Int) -> Bool {
  x == "hello"
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2202"], "diagnostics: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(
        d.message,
        "binary operation type mismatch: left `Int`, right `Str`"
    );
    assert_eq!(d.span.line_start, 2, "span should point at the comparison");
}

#[test]
fn ordering_int_vs_str_is_rejected() {
    let diags = check(
        r#"pub main() -> Int {
  x := 1
  mut y := 0
  if x > "hello" {
    y = 1
  }
  y
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2202"], "diagnostics: {diags:?}");
    assert_eq!(diags[0].span.line_start, 4);
}

#[test]
fn require_int_vs_str_is_rejected() {
    let diags = check(
        r#"guarded(x: Int) -> Int {
  @require x > "zero"
  x
}

pub main() -> Int {
  guarded(1)
}
"#,
    );
    assert!(
        codes(&diags).contains(&"E2202"),
        "expected E2202 in @require, got: {diags:?}"
    );
    assert!(
        !codes(&diags).contains(&"E3004"),
        "comparison should still type as Bool for E3004: {diags:?}"
    );
}

#[test]
fn ensure_result_placeholder_int_vs_str_is_rejected() {
    let diags = check(
        r#"answer(x: Int) -> Int {
  @ensure . == "a string"
  x
}

pub main() -> Int {
  answer(1)
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2202"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "binary operation type mismatch: left `Int`, right `Str`"
    );
}

#[test]
fn ensure_param_int_vs_str_is_rejected() {
    let diags = check(
        r#"answer(x: Int) -> Int {
  @ensure x != "nope"
  x
}

pub main() -> Int {
  answer(1)
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2202"], "diagnostics: {diags:?}");
}

#[test]
fn and_with_int_operand_is_rejected() {
    let diags = check(
        r#"pub main() -> Int {
  x := 1
  mut y := 0
  if x && true {
    y = 1
  }
  y
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2273"], "diagnostics: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(
        d.message,
        "logical operator expects Bool operands, left is `Int`"
    );
    assert_eq!(d.span.line_start, 4);
}

#[test]
fn or_with_str_operand_is_rejected() {
    let diags = check(
        r#"pub main() -> Int {
  mut y := 0
  if true || "nope" {
    y = 1
  }
  y
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2273"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "logical operator expects Bool operands, right is `Str`"
    );
}

#[test]
fn int_comparisons_are_accepted() {
    let diags = check(
        r#"pub main() -> Int {
  x := 5
  mut y := 0
  if 1 < 2 {
    y = 1
  }
  if x == 7 {
    y = 2
  }
  y
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn str_equality_is_accepted() {
    let diags = check(
        r#"pub main() -> Int {
  s := "x"
  mut y := 0
  if s == "y" {
    y = 1
  }
  y
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn bool_logic_is_accepted() {
    let diags = check(
        r#"pub main() -> Int {
  x := 5
  mut y := 0
  if true && false {
    y = 1
  }
  if x > 0 && x < 10 {
    y = 2
  }
  y
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn unknown_operand_comparison_is_accepted() {
    // A call to a function without a declared return type infers `Unknown`;
    // comparisons against `Unknown` must stay permissive (stdlib-returning
    // expressions rely on this).
    let diags = check(
        r#"mystery(x: Int) {
  x
}

pub main() -> Int {
  mut y := 0
  if mystery(1) == "whatever" {
    y = 1
  }
  y
}
"#,
    );
    assert!(
        !codes(&diags).contains(&"E2202") && !codes(&diags).contains(&"E2273"),
        "Unknown operands must not be rejected: {diags:?}"
    );
}

#[test]
fn ensure_result_placeholder_matching_type_is_accepted() {
    let diags = check(
        r#"increment(x: Int) -> Int {
  @ensure . == x + 1
  x + 1
}

pub main() -> Int {
  increment(1)
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn ensure_len_of_result_placeholder_is_accepted() {
    let diags = check(
        r#"build(n: Int) -> List<Int> {
  @effect alloc
  @ensure len(.) == n
  [n]
}

pub main() -> Int {
  @effect alloc
  len(build(3))
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}
