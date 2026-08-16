// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Immutability by default (`docs/spec/mutability_model.md`).
//!
//! A binding is immutable unless it is declared `mut`. Reassigning an
//! immutable binding is `E2110`; mutating a field through one is `E2111`;
//! calling an in-place container method through one is `E2112`.
//! Re-binding with `:=` is shadowing, not reassignment, and stays legal.

use vibe_ast::{Param, Stmt};
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
fn reassigning_immutable_local_is_rejected_with_span() {
    let diags = check(
        r#"pub main() -> Int {
  x := 1
  x = 2
  x
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2110"], "diagnostics: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(
        d.message,
        "cannot assign to immutable binding `x`; declared at line 2, column 3 \
         — declare it `mut x := ...` to allow mutation"
    );
    assert_eq!(d.span.line_start, 3, "span points at the reassignment");
    assert_eq!(d.span.col_start, 3);
}

#[test]
fn reassigning_mut_local_is_accepted() {
    let diags = check(
        r#"pub main() -> Int {
  mut x := 1
  x = 2
  x
}
"#,
    );
    assert!(diags.is_empty(), "diagnostics: {diags:?}");
}

#[test]
fn parser_records_the_mutability_bit() {
    let parsed = parse_source(
        r#"pub main() -> Int {
  mut x := 1
  y := 2
  x
}
"#,
    );
    assert!(!parsed.diagnostics.has_errors());
    let vibe_ast::Declaration::Function(func) = &parsed.ast.declarations[0] else {
        panic!("expected a function declaration");
    };
    let flags: Vec<(String, bool)> = func
        .body
        .iter()
        .filter_map(|s| match s {
            Stmt::Binding { name, is_mut, .. } => Some((name.clone(), *is_mut)),
            _ => None,
        })
        .collect();
    assert_eq!(
        flags,
        vec![("x".to_string(), true), ("y".to_string(), false)]
    );
}

#[test]
fn rebinding_with_walrus_is_shadowing_not_reassignment() {
    // `x := ...` a second time introduces a fresh binding, which the model
    // documents as legal and distinct from reassignment.
    let plain = check(
        r#"pub main() -> Int {
  x := 1
  x := 2
  x
}
"#,
    );
    assert!(plain.is_empty(), "diagnostics: {plain:?}");

    // The escape hatch: shadowing an immutable binding with a `mut` one makes
    // the *new* binding writable without touching the old declaration.
    let upgraded = check(
        r#"pub main() -> Int {
  x := 1
  mut x := x
  x = 2
  x
}
"#,
    );
    assert!(upgraded.is_empty(), "diagnostics: {upgraded:?}");
}

#[test]
fn rebinding_resets_mutability_to_immutable() {
    // The fresh binding carries its own mutability, so dropping `mut` on a
    // re-bind makes later reassignment an error again.
    let diags = check(
        r#"pub main() -> Int {
  mut x := 1
  x = 2
  x := 3
  x = 4
  x
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2110"], "diagnostics: {diags:?}");
    assert_eq!(diags[0].span.line_start, 5, "only the post-rebind write");
    assert!(
        diags[0].message.contains("declared at line 4"),
        "message points at the immutable re-bind: {}",
        diags[0].message
    );
}

#[test]
fn parameters_are_immutable_by_default() {
    let diags = check(
        r#"pub bump(n: Int) -> Int {
  n = n + 1
  n
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2110"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "cannot assign to immutable binding `n`; declared at line 1, column 10 \
         — declare the parameter `mut n` to allow mutation"
    );
}

#[test]
fn mut_parameter_reassignment_is_accepted() {
    let diags = check(
        r#"pub bump(mut n: Int) -> Int {
  n = n + 1
  n
}
"#,
    );
    assert!(diags.is_empty(), "diagnostics: {diags:?}");
    let parsed = parse_source("pub bump(mut n: Int, m: Int) -> Int {\n  n\n}\n");
    let vibe_ast::Declaration::Function(func) = &parsed.ast.declarations[0] else {
        panic!("expected a function declaration");
    };
    let flags: Vec<bool> = func.params.iter().map(|p: &Param| p.is_mut).collect();
    assert_eq!(flags, vec![true, false]);
}

