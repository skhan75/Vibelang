// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Immutability-by-default enforcement.
//!
//! `docs/spec/mutability_model.md` requires that a binding is immutable unless
//! it is declared `mut`, and that reassigning an immutable binding is a
//! compile-time error. This pass implements the binding half of that model:
//!
//! - `x := expr` introduces an immutable binding; `mut x := expr` a mutable one.
//! - `fn f(a: T)` takes an immutable parameter; `fn f(mut a: T)` a mutable one.
//! - `x = expr` on an immutable binding is `E2110`.
//! - `x.field = expr` through an immutable binding is `E2111`.
//! - `x.append(v)` / `x.set(k, v)` / `x.remove(k)` through an immutable binding
//!   is `E2112`: those three are the container operations that mutate the
//!   receiver in place, so they need the same permission a field write needs.
//! - Re-binding with `:=` is *not* reassignment: it introduces a fresh binding
//!   that shadows the old one, and the fresh binding carries its own
//!   mutability. That is the documented escape hatch and it stays legal.
//!
//! Known holes, all recorded in `docs/spec/mutability_model.md`: aliasing is
//! not tracked (`mut b := a` hands out a mutable view of an immutable `a`),
//! and a receiver that is not rooted in a name (`f().append(v)`) has no
//! binding to check.
//!
//! Scoping deliberately mirrors [`crate::check_stmt`]: the binding environment
//! is flat, so a name introduced inside an `if`/`for` body stays visible after
//! it, exactly as the type environment already behaves. Using a stricter scope
//! stack here would reject programs the type checker accepts.

use std::collections::BTreeMap;

use vibe_ast::{Expr, FunctionDecl, Param, SelectPattern, Stmt};
use vibe_diagnostics::{Diagnostic, Diagnostics, Severity, Span};

/// How a name entered scope, which decides what the diagnostic tells the user
/// to write to make it mutable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BindingOrigin {
    /// `x := expr` — can become `mut x := expr`.
    Local,
    /// A function or function-literal parameter — can become `mut x: T`.
    Param,
    /// A `for` loop variable, `select` receive binding, or `match` pattern
    /// field. The grammar has no `mut` form for these, so the fix is to bind a
    /// mutable local from them instead. The payload names the binder so the
    /// hint does not misdescribe which declaration it is pointing at.
    Implicit(&'static str),
}

#[derive(Debug, Clone)]
struct BindingInfo {
    is_mut: bool,
    origin: BindingOrigin,
    decl: Span,
}

type Scope = BTreeMap<String, BindingInfo>;

/// Checks one function body for assignments to immutable bindings.
pub(crate) fn check_function_mutability(func: &FunctionDecl, diagnostics: &mut Diagnostics) {
    let mut scope = Scope::new();
    seed_params(&func.params, &mut scope);
    check_block(&func.body, &mut scope, diagnostics);
    if let Some(tail) = &func.tail_expr {
        check_expr(tail, &scope, diagnostics);
    }
}

fn seed_params(params: &[Param], scope: &mut Scope) {
    for p in params {
        scope.insert(
            p.name.clone(),
            BindingInfo {
                is_mut: p.is_mut,
                origin: BindingOrigin::Param,
                decl: p.span,
            },
        );
    }
}

fn declare_implicit(name: &str, binder: &'static str, span: Span, scope: &mut Scope) {
    scope.insert(
        name.to_string(),
        BindingInfo {
            is_mut: false,
            origin: BindingOrigin::Implicit(binder),
            decl: span,
        },
    );
}

fn check_block(stmts: &[Stmt], scope: &mut Scope, diagnostics: &mut Diagnostics) {
    for stmt in stmts {
        check_stmt(stmt, scope, diagnostics);
    }
}

