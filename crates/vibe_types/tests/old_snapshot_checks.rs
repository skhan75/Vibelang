// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! `old(...)` operand restrictions (audit P0-06): entry snapshots are plain
//! value copies, which is only faithful for scalars. Heap-reference operands
//! (Str, List, Map, records) must be rejected honestly with E2208 instead of
//! silently aliasing post-state, and `.` inside `old(...)` is E2209 because
//! the result does not exist when the snapshot is taken. Scalar operands —
//! the README/book pattern, including record field reads — stay accepted.

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

fn error_codes(diags: &[Diagnostic]) -> Vec<&str> {
    diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.as_str())
        .collect()
}

#[test]
fn old_on_str_param_is_rejected() {
    let diags = check(
        r#"shout(s: Str) -> Str {
  @ensure old(s) == s
  s + "!"
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(error_codes(&diags), vec!["E2208"], "diagnostics: {diags:?}");
    let d = diags
        .iter()
        .find(|d| d.code == "E2208")
        .expect("E2208 diagnostic");
    assert!(
        d.message.contains("operand has type `Str`"),
        "message should name the offending type: {}",
        d.message
    );
}

#[test]
fn old_on_list_param_is_rejected() {
    let diags = check(
        r#"grow(xs: List<Int>) -> Int {
  @ensure xs.len > old(xs).len
  xs.push(1)
  xs.len
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(error_codes(&diags), vec!["E2208"], "diagnostics: {diags:?}");
}

#[test]
fn old_on_record_param_is_rejected() {
    let diags = check(
        r#"type Account {
  balance: Int
}

touch(a: Account) -> Int {
  @ensure old(a) == a
  a.balance
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(error_codes(&diags), vec!["E2208"], "diagnostics: {diags:?}");
    let d = diags
        .iter()
        .find(|d| d.code == "E2208")
        .expect("E2208 diagnostic");
    assert!(
        d.message.contains("operand has type `Account`"),
        "message should name the offending type: {}",
        d.message
    );
}

#[test]
fn dot_result_inside_old_is_rejected() {
    let diags = check(
        r#"same(x: Int) -> Int {
  @ensure old(.) == x
  x
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(error_codes(&diags), vec!["E2209"], "diagnostics: {diags:?}");
}

#[test]
fn old_on_scalar_record_field_is_accepted() {
    // The flagship README pattern: `old(...)` over a mutated record field.
    let diags = check(
        r#"type Box {
  v: Int
}

bump(mut b: Box) -> Int {
  @ensure b.v == old(b.v) + 1
  b.v = b.v + 1
  b.v
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(
        error_codes(&diags),
        Vec::<&str>::new(),
        "diagnostics: {diags:?}"
    );
}

#[test]
fn old_on_scalar_params_and_larger_expressions_accepted() {
    // `old(x) + 1` and `old(x + 1)` must both check clean, as must Bool and
    // Float operands.
    let diags = check(
        r#"bump(mut x: Int) -> Int {
  @ensure x == old(x) + 1
  @ensure x == old(x + 1)
  x = x + 1
  x
}

keep(flag: Bool) -> Bool {
  @ensure flag == old(flag)
  flag
}

scale(f: Float) -> Float {
  @ensure . >= old(f)
  f * 2.0
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(
        error_codes(&diags),
        Vec::<&str>::new(),
        "diagnostics: {diags:?}"
    );
}

#[test]
fn old_outside_ensure_is_still_rejected_with_existing_codes() {
    let diags = check(
        r#"bad(x: Int) -> Int {
  @require old(x) > 0
  x
}

pub main() -> Int {
  0
}
"#,
    );
    let codes = error_codes(&diags);
    assert!(
        codes.contains(&"E2207") && codes.contains(&"E2205"),
        "old() outside @ensure must keep E2205/E2207: {diags:?}"
    );
    assert!(
        !codes.contains(&"E2208") && !codes.contains(&"E2209"),
        "operand checks must not fire outside @ensure clauses: {diags:?}"
    );
}
