// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Arity and argument type checking for direct calls to named functions
//! (audit P0-01): `add_one("not a number")` must be rejected at check time
//! with the same E2264/E2265 codes used for function-value calls, while
//! generic calls, recursion, coercions, and shadowed names stay unaffected.

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
fn wrong_arg_type_is_rejected() {
    // Audit repro: this previously compiled clean and printed pointer garbage.
    let diags = check(
        r#"add_one(x: Int) -> Int {
  x + 1
}

pub main() -> Int {
  add_one("not a number")
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2265"], "diagnostics: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(
        d.message,
        "argument 1 type mismatch in call to `add_one`: expected `Int`, got `Str`"
    );
    assert_eq!(d.span.line_start, 6, "span should point at the argument");
}

#[test]
fn too_few_args_is_rejected() {
    let diags = check(
        r#"add(x: Int, y: Int) -> Int {
  x + y
}

pub main() -> Int {
  add(1)
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2264"], "diagnostics: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(d.message, "function `add` expects 2 argument(s), got 1");
    assert_eq!(d.span.line_start, 6, "span should point at the call");
}

#[test]
fn too_many_args_is_rejected() {
    let diags = check(
        r#"add(x: Int, y: Int) -> Int {
  x + y
}

pub main() -> Int {
  add(1, 2, 3)
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2264"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "function `add` expects 2 argument(s), got 3"
    );
}

#[test]
fn multiple_bad_args_report_each_position() {
    let diags = check(
        r#"combine(a: Int, b: Str) -> Int {
  a + len(b)
}

pub main() -> Int {
  combine("one", 2)
}
"#,
    );
    assert_eq!(
        codes(&diags),
        vec!["E2265", "E2265"],
        "diagnostics: {diags:?}"
    );
    assert_eq!(
        diags[0].message,
        "argument 1 type mismatch in call to `combine`: expected `Int`, got `Str`"
    );
    assert_eq!(
        diags[1].message,
        "argument 2 type mismatch in call to `combine`: expected `Str`, got `Int`"
    );
}

#[test]
fn correct_calls_are_accepted() {
    let diags = check(
        r#"add(x: Int, y: Int) -> Int {
  x + y
}

greet(name: Str) -> Str {
  name
}

pub main() -> Int {
  s := greet("vibe")
  add(len(s), 2)
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn int_to_float_coercion_is_accepted() {
    let diags = check(
        r#"half(x: Float) -> Float {
  x / 2.0
}

pub main() -> Int {
  y := half(4)
  if y > 1.0 {
    return 1
  }
  0
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn generic_calls_are_accepted() {
    // Single-type-param generics go through the E2269/E2272 template path
    // and must not be re-checked against their `TypeParam` signature.
    let diags = check(
        r#"identity<T>(x: T) -> T { x }

first<T>(items: List<T>) -> T { items[0] }

main() -> Int {
  @effect alloc
  a := identity(0)
  xs := [0, 1, 2]
  b := first(xs)
  a + b
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn mutual_recursion_calls_are_accepted() {
    let diags = check(
        r#"is_even(n: Int) -> Bool {
  if n == 0 {
    return true
  }
  is_odd(n - 1)
}

is_odd(n: Int) -> Bool {
  if n == 0 {
    return false
  }
  is_even(n - 1)
}

pub main() -> Int {
  if is_even(4) {
    return 1
  }
  0
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn unannotated_return_call_reports_single_mismatch() {
    // Functions without a declared return type used to reach the
    // function-value fall-through; the direct-call check must not
    // double-report the same argument mismatch.
    let diags = check(
        r#"mystery(x: Int) {
  x
}

pub main() -> Int {
  mystery("nope")
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2265"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "argument 1 type mismatch in call to `mystery`: expected `Int`, got `Str`"
    );
}

#[test]
fn function_value_call_keeps_value_path_check() {
    let diags = check(
        r#"apply(cb: fn(Int) -> Int) -> Int {
  cb("bad")
}

pub main() -> Int {
  apply(fn(z: Int) -> Int { z })
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2265"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "argument 1 type mismatch: expected `Int`, got `Str`"
    );
}

#[test]
fn local_shadowing_binding_is_not_checked_against_outer_function() {
    let diags = check(
        r#"double(x: Int) -> Int {
  x * 2
}

pub main() -> Int {
  double := fn(s: Str) -> Int { 7 }
  double("hi")
}
"#,
    );
    assert!(
        !codes(&diags).contains(&"E2264") && !codes(&diags).contains(&"E2265"),
        "shadowed call must not be checked against the outer signature: {diags:?}"
    );
}