#[test]
fn field_mutation_through_immutable_parameter_is_rejected() {
    // The audit's measured hole: `b.v = ...` used to compile with `b` not
    // declared `mut`.
    let diags = check(
        r#"type Box {
  v: Int,
}

pub bump(b: Box) -> Int {
  @effect mut_state
  b.v = b.v + 1
  b.v
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2111"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "cannot mutate through immutable binding `b`; declared at line 5, column 10 \
         — declare the parameter `mut b` to allow mutation"
    );
    assert_eq!(diags[0].span.line_start, 7);
}

#[test]
fn field_mutation_through_mut_parameter_is_accepted() {
    let diags = check(
        r#"type Box {
  v: Int,
}

pub bump(mut b: Box) -> Int {
  @effect mut_state
  b.v = b.v + 1
  b.v
}
"#,
    );
    assert!(diags.is_empty(), "diagnostics: {diags:?}");
}

#[test]
fn field_mutation_through_immutable_local_is_rejected() {
    let diags = check(
        r#"type Box {
  v: Int,
}

pub main() -> Int {
  @effect mut_state
  @effect alloc
  b := Box { v: 1 }
  b.v = 2
  b.v
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2111"], "diagnostics: {diags:?}");
    assert!(
        diags[0].message.contains("declare it `mut b := ...`"),
        "local hint expected: {}",
        diags[0].message
    );
}

#[test]
fn for_loop_variable_is_immutable_and_has_no_mut_form() {
    let diags = check(
        r#"pub main() -> Int {
  @effect alloc
  values := [1, 2]
  for v in values {
    v = 0
  }
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2110"], "diagnostics: {diags:?}");
    assert!(
        diags[0].message.contains("bind a mutable copy instead"),
        "pattern-bound hint expected: {}",
        diags[0].message
    );
}

#[test]
fn closure_body_respects_the_capture_mutability() {
    let rejected = check(
        r#"pub main() -> Int {
  total := 0
  f := fn () -> Int {
    total = total + 1
    total
  }
  f()
}
"#,
    );
    assert_eq!(codes(&rejected), vec!["E2110"], "diagnostics: {rejected:?}");

    let accepted = check(
        r#"pub main() -> Int {
  mut total := 0
  f := fn () -> Int {
    total = total + 1
    total
  }
  f()
}
"#,
    );
    assert!(accepted.is_empty(), "diagnostics: {accepted:?}");
}

#[test]
fn generic_function_body_is_reported_exactly_once() {
    // The template and every monomorph built from it share one body; the pass
    // must not report the same write once per instantiation.
    let diags = check(
        r#"identity<T>(v: T) -> T {
  v = v
  v
}

pub main() -> Int {
  identity(1)
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2110"], "diagnostics: {diags:?}");
}

#[test]
fn book_bank_transfer_shape_compiles() {
    // `book/src/ch06_contracts_and_intent.md` — mutable parameters plus field
    // writes through them, with `old()` postconditions.
    let diags = check(
        r#"type Account {
  id: Str,
  balance: Int,
}

pub transfer(mut from: Account, mut to: Account, amount: Int) -> Bool {
  @intent "move amount from source account to destination account"
  @effect mut_state
  @require amount > 0
  @require from.balance >= amount
  @require from.id != to.id
  @ensure from.balance == old(from.balance) - amount
  @ensure to.balance == old(to.balance) + amount

  from.balance = from.balance - amount
  to.balance = to.balance + amount
  true
}
"#,
    );
    assert!(diags.is_empty(), "diagnostics: {diags:?}");
}

#[test]
fn book_bank_transfer_without_mut_params_is_rejected() {
    let diags = check(
        r#"type Account {
  id: Str,
  balance: Int,
}

pub transfer(from: Account, to: Account, amount: Int) -> Bool {
  @effect mut_state
  from.balance = from.balance - amount
  to.balance = to.balance + amount
  true
}
"#,
    );
    assert_eq!(
        codes(&diags),
        vec!["E2111", "E2111"],
        "diagnostics: {diags:?}"
    );
    assert_eq!(diags[0].span.line_start, 8);
    assert_eq!(diags[1].span.line_start, 9);
}

