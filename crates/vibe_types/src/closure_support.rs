// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

use vibe_ast::{Expr, Stmt};
use vibe_diagnostics::{Diagnostic, Diagnostics, Severity};
use vibe_hir::{HirExpr, HirExprKind, HirFunction, HirParam, HirStmt};

use crate::{
    check_stmt, infer_expr, lower_expr, type_compatible, type_name, unify_return_types,
    ContractContext, TypeContext, TypeKind,
};

pub struct FnLiteralCache {
    pub owner: String,
    pub next_id: u32,
    pub entries: BTreeMap<usize, FnLiteralEntry>,
    pub synthetic: Vec<HirFunction>,
}

#[derive(Clone)]
pub struct FnLiteralEntry {
    pub ty: TypeKind,
    pub make_hir: HirExpr,
    pub capture_types: Vec<TypeKind>,
}

pub fn process_fn_literal(
    expr: &Expr,
    env: &BTreeMap<String, TypeKind>,
    ctx: &TypeContext<'_>,
    diagnostics: &mut Diagnostics,
    observed_effects: &mut BTreeSet<String>,
    context: ContractContext,
) -> TypeKind {
    let Some(cache_rc) = ctx.fn_literal_cache.as_ref() else {
        if let Expr::FnLiteral { span, .. } = expr {
            diagnostics.push(Diagnostic::new(
                "E2260",
                Severity::Error,
                "function literals are not allowed in this context",
                *span,
            ));
        }
        return TypeKind::Unknown;
    };
    let Expr::FnLiteral {
        params,
        return_type,
        body,
        tail_expr,
        span,
    } = expr
    else {
        return TypeKind::Unknown;
    };

    let ptr = expr as *const Expr as usize;
    if let Some(ent) = cache_rc.borrow().entries.get(&ptr) {
        return ent.ty.clone();
    }

    for p in params {
        if p.ty.is_none() {
            diagnostics.push(Diagnostic::new(
                "E2261",
                Severity::Error,
                format!(
                    "closure parameter `{}` requires an explicit type",
                    p.name
                ),
                *span,
            ));
        }
    }

    let param_names: BTreeSet<String> = params.iter().map(|p| p.name.clone()).collect();
    let param_types: Vec<TypeKind> = params
        .iter()
        .map(|p| {
            p.ty
                .as_ref()
                .map(|t| crate::resolve_type_ref(t, ctx.type_defs, ctx.enum_defs))
                .unwrap_or(TypeKind::Unknown)
        })
        .collect();

    let capture_names =
        collect_capture_names(body, tail_expr.as_ref().map(|b| b.as_ref()), &param_names, env);
    let mut cap_map: BTreeMap<String, u32> = BTreeMap::new();
    for (i, name) in capture_names.iter().enumerate() {
        cap_map.insert(name.clone(), (i + 1) as u32);
    }
    let capture_types: Vec<TypeKind> = capture_names
        .iter()
        .map(|n| env.get(n).cloned().unwrap_or(TypeKind::Unknown))
        .collect();

    let mut inner_env = env.clone();
    for (p, t) in params.iter().zip(&param_types) {
        inner_env.insert(p.name.clone(), t.clone());
    }

    let mut inferred_returns: Vec<TypeKind> = Vec::new();
    let mut hir_body: Vec<HirStmt> = Vec::new();
    let ensure_empty: Vec<vibe_ast::Expr> = Vec::new();

    let prev = ctx.closure_slot_map.replace(Some(cap_map.clone()));
    for stmt in body {
        check_stmt(
            stmt,
            &mut inner_env,
            ctx,
            diagnostics,
            observed_effects,
            &mut inferred_returns,
            &mut hir_body,
            &ensure_empty,
            0,
        );
    }
    let mut hir_tail = None;
    if let Some(te) = tail_expr {
        let t = infer_expr(
            te.as_ref(),
            &inner_env,
            ctx,
            context,
            diagnostics,
            observed_effects,
        );
        inferred_returns.push(t);
        hir_tail = Some(lower_expr(te.as_ref(), &inner_env, ctx));
    }
    ctx.closure_slot_map.replace(prev);

    let inferred_ret = unify_return_types(&inferred_returns);
    let declared_ret = return_type
        .as_ref()
        .map(|t| crate::resolve_type_ref(t, ctx.type_defs, ctx.enum_defs));
    let effective_ret = match (&declared_ret, &inferred_ret) {
        (Some(d), _) if !matches!(d, TypeKind::Unknown) => {
            if !type_compatible(d, &inferred_ret) && !matches!(inferred_ret, TypeKind::Unknown) {
                diagnostics.push(Diagnostic::new(
                    "E2262",
                    Severity::Error,
                    format!(
                        "closure return type mismatch: declared `{}`, inferred `{}`",
                        type_name(d),
                        type_name(&inferred_ret)
                    ),
                    *span,
                ));
            }
            d.clone()
        }
        _ => inferred_ret,
    };

    let fn_ty = TypeKind::Fn(param_types.clone(), Box::new(effective_ret.clone()));
    let ty_str = type_name(&fn_ty);

    let mut c = cache_rc.borrow_mut();
    let id = c.next_id;
    c.next_id += 1;
    let owner = sanitize_owner(&c.owner);
    let synthetic = format!("__closure_{owner}_{id}");

    let capture_hirs: Vec<HirExpr> = capture_names
        .iter()
        .map(|n| {
            lower_expr(
                &Expr::Ident {
                    name: n.clone(),
                    span: *span,
                },
                env,
                ctx,
            )
        })
        .collect();

    let make_hir = HirExpr::new(
        HirExprKind::MakeClosure {
            closure_fn: synthetic.clone(),
            captures: capture_hirs,
        },
        ty_str,
    );

    let mut hir_params = vec![HirParam {
        name: "__env".to_string(),
        ty: Some(vibe_ast::TypeRef {
            raw: "Ptr".to_string(),
        }),
    }];
    for p in params {
        hir_params.push(HirParam {
            name: p.name.clone(),
            ty: p.ty.clone(),
        });
    }

    c.synthetic.push(HirFunction {
        name: synthetic,
        is_public: false,
        params: hir_params,
        return_type: return_type.clone(),
        inferred_return_type: Some(type_name(&effective_ret)),
        effects_declared: BTreeSet::new(),
        effects_observed: BTreeSet::new(),
        body: hir_body,
        tail_expr: hir_tail,
        native_symbol: None,
    });

    c.entries.insert(
        ptr,
        FnLiteralEntry {
            ty: fn_ty.clone(),
            make_hir,
            capture_types,
        },
    );
    fn_ty
}

