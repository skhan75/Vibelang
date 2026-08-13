// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Unknown type names in declared signatures and fields must be reported with
//! E2005 (audit P0-04): a typo like `Strr` previously became a phantom user
//! type and an unsupported annotation like `Option<Int>` silently became
//! `Unknown`, disabling every downstream check that touched the annotation.

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
fn typo_in_parameter_type_is_rejected() {
    let diags = check(
        r#"parse_flag(s: Strr) -> Bool {
  true
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2005"], "diagnostics: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(
        d.message,
        "unknown type `Strr` in parameter `s` of function `parse_flag`"
    );
    assert_eq!(d.span.line_start, 1, "span should point at the function");
}

#[test]
fn unsupported_generic_in_return_type_is_rejected() {
    // `Option<Int>` is not a supported type; it previously resolved to
    // `Unknown` and made the declared-return check vacuous.
    let diags = check(
        r#"wrap(x: Int) -> Option<Int> {
  x
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2005"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "unknown type `Option<Int>` in return type of function `wrap`"
    );
}

#[test]
fn typo_in_return_type_is_rejected() {
    // The phantom `Strr` user type also fails the declared-vs-inferred check
    // now that it no longer erases to a compatible-with-everything type.
    let diags = check(
        r#"label() -> Strr {
  "x"
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(
        codes(&diags),
        vec!["E2005", "E2201"],
        "diagnostics: {diags:?}"
    );
    assert_eq!(
        diags[0].message,
        "unknown type `Strr` in return type of function `label`"
    );
}

#[test]
fn unknown_type_in_type_field_is_rejected() {
    let diags = check(
        r#"type P {
  pos: Vec3
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2005"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "unknown type `Vec3` in field `pos` of type `P`"
    );
}

#[test]
fn unknown_type_in_enum_variant_field_is_rejected() {
    let diags = check(
        r#"enum Shape {
  Circle { r: Flaot }
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2005"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "unknown type `Flaot` in field `r` of enum variant `Shape.Circle`"
    );
}

#[test]
fn known_types_and_generics_are_accepted() {
    let diags = check(
        r#"type Point {
  x: Int,
  y: Int
}

enum Color {
  Red,
  Green
}

enum Shape {
  Dot,
  Circle { r: Float }
}

identity<T>(v: T) -> T {
  v
}

use_all(p: Point, c: Color, items: List<Str>, table: Map<Str, Int>, r: Result<Int, Str>, f: fn(Int) -> Int) -> Int {
  0
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(codes(&diags), Vec::<&str>::new(), "diagnostics: {diags:?}");
}
