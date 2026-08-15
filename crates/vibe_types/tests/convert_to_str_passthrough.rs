// Copyright 2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! HIR lowering decision for the `convert.to_str(x)` passthrough
//! (audit P0-12): nested cross-namespace stdlib calls such as
//! `convert.to_str(text.byte_len("abc"))` miscompiled to SIGSEGV because the
//! passthrough hint guessed a member call's type from its FIRST ARGUMENT
//! (`"abc"` -> Str), elided the `to_str` call, and let the raw Int flow into
//! Str-consuming natives (`strlen(3)` -> EXC_BAD_ACCESS). The passthrough must
//! consult the stdlib return-type registry for namespace-call arguments:
//! Int-returning inner calls keep the real `to_str` call, while Str-returning
//! inner calls stay a passthrough (Str interpolation depends on that).

use std::collections::BTreeMap;

use vibe_hir::{HirExpr, HirExprKind, HirStmt};
use vibe_parser::parse_source;
use vibe_types::{check_and_lower_with_ns, CheckOutput};

/// Mangled stdlib declarations mirroring what the CLI module resolver injects
/// for native-only stdlib functions (see
/// `crates/vibe_cli/src/module_resolver.rs::load_stdlib_namespace_functions`).
/// Bodies are stand-ins; only the signatures matter for the lowering decision.
const STDLIB_DECLS: &str = r#"
__stdlib_convert__to_str(n: Int) -> Str { "" }
__stdlib_text__byte_len(s: Str) -> Int { 0 }
__stdlib_text__index_of(haystack: Str, needle: Str) -> Int { 0 }
__stdlib_text__trim(s: Str) -> Str { s }
__stdlib_crypto__sha256(s: Str) -> Str { s }
"#;

fn namespace_map() -> BTreeMap<(String, String), String> {
    let mut map = BTreeMap::new();
    for (ns, field) in [
        ("convert", "to_str"),
        ("text", "byte_len"),
        ("text", "index_of"),
        ("text", "trim"),
        ("crypto", "sha256"),
    ] {
        map.insert(
            (ns.to_string(), field.to_string()),
            format!("__stdlib_{ns}__{field}"),
        );
    }
    map
}

fn check(src: &str) -> CheckOutput {
    let full = format!("{src}\n{STDLIB_DECLS}");
    let parsed = parse_source(&full);
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors:\n{}",
        parsed.diagnostics.to_golden()
    );
    check_and_lower_with_ns(&parsed.ast, &namespace_map())
}

fn check_clean(src: &str) -> CheckOutput {
    let out = check(src);
    assert!(
        !out.diagnostics.has_errors(),
        "unexpected type errors:\n{}",
        out.diagnostics.to_golden()
    );
    out
}

/// Count `convert.to_str(...)` calls in the lowered `main` body.
fn to_str_calls_in_main(out: &CheckOutput) -> usize {
    let main = out
        .hir
        .functions
        .iter()
        .find(|f| f.name == "main")
        .expect("lowered main function");
    let mut count = 0;
    for stmt in &main.body {
        count_in_stmt(stmt, &mut count);
    }
    if let Some(tail) = &main.tail_expr {
        count_in_expr(tail, &mut count);
    }
    count
}

fn count_in_stmt(stmt: &HirStmt, count: &mut usize) {
    match stmt {
        HirStmt::Binding { expr, .. }
        | HirStmt::Return { expr }
        | HirStmt::Expr { expr }
        | HirStmt::Go { expr }
        | HirStmt::Thread { expr }
        | HirStmt::ContractCheck { expr, .. } => count_in_expr(expr, count),
        HirStmt::Assignment { target, expr } => {
            count_in_expr(target, count);
            count_in_expr(expr, count);
        }
        HirStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            count_in_expr(cond, count);
            for s in then_body.iter().chain(else_body) {
                count_in_stmt(s, count);
            }
        }
        HirStmt::For { iter, body, .. } => {
            count_in_expr(iter, count);
            for s in body {
                count_in_stmt(s, count);
            }
        }
        HirStmt::While { cond, body } => {
            count_in_expr(cond, count);
            for s in body {
                count_in_stmt(s, count);
            }
        }
        HirStmt::Repeat { count: n, body } => {
            count_in_expr(n, count);
            for s in body {
                count_in_stmt(s, count);
            }
        }
        _ => {}
    }
}