fn sanitize_owner(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "fn".to_string()
    } else {
        out
    }
}

pub(crate) fn fn_literal_capture_type_kinds(
    expr: &Expr,
    env: &BTreeMap<String, TypeKind>,
) -> Option<Vec<TypeKind>> {
    let Expr::FnLiteral {
        params,
        body,
        tail_expr,
        ..
    } = expr
    else {
        return None;
    };
    let param_names: BTreeSet<String> = params.iter().map(|p| p.name.clone()).collect();
    let names =
        collect_capture_names(body, tail_expr.as_ref().map(|b| b.as_ref()), &param_names, env);
    Some(
        names
            .into_iter()
            .map(|n| env.get(&n).cloned().unwrap_or(TypeKind::Unknown))
            .collect(),
    )
}

fn collect_capture_names(
    body: &[Stmt],
    tail: Option<&Expr>,
    param_names: &BTreeSet<String>,
    outer_env: &BTreeMap<String, TypeKind>,
) -> Vec<String> {
    let mut caps = BTreeSet::new();
    let mut locals = param_names.clone();
    for stmt in body {
        walk_stmt(stmt, outer_env, &mut locals, &mut caps);
    }
    if let Some(e) = tail {
        walk_expr(e, outer_env, &locals, &mut caps);
    }
    caps.into_iter().collect()
}