#[test]
fn container_append_through_immutable_local_is_rejected() {
    // The measured hole this closes: before `E2112`, an immutable binding's
    // list could be grown in place with zero diagnostics.
    let diags = check(
        r#"pub main() -> Int {
  @effect alloc
  @effect mut_state
  xs := [10, 20]
  xs.append(30)
  xs.len()
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2112"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "`.append(...)` mutates its receiver in place, but `xs` is an immutable binding; \
         declared at line 4, column 3 — declare it `mut xs := ...` to allow mutation"
    );
    assert_eq!(diags[0].span.line_start, 5, "span points at the call");
}

#[test]
fn container_mutation_through_mut_local_is_accepted() {
    let diags = check(
        r#"pub main() -> Int {
  @effect alloc
  @effect mut_state
  mut xs := [10, 20]
  xs.append(30)
  xs.set(0, 99)
  mut m := {1: 2}
  m.set(3, 4)
  m.remove(1)
  xs.len()
}
"#,
    );
    assert!(diags.is_empty(), "diagnostics: {diags:?}");
}

#[test]
fn every_in_place_container_method_is_gated() {
    // `set` is VibeLang's index assignment and `remove` is its map delete, so
    // all three mutators have to be covered, not just `append`.
    let diags = check(
        r#"pub main() -> Int {
  @effect alloc
  @effect mut_state
  xs := [10, 20]
  xs.set(0, 99)
  m := {1: 2}
  m.set(3, 4)
  m.remove(1)
  xs.len()
}
"#,
    );
    assert_eq!(
        codes(&diags),
        vec!["E2112", "E2112", "E2112"],
        "diagnostics: {diags:?}"
    );
    assert!(diags[0].message.starts_with("`.set(...)`"));
    assert!(diags[2].message.starts_with("`.remove(...)`"));
}

#[test]
fn container_reads_are_not_gated() {
    // Guard against over-gating: `sort_desc` and `take` build fresh containers
    // and leave the receiver alone (measured), so they are reads.
    let diags = check(
        r#"pub main() -> Int {
  @effect alloc
  ys := [3, 1, 2]
  sorted := ys.sort_desc()
  head := ys.take(1)
  sorted.len() + head.len() + ys.len() + ys.get(0)
}
"#,
    );
    assert!(diags.is_empty(), "diagnostics: {diags:?}");
}

#[test]
fn channel_operations_do_not_require_a_mut_binding() {
    // Decided in `docs/spec/mutability_model.md`: a channel handle is a shared
    // endpoint, not a value the binding owns.
    let diags = check(
        r#"pub main() -> Int {
  @effect concurrency
  @effect alloc
  ch := chan(1)
  ch.send(7)
  v := ch.recv()
  ch.close()
  v
}
"#,
    );
    assert!(diags.is_empty(), "diagnostics: {diags:?}");
}

#[test]
fn container_mutation_through_immutable_parameter_is_rejected() {
    let diags = check(
        r#"pub grow(zs: List<Int>) -> Int {
  @effect alloc
  @effect mut_state
  zs.append(7)
  zs.len()
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2112"], "diagnostics: {diags:?}");
    assert!(
        diags[0].message.contains("declare the parameter `mut zs`"),
        "parameter hint expected: {}",
        diags[0].message
    );
}

#[test]
fn nested_container_mutation_is_charged_to_the_root_binding() {
    // `.get(...)` hands back a view, so growing the inner list is a write to
    // data `rows` owns and needs `mut rows`.
    let rejected = check(
        r#"pub main() -> Int {
  @effect alloc
  @effect mut_state
  rows := [[1], [2]]
  rows.get(0).append(9)
  rows.len()
}
"#,
    );
    assert_eq!(codes(&rejected), vec!["E2112"], "diagnostics: {rejected:?}");
    assert!(
        rejected[0]
            .message
            .contains("`rows` is an immutable binding"),
        "root binding named: {}",
        rejected[0].message
    );

    let accepted = check(
        r#"pub main() -> Int {
  @effect alloc
  @effect mut_state
  mut rows := [[1], [2]]
  rows.get(0).append(9)
  rows.len()
}
"#,
    );
    assert!(accepted.is_empty(), "diagnostics: {accepted:?}");

    // The bracket spelling of the same read reaches the same root.
    let bracketed = check(
        r#"pub main() -> Int {
  @effect alloc
  @effect mut_state
  rows := [[1], [2]]
  rows[0].append(9)
  rows.len()
}
"#,
    );
    assert_eq!(
        codes(&bracketed),
        vec!["E2112"],
        "diagnostics: {bracketed:?}"
    );
}

