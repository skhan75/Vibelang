// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Argument checking for the `print`/`println` builtins (audit P0-14a):
//! `println(42)` used to type-check clean and then SIGSEGV at runtime
//! because codegen passes the raw value to the runtime's string-pointer
//! ABI. The builtins must take exactly one `Str` argument; `Unknown`
//! stays accepted (permissive rule), and interpolated strings keep
//! working because the parser desugars them to `Str`.

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
fn println_int_variable_is_rejected() {
    // Audit repro: this previously built clean and the binary exited 139.
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  n := 42
  println(n)
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2265"], "diagnostics: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(
        d.message,
        "argument 1 type mismatch in call to `println`: expected `Str`, got `Int`; interpolate with `\"{name}\"` or convert with `convert.to_str`"
    );
    assert_eq!(d.span.line_start, 4, "span should point at the argument");
}

#[test]
fn println_int_literal_is_rejected() {
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  println(42)
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2265"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "argument 1 type mismatch in call to `println`: expected `Str`, got `Int`; interpolate with `\"{name}\"` or convert with `convert.to_str`"
    );
}

#[test]
fn println_bool_float_and_list_are_rejected() {
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  @effect alloc
  println(true)
  println(1.5)
  println([1, 2])
  0
}
"#,
    );
    assert_eq!(
        codes(&diags),
        vec!["E2265", "E2265", "E2265"],
        "diagnostics: {diags:?}"
    );
    assert_eq!(
        diags[0].message,
        "argument 1 type mismatch in call to `println`: expected `Str`, got `Bool`; interpolate with `\"{name}\"` or convert with `convert.to_str`"
    );
    assert_eq!(
        diags[1].message,
        "argument 1 type mismatch in call to `println`: expected `Str`, got `Float`; interpolate with `\"{name}\"` or convert with `convert.to_str`"
    );
    assert_eq!(
        diags[2].message,
        "argument 1 type mismatch in call to `println`: expected `Str`, got `List<Int>`; interpolate with `\"{name}\"` or convert with `convert.to_str`"
    );
}

#[test]
fn print_int_is_rejected() {
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  print(7)
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2265"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "argument 1 type mismatch in call to `print`: expected `Str`, got `Int`; interpolate with `\"{name}\"` or convert with `convert.to_str`"
    );
}

#[test]
fn println_zero_args_is_rejected() {
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  println()
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2264"], "diagnostics: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(d.message, "`println` expects exactly 1 argument, got 0");
    assert_eq!(d.span.line_start, 3, "span should point at the call");
}

#[test]
fn println_two_args_is_rejected() {
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  println("a", "b")
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2264"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "`println` expects exactly 1 argument, got 2"
    );
}

#[test]
fn print_two_args_is_rejected() {
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  print("a", "b")
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2264"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "`print` expects exactly 1 argument, got 2"
    );
}

#[test]
fn println_str_literal_and_variable_are_accepted() {
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  msg := "hello"
  println("direct")
  println(msg)
  print("no newline")
  0
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn println_interpolated_string_is_accepted() {
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  count := 3
  println("you have {count} items")
  0
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn println_str_returning_call_is_accepted() {
    let diags = check(
        r#"greet(name: Str) -> Str {
  name
}

pub main() -> Int {
  @effect io
  println(greet("vibe"))
  println(convert.to_str(42))
  0
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn println_unknown_typed_value_is_accepted() {
    // Permissive rule: `Unknown` stays accepted so untyped/ambiguous
    // values do not produce false rejections.
    let diags = check(
        r#"mystery(x) {
  x
}

pub main() -> Int {
  @effect io
  println(mystery("s"))
  0
}
"#,
    );
    assert!(
        !codes(&diags).contains(&"E2264") && !codes(&diags).contains(&"E2265"),
        "Unknown-typed argument must stay accepted: {diags:?}"
    );
}