fn walk_stmt(
    stmt: &Stmt,
    outer_env: &BTreeMap<String, TypeKind>,
    locals: &mut BTreeSet<String>,
    caps: &mut BTreeSet<String>,
) {
    match stmt {
        Stmt::Binding { name, expr, .. } => {
            walk_expr(expr, outer_env, locals, caps);
            locals.insert(name.clone());
        }
        Stmt::Assignment { target, expr, .. } => {
            walk_expr(target, outer_env, locals, caps);
            walk_expr(expr, outer_env, locals, caps);
        }
        Stmt::Return { expr, .. } | Stmt::ExprStmt { expr, .. } => {
            walk_expr(expr, outer_env, locals, caps);
        }
        Stmt::For { var, iter, body, .. } => {
            walk_expr(iter, outer_env, locals, caps);
            let mut inner = locals.clone();
            inner.insert(var.clone());
            for s in body {
                walk_stmt(s, outer_env, &mut inner, caps);
            }
        }
        Stmt::If {
            cond,
            then_body,
            else_body,
            ..
        } => {
            walk_expr(cond, outer_env, locals, caps);
            let mut then_loc = locals.clone();
            for s in then_body {
                walk_stmt(s, outer_env, &mut then_loc, caps);
            }
            let mut else_loc = locals.clone();
            for s in else_body {
                walk_stmt(s, outer_env, &mut else_loc, caps);
            }
        }
        Stmt::While { cond, body, .. } | Stmt::Repeat { count: cond, body, .. } => {
            walk_expr(cond, outer_env, locals, caps);
            let mut inner = locals.clone();
            for s in body {
                walk_stmt(s, outer_env, &mut inner, caps);
            }
        }
        Stmt::Go { expr, .. } | Stmt::Thread { expr, .. } => {
            walk_expr(expr, outer_env, locals, caps);
        }
        Stmt::Select { cases, .. } => {
            for c in cases {
                if let vibe_ast::SelectPattern::Receive { expr, .. } = &c.pattern {
                    walk_expr(expr, outer_env, locals, caps);
                }
                walk_expr(&c.action, outer_env, locals, caps);
            }
        }
        Stmt::Match {
            scrutinee,
            arms,
            default_action,
            ..
        } => {
            walk_expr(scrutinee, outer_env, locals, caps);
            for a in arms {
                walk_expr(&a.pattern, outer_env, locals, caps);
                walk_expr(&a.action, outer_env, locals, caps);
            }
            if let Some(e) = default_action {
                walk_expr(e, outer_env, locals, caps);
            }
        }
        Stmt::Break { .. } | Stmt::Continue { .. } => {}
    }
}

fn walk_expr(
    expr: &Expr,
    outer_env: &BTreeMap<String, TypeKind>,
    locals: &BTreeSet<String>,
    caps: &mut BTreeSet<String>,
) {
    match expr {
        Expr::Ident { name, .. } => {
            if outer_env.contains_key(name) && !locals.contains(name) {
                caps.insert(name.clone());
            }
        }
        Expr::Call { callee, args, .. } => {
            walk_expr(callee, outer_env, locals, caps);
            for a in args {
                walk_expr(a, outer_env, locals, caps);
            }
        }
        Expr::Member { object, .. }
        | Expr::Unary { expr: object, .. }
        | Expr::Async { expr: object, .. }
        | Expr::Await { expr: object, .. }
        | Expr::Question { expr: object, .. }
        | Expr::Old { expr: object, .. } => {
            walk_expr(object, outer_env, locals, caps);
        }
        Expr::Index { object, index, .. } => {
            walk_expr(object, outer_env, locals, caps);
            walk_expr(index, outer_env, locals, caps);
        }
        Expr::Slice {
            object,
            start,
            end,
            ..
        } => {
            walk_expr(object, outer_env, locals, caps);
            if let Some(s) = start {
                walk_expr(s, outer_env, locals, caps);
            }
            if let Some(e) = end {
                walk_expr(e, outer_env, locals, caps);
            }
        }
        Expr::Binary { left, right, .. } => {
            walk_expr(left, outer_env, locals, caps);
            walk_expr(right, outer_env, locals, caps);
        }
        Expr::List { items, .. } => {
            for i in items {
                walk_expr(i, outer_env, locals, caps);
            }
        }
        Expr::Map { entries, .. } => {
            for (k, v) in entries {
                walk_expr(k, outer_env, locals, caps);
                walk_expr(v, outer_env, locals, caps);
            }
        }
        Expr::Constructor { fields, .. } => {
            for (_, e) in fields {
                walk_expr(e, outer_env, locals, caps);
            }
        }
        Expr::FnLiteral {
            params,
            body,
            tail_expr,
            ..
        } => {
            let mut inner = locals.clone();
            for p in params {
                inner.insert(p.name.clone());
            }
            for s in body {
                walk_stmt(s, outer_env, &mut inner, caps);
            }
            if let Some(t) = tail_expr {
                walk_expr(t.as_ref(), outer_env, &inner, caps);
            }
        }
        Expr::Int { .. }
        | Expr::Float { .. }
        | Expr::Bool { .. }
        | Expr::String { .. }
        | Expr::DotResult { .. } => {}
        Expr::EnumVariant { fields, .. } => {
            for (_, e) in fields {
                walk_expr(e, outer_env, locals, caps);
            }
        }
    }
}
