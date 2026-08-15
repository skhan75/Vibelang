// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! `check_and_lower_with_prelude` is handed the number of declarations the
//! compiler appended to the unit (the stdlib prelude, always the tail). Those
//! declarations come from source the user never wrote and cannot open, so their
//! diagnostics are dropped — and nothing else is.

use std::collections::BTreeMap;

use vibe_diagnostics::Diagnostic;
use vibe_parser::parse_source;
use vibe_types::{check_and_lower_with_ns, check_and_lower_with_prelude};

/// Two user declarations followed by two stand-ins for injected prelude
/// declarations. Each of the four trips the same warning pair (E2004 for the
/// missing return type, E3003 for the effect it declares but never performs).
const SOURCE: &str = r#"pub user_one() {
  @effect net
  1
}

pub user_two() {
  @effect net
  2
}

pub prelude_one() {
  @effect net
  3
}

pub prelude_two() {
  @effect net
  4
}
"#;

fn diagnostics(injected_prelude_decls: usize) -> Vec<Diagnostic> {
    let parsed = parse_source(SOURCE);
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors:\n{}",
        parsed.diagnostics.to_golden()
    );
    assert_eq!(parsed.ast.declarations.len(), 4);
    check_and_lower_with_prelude(&parsed.ast, &BTreeMap::new(), injected_prelude_decls)
        .diagnostics
        .sorted()
}

fn messages(diags: &[Diagnostic]) -> Vec<String> {
    diags
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect()
}

#[test]
fn zero_injected_declarations_suppresses_nothing() {
    let diags = diagnostics(0);
    assert_eq!(
        messages(&diags),
        vec![
            "E2004: public function `user_one` should have explicit return type",
            "E3003: declared effect `net` was not observed",
            "E2004: public function `user_two` should have explicit return type",
            "E3003: declared effect `net` was not observed",
            "E2004: public function `prelude_one` should have explicit return type",
            "E3003: declared effect `net` was not observed",
            "E2004: public function `prelude_two` should have explicit return type",
            "E3003: declared effect `net` was not observed",
        ],
        "diagnostics: {diags:?}"
    );
}

#[test]
fn injected_tail_declarations_are_silent() {
    let diags = diagnostics(2);
    assert_eq!(
        messages(&diags),
        vec![
            "E2004: public function `user_one` should have explicit return type",
            "E3003: declared effect `net` was not observed",
            "E2004: public function `user_two` should have explicit return type",
            "E3003: declared effect `net` was not observed",
        ],
        "only the two user declarations may report; diagnostics: {diags:?}"
    );
}

#[test]
fn default_entry_point_matches_zero_injected_declarations() {
    let parsed = parse_source(SOURCE);
    let via_ns = check_and_lower_with_ns(&parsed.ast, &BTreeMap::new())
        .diagnostics
        .sorted();
    assert_eq!(messages(&via_ns), messages(&diagnostics(0)));
}

/// Effect propagation must still walk the prelude: a user function that calls a
/// prelude function inherits its effects, so `@effect`-related diagnostics on
/// the *user's* function stay accurate even though the prelude itself is mute.
#[test]
fn prelude_effects_still_propagate_into_user_functions() {
    let src = r#"pub caller() -> Int {
  prelude_worker()
}

pub prelude_worker() -> Int {
  @effect io
  println("x")
  0
}
"#;
    let parsed = parse_source(src);
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors:\n{}",
        parsed.diagnostics.to_golden()
    );
    // The last declaration stands in for the injected prelude.
    let diags = check_and_lower_with_prelude(&parsed.ast, &BTreeMap::new(), 1)
        .diagnostics
        .sorted();
    assert_eq!(
        messages(&diags),
        vec![
            "E3002: observed effect `io` is not declared in `@effect` (including transitive calls)",
            "E3101: call to `prelude_worker` requires transitive effect `io` to be declared",
        ],
        "the user's caller must still be told about the inherited effect; diagnostics: {diags:?}"
    );
}

/// A user declaration that collides with a prelude name still reports the
/// redefinition: suppression drops warnings and infos, never errors.
#[test]
fn name_collision_with_user_declaration_is_reported() {
    let src = r#"pub helper() -> Int {
  1
}

pub helper() -> Int {
  2
}
"#;
    let parsed = parse_source(src);
    let diags = check_and_lower_with_prelude(&parsed.ast, &BTreeMap::new(), 1)
        .diagnostics
        .sorted();
    assert!(
        messages(&diags).contains(&"E2002: duplicate function `helper`".to_string()),
        "duplicate definition must survive; diagnostics: {diags:?}"
    );
}

/// Suppression must never turn a loud failure into a silent miscompile: an
/// error inside a prelude declaration is still reported, because the compiler
/// would otherwise lower a function whose body does not match its signature.
#[test]
fn prelude_errors_are_never_suppressed() {
    let src = r#"pub main() -> Int {
  0
}

pub broken_ret() -> Int {
  "not an int"
}
"#;
    let parsed = parse_source(src);
    // The last declaration stands in for an injected prelude declaration.
    let diags = check_and_lower_with_prelude(&parsed.ast, &BTreeMap::new(), 1)
        .diagnostics
        .sorted();
    assert!(
        messages(&diags).contains(
            &"E2201: return type mismatch in `broken_ret`: declared `Int`, inferred `Str`"
                .to_string()
        ),
        "a prelude-origin error must survive; diagnostics: {diags:?}"
    );
    assert!(
        diags
            .iter()
            .any(|d| d.severity == vibe_diagnostics::Severity::Error),
        "the unit must still fail to compile; diagnostics: {diags:?}"
    );
}

/// Regression: the prelude-owned set is keyed per declaration index, and the
/// one name-based carve-out it keeps compares functions against functions. A
/// user *type* that happens to share a prelude *function*'s name must not
/// resurrect that function's warnings, which would point past the end of the
/// user's file at source they cannot open.
#[test]
fn user_type_sharing_a_prelude_function_name_stays_silent() {
    let src = r#"pub main() -> Int {
  0
}

type spawn {
  x: Int
}

pub spawn() {
  @effect net
  1
}
"#;
    let parsed = parse_source(src);
    // The trailing function stands in for the injected prelude's `spawn`.
    let diags = check_and_lower_with_prelude(&parsed.ast, &BTreeMap::new(), 1)
        .diagnostics
        .sorted();
    assert_eq!(
        messages(&diags),
        Vec::<String>::new(),
        "a user type colliding with a prelude function name must not un-mute it; \
         diagnostics: {diags:?}"
    );
}