fn check_stmt(stmt: &Stmt, scope: &mut Scope, diagnostics: &mut Diagnostics) {
    match stmt {
        Stmt::Binding {
            name,
            is_mut,
            expr,
            span,
        } => {
            check_expr(expr, scope, diagnostics);
            // A fresh `:=` replaces any earlier binding of the same name,
            // including its mutability. This is shadowing, not reassignment.
            scope.insert(
                name.clone(),
                BindingInfo {
                    is_mut: *is_mut,
                    origin: BindingOrigin::Local,
                    decl: *span,
                },
            );
        }
        Stmt::Assignment { target, expr, span } => {
            check_expr(expr, scope, diagnostics);
            check_assign_target(target, *span, scope, diagnostics);
        }
        Stmt::Return { expr, .. }
        | Stmt::ExprStmt { expr, .. }
        | Stmt::Go { expr, .. }
        | Stmt::Thread { expr, .. } => check_expr(expr, scope, diagnostics),
        Stmt::For {
            var,
            iter,
            body,
            span,
        } => {
            check_expr(iter, scope, diagnostics);
            declare_implicit(var, "`for` loop", *span, scope);
            check_block(body, scope, diagnostics);
        }
        Stmt::If {
            cond,
            then_body,
            else_body,
            ..
        } => {
            check_expr(cond, scope, diagnostics);
            check_block(then_body, scope, diagnostics);
            check_block(else_body, scope, diagnostics);
        }
        Stmt::While { cond, body, .. } => {
            check_expr(cond, scope, diagnostics);
            check_block(body, scope, diagnostics);
        }
        Stmt::Repeat { count, body, .. } => {
            check_expr(count, scope, diagnostics);
            check_block(body, scope, diagnostics);
        }
        Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::Select { cases, .. } => {
            for case in cases {
                match &case.pattern {
                    SelectPattern::Receive { binding, expr } => {
                        check_expr(expr, scope, diagnostics);
                        declare_implicit(binding, "`select` receive", case.span, scope);
                    }
                    SelectPattern::After { .. }
                    | SelectPattern::Closed { .. }
                    | SelectPattern::Default => {}
                }
                check_expr(&case.action, scope, diagnostics);
            }
        }
        Stmt::Match {
            scrutinee,
            arms,
            default_action,
            ..
        } => {
            check_expr(scrutinee, scope, diagnostics);
            for arm in arms {
                // Pattern field bindings are visible to the arm action only.
                let mut arm_scope = scope.clone();
                if let Expr::EnumVariant { fields, .. } = &arm.pattern {
                    for (_, bound) in fields {
                        if let Expr::Ident { name, .. } = bound {
                            declare_implicit(name, "`match` pattern", arm.span, &mut arm_scope);
                        }
                    }
                }
                check_expr(&arm.action, &arm_scope, diagnostics);
            }
            if let Some(action) = default_action {
                check_expr(action, scope, diagnostics);
            }
        }
    }
}

/// Reports `E2110`/`E2111` when an assignment writes through an immutable
/// binding. Targets whose root is not a known binding are left alone: the type
/// checker already reports those as `E2101`.
fn check_assign_target(target: &Expr, span: Span, scope: &Scope, diagnostics: &mut Diagnostics) {
    let Some((root, is_direct)) = lvalue_root(target) else {
        return;
    };
    let Some(info) = scope.get(root) else {
        return;
    };
    if info.is_mut {
        return;
    }
    let (code, what) = if is_direct {
        (
            "E2110",
            format!("cannot assign to immutable binding `{root}`"),
        )
    } else {
        (
            "E2111",
            format!("cannot mutate through immutable binding `{root}`"),
        )
    };
    diagnostics.push(Diagnostic::new(
        code,
        Severity::Error,
        format!("{what}; {}", fix_hint(root, info)),
        span,
    ));
}

fn fix_hint(name: &str, info: &BindingInfo) -> String {
    let at = format!(
        "declared at line {}, column {}",
        info.decl.line_start, info.decl.col_start
    );
    match info.origin {
        BindingOrigin::Local => {
            format!("{at} — declare it `mut {name} := ...` to allow mutation")
        }
        BindingOrigin::Param => {
            format!("{at} — declare the parameter `mut {name}` to allow mutation")
        }
        BindingOrigin::Implicit(binder) => format!(
            "{at} by a {binder} binder, which has no `mut` form; bind a mutable copy \
             instead (`mut local := {name}`)"
        ),
    }
}

/// Container operations that mutate the receiver in place, so calling one is a
/// write to whatever binding the receiver is reached through.
///
/// This set is measured, not assumed. It is exactly the container surface the
/// type checker already tags with the `mut_state` effect on its receiver
/// (`crate::infer_expr`), and a runtime probe confirms the rest of the
/// container API returns a fresh value instead: `sort_desc` and `take` leave
/// the receiver untouched, and `get`/`contains`/`len` only read.
///
/// Channel operations (`send`, `close`, `recv`) are deliberately excluded: a
/// channel handle is a communication endpoint shared across tasks by design,
/// not a value whose contents the binding owns. Requiring `mut ch := chan(1)`
/// would make every `go`/`select` program declare a mutability it does not
/// mean. See `docs/spec/mutability_model.md`.
fn is_in_place_container_mutation(method: &str) -> bool {
    matches!(method, "append" | "set" | "remove")
}

