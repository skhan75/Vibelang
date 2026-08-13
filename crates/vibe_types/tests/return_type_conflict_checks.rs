// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Conflicting inferred return types must be reported instead of silently
//! unifying to `Unknown` (audit P0-04): a `Result`-declared function with a
//! bare `Int` tail previously compiled with zero diagnostics, and the raw int
//! was treated as a heap Result pointer downstream. `Unknown` members (e.g.
//! results of unresolved calls) must stay permissive.

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
fn result_declared_function_with_bare_int_tail_is_rejected() {
    // Audit repro b1b: previously compiled with zero diagnostics.
    let diags = check(
        r#"get_num(flag: Bool) -> Result<Int, Str> {
  if flag {
    return ok(1)
  }
  42
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2201"], "diagnostics: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(
        d.message,
        "conflicting return types in `get_num`: this path returns `Int`, but an earlier path returns `Result<Int, Unknown>`"
    );
    assert_eq!(
        d.span.line_start, 5,
        "span should point at the conflicting tail expression"
    );
}

#[test]
fn conflicting_str_and_int_paths_are_rejected() {
    let diags = check(
        r#"describe(flag: Bool) -> Str {
  if flag {
    return "yes"
  }
  7
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2201"], "diagnostics: {diags:?}");
    let d = &diags[0];
    assert_eq!(
        d.message,
        "conflicting return types in `describe`: this path returns `Int`, but an earlier path returns `Str`"
    );
    assert_eq!(
        d.span.line_start, 5,
        "span should point at the conflicting return site"
    );
}

#[test]
fn int_returned_on_two_paths_is_accepted() {
    let diags = check(
        r#"pick(flag: Bool) -> Int {
  if flag {
    return 1
  }
  2
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(codes(&diags), Vec::<&str>::new(), "diagnostics: {diags:?}");
}

#[test]
fn int_and_float_paths_stay_compatible() {
    // Int/Float coercion is allowed by `type_compatible` and must not be
    // reported as a conflict.
    let diags = check(
        r#"measure(flag: Bool) -> Float {
  if flag {
    return 1
  }
  2.5
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(codes(&diags), Vec::<&str>::new(), "diagnostics: {diags:?}");
}

#[test]
fn enum_returned_on_all_paths_is_accepted() {
    let diags = check(
        r#"enum Color {
  Red,
  Green,
  Blue
}

pick(n: Int) -> Color {
  if n == 0 {
    return Color.Red
  }
  if n == 1 {
    return Color.Green
  }
  Color.Blue
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(codes(&diags), Vec::<&str>::new(), "diagnostics: {diags:?}");
}

#[test]
fn unknown_members_do_not_conflict() {
    // `x` has no type annotation, so `return x` infers `Unknown`; mixing it
    // with a concrete `Int` path must stay permissive.
    let diags = check(
        r#"passthrough(x, flag: Bool) -> Int {
  if flag {
    return x
  }
  0
}

pub main() -> Int {
  0
}
"#,
    );
    assert_eq!(codes(&diags), Vec::<&str>::new(), "diagnostics: {diags:?}");
}