#[test]
fn book_ch09_append_section_matches_the_compiler() {
    // `book/src/ch09_collections.md` §9.2.2 prints this exact program and this
    // exact diagnostic; it used to print a fabricated `error[E0305]` block for
    // a rule the compiler did not have.
    let rejected = check(
        r#"pub main() -> Int {
  @effect alloc
  @effect mut_state
  scores := [90, 85, 92]
  scores.append(88)
  scores.len()
}
"#,
    );
    assert_eq!(codes(&rejected), vec!["E2112"], "diagnostics: {rejected:?}");
    assert_eq!(
        rejected[0].message,
        "`.append(...)` mutates its receiver in place, but `scores` is an immutable binding; \
         declared at line 4, column 3 — declare it `mut scores := ...` to allow mutation"
    );

    let accepted = check(
        r#"pub main() -> Int {
  @effect alloc
  @effect mut_state
  mut scores := [90, 85, 92]
  scores.append(88)
  scores.append(95)
  scores.len()
}
"#,
    );
    assert!(accepted.is_empty(), "diagnostics: {accepted:?}");
}

#[test]
fn book_ch06_rate_limiter_shape_compiles() {
    // `book/src/ch06_contracts_and_intent.md` §6.12: one `mut` parameter has to
    // license both the field write and the in-place append reached through it.
    let accepted = check(
        r#"type RateLimiter {
  max_requests: Int,
  window_ms: Int,
  requests: List<Int>,
}

pub allow_request(mut limiter: RateLimiter, now_ms: Int) -> Bool {
  @effect mut_state
  @effect alloc
  limiter.window_ms = now_ms
  limiter.requests.append(now_ms)
  true
}
"#,
    );
    assert!(accepted.is_empty(), "diagnostics: {accepted:?}");

    let rejected = check(
        r#"type RateLimiter {
  max_requests: Int,
  window_ms: Int,
  requests: List<Int>,
}

pub allow_request(limiter: RateLimiter, now_ms: Int) -> Bool {
  @effect mut_state
  @effect alloc
  limiter.window_ms = now_ms
  limiter.requests.append(now_ms)
  true
}
"#,
    );
    assert_eq!(
        codes(&rejected),
        vec!["E2111", "E2112"],
        "both writes through the same non-mut parameter are reported: {rejected:?}"
    );
}

#[test]
fn select_receive_name_gets_one_consistent_diagnostic() {
    // The receive binding exists, so `E2101` ("unknown variable") must not fire
    // alongside `E2110` ("declared at line ..."): the two contradicted each
    // other about whether the name was bound.
    let diags = check(
        r#"pub main() -> Int {
  @effect concurrency
  @effect nondet
  @effect alloc
  signal := chan(1)
  select {
    case value := signal.recv() => value
    case after 1 => 0
  }
  value = 9
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2110"], "diagnostics: {diags:?}");
    assert!(
        diags[0].message.contains("`select` receive binder"),
        "binder named precisely: {}",
        diags[0].message
    );
}

#[test]
fn implicit_binder_hint_names_the_binder_that_bound_the_name() {
    // Scopes are flat, so an outer `mut i` can be superseded by a `for` binder
    // of the same name. The hint has to describe the binding it is actually
    // pointing at, not "the surrounding pattern".
    let diags = check(
        r#"pub main() -> Int {
  @effect alloc
  mut i := 0
  values := [1, 2]
  for i in values {
    i
  }
  i = 5
  i
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2110"], "diagnostics: {diags:?}");
    assert!(
        diags[0]
            .message
            .contains("by a `for` loop binder, which has no `mut` form"),
        "binder named precisely: {}",
        diags[0].message
    );
}

#[test]
fn diagnostics_are_deterministic_across_runs() {
    let src = r#"pub main() -> Int {
  a := 1
  b := 2
  b = 3
  a = 4
  a + b
}
"#;
    let first: Vec<String> = check(src)
        .iter()
        .map(|d| format!("{}@{}:{}", d.code, d.span.line_start, d.span.col_start))
        .collect();
    let second: Vec<String> = check(src)
        .iter()
        .map(|d| format!("{}@{}:{}", d.code, d.span.line_start, d.span.col_start))
        .collect();
    assert_eq!(first, vec!["E2110@4:3", "E2110@5:3"]);
    assert_eq!(first, second);
}