/// Reports `E2112` for `xs.append(...)`-style in-place mutation reached
/// through an immutable binding. Receivers that are not rooted in a name
/// (`f().append(v)`) have no binding to check and are skipped, as are names
/// that are not bindings at all (module paths such as `json.builder`).
fn check_container_mutation(
    receiver: &Expr,
    method: &str,
    span: Span,
    scope: &Scope,
    diagnostics: &mut Diagnostics,
) {
    let Some((root, _)) = lvalue_root(receiver) else {
        return;
    };
    let Some(info) = scope.get(root) else {
        return;
    };
    if info.is_mut {
        return;
    }
    diagnostics.push(Diagnostic::new(
        "E2112",
        Severity::Error,
        format!(
            "`.{method}(...)` mutates its receiver in place, but `{root}` is an immutable binding; {}",
            fix_hint(root, info)
        ),
        span,
    ));
}

/// Root binding written by an lvalue, plus whether the write rebinds the name
/// itself (`x = ...`) rather than reaching through it (`x.f = ...`).
fn lvalue_root(target: &Expr) -> Option<(&str, bool)> {
    match target {
        Expr::Ident { name, .. } => Some((name.as_str(), true)),
        Expr::Member { object, .. } | Expr::Index { object, .. } => {
            lvalue_root(object).map(|(name, _)| (name, false))
        }
        // `xs.get(i).append(v)` writes into data `xs` owns: `.get(...)` on a
        // nested container hands back a view, not a copy (measured — mutating
        // the result is observable through the outer binding), so permission
        // still has to come from `xs`. `.take(...)` and `.sort_desc()` build
        // fresh containers and are deliberately not followed.
        Expr::Call { callee, .. } => match callee.as_ref() {
            Expr::Member { object, field, .. } if field == "get" => {
                lvalue_root(object).map(|(name, _)| (name, false))
            }
            _ => None,
        },
        _ => None,
    }
}

/// Function literals carry their own statement bodies, so assignments inside a
/// closure are checked with the enclosing bindings still visible (a closure may
/// mutate a captured binding only if that binding is `mut`).
fn check_expr(expr: &Expr, scope: &Scope, diagnostics: &mut Diagnostics) {
    match expr {
        Expr::FnLiteral {
            params,
            body,
            tail_expr,
            ..
        } => {
            let mut inner = scope.clone();
            seed_params(params, &mut inner);
            check_block(body, &mut inner, diagnostics);
            if let Some(tail) = tail_expr {
                check_expr(tail, &inner, diagnostics);
            }
        }
        Expr::Call { callee, args, span } => {
            if let Expr::Member { object, field, .. } = callee.as_ref() {
                if is_in_place_container_mutation(field) {
                    check_container_mutation(object, field, *span, scope, diagnostics);
                }
            }
            check_expr(callee, scope, diagnostics);
            for a in args {
                check_expr(a, scope, diagnostics);
            }
        }
        Expr::Binary { left, right, .. } => {
            check_expr(left, scope, diagnostics);
            check_expr(right, scope, diagnostics);
        }
        Expr::Index { object, index, .. } => {
            check_expr(object, scope, diagnostics);
            check_expr(index, scope, diagnostics);
        }
        Expr::Slice {
            object, start, end, ..
        } => {
            check_expr(object, scope, diagnostics);
            if let Some(s) = start {
                check_expr(s, scope, diagnostics);
            }
            if let Some(e) = end {
                check_expr(e, scope, diagnostics);
            }
        }
        Expr::Member { object, .. } => check_expr(object, scope, diagnostics),
        Expr::Unary { expr, .. }
        | Expr::Async { expr, .. }
        | Expr::Await { expr, .. }
        | Expr::Question { expr, .. }
        | Expr::Old { expr, .. } => check_expr(expr, scope, diagnostics),
        Expr::List { items, .. } => {
            for i in items {
                check_expr(i, scope, diagnostics);
            }
        }
        Expr::Map { entries, .. } => {
            for (k, v) in entries {
                check_expr(k, scope, diagnostics);
                check_expr(v, scope, diagnostics);
            }
        }
        Expr::Constructor { fields, .. } | Expr::EnumVariant { fields, .. } => {
            for (_, v) in fields {
                check_expr(v, scope, diagnostics);
            }
        }
        Expr::Ident { .. }
        | Expr::Int { .. }
        | Expr::Float { .. }
        | Expr::Bool { .. }
        | Expr::String { .. }
        | Expr::DotResult { .. } => {}
    }
}