fn count_in_expr(expr: &HirExpr, count: &mut usize) {
    match &expr.kind {
        HirExprKind::Call { callee, args } => {
            if let HirExprKind::Member { object, field } = &callee.kind {
                if field == "to_str"
                    && matches!(&object.kind, HirExprKind::Ident(ns) if ns == "convert")
                {
                    *count += 1;
                }
            }
            count_in_expr(callee, count);
            for a in args {
                count_in_expr(a, count);
            }
        }
        HirExprKind::Member { object, .. } => count_in_expr(object, count),
        HirExprKind::Index { object, index } => {
            count_in_expr(object, count);
            count_in_expr(index, count);
        }
        HirExprKind::Binary { left, right, .. } => {
            count_in_expr(left, count);
            count_in_expr(right, count);
        }
        HirExprKind::Unary { expr, .. }
        | HirExprKind::Async { expr }
        | HirExprKind::Await { expr }
        | HirExprKind::Question { expr }
        | HirExprKind::Old { expr } => count_in_expr(expr, count),
        HirExprKind::List(items) => {
            for e in items {
                count_in_expr(e, count);
            }
        }
        _ => {}
    }
}

#[test]
fn int_returning_ns_call_keeps_to_str_call() {
    // Audit P0-12 repro 1: previously lowered to `println(text.byte_len("abc"))`
    // (passthrough elided the conversion), so println received Int 3 as char*.
    let out = check_clean(
        r#"pub main() -> Int {
  println(convert.to_str(text.byte_len("abc")))
  0
}
"#,
    );
    assert_eq!(to_str_calls_in_main(&out), 1);
}

#[test]
fn int_returning_ns_call_in_binding_keeps_to_str_call() {
    // Audit P0-12 repro 2: the binding form crashed identically.
    let out = check_clean(
        r#"pub main() -> Int {
  s := convert.to_str(text.index_of("abc", "b"))
  println(s)
  0
}
"#,
    );
    assert_eq!(to_str_calls_in_main(&out), 1);
}

#[test]
fn three_deep_nesting_keeps_to_str_call() {
    // Audit P0-12 repro 4: crypto.sha256 returns Str, text.byte_len returns
    // Int, so exactly the outer `to_str` must survive.
    let out = check_clean(
        r#"pub main() -> Int {
  println(convert.to_str(text.byte_len(crypto.sha256("x"))))
  0
}
"#,
    );
    assert_eq!(to_str_calls_in_main(&out), 1);
}

#[test]
fn str_returning_ns_call_stays_passthrough() {
    // The registry says text.trim returns Str, so the passthrough (which Str
    // interpolation relies on) must still elide the `to_str` call.
    let out = check_clean(
        r#"pub main() -> Int {
  println(convert.to_str(text.trim("  hi  ")))
  0
}
"#,
    );
    assert_eq!(to_str_calls_in_main(&out), 0);
}

#[test]
fn str_literal_stays_passthrough() {
    // Interpolation desugars `{x}` to `convert.to_str(x)`; literal Str
    // segments must keep eliding the call.
    let out = check_clean(
        r#"pub main() -> Int {
  println(convert.to_str("hi"))
  0
}
"#,
    );
    assert_eq!(to_str_calls_in_main(&out), 0);
}

#[test]
fn int_literal_keeps_to_str_call() {
    let out = check_clean(
        r#"pub main() -> Int {
  println(convert.to_str(42))
  0
}
"#,
    );
    assert_eq!(to_str_calls_in_main(&out), 1);
}

#[test]
fn int_returning_inner_into_str_taking_outer_is_type_error() {
    // Nesting an Int-returning stdlib call into a Str-taking stdlib parameter
    // must stay a compile-time E2265, not a runtime crash (and not a dodge:
    // the rejection is the declared-signature mismatch).
    let out = check(
        r#"pub main() -> Int {
  s := text.trim(text.byte_len("x"))
  println(s)
  0
}
"#,
    );
    let errors: Vec<_> = out
        .diagnostics
        .sorted()
        .into_iter()
        .filter(|d| d.severity == vibe_diagnostics::Severity::Error)
        .collect();
    assert_eq!(
        errors.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(),
        vec!["E2265"],
        "diagnostics: {errors:?}"
    );
    assert_eq!(
        errors[0].message,
        "argument 1 type mismatch in call to `text.trim`: expected `Str`, got `Int`"
    );
}
