// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Grammar for the `mut` modifier: it introduces a binding (`mut x := e`) or
//! marks a parameter (`fn f(mut a: T)`). Anything else is `E1213` rather than a
//! confusing expression error.

use vibe_ast::{Declaration, Expr, Stmt};
use vibe_diagnostics::Severity;
use vibe_parser::parse_source;

fn error_codes(src: &str) -> Vec<String> {
    parse_source(src)
        .diagnostics
        .sorted()
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

#[test]
fn mut_binding_parses_without_errors() {
    let out = parse_source("pub main() -> Int {\n  mut x := 1\n  x\n}\n");
    assert!(
        !out.diagnostics.has_errors(),
        "unexpected diagnostics:\n{}",
        out.diagnostics.to_golden()
    );
    let Declaration::Function(func) = &out.ast.declarations[0] else {
        panic!("expected a function declaration");
    };
    assert!(matches!(
        &func.body[0],
        Stmt::Binding { name, is_mut: true, .. } if name == "x"
    ));
}

#[test]
fn mut_is_reserved_and_cannot_be_used_as_an_identifier() {
    // `mut` lexes as a keyword now, so a bare `mut := 1` is a grammar error
    // rather than a binding named `mut`.
    assert_eq!(
        error_codes("pub main() -> Int {\n  mut := 1\n  0\n}\n"),
        vec!["E1213"]
    );
}

#[test]
fn mut_before_a_reassignment_is_rejected() {
    assert_eq!(
        error_codes("pub main() -> Int {\n  x := 1\n  mut x = 2\n  x\n}\n"),
        vec!["E1213"]
    );
}

#[test]
fn bare_mut_in_statement_position_is_rejected() {
    // `mut c` on its own is not a binding, so it is a grammar error rather
    // than a statement that silently evaluates `c`.
    let codes = error_codes("pub main() -> Int {\n  c := 1\n  mut c\n  0\n}\n");
    assert_eq!(codes, vec!["E1213"]);
}

#[test]
fn mut_in_argument_position_is_rejected_without_a_cascade() {
    // `f(n, mut cache)` — the call-site mutable borrow other languages have and
    // VibeLang does not. It has to produce exactly one diagnostic: the operand
    // is still parsed, so the argument list closes normally instead of
    // unwinding into `expected expression` / `expected )` / `unexpected token`.
    let codes = error_codes(
        "pub g(n: Int) -> Int {\n  n\n}\n\npub main() -> Int {\n  c := 1\n  g(mut c)\n}\n",
    );
    assert_eq!(codes, vec!["E1213"]);

    let out = parse_source(
        "pub g(n: Int) -> Int {\n  n\n}\n\npub main() -> Int {\n  c := 1\n  g(mut c)\n}\n",
    );
    let Declaration::Function(main) = &out.ast.declarations[1] else {
        panic!("expected a function declaration");
    };
    // Recovery keeps the call, with `c` as its single argument.
    let tail = main.tail_expr.as_ref().expect("tail expression");
    let Expr::Call { args, .. } = tail else {
        panic!("expected the call to survive recovery, got {tail:?}");
    };
    assert!(
        matches!(&args[..], [Expr::Ident { name, .. }] if name == "c"),
        "argument recovered as the bare operand: {args:?}"
    );
}

#[test]
fn type_annotated_local_binding_says_so() {
    // `mut result: List<Int> := []` appears in the book but the grammar takes
    // only inferred locals. The diagnostic has to name that gap rather than
    // send the reader hunting for a typo.
    let out = parse_source("pub main() -> Int {\n  mut result: List<Int> := []\n  0\n}\n");
    let first = out
        .diagnostics
        .sorted()
        .into_iter()
        .find(|d| d.code == "E1213")
        .expect("E1213 reported");
    assert_eq!(
        first.message,
        "type-annotated local bindings are not supported yet: write \
         `mut name := expr` and annotate the value instead"
    );

    // The generic wording still covers every other malformed shape.
    let other = parse_source("pub main() -> Int {\n  x := 1\n  mut x = 2\n  x\n}\n");
    let generic = other
        .diagnostics
        .sorted()
        .into_iter()
        .find(|d| d.code == "E1213")
        .expect("E1213 reported");
    assert_eq!(
        generic.message,
        "`mut` must introduce a binding: write `mut name := expr`"
    );
}

#[test]
fn mut_parameter_parses_and_is_flagged() {
    let out = parse_source("pub f(mut a: Int, b: Int) -> Int {\n  a\n}\n");
    assert!(
        !out.diagnostics.has_errors(),
        "unexpected diagnostics:\n{}",
        out.diagnostics.to_golden()
    );
    let Declaration::Function(func) = &out.ast.declarations[0] else {
        panic!("expected a function declaration");
    };
    assert_eq!(
        func.params
            .iter()
            .map(|p| (p.name.as_str(), p.is_mut))
            .collect::<Vec<_>>(),
        vec![("a", true), ("b", false)]
    );
    assert_eq!(func.params[0].span.line_start, 1);
    assert_eq!(
        func.params[0].span.col_start, 11,
        "param span points at the name, not at `mut`"
    );
}

#[test]
fn mut_function_literal_parameter_parses() {
    let out = parse_source(
        "pub main() -> Int {\n  f := fn (mut a: Int) -> Int {\n    a = a + 1\n    a\n  }\n  f(1)\n}\n",
    );
    assert!(
        !out.diagnostics.has_errors(),
        "unexpected diagnostics:\n{}",
        out.diagnostics.to_golden()
    );
}
