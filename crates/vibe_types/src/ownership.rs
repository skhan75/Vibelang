// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use vibe_ast::{Expr, Stmt};
use vibe_diagnostics::{Diagnostic, Diagnostics, Severity, Span};

use crate::closure_support::fn_literal_capture_type_kinds;
use crate::TypeKind;

/// Captures allowed inside a `go` closure body. Function values (`fn(...) -> T`) are permitted
/// here so callers can invoke a passed task/worker inside the spawned task; they remain
/// non-sendable for `chan.send` and other cross-task value transfers.
fn is_sendable_go_closure_capture(ty: &TypeKind) -> bool {
    match ty {
        TypeKind::Fn(_, _) => true,
        _ => is_sendable_type(ty),
    }
}

pub fn is_sendable_type(ty: &TypeKind) -> bool {
    match ty {
        TypeKind::Int
        | TypeKind::Float
        | TypeKind::Bool
        | TypeKind::Str
        | TypeKind::Json
        | TypeKind::JsonBuilder
        | TypeKind::Void => true,
        TypeKind::List(inner) => is_sendable_type(inner),
        TypeKind::Map(key, value) => is_sendable_type(key) && is_sendable_type(value),
        TypeKind::Result(ok, err) => is_sendable_type(ok) && is_sendable_type(err),
        TypeKind::Chan(_) => true,
        TypeKind::UserType(_) | TypeKind::Enum(_) => true,
        // Unknown types are treated as non-sendable so unresolved values do not silently cross
        // concurrency boundaries.
        TypeKind::Unknown => false,
        // Function values may close over non-sendable state; checked separately for known closures.
        TypeKind::Fn(_, _) => false,
        TypeKind::TypeParam(_) => false,
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
        | Expr::DotResult { .. }
        | Expr::Constructor { .. } => false,
        Expr::EnumVariant { fields, .. } => {
            fields.iter().any(|(_, e)| expr_contains_member_access(e))
        }
        Expr::FnLiteral { body, tail_expr, .. } => {
            body.iter().any(|s| stmt_contains_member_access(s))
                || tail_expr
                    .as_ref()
                    .is_some_and(|e| expr_contains_member_access(e.as_ref()))
        }
    }
}

fn stmt_contains_member_access(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Binding { expr, .. }
        | Stmt::ExprStmt { expr, .. }
        | Stmt::Return { expr, .. }
        | Stmt::Go { expr, .. }
        | Stmt::Thread { expr, .. } => expr_contains_member_access(expr),
        Stmt::Assignment { target, expr, .. } => {
            expr_contains_member_access(target) || expr_contains_member_access(expr)
        }
        Stmt::For { iter, body, .. } | Stmt::While { cond: iter, body, .. } => {
            expr_contains_member_access(iter) || body.iter().any(stmt_contains_member_access)
        }
        Stmt::Repeat { count, body, .. } => {
            expr_contains_member_access(count) || body.iter().any(stmt_contains_member_access)
        }
        Stmt::If {
            cond,
            then_body,
            else_body,
            ..
        } => {
            expr_contains_member_access(cond)
                || then_body.iter().any(stmt_contains_member_access)
                || else_body.iter().any(stmt_contains_member_access)
        }
        Stmt::Select { cases, .. } => cases.iter().any(|c| {
            matches!(&c.pattern, vibe_ast::SelectPattern::Receive { expr, .. } if expr_contains_member_access(expr))
                || expr_contains_member_access(&c.action)
        }),
        Stmt::Match {
            scrutinee,
            arms,
            default_action,
            ..
        } => {
            expr_contains_member_access(scrutinee)
                || arms.iter().any(|a| {
                    expr_contains_member_access(&a.pattern) || expr_contains_member_access(&a.action)
                })
                || default_action
                    .as_ref()
                    .is_some_and(|e| expr_contains_member_access(e))
        }
        Stmt::Break { .. } | Stmt::Continue { .. } => false,
    }
}

pub fn check_go_sendability(
    expr: &Expr,
    env: &BTreeMap<String, TypeKind>,
    expr_type_hint: impl Fn(&Expr, &BTreeMap<String, TypeKind>) -> TypeKind,
    closure_binding_meta: &BTreeMap<String, Vec<TypeKind>>,
    diagnostics: &mut Diagnostics,
) {
    let Expr::Call { callee, args, .. } = expr else {
        return;
    };

    let mut check_captures = |caps: &[TypeKind], err_span: vibe_diagnostics::Span| {
        for ty in caps {
            if !is_sendable_go_closure_capture(ty) {
                diagnostics.push(Diagnostic::new(
                    "E3205",
                    Severity::Error,
                    format!(
                        "non-sendable captured value in `go` closure: `{}`",
                        type_name(ty)
                    ),
                    err_span,
                ));
            }
        }
    };

    match &**callee {
        Expr::Ident { name, span: sp } => {
            if let Some(caps) = closure_binding_meta.get(name) {
                check_captures(caps, *sp);
            }
        }
        lit @ Expr::FnLiteral { span: sp, .. } => {
            if let Some(caps) = fn_literal_capture_type_kinds(lit, env) {
                check_captures(&caps, *sp);
            }
        }
        _ => {}
    }

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
            | Stmt::ExprStmt { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Match { .. } => {}
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
        TypeKind::Json => "Json".to_string(),
        TypeKind::JsonBuilder => "JsonBuilder".to_string(),
        TypeKind::List(inner) => format!("List<{}>", type_name(inner)),
        TypeKind::Map(key, value) => format!("Map<{}, {}>", type_name(key), type_name(value)),
        TypeKind::Result(ok, err) => format!("Result<{}, {}>", type_name(ok), type_name(err)),
        TypeKind::Chan(inner) => format!("Chan<{}>", type_name(inner)),
        TypeKind::UserType(name) => name.clone(),
        TypeKind::Enum(name) => name.clone(),
        TypeKind::Void => "Void".to_string(),
        TypeKind::Unknown => "Unknown".to_string(),
        TypeKind::Fn(ps, r) => {
            let inner: Vec<String> = ps.iter().map(type_name).collect();
            format!("fn({})->{}", inner.join(","), type_name(r))
        }
        TypeKind::TypeParam(name) => name.clone(),
    }
}
