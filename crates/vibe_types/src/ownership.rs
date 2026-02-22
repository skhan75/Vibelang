use std::collections::BTreeMap;

use vibe_ast::{Expr, Stmt};
use vibe_diagnostics::{Diagnostic, Diagnostics, Severity, Span};

use crate::TypeKind;

pub fn is_sendable_type(ty: &TypeKind) -> bool {
    match ty {
        TypeKind::Int | TypeKind::Float | TypeKind::Bool | TypeKind::Str | TypeKind::Void => true,
        TypeKind::List(inner) => is_sendable_type(inner),
        TypeKind::Map(key, value) => is_sendable_type(key) && is_sendable_type(value),
        TypeKind::Result(ok, err) => is_sendable_type(ok) && is_sendable_type(err),
        TypeKind::Chan(_) => true,
        // Unknown types are treated as non-sendable so unresolved values do not silently cross
        // concurrency boundaries.
        TypeKind::Unknown => false,
    }
}

pub fn expr_contains_member_access(expr: &Expr) -> bool {
    match expr {
        Expr::Member { .. } => true,
        Expr::Call { callee, args, .. } => {
            expr_contains_member_access(callee) || args.iter().any(expr_contains_member_access)
        }
        Expr::Binary { left, right, .. } => {
            expr_contains_member_access(left) || expr_contains_member_access(right)
        }
        Expr::Index { object, index, .. } => {
            expr_contains_member_access(object) || expr_contains_member_access(index)
        }
        Expr::Slice {
            object, start, end, ..
        } => {
            expr_contains_member_access(object)
                || start
                    .as_ref()
                    .is_some_and(|e| expr_contains_member_access(e))
                || end.as_ref().is_some_and(|e| expr_contains_member_access(e))
        }
        Expr::Unary { expr, .. }
        | Expr::Async { expr, .. }
        | Expr::Await { expr, .. }
        | Expr::Question { expr, .. }
        | Expr::Old { expr, .. } => expr_contains_member_access(expr),
        Expr::List { items, .. } => items.iter().any(expr_contains_member_access),
        Expr::Map { entries, .. } => entries
            .iter()
            .any(|(k, v)| expr_contains_member_access(k) || expr_contains_member_access(v)),
        Expr::Ident { .. }
        | Expr::Int { .. }
        | Expr::Float { .. }
        | Expr::Bool { .. }
        | Expr::String { .. }
        | Expr::DotResult { .. } => false,
    }
}

pub fn check_go_sendability(
    expr: &Expr,
    env: &BTreeMap<String, TypeKind>,
    expr_type_hint: impl Fn(&Expr, &BTreeMap<String, TypeKind>) -> TypeKind,
    diagnostics: &mut Diagnostics,
) {
    let Expr::Call { args, .. } = expr else {
        return;
    };

    for arg in args {
        let inferred = expr_type_hint(arg, env);
        if !is_sendable_type(&inferred) {
            diagnostics.push(Diagnostic::new(
                "E3201",
                Severity::Error,
                format!(
                    "non-sendable value passed to `go`: inferred `{}`",
                    type_name(&inferred)
                ),
                arg.span(),
            ));
        }
        if expr_contains_member_access(arg) {
            diagnostics.push(Diagnostic::new(
                "E3202",
                Severity::Error,
                "capturing member access in `go` may alias shared mutable state; use explicit synchronization",
                arg.span(),
            ));
        }
    }
}

pub fn check_shared_mutation_in_concurrent_context(
    body: &[Stmt],
    has_concurrency: bool,
    diagnostics: &mut Diagnostics,
    function_span: Span,
) {
    if !has_concurrency {
        return;
    }
    if !contains_member_assignment(body) {
        return;
    }
    diagnostics.push(Diagnostic::new(
        "E3203",
        Severity::Error,
        "shared mutable member assignment in concurrent function requires explicit synchronization primitive",
        function_span,
    ));
}

fn contains_member_assignment(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        match stmt {
            Stmt::Assignment { target, .. } => {
                if matches!(target, Expr::Member { .. }) {
                    return true;
                }
            }
            Stmt::For { body, .. } | Stmt::While { body, .. } | Stmt::Repeat { body, .. } => {
                if contains_member_assignment(body) {
                    return true;
                }
            }
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                if contains_member_assignment(then_body) || contains_member_assignment(else_body) {
                    return true;
                }
            }
            Stmt::Select { .. }
            | Stmt::Go { .. }
            | Stmt::Thread { .. }
            | Stmt::Binding { .. }
            | Stmt::Return { .. }
            | Stmt::ExprStmt { .. } => {}
        }
    }
    false
}

fn type_name(t: &TypeKind) -> String {
    match t {
        TypeKind::Int => "Int".to_string(),
        TypeKind::Float => "Float".to_string(),
        TypeKind::Bool => "Bool".to_string(),
        TypeKind::Str => "Str".to_string(),
        TypeKind::List(inner) => format!("List<{}>", type_name(inner)),
        TypeKind::Map(key, value) => format!("Map<{}, {}>", type_name(key), type_name(value)),
        TypeKind::Result(ok, err) => format!("Result<{}, {}>", type_name(ok), type_name(err)),
        TypeKind::Chan(inner) => format!("Chan<{}>", type_name(inner)),
        TypeKind::Void => "Void".to_string(),
        TypeKind::Unknown => "Unknown".to_string(),
    }
}
