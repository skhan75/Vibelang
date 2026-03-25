// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

mod effect_diagnostics;
mod effect_propagation;
mod ownership;

use vibe_ast::{
    BinaryOp, Contract, Declaration, Expr, FileAst, SelectPattern, Stmt, TypeRef, UnaryOp,
};
use vibe_diagnostics::{Diagnostic, Diagnostics, Severity, Span};
use vibe_hir::{
    verify_hir, HirContractKind, HirExpr, HirExprKind, HirFunction, HirMatchArm, HirParam,
    HirProgram, HirSelectCase, HirSelectPattern, HirStmt,
};

use crate::effect_diagnostics::emit_effect_diagnostics;
use crate::effect_propagation::{
    collect_direct_calls, compute_transitive_effects, FunctionEffectSummary,
};
use crate::ownership::{
    check_go_sendability, check_shared_mutation_in_concurrent_context, is_sendable_type,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Int,
    Float,
    Bool,
    Str,
    Json,
    JsonBuilder,
    List(Box<TypeKind>),
    Map(Box<TypeKind>, Box<TypeKind>),
    Result(Box<TypeKind>, Box<TypeKind>),
    Chan(Box<TypeKind>),
    UserType(String),
    Enum(String),
    Void,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CheckOutput {
    pub diagnostics: Diagnostics,
    pub hir: HirProgram,
    pub type_defs: BTreeMap<String, Vec<(String, TypeKind)>>,
    pub enum_defs: BTreeMap<String, Vec<String>>,
}

/// Convert TypeKind to a string for codegen (Int, Bool, Str, etc).
pub fn type_kind_to_codegen_str(t: &TypeKind) -> String {
    match t {
        TypeKind::Int => "Int".to_string(),
        TypeKind::Float => "Float".to_string(),
        TypeKind::Bool => "Bool".to_string(),
        TypeKind::Str => "Str".to_string(),
        TypeKind::Json => "Json".to_string(),
        TypeKind::JsonBuilder => "JsonBuilder".to_string(),
        TypeKind::UserType(name) => name.clone(),
        TypeKind::Enum(name) => name.clone(),
        _ => "Unknown".to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContractContext {
    Require,
    Ensure,
    Other,
}

struct TypeContext<'a> {
    sigs: &'a BTreeMap<String, Option<TypeKind>>,
    type_defs: &'a BTreeMap<String, Vec<(String, TypeKind)>>,
    enum_defs: &'a BTreeMap<String, Vec<String>>,
}

pub fn check_and_lower(ast: &FileAst) -> CheckOutput {
    let mut diagnostics = Diagnostics::default();
    let mut signatures: BTreeMap<String, Option<TypeKind>> = BTreeMap::new();
    let mut type_defs: BTreeMap<String, Vec<(String, TypeKind)>> = BTreeMap::new();
    let mut enum_defs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut hir = HirProgram::default();
    let mut effect_summaries: Vec<FunctionEffectSummary> = Vec::new();

    for decl in &ast.declarations {
        match decl {
            Declaration::Type(t) => {
                if type_defs.contains_key(&t.name) {
                    diagnostics.push(Diagnostic::new(
                        "E2002",
                        Severity::Error,
                        format!("duplicate type `{}`", t.name),
                        t.span,
                    ));
                }
                let mut fields = Vec::new();
                for f in &t.fields {
                    let field_ty = parse_type_ref(&f.ty);
                    fields.push((f.name.clone(), field_ty));
                }
                type_defs.insert(t.name.clone(), fields);
            }
            Declaration::Enum(e) => {
                if enum_defs.contains_key(&e.name) {
                    diagnostics.push(Diagnostic::new(
                        "E2002",
                        Severity::Error,
                        format!("duplicate enum `{}`", e.name),
                        e.span,
                    ));
                }
                enum_defs.insert(e.name.clone(), e.variants.clone());
            }
            Declaration::Function(f) => {
                if signatures.contains_key(&f.name) {
                    diagnostics.push(Diagnostic::new(
                        "E2002",
                        Severity::Error,
                        format!("duplicate function `{}`", f.name),
                        f.span,
                    ));
                }
                signatures.insert(
                    f.name.clone(),
                    f.return_type
                        .as_ref()
                        .map(|t| resolve_type_ref(t, &type_defs, &enum_defs)),
                );
            }
        }
    }

    let ctx = TypeContext {
        sigs: &signatures,
        type_defs: &type_defs,
        enum_defs: &enum_defs,
    };

    for decl in &ast.declarations {
        let Declaration::Function(func) = decl else {
            continue;
        };
        let mut env: BTreeMap<String, TypeKind> = BTreeMap::new();
        let mut observed_effects: BTreeSet<String> = BTreeSet::new();
        let mut declared_effects: BTreeSet<String> = BTreeSet::new();
        let mut require_contract_exprs: Vec<Expr> = Vec::new();
        let mut ensure_contract_exprs: Vec<Expr> = Vec::new();

        if func.is_public {
            for p in &func.params {
                if p.ty.is_none() {
                    diagnostics.push(Diagnostic::new(
                        "E2003",
                        Severity::Warning,
                        format!(
                            "public function `{}` parameter `{}` should have explicit type",
                            func.name, p.name
                        ),
                        func.span,
                    ));
                }
            }
            if func.return_type.is_none() {
                diagnostics.push(Diagnostic::new(
                    "E2004",
                    Severity::Warning,
                    format!(
                        "public function `{}` should have explicit return type",
                        func.name
                    ),
                    func.span,
                ));
            }
        }

        for p in &func.params {
            env.insert(
                p.name.clone(),
                p.ty.as_ref()
                    .map(|t| resolve_type_ref(t, &type_defs, &enum_defs))
                    .unwrap_or(TypeKind::Unknown),
            );
        }

        for c in &func.contracts {
            match c {
                Contract::Effect { name, span } => {
                    if !is_known_effect(name) {
                        diagnostics.push(Diagnostic::new(
                            "E3001",
                            Severity::Error,
                            format!("unknown effect `{name}`"),
                            *span,
                        ));
                    }
                    declared_effects.insert(name.clone());
                }
                Contract::Require { expr, span } => {
                    validate_contract_expr(expr, ContractContext::Require, &mut diagnostics);
                    infer_expr(
                        expr,
                        &env,
                        &ctx,
                        ContractContext::Require,
                        &mut diagnostics,
                        &mut observed_effects,
                    );
                    if !matches!(
                        infer_expr(
                            expr,
                            &env,
                            &ctx,
                            ContractContext::Require,
                            &mut diagnostics,
                            &mut observed_effects
                        ),
                        TypeKind::Bool | TypeKind::Unknown
                    ) {
                        diagnostics.push(Diagnostic::new(
                            "E3004",
                            Severity::Error,
                            "@require expression should evaluate to Bool",
                            *span,
                        ));
                    }
                    require_contract_exprs.push(expr.clone());
                }
                Contract::Ensure { expr, span } => {
                    validate_contract_expr(expr, ContractContext::Ensure, &mut diagnostics);
                    if !matches!(
                        infer_expr(
                            expr,
                            &env,
                            &ctx,
                            ContractContext::Ensure,
                            &mut diagnostics,
                            &mut observed_effects
                        ),
                        TypeKind::Bool | TypeKind::Unknown
                    ) {
                        diagnostics.push(Diagnostic::new(
                            "E3005",
                            Severity::Error,
                            "@ensure expression should evaluate to Bool",
                            *span,
                        ));
                    }
                    ensure_contract_exprs.push(expr.clone());
                }
                Contract::Examples { cases, .. } => {
                    for case in cases {
                        infer_expr(
                            &case.call,
                            &env,
                            &ctx,
                            ContractContext::Other,
                            &mut diagnostics,
                            &mut observed_effects,
                        );
                        infer_expr(
                            &case.expected,
                            &env,
                            &ctx,
                            ContractContext::Other,
                            &mut diagnostics,
                            &mut observed_effects,
                        );
                    }
                }
                Contract::Intent { .. } => {}
            }
        }

        let mut inferred_returns: Vec<TypeKind> = Vec::new();
        let mut hir_body: Vec<HirStmt> = Vec::new();
        for expr in &require_contract_exprs {
            hir_body.push(HirStmt::ContractCheck {
                kind: HirContractKind::Require,
                expr: lower_contract_expr(expr, &env, None, &ctx),
            });
        }
        for stmt in &func.body {
            check_stmt(
                stmt,
                &mut env,
                &ctx,
                &mut diagnostics,
                &mut observed_effects,
                &mut inferred_returns,
                &mut hir_body,
                &ensure_contract_exprs,
                0,
            );
        }
        let mut hir_tail_expr = None;
        if let Some(expr) = &func.tail_expr {
            let t = infer_expr(
                expr,
                &env,
                &ctx,
                ContractContext::Other,
                &mut diagnostics,
                &mut observed_effects,
            );
            inferred_returns.push(t);
            let lowered_tail = lower_expr(expr, &env, &ctx);
            for contract_expr in &ensure_contract_exprs {
                hir_body.push(HirStmt::ContractCheck {
                    kind: HirContractKind::Ensure,
                    expr: lower_contract_expr(contract_expr, &env, Some(&lowered_tail), &ctx),
                });
            }
            hir_tail_expr = Some(lowered_tail);
        }

        let inferred_return = unify_return_types(&inferred_returns);
        if let Some(declared) = func.return_type.as_ref() {
            let declared = resolve_type_ref(declared, &type_defs, &enum_defs);
            if !type_compatible(&declared, &inferred_return) {
                diagnostics.push(Diagnostic::new(
                    "E2201",
                    Severity::Error,
                    format!(
                        "return type mismatch in `{}`: declared `{}`, inferred `{}`",
                        func.name,
                        type_name(&declared),
                        type_name(&inferred_return)
                    ),
                    func.span,
                ));
            }
        }

        check_shared_mutation_in_concurrent_context(
            &func.body,
            observed_effects.contains("concurrency"),
            &mut diagnostics,
            func.span,
        );

        effect_summaries.push(FunctionEffectSummary {
            name: func.name.clone(),
            span: func.span,
            declared_effects: declared_effects.clone(),
            direct_observed_effects: observed_effects.clone(),
            direct_calls: collect_direct_calls(func),
        });

        hir.functions.push(HirFunction {
            name: func.name.clone(),
            is_public: func.is_public,
            params: func
                .params
                .iter()
                .map(|p| HirParam {
                    name: p.name.clone(),
                    ty: p.ty.clone(),
                })
                .collect(),
            return_type: func.return_type.clone(),
            inferred_return_type: Some(type_name(&inferred_return)),
            effects_declared: declared_effects,
            effects_observed: observed_effects,
            body: hir_body,
            tail_expr: hir_tail_expr,
        });
    }

    let transitive_effects = compute_transitive_effects(&effect_summaries);
    emit_effect_diagnostics(&effect_summaries, &transitive_effects, &mut diagnostics);
    for f in &mut hir.functions {
        if let Some(transitive) = transitive_effects.get(&f.name) {
            f.effects_observed = transitive.clone();
        }
    }

    if let Err(msg) = verify_hir(&hir) {
        diagnostics.push(Diagnostic::new(
            "E2301",
            Severity::Error,
            format!("HIR verification failed: {msg}"),
            Default::default(),
        ));
    }

    CheckOutput {
        diagnostics,
        hir,
        type_defs,
        enum_defs,
    }
}

#[allow(clippy::too_many_arguments)]
fn check_stmt(
    stmt: &Stmt,
    env: &mut BTreeMap<String, TypeKind>,
    ctx: &TypeContext,
    diagnostics: &mut Diagnostics,
    observed_effects: &mut BTreeSet<String>,
    inferred_returns: &mut Vec<TypeKind>,
    hir_out: &mut Vec<HirStmt>,
    ensure_contract_exprs: &[Expr],
    loop_depth: usize,
) {
    match stmt {
        Stmt::Binding { name, expr, .. } => {
            let t = infer_expr(
                expr,
                env,
                ctx,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            let lowered_expr = lower_expr(expr, env, ctx);
            env.insert(name.clone(), t);
            hir_out.push(HirStmt::Binding {
                name: name.clone(),
                expr: lowered_expr,
            });
        }
        Stmt::Assignment { target, expr, span } => {
            let rhs = infer_expr(
                expr,
                env,
                ctx,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            match target {
                Expr::Ident { name, .. } => {
                    let lhs = env.get(name).cloned().unwrap_or(TypeKind::Unknown);
                    if lhs == TypeKind::Unknown {
                        diagnostics.push(Diagnostic::new(
                            "E2101",
                            Severity::Error,
                            format!("assignment to unknown variable `{name}`"),
                            *span,
                        ));
                    } else if !type_compatible(&lhs, &rhs) {
                        diagnostics.push(Diagnostic::new(
                            "E2102",
                            Severity::Error,
                            format!(
                                "type mismatch in assignment to `{name}`: lhs `{}`, rhs `{}`",
                                type_name(&lhs),
                                type_name(&rhs)
                            ),
                            *span,
                        ));
                    }
                }
                Expr::Member { object, field, .. } => {
                    observed_effects.insert("mut_state".to_string());
                    let base_ty = infer_expr(
                        object,
                        env,
                        ctx,
                        ContractContext::Other,
                        diagnostics,
                        observed_effects,
                    );
                    if let TypeKind::UserType(user_type_name) = base_ty {
                        if let Some(fields) = ctx.type_defs.get(&user_type_name) {
                            if let Some((_, field_ty)) = fields.iter().find(|(n, _)| n == field) {
                                if !type_compatible(field_ty, &rhs)
                                    && !matches!(rhs, TypeKind::Unknown)
                                    && !matches!(field_ty, TypeKind::Unknown)
                                {
                                    diagnostics.push(Diagnostic::new(
                                        "E2259",
                                        Severity::Error,
                                        format!(
                                            "field assignment type mismatch `{user_type_name}.{field}`: expected `{}`, got `{}`",
                                            type_name(field_ty),
                                            type_name(&rhs)
                                        ),
                                        *span,
                                    ));
                                }
                            } else {
                                diagnostics.push(Diagnostic::new(
                                    "E2251",
                                    Severity::Error,
                                    format!("type `{user_type_name}` has no field `{field}`"),
                                    *span,
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
            hir_out.push(HirStmt::Assignment {
                target: lower_expr(target, env, ctx),
                expr: lower_expr(expr, env, ctx),
            });
        }
        Stmt::Return { expr, .. } => {
            let ret_ty = infer_expr(
                expr,
                env,
                ctx,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            inferred_returns.push(ret_ty);
            let lowered_return_expr = lower_expr(expr, env, ctx);
            for ensure_expr in ensure_contract_exprs {
                hir_out.push(HirStmt::ContractCheck {
                    kind: HirContractKind::Ensure,
                    expr: lower_contract_expr(ensure_expr, env, Some(&lowered_return_expr), ctx),
                });
            }
            hir_out.push(HirStmt::Return {
                expr: lowered_return_expr,
            });
        }
        Stmt::ExprStmt { expr, .. } => {
            let _ = infer_expr(
                expr,
                env,
                ctx,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            hir_out.push(HirStmt::Expr {
                expr: lower_expr(expr, env, ctx),
            });
        }
        Stmt::For {
            var, iter, body, ..
        } => {
            let iter_ty = infer_expr(
                iter,
                env,
                ctx,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            let item_ty = match iter_ty {
                TypeKind::List(inner) => *inner,
                TypeKind::Map(key, _value) => *key,
                _ => TypeKind::Unknown,
            };
            env.insert(var.clone(), item_ty);
            let mut child_hir = Vec::new();
            for s in body {
                check_stmt(
                    s,
                    env,
                    ctx,
                    diagnostics,
                    observed_effects,
                    inferred_returns,
                    &mut child_hir,
                    ensure_contract_exprs,
                    loop_depth + 1,
                );
            }
            hir_out.push(HirStmt::For {
                var: var.clone(),
                iter: lower_expr(iter, env, ctx),
                body: child_hir,
            });
        }
        Stmt::If {
            cond,
            then_body,
            else_body,
            ..
        } => {
            let cond_ty = infer_expr(
                cond,
                env,
                ctx,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            if !matches!(cond_ty, TypeKind::Bool | TypeKind::Unknown) {
                diagnostics.push(Diagnostic::new(
                    "E2103",
                    Severity::Error,
                    "if condition should be Bool",
                    cond.span(),
                ));
            }
            let mut then_hir = Vec::new();
            for s in then_body {
                check_stmt(
                    s,
                    env,
                    ctx,
                    diagnostics,
                    observed_effects,
                    inferred_returns,
                    &mut then_hir,
                    ensure_contract_exprs,
                    loop_depth,
                );
            }
            let mut else_hir = Vec::new();
            for s in else_body {
                check_stmt(
                    s,
                    env,
                    ctx,
                    diagnostics,
                    observed_effects,
                    inferred_returns,
                    &mut else_hir,
                    ensure_contract_exprs,
                    loop_depth,
                );
            }
            hir_out.push(HirStmt::If {
                cond: lower_expr(cond, env, ctx),
                then_body: then_hir,
                else_body: else_hir,
            });
        }
        Stmt::While { cond, body, .. } => {
            let cond_ty = infer_expr(
                cond,
                env,
                ctx,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            if !matches!(cond_ty, TypeKind::Bool | TypeKind::Unknown) {
                diagnostics.push(Diagnostic::new(
                    "E2104",
                    Severity::Error,
                    "while condition should be Bool",
                    cond.span(),
                ));
            }
            let mut child_hir = Vec::new();
            for s in body {
                check_stmt(
                    s,
                    env,
                    ctx,
                    diagnostics,
                    observed_effects,
                    inferred_returns,
                    &mut child_hir,
                    ensure_contract_exprs,
                    loop_depth + 1,
                );
            }
            hir_out.push(HirStmt::While {
                cond: lower_expr(cond, env, ctx),
                body: child_hir,
            });
        }
        Stmt::Repeat { count, body, .. } => {
            let count_ty = infer_expr(
                count,
                env,
                ctx,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            if !matches!(count_ty, TypeKind::Int | TypeKind::Unknown) {
                diagnostics.push(Diagnostic::new(
                    "E2105",
                    Severity::Error,
                    "repeat count should be Int",
                    count.span(),
                ));
            }
            let mut child_hir = Vec::new();
            for s in body {
                check_stmt(
                    s,
                    env,
                    ctx,
                    diagnostics,
                    observed_effects,
                    inferred_returns,
                    &mut child_hir,
                    ensure_contract_exprs,
                    loop_depth + 1,
                );
            }
            hir_out.push(HirStmt::Repeat {
                count: lower_expr(count, env, ctx),
                body: child_hir,
            });
        }
        Stmt::Break { span } => {
            if loop_depth == 0 {
                diagnostics.push(Diagnostic::new(
                    "E2107",
                    Severity::Error,
                    "`break` is only valid inside `for`, `while`, or `repeat` loops",
                    *span,
                ));
            }
            hir_out.push(HirStmt::Break);
        }
        Stmt::Continue { span } => {
            if loop_depth == 0 {
                diagnostics.push(Diagnostic::new(
                    "E2108",
                    Severity::Error,
                    "`continue` is only valid inside `for`, `while`, or `repeat` loops",
                    *span,
                ));
            }
            hir_out.push(HirStmt::Continue);
        }
        Stmt::Select { cases, .. } => {
            observed_effects.insert("concurrency".to_string());
            let mut lowered_cases = Vec::new();
            for c in cases {
                match &c.pattern {
                    SelectPattern::Receive { binding, expr } => {
                        let _ = infer_expr(
                            expr,
                            env,
                            ctx,
                            ContractContext::Other,
                            diagnostics,
                            observed_effects,
                        );
                        env.insert(binding.clone(), TypeKind::Unknown);
                    }
                    SelectPattern::After { .. } => {
                        observed_effects.insert("nondet".to_string());
                    }
                    SelectPattern::Closed { ident } => {
                        if !env.contains_key(ident) {
                            diagnostics.push(Diagnostic::new(
                                "E2106",
                                Severity::Warning,
                                format!("closed case references unknown `{ident}`"),
                                c.span,
                            ));
                        }
                    }
                    SelectPattern::Default => {}
                }
                let _ = infer_expr(
                    &c.action,
                    env,
                    ctx,
                    ContractContext::Other,
                    diagnostics,
                    observed_effects,
                );
                lowered_cases.push(HirSelectCase {
                    pattern: lower_select_pattern(&c.pattern, env, ctx),
                    action: lower_expr(&c.action, env, ctx),
                });
            }
            hir_out.push(HirStmt::Select {
                cases: lowered_cases,
            });
        }
        Stmt::Go { expr, .. } => {
            observed_effects.insert("concurrency".to_string());
            let _ = infer_expr(
                expr,
                env,
                ctx,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            check_go_sendability(expr, env, expr_type_hint, diagnostics);
            hir_out.push(HirStmt::Go {
                expr: lower_expr(expr, env, ctx),
            });
        }
        Stmt::Thread { expr, .. } => {
            observed_effects.insert("concurrency".to_string());
            let _ = infer_expr(
                expr,
                env,
                ctx,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            check_go_sendability(expr, env, expr_type_hint, diagnostics);
            hir_out.push(HirStmt::Thread {
                expr: lower_expr(expr, env, ctx),
            });
        }
        Stmt::Match {
            scrutinee,
            arms,
            default_action,
            span,
        } => {
            let scrutinee_ty = infer_expr(
                scrutinee,
                env,
                ctx,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            if let TypeKind::Enum(enum_name) = &scrutinee_ty {
                if default_action.is_none() {
                    if let Some(variants) = ctx.enum_defs.get(enum_name) {
                        let mut covered: BTreeSet<String> = BTreeSet::new();
                        for arm in arms {
                            match &arm.pattern {
                                Expr::EnumVariant {
                                    enum_name: en,
                                    variant,
                                    ..
                                } if en == enum_name => {
                                    covered.insert(variant.clone());
                                }
                                Expr::Member {
                                    object,
                                    field: variant,
                                    ..
                                } if matches!(&**object, Expr::Ident { name, .. } if name == enum_name) =>
                                {
                                    covered.insert(variant.clone());
                                }
                                _ => {}
                            }
                        }
                        for v in variants {
                            if !covered.contains(v) {
                                diagnostics.push(Diagnostic::new(
                                    "E2258",
                                    Severity::Error,
                                    format!(
                                        "match on enum `{enum_name}` is not exhaustive: missing variant `{v}`",
                                    ),
                                    *span,
                                ));
                            }
                        }
                    }
                }
            }
            let scrutinee_hir = lower_expr(scrutinee, env, ctx);
            let hir_arms: Vec<_> = arms
                .iter()
                .map(|a| HirMatchArm {
                    pattern: lower_expr(&a.pattern, env, ctx),
                    action: lower_expr(&a.action, env, ctx),
                })
                .collect();
            let default_hir = default_action.as_ref().map(|e| lower_expr(e, env, ctx));
            hir_out.push(HirStmt::Match {
                scrutinee: scrutinee_hir,
                arms: hir_arms,
                default_action: default_hir,
            });
        }
    }
}

fn lower_select_pattern(
    pattern: &SelectPattern,
    env: &BTreeMap<String, TypeKind>,
    ctx: &TypeContext,
) -> HirSelectPattern {
    match pattern {
        SelectPattern::Receive { binding, expr } => HirSelectPattern::Receive {
            binding: binding.clone(),
            expr: lower_expr(expr, env, ctx),
        },
        SelectPattern::After { duration_literal } => HirSelectPattern::After {
            duration_literal: duration_literal.clone(),
        },
        SelectPattern::Closed { ident } => HirSelectPattern::Closed {
            ident: ident.clone(),
        },
        SelectPattern::Default => HirSelectPattern::Default,
    }
}

fn lower_contract_expr(
    expr: &Expr,
    env: &BTreeMap<String, TypeKind>,
    dot_result: Option<&HirExpr>,
    ctx: &TypeContext,
) -> HirExpr {
    if let Expr::DotResult { .. } = expr {
        if let Some(result) = dot_result {
            return result.clone();
        }
    }
    let ty = type_name(&expr_type_hint(expr, env));
    let kind = match expr {
        Expr::Ident { name, .. } => HirExprKind::Ident(name.clone()),
        Expr::Int { value, .. } => HirExprKind::Int(*value),
        Expr::Float { value, .. } => HirExprKind::Float(*value),
        Expr::Bool { value, .. } => HirExprKind::Bool(*value),
        Expr::String { value, .. } => HirExprKind::String(value.clone()),
        Expr::List { items, .. } => HirExprKind::List(
            items
                .iter()
                .map(|e| lower_contract_expr(e, env, dot_result, ctx))
                .collect(),
        ),
        Expr::Map { entries, .. } => HirExprKind::Map(
            entries
                .iter()
                .map(|(k, v)| {
                    (
                        lower_contract_expr(k, env, dot_result, ctx),
                        lower_contract_expr(v, env, dot_result, ctx),
                    )
                })
                .collect(),
        ),
        Expr::Member { object, field, .. } => {
            if let Expr::Ident {
                name: enum_name, ..
            } = &**object
            {
                if ctx.enum_defs.contains_key(enum_name) {
                    return HirExpr::new(
                        HirExprKind::EnumVariant {
                            enum_name: enum_name.clone(),
                            variant: field.clone(),
                        },
                        enum_name.clone(),
                    );
                }
            }
            HirExprKind::Member {
                object: Box::new(lower_contract_expr(object, env, dot_result, ctx)),
                field: field.clone(),
            }
        }
        Expr::Index { object, index, .. } => HirExprKind::Index {
            object: Box::new(lower_contract_expr(object, env, dot_result, ctx)),
            index: Box::new(lower_contract_expr(index, env, dot_result, ctx)),
        },
        Expr::Slice {
            object, start, end, ..
        } => HirExprKind::Slice {
            object: Box::new(lower_contract_expr(object, env, dot_result, ctx)),
            start: start
                .as_ref()
                .map(|e| Box::new(lower_contract_expr(e, env, dot_result, ctx))),
            end: end
                .as_ref()
                .map(|e| Box::new(lower_contract_expr(e, env, dot_result, ctx))),
        },
        Expr::Call { callee, args, .. } => HirExprKind::Call {
            callee: Box::new(lower_contract_expr(callee, env, dot_result, ctx)),
            args: args
                .iter()
                .map(|a| lower_contract_expr(a, env, dot_result, ctx))
                .collect(),
        },
        Expr::Binary {
            left, op, right, ..
        } => HirExprKind::Binary {
            left: Box::new(lower_contract_expr(left, env, dot_result, ctx)),
            op: *op,
            right: Box::new(lower_contract_expr(right, env, dot_result, ctx)),
        },
        Expr::Unary { op, expr, .. } => HirExprKind::Unary {
            op: *op,
            expr: Box::new(lower_contract_expr(expr, env, dot_result, ctx)),
        },
        Expr::Async { expr, .. } => HirExprKind::Async {
            expr: Box::new(lower_contract_expr(expr, env, dot_result, ctx)),
        },
        Expr::Await { expr, .. } => HirExprKind::Await {
            expr: Box::new(lower_contract_expr(expr, env, dot_result, ctx)),
        },
        Expr::Question { expr, .. } => HirExprKind::Question {
            expr: Box::new(lower_contract_expr(expr, env, dot_result, ctx)),
        },
        Expr::DotResult { .. } => HirExprKind::DotResult,
        Expr::Old { expr, .. } => HirExprKind::Old {
            expr: Box::new(lower_contract_expr(expr, env, dot_result, ctx)),
        },
        Expr::Constructor {
            type_name, fields, ..
        } => HirExprKind::Constructor {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(n, e)| (n.clone(), lower_contract_expr(e, env, dot_result, ctx)))
                .collect(),
        },
        Expr::EnumVariant {
            enum_name, variant, ..
        } => HirExprKind::EnumVariant {
            enum_name: enum_name.clone(),
            variant: variant.clone(),
        },
    };
    HirExpr::new(kind, ty)
}

fn lower_expr(expr: &Expr, env: &BTreeMap<String, TypeKind>, ctx: &TypeContext) -> HirExpr {
    let ty = type_name(&expr_type_hint(expr, env));
    let kind = match expr {
        Expr::Ident { name, .. } => HirExprKind::Ident(name.clone()),
        Expr::Int { value, .. } => HirExprKind::Int(*value),
        Expr::Float { value, .. } => HirExprKind::Float(*value),
        Expr::Bool { value, .. } => HirExprKind::Bool(*value),
        Expr::String { value, .. } => HirExprKind::String(value.clone()),
        Expr::List { items, .. } => {
            HirExprKind::List(items.iter().map(|e| lower_expr(e, env, ctx)).collect())
        }
        Expr::Map { entries, .. } => HirExprKind::Map(
            entries
                .iter()
                .map(|(k, v)| (lower_expr(k, env, ctx), lower_expr(v, env, ctx)))
                .collect(),
        ),
        Expr::Member { object, field, .. } => {
            if let Expr::Ident {
                name: enum_name, ..
            } = &**object
            {
                if ctx.enum_defs.contains_key(enum_name) {
                    return HirExpr::new(
                        HirExprKind::EnumVariant {
                            enum_name: enum_name.clone(),
                            variant: field.clone(),
                        },
                        enum_name.clone(),
                    );
                }
            }
            HirExprKind::Member {
                object: Box::new(lower_expr(object, env, ctx)),
                field: field.clone(),
            }
        }
        Expr::Index { object, index, .. } => HirExprKind::Index {
            object: Box::new(lower_expr(object, env, ctx)),
            index: Box::new(lower_expr(index, env, ctx)),
        },
        Expr::Slice {
            object, start, end, ..
        } => HirExprKind::Slice {
            object: Box::new(lower_expr(object, env, ctx)),
            start: start.as_ref().map(|e| Box::new(lower_expr(e, env, ctx))),
            end: end.as_ref().map(|e| Box::new(lower_expr(e, env, ctx))),
        },
        Expr::Constructor {
            type_name, fields, ..
        } => HirExprKind::Constructor {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(n, e)| (n.clone(), lower_expr(e, env, ctx)))
                .collect(),
        },
        Expr::Call { callee, args, .. } => HirExprKind::Call {
            callee: Box::new(lower_expr(callee, env, ctx)),
            args: args.iter().map(|a| lower_expr(a, env, ctx)).collect(),
        },
        Expr::Binary {
            left, op, right, ..
        } => HirExprKind::Binary {
            left: Box::new(lower_expr(left, env, ctx)),
            op: *op,
            right: Box::new(lower_expr(right, env, ctx)),
        },
        Expr::Unary { op, expr, .. } => HirExprKind::Unary {
            op: *op,
            expr: Box::new(lower_expr(expr, env, ctx)),
        },
        Expr::Async { expr, .. } => HirExprKind::Async {
            expr: Box::new(lower_expr(expr, env, ctx)),
        },
        Expr::Await { expr, .. } => HirExprKind::Await {
            expr: Box::new(lower_expr(expr, env, ctx)),
        },
        Expr::Question { expr, .. } => HirExprKind::Question {
            expr: Box::new(lower_expr(expr, env, ctx)),
        },
        Expr::DotResult { .. } => HirExprKind::DotResult,
        Expr::Old { expr, .. } => HirExprKind::Old {
            expr: Box::new(lower_expr(expr, env, ctx)),
        },
        Expr::EnumVariant {
            enum_name, variant, ..
        } => HirExprKind::EnumVariant {
            enum_name: enum_name.clone(),
            variant: variant.clone(),
        },
    };
    HirExpr::new(kind, ty)
}

fn expr_type_hint(expr: &Expr, env: &BTreeMap<String, TypeKind>) -> TypeKind {
    match expr {
        Expr::Ident { name, .. } => env.get(name).cloned().unwrap_or(TypeKind::Unknown),
        Expr::Int { .. } => TypeKind::Int,
        Expr::Float { .. } => TypeKind::Float,
        Expr::Bool { .. } => TypeKind::Bool,
        Expr::String { .. } => TypeKind::Str,
        Expr::List { items, .. } => {
            if let Some(first) = items.first() {
                TypeKind::List(Box::new(expr_type_hint(first, env)))
            } else {
                TypeKind::List(Box::new(TypeKind::Unknown))
            }
        }
        Expr::Map { entries, .. } => {
            if let Some((first_key, first_value)) = entries.first() {
                TypeKind::Map(
                    Box::new(expr_type_hint(first_key, env)),
                    Box::new(expr_type_hint(first_value, env)),
                )
            } else {
                TypeKind::Map(Box::new(TypeKind::Unknown), Box::new(TypeKind::Unknown))
            }
        }
        Expr::Member { object, field, .. } => {
            let object_ty = expr_type_hint(object, env);
            match field.as_str() {
                "len" | "balance" => TypeKind::Int,
                _ => match object_ty {
                    TypeKind::Map(key_ty, value_ty)
                        if matches!(*key_ty, TypeKind::Str) && !is_container_member_api(field) =>
                    {
                        *value_ty
                    }
                    other => other,
                },
            }
        }
        Expr::Index { object, .. } => match expr_type_hint(object, env) {
            TypeKind::List(inner) => *inner,
            TypeKind::Map(_, value) => *value,
            TypeKind::Str => TypeKind::Int,
            _ => TypeKind::Unknown,
        },
        Expr::Slice { object, .. } => match expr_type_hint(object, env) {
            TypeKind::List(inner) => TypeKind::List(inner),
            TypeKind::Str => TypeKind::Str,
            _ => TypeKind::Unknown,
        },
        Expr::Call { callee, args, .. } => {
            if let Expr::Ident { name, .. } = &**callee {
                return match name.as_str() {
                    "len" | "min" | "cpu_count" => TypeKind::Int,
                    "sorted_desc" => TypeKind::Bool,
                    "print" | "println" => TypeKind::Void,
                    _ => TypeKind::Unknown,
                };
            }
            if let Some((namespace, field)) = extract_stdlib_call_target(callee) {
                if let Some(ty) = stdlib_namespace_return_hint(&namespace, &field) {
                    return ty;
                }
            }
            if let Expr::Member { object, field, .. } = &**callee {
                let object_ty = expr_type_hint(object, env);
                return match field.as_str() {
                    "len" => TypeKind::Int,
                    "append" | "set" => TypeKind::Void,
                    "contains" | "remove" => TypeKind::Bool,
                    "get" => match object_ty {
                        TypeKind::List(inner) => *inner,
                        TypeKind::Map(_, value) => *value,
                        _ => TypeKind::Unknown,
                    },
                    _ => {
                        // Keep a conservative fallback for call-like members outside
                        // the container/channels surface.
                        if field == "recv" {
                            if let TypeKind::Chan(inner) = object_ty {
                                *inner
                            } else {
                                TypeKind::Unknown
                            }
                        } else if field == "send" || field == "close" || field == "warn" {
                            TypeKind::Void
                        } else if field == "sort_desc" || field == "take" || field == "listen" {
                            TypeKind::Unknown
                        } else if let Some(first_arg) = args.first() {
                            expr_type_hint(first_arg, env)
                        } else {
                            TypeKind::Unknown
                        }
                    }
                };
            }
            TypeKind::Unknown
        }
        Expr::Binary {
            op, left, right, ..
        } => match op {
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => TypeKind::Bool,
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                let lt = expr_type_hint(left, env);
                let rt = expr_type_hint(right, env);
                if matches!(lt, TypeKind::Str)
                    && matches!(rt, TypeKind::Str)
                    && matches!(op, BinaryOp::Add)
                {
                    return TypeKind::Str;
                }
                if matches!(lt, TypeKind::Float) || matches!(rt, TypeKind::Float) {
                    TypeKind::Float
                } else {
                    TypeKind::Int
                }
            }
        },
        Expr::Unary { op, expr, .. } => match op {
            UnaryOp::Not => TypeKind::Bool,
            UnaryOp::Neg => expr_type_hint(expr, env),
        },
        Expr::Async { expr, .. } | Expr::Await { expr, .. } => expr_type_hint(expr, env),
        Expr::Question { expr, .. } => match expr_type_hint(expr, env) {
            TypeKind::Result(ok, _) => *ok,
            _ => TypeKind::Unknown,
        },
        Expr::DotResult { .. } => TypeKind::Unknown,
        Expr::Old { expr, .. } => expr_type_hint(expr, env),
        Expr::Constructor { type_name, .. } => TypeKind::UserType(type_name.clone()),
        Expr::EnumVariant { enum_name, .. } => TypeKind::Enum(enum_name.clone()),
    }
}

fn infer_expr(
    expr: &Expr,
    env: &BTreeMap<String, TypeKind>,
    ctx: &TypeContext,
    context: ContractContext,
    diagnostics: &mut Diagnostics,
    observed_effects: &mut BTreeSet<String>,
) -> TypeKind {
    let sigs = ctx.sigs;
    match expr {
        Expr::Ident { name, span } => {
            if let Some(t) = env.get(name) {
                return t.clone();
            }
            if sigs.contains_key(name) || is_builtin_ident(name) {
                return TypeKind::Unknown;
            }
            diagnostics.push(Diagnostic::new(
                "E2001",
                Severity::Error,
                format!("unknown identifier `{name}`"),
                *span,
            ));
            TypeKind::Unknown
        }
        Expr::Int { .. } => TypeKind::Int,
        Expr::Float { .. } => TypeKind::Float,
        Expr::Bool { .. } => TypeKind::Bool,
        Expr::String { .. } => TypeKind::Str,
        Expr::List { items, .. } => {
            observed_effects.insert("alloc".to_string());
            if let Some(first) = items.first() {
                TypeKind::List(Box::new(infer_expr(
                    first,
                    env,
                    ctx,
                    context,
                    diagnostics,
                    observed_effects,
                )))
            } else {
                TypeKind::List(Box::new(TypeKind::Unknown))
            }
        }
        Expr::Map { entries, .. } => {
            observed_effects.insert("alloc".to_string());
            if entries.is_empty() {
                return TypeKind::Map(Box::new(TypeKind::Unknown), Box::new(TypeKind::Unknown));
            }

            let mut key_ty = TypeKind::Unknown;
            let mut value_ty = TypeKind::Unknown;
            for (idx, (key, value)) in entries.iter().enumerate() {
                let inferred_key =
                    infer_expr(key, env, ctx, context, diagnostics, observed_effects);
                let inferred_value =
                    infer_expr(value, env, ctx, context, diagnostics, observed_effects);
                if idx == 0 {
                    key_ty = inferred_key;
                    value_ty = inferred_value;
                    continue;
                }
                if !type_compatible(&key_ty, &inferred_key)
                    && !matches!(key_ty, TypeKind::Unknown)
                    && !matches!(inferred_key, TypeKind::Unknown)
                {
                    diagnostics.push(Diagnostic::new(
                        "E2208",
                        Severity::Error,
                        format!(
                            "map literal key type mismatch: expected `{}`, got `{}`",
                            type_name(&key_ty),
                            type_name(&inferred_key)
                        ),
                        key.span(),
                    ));
                    key_ty = TypeKind::Unknown;
                } else if matches!(key_ty, TypeKind::Unknown) {
                    key_ty = inferred_key;
                }

                if !type_compatible(&value_ty, &inferred_value)
                    && !matches!(value_ty, TypeKind::Unknown)
                    && !matches!(inferred_value, TypeKind::Unknown)
                {
                    diagnostics.push(Diagnostic::new(
                        "E2209",
                        Severity::Error,
                        format!(
                            "map literal value type mismatch: expected `{}`, got `{}`",
                            type_name(&value_ty),
                            type_name(&inferred_value)
                        ),
                        value.span(),
                    ));
                    value_ty = TypeKind::Unknown;
                } else if matches!(value_ty, TypeKind::Unknown) {
                    value_ty = inferred_value;
                }
            }
            TypeKind::Map(Box::new(key_ty), Box::new(value_ty))
        }
        Expr::Constructor {
            type_name: ctor_type_name,
            fields,
            span,
        } => {
            observed_effects.insert("alloc".to_string());
            if let Some(type_fields) = ctx.type_defs.get(ctor_type_name) {
                let mut provided: BTreeSet<&str> = BTreeSet::new();
                for (field_name, field_expr) in fields {
                    if !type_fields.iter().any(|(f, _)| f == field_name) {
                        diagnostics.push(Diagnostic::new(
                            "E2252",
                            Severity::Error,
                            format!("type `{ctor_type_name}` has no field `{field_name}`"),
                            *span,
                        ));
                    } else if !provided.insert(field_name.as_str()) {
                        diagnostics.push(Diagnostic::new(
                            "E2253",
                            Severity::Error,
                            format!("duplicate field `{field_name}` in constructor"),
                            *span,
                        ));
                    } else {
                        let expected_ty = type_fields
                            .iter()
                            .find(|(f, _)| f == field_name)
                            .map(|(_, t)| t);
                        let actual = infer_expr(
                            field_expr,
                            env,
                            ctx,
                            context,
                            diagnostics,
                            observed_effects,
                        );
                        if let Some(exp) = expected_ty {
                            if !type_compatible(exp, &actual)
                                && !matches!(actual, TypeKind::Unknown)
                                && !matches!(exp, TypeKind::Unknown)
                            {
                                diagnostics.push(Diagnostic::new(
                                    "E2254",
                                    Severity::Error,
                                    format!(
                                        "field `{field_name}` type mismatch: expected `{}`, got `{}`",
                                        type_name(exp),
                                        type_name(&actual)
                                    ),
                                    field_expr.span(),
                                ));
                            }
                        }
                    }
                }
                for (fname, _) in type_fields {
                    if !fields.iter().any(|(n, _)| n == fname) {
                        diagnostics.push(Diagnostic::new(
                            "E2255",
                            Severity::Error,
                            format!("missing required field `{fname}` in constructor for type `{ctor_type_name}`"),
                            *span,
                        ));
                    }
                }
            } else {
                diagnostics.push(Diagnostic::new(
                    "E2256",
                    Severity::Error,
                    format!("unknown type `{ctor_type_name}`"),
                    *span,
                ));
            }
            TypeKind::UserType(ctor_type_name.clone())
        }
        Expr::EnumVariant {
            enum_name,
            variant,
            span,
        } => {
            if let Some(variants) = ctx.enum_defs.get(enum_name) {
                if !variants.contains(variant) {
                    diagnostics.push(Diagnostic::new(
                        "E2250",
                        Severity::Error,
                        format!("enum `{enum_name}` has no variant `{variant}`"),
                        *span,
                    ));
                }
            } else {
                diagnostics.push(Diagnostic::new(
                    "E2257",
                    Severity::Error,
                    format!("unknown enum `{enum_name}`"),
                    *span,
                ));
            }
            TypeKind::Enum(enum_name.clone())
        }
        Expr::Member {
            object,
            field,
            span,
        } => {
            if let Expr::Ident {
                name: enum_name, ..
            } = &**object
            {
                if ctx.enum_defs.contains_key(enum_name) {
                    if let Some(variants) = ctx.enum_defs.get(enum_name) {
                        if !variants.contains(field) {
                            diagnostics.push(Diagnostic::new(
                                "E2250",
                                Severity::Error,
                                format!("enum `{enum_name}` has no variant `{field}`",),
                                *span,
                            ));
                        }
                    }
                    return TypeKind::Enum(enum_name.clone());
                }
            }
            let base = infer_expr(object, env, ctx, context, diagnostics, observed_effects);
            match field.as_str() {
                "len" => TypeKind::Int,
                "balance" => TypeKind::Int,
                _ => match &base {
                    TypeKind::UserType(type_name) => {
                        if let Some(fields) = ctx.type_defs.get(type_name) {
                            if let Some((_, ty)) = fields.iter().find(|(n, _)| n == field) {
                                return ty.clone();
                            }
                            diagnostics.push(Diagnostic::new(
                                "E2251",
                                Severity::Error,
                                format!("type `{type_name}` has no field `{field}`",),
                                *span,
                            ));
                            TypeKind::Unknown
                        } else {
                            base
                        }
                    }
                    TypeKind::Map(key_ty, value_ty)
                        if matches!(**key_ty, TypeKind::Str) && !is_container_member_api(field) =>
                    {
                        (**value_ty).clone()
                    }
                    other => other.clone(),
                },
            }
        }
        Expr::Index {
            object,
            index,
            span,
        } => {
            let object_ty = infer_expr(object, env, ctx, context, diagnostics, observed_effects);
            let index_ty = infer_expr(index, env, ctx, context, diagnostics, observed_effects);
            match object_ty {
                TypeKind::List(inner) => {
                    if !matches!(index_ty, TypeKind::Int | TypeKind::Unknown) {
                        diagnostics.push(Diagnostic::new(
                            "E2230",
                            Severity::Error,
                            "list index must be Int",
                            index.span(),
                        ));
                    }
                    *inner
                }
                TypeKind::Map(key_ty, value_ty) => {
                    if !type_compatible(&key_ty, &index_ty)
                        && !matches!(*key_ty, TypeKind::Unknown)
                        && !matches!(index_ty, TypeKind::Unknown)
                    {
                        diagnostics.push(Diagnostic::new(
                            "E2231",
                            Severity::Error,
                            format!(
                                "map index key type mismatch: expected `{}`, got `{}`",
                                type_name(&key_ty),
                                type_name(&index_ty)
                            ),
                            index.span(),
                        ));
                    }
                    *value_ty
                }
                TypeKind::Str => {
                    if !matches!(index_ty, TypeKind::Int | TypeKind::Unknown) {
                        diagnostics.push(Diagnostic::new(
                            "E2232",
                            Severity::Error,
                            "string index must be Int byte offset",
                            index.span(),
                        ));
                    }
                    TypeKind::Int
                }
                TypeKind::Unknown => TypeKind::Unknown,
                other => {
                    diagnostics.push(Diagnostic::new(
                        "E2233",
                        Severity::Error,
                        format!(
                            "indexing is only supported for List<T>, Map<K,V>, and Str; got `{}`",
                            type_name(&other)
                        ),
                        *span,
                    ));
                    TypeKind::Unknown
                }
            }
        }
        Expr::Slice {
            object,
            start,
            end,
            span,
        } => {
            let object_ty = infer_expr(object, env, ctx, context, diagnostics, observed_effects);
            if let Some(start) = start {
                let start_ty = infer_expr(start, env, ctx, context, diagnostics, observed_effects);
                if !matches!(start_ty, TypeKind::Int | TypeKind::Unknown) {
                    diagnostics.push(Diagnostic::new(
                        "E2234",
                        Severity::Error,
                        "slice start index must be Int",
                        start.span(),
                    ));
                }
            }
            if let Some(end) = end {
                let end_ty = infer_expr(end, env, ctx, context, diagnostics, observed_effects);
                if !matches!(end_ty, TypeKind::Int | TypeKind::Unknown) {
                    diagnostics.push(Diagnostic::new(
                        "E2235",
                        Severity::Error,
                        "slice end index must be Int",
                        end.span(),
                    ));
                }
            }
            match object_ty {
                TypeKind::Str => TypeKind::Str,
                TypeKind::List(inner) => TypeKind::List(inner),
                TypeKind::Unknown => TypeKind::Unknown,
                other => {
                    diagnostics.push(Diagnostic::new(
                        "E2236",
                        Severity::Error,
                        format!(
                            "slicing is only supported for List<T> and Str; got `{}`",
                            type_name(&other)
                        ),
                        *span,
                    ));
                    TypeKind::Unknown
                }
            }
        }
        Expr::Call { callee, args, .. } => {
            let callee_ty = infer_expr(callee, env, ctx, context, diagnostics, observed_effects);
            let mut arg_types = Vec::with_capacity(args.len());
            for arg in args {
                arg_types.push(infer_expr(
                    arg,
                    env,
                    ctx,
                    context,
                    diagnostics,
                    observed_effects,
                ));
            }
            if let Expr::Ident { name, .. } = &**callee {
                match name.as_str() {
                    "chan" => {
                        observed_effects.insert("alloc".to_string());
                        observed_effects.insert("concurrency".to_string());
                        return TypeKind::Chan(Box::new(TypeKind::Unknown));
                    }
                    "len" | "min" | "cpu_count" => return TypeKind::Int,
                    "sorted_desc" => return TypeKind::Bool,
                    "print" | "println" => {
                        observed_effects.insert("io".to_string());
                        return TypeKind::Void;
                    }
                    "ok" => {
                        return TypeKind::Result(
                            Box::new(TypeKind::Void),
                            Box::new(TypeKind::Unknown),
                        )
                    }
                    _ => {}
                }
                if let Some(ret) = sigs.get(name).and_then(|r| r.clone()) {
                    return ret;
                }
            }
            if let Expr::Member { field, .. } = &**callee {
                if let Some((namespace, stdlib_field)) = extract_stdlib_call_target(callee) {
                    if let Some(ret) = infer_stdlib_namespace_call(
                        &namespace,
                        &stdlib_field,
                        args,
                        &arg_types,
                        ctx.type_defs,
                        diagnostics,
                        observed_effects,
                        callee.span(),
                    ) {
                        return ret;
                    }
                }
                match field.as_str() {
                    "sort_desc" | "take" => {
                        observed_effects.insert("alloc".to_string());
                        return TypeKind::Unknown;
                    }
                    "recv" => {
                        observed_effects.insert("concurrency".to_string());
                        if let TypeKind::Chan(inner) = callee_ty {
                            return *inner;
                        }
                        return TypeKind::Unknown;
                    }
                    "send" => {
                        observed_effects.insert("concurrency".to_string());
                        if let Some(value_ty) = arg_types.first() {
                            if !is_sendable_type(value_ty) {
                                diagnostics.push(Diagnostic::new(
                                    "E3201",
                                    Severity::Error,
                                    format!(
                                        "non-sendable value passed to channel send: inferred `{}`",
                                        type_name(value_ty)
                                    ),
                                    args.first().map(|arg| arg.span()).unwrap_or(callee.span()),
                                ));
                            }
                        }
                        return TypeKind::Void;
                    }
                    "close" => {
                        observed_effects.insert("concurrency".to_string());
                        return TypeKind::Void;
                    }
                    "listen" => {
                        observed_effects.insert("io".to_string());
                        return TypeKind::Unknown;
                    }
                    "warn" => {
                        observed_effects.insert("io".to_string());
                        return TypeKind::Void;
                    }
                    "append" => {
                        observed_effects.insert("mut_state".to_string());
                        observed_effects.insert("alloc".to_string());
                        let Some(value_ty) = arg_types.first() else {
                            diagnostics.push(Diagnostic::new(
                                "E2210",
                                Severity::Error,
                                "list.append expects one argument",
                                callee.span(),
                            ));
                            return TypeKind::Unknown;
                        };
                        match callee_ty {
                            TypeKind::List(inner) => {
                                if !type_compatible(&inner, value_ty)
                                    && !matches!(*inner, TypeKind::Unknown)
                                    && !matches!(value_ty, TypeKind::Unknown)
                                {
                                    diagnostics.push(Diagnostic::new(
                                        "E2211",
                                        Severity::Error,
                                        format!(
                                            "list.append value type mismatch: expected `{}`, got `{}`",
                                            type_name(&inner),
                                            type_name(value_ty)
                                        ),
                                        args[0].span(),
                                    ));
                                }
                                return TypeKind::Void;
                            }
                            TypeKind::Unknown => return TypeKind::Unknown,
                            other => {
                                diagnostics.push(Diagnostic::new(
                                    "E2212",
                                    Severity::Error,
                                    format!(
                                        "`.append(...)` is only supported for List<T>, got `{}`",
                                        type_name(&other)
                                    ),
                                    callee.span(),
                                ));
                                return TypeKind::Unknown;
                            }
                        }
                    }
                    "get" => match callee_ty {
                        TypeKind::List(inner) => {
                            if arg_types.len() != 1 {
                                diagnostics.push(Diagnostic::new(
                                    "E2213",
                                    Severity::Error,
                                    "list.get expects one index argument",
                                    callee.span(),
                                ));
                                return TypeKind::Unknown;
                            }
                            if !matches!(arg_types[0], TypeKind::Int | TypeKind::Unknown) {
                                diagnostics.push(Diagnostic::new(
                                    "E2214",
                                    Severity::Error,
                                    "list.get index must be Int",
                                    args[0].span(),
                                ));
                            }
                            return *inner;
                        }
                        TypeKind::Map(key_ty, value_ty) => {
                            if arg_types.len() != 1 {
                                diagnostics.push(Diagnostic::new(
                                    "E2215",
                                    Severity::Error,
                                    "map.get expects one key argument",
                                    callee.span(),
                                ));
                                return TypeKind::Unknown;
                            }
                            if !type_compatible(&key_ty, &arg_types[0])
                                && !matches!(*key_ty, TypeKind::Unknown)
                                && !matches!(arg_types[0], TypeKind::Unknown)
                            {
                                diagnostics.push(Diagnostic::new(
                                    "E2216",
                                    Severity::Error,
                                    format!(
                                        "map.get key type mismatch: expected `{}`, got `{}`",
                                        type_name(&key_ty),
                                        type_name(&arg_types[0])
                                    ),
                                    args[0].span(),
                                ));
                            }
                            return *value_ty;
                        }
                        TypeKind::Unknown => return TypeKind::Unknown,
                        other => {
                            diagnostics.push(Diagnostic::new(
                                "E2217",
                                Severity::Error,
                                format!(
                                    "`.get(...)` is only supported for List<T> and Map<K,V>, got `{}`",
                                    type_name(&other)
                                ),
                                callee.span(),
                            ));
                            return TypeKind::Unknown;
                        }
                    },
                    "set" => {
                        observed_effects.insert("mut_state".to_string());
                        match callee_ty {
                            TypeKind::List(inner) => {
                                if arg_types.len() != 2 {
                                    diagnostics.push(Diagnostic::new(
                                        "E2218",
                                        Severity::Error,
                                        "list.set expects index and value arguments",
                                        callee.span(),
                                    ));
                                    return TypeKind::Unknown;
                                }
                                if !matches!(arg_types[0], TypeKind::Int | TypeKind::Unknown) {
                                    diagnostics.push(Diagnostic::new(
                                        "E2219",
                                        Severity::Error,
                                        "list.set index must be Int",
                                        args[0].span(),
                                    ));
                                }
                                if !type_compatible(&inner, &arg_types[1])
                                    && !matches!(*inner, TypeKind::Unknown)
                                    && !matches!(arg_types[1], TypeKind::Unknown)
                                {
                                    diagnostics.push(Diagnostic::new(
                                        "E2220",
                                        Severity::Error,
                                        format!(
                                            "list.set value type mismatch: expected `{}`, got `{}`",
                                            type_name(&inner),
                                            type_name(&arg_types[1])
                                        ),
                                        args[1].span(),
                                    ));
                                }
                                return TypeKind::Void;
                            }
                            TypeKind::Map(key_ty, value_ty) => {
                                if arg_types.len() != 2 {
                                    diagnostics.push(Diagnostic::new(
                                        "E2221",
                                        Severity::Error,
                                        "map.set expects key and value arguments",
                                        callee.span(),
                                    ));
                                    return TypeKind::Unknown;
                                }
                                if !type_compatible(&key_ty, &arg_types[0])
                                    && !matches!(*key_ty, TypeKind::Unknown)
                                    && !matches!(arg_types[0], TypeKind::Unknown)
                                {
                                    diagnostics.push(Diagnostic::new(
                                        "E2222",
                                        Severity::Error,
                                        format!(
                                            "map.set key type mismatch: expected `{}`, got `{}`",
                                            type_name(&key_ty),
                                            type_name(&arg_types[0])
                                        ),
                                        args[0].span(),
                                    ));
                                }
                                if !type_compatible(&value_ty, &arg_types[1])
                                    && !matches!(*value_ty, TypeKind::Unknown)
                                    && !matches!(arg_types[1], TypeKind::Unknown)
                                {
                                    diagnostics.push(Diagnostic::new(
                                        "E2223",
                                        Severity::Error,
                                        format!(
                                            "map.set value type mismatch: expected `{}`, got `{}`",
                                            type_name(&value_ty),
                                            type_name(&arg_types[1])
                                        ),
                                        args[1].span(),
                                    ));
                                }
                                return TypeKind::Void;
                            }
                            TypeKind::Unknown => return TypeKind::Unknown,
                            other => {
                                diagnostics.push(Diagnostic::new(
                                    "E2224",
                                    Severity::Error,
                                    format!(
                                        "`.set(...)` is only supported for List<T> and Map<K,V>, got `{}`",
                                        type_name(&other)
                                    ),
                                    callee.span(),
                                ));
                                return TypeKind::Unknown;
                            }
                        }
                    }
                    "contains" | "remove" => {
                        if arg_types.len() != 1 {
                            diagnostics.push(Diagnostic::new(
                                "E2225",
                                Severity::Error,
                                format!("{field} expects one key argument"),
                                callee.span(),
                            ));
                            return TypeKind::Unknown;
                        }
                        match callee_ty {
                            TypeKind::Map(key_ty, _) => {
                                if !type_compatible(&key_ty, &arg_types[0])
                                    && !matches!(*key_ty, TypeKind::Unknown)
                                    && !matches!(arg_types[0], TypeKind::Unknown)
                                {
                                    diagnostics.push(Diagnostic::new(
                                        "E2226",
                                        Severity::Error,
                                        format!(
                                            "map.{field} key type mismatch: expected `{}`, got `{}`",
                                            type_name(&key_ty),
                                            type_name(&arg_types[0])
                                        ),
                                        args[0].span(),
                                    ));
                                }
                                if field == "remove" {
                                    observed_effects.insert("mut_state".to_string());
                                }
                                return TypeKind::Bool;
                            }
                            TypeKind::Unknown => return TypeKind::Unknown,
                            other => {
                                diagnostics.push(Diagnostic::new(
                                    "E2227",
                                    Severity::Error,
                                    format!(
                                        "`.{field}(...)` is only supported for Map<K,V>, got `{}`",
                                        type_name(&other)
                                    ),
                                    callee.span(),
                                ));
                                return TypeKind::Unknown;
                            }
                        }
                    }
                    "len" => {
                        if !arg_types.is_empty() {
                            diagnostics.push(Diagnostic::new(
                                "E2228",
                                Severity::Error,
                                "`.len()` expects no arguments",
                                callee.span(),
                            ));
                            return TypeKind::Unknown;
                        }
                        return TypeKind::Int;
                    }
                    _ => {}
                }
            }
            TypeKind::Unknown
        }
        Expr::Binary {
            left,
            op,
            right,
            span,
        } => {
            let lt = infer_expr(left, env, ctx, context, diagnostics, observed_effects);
            let rt = infer_expr(right, env, ctx, context, diagnostics, observed_effects);
            match op {
                BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge => TypeKind::Bool,
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                    if matches!(op, BinaryOp::Add)
                        && matches!(lt, TypeKind::Str)
                        && matches!(rt, TypeKind::Str)
                    {
                        return TypeKind::Str;
                    }
                    if (matches!(lt, TypeKind::Str) || matches!(rt, TypeKind::Str))
                        && !matches!(lt, TypeKind::Unknown)
                        && !matches!(rt, TypeKind::Unknown)
                    {
                        diagnostics.push(Diagnostic::new(
                            "E2229",
                            Severity::Error,
                            "string operands are only supported with `+` and both sides must be Str",
                            *span,
                        ));
                        return TypeKind::Unknown;
                    }
                    if !type_compatible(&lt, &rt)
                        && !matches!(lt, TypeKind::Unknown)
                        && !matches!(rt, TypeKind::Unknown)
                    {
                        diagnostics.push(Diagnostic::new(
                            "E2202",
                            Severity::Error,
                            format!(
                                "binary operation type mismatch: left `{}`, right `{}`",
                                type_name(&lt),
                                type_name(&rt)
                            ),
                            *span,
                        ));
                    }
                    if matches!(lt, TypeKind::Float) || matches!(rt, TypeKind::Float) {
                        TypeKind::Float
                    } else {
                        TypeKind::Int
                    }
                }
            }
        }
        Expr::Unary { op, expr, .. } => {
            let t = infer_expr(expr, env, ctx, context, diagnostics, observed_effects);
            match op {
                UnaryOp::Neg => t,
                UnaryOp::Not => TypeKind::Bool,
            }
        }
        Expr::Async { expr, span } => {
            observed_effects.insert("concurrency".to_string());
            if !matches!(&**expr, Expr::Call { .. }) {
                diagnostics.push(Diagnostic::new(
                    "E3204",
                    Severity::Error,
                    "`async` expects a call expression",
                    *span,
                ));
            }
            check_go_sendability(expr, env, expr_type_hint, diagnostics);
            infer_expr(expr, env, ctx, context, diagnostics, observed_effects)
        }
        Expr::Await { expr, .. } => {
            observed_effects.insert("concurrency".to_string());
            infer_expr(expr, env, ctx, context, diagnostics, observed_effects)
        }
        Expr::Question { expr, span } => {
            let inner = infer_expr(expr, env, ctx, context, diagnostics, observed_effects);
            match inner {
                TypeKind::Result(ok, _err) => *ok,
                TypeKind::Unknown => TypeKind::Unknown,
                other => {
                    diagnostics.push(Diagnostic::new(
                        "E2203",
                        Severity::Error,
                        format!("`?` expects Result<T,E>, got `{}`", type_name(&other)),
                        *span,
                    ));
                    TypeKind::Unknown
                }
            }
        }
        Expr::DotResult { span } => {
            if context != ContractContext::Ensure {
                diagnostics.push(Diagnostic::new(
                    "E2204",
                    Severity::Error,
                    "`.` result placeholder is only valid inside `@ensure`",
                    *span,
                ));
            }
            TypeKind::Unknown
        }
        Expr::Old { expr, span } => {
            if context != ContractContext::Ensure {
                diagnostics.push(Diagnostic::new(
                    "E2205",
                    Severity::Error,
                    "`old(...)` is only valid inside `@ensure`",
                    *span,
                ));
            }
            infer_expr(expr, env, ctx, context, diagnostics, observed_effects)
        }
    }
}

fn validate_contract_expr(expr: &Expr, context: ContractContext, diagnostics: &mut Diagnostics) {
    match expr {
        Expr::DotResult { span } if context != ContractContext::Ensure => {
            diagnostics.push(Diagnostic::new(
                "E2206",
                Severity::Error,
                "`.` is only valid in `@ensure`",
                *span,
            ))
        }
        Expr::Old { span, .. } if context != ContractContext::Ensure => {
            diagnostics.push(Diagnostic::new(
                "E2207",
                Severity::Error,
                "`old(...)` is only valid in `@ensure`",
                *span,
            ))
        }
        Expr::Member { object, .. }
        | Expr::Question { expr: object, .. }
        | Expr::Unary { expr: object, .. }
        | Expr::Async { expr: object, .. }
        | Expr::Await { expr: object, .. } => {
            validate_contract_expr(object, context, diagnostics);
        }
        Expr::Index { object, index, .. } => {
            validate_contract_expr(object, context, diagnostics);
            validate_contract_expr(index, context, diagnostics);
        }
        Expr::Slice {
            object, start, end, ..
        } => {
            validate_contract_expr(object, context, diagnostics);
            if let Some(start) = start {
                validate_contract_expr(start, context, diagnostics);
            }
            if let Some(end) = end {
                validate_contract_expr(end, context, diagnostics);
            }
        }
        Expr::Call { callee, args, .. } => {
            validate_contract_expr(callee, context, diagnostics);
            for arg in args {
                validate_contract_expr(arg, context, diagnostics);
            }
        }
        Expr::Binary { left, right, .. } => {
            validate_contract_expr(left, context, diagnostics);
            validate_contract_expr(right, context, diagnostics);
        }
        Expr::List { items, .. } => {
            for item in items {
                validate_contract_expr(item, context, diagnostics);
            }
        }
        Expr::Map { entries, .. } => {
            for (k, v) in entries {
                validate_contract_expr(k, context, diagnostics);
                validate_contract_expr(v, context, diagnostics);
            }
        }
        _ => {}
    }
}

fn resolve_type_ref(
    t: &TypeRef,
    type_defs: &BTreeMap<String, Vec<(String, TypeKind)>>,
    enum_defs: &BTreeMap<String, Vec<String>>,
) -> TypeKind {
    let raw = t.raw.replace(' ', "");
    if raw.is_empty() {
        return TypeKind::Unknown;
    }
    if type_defs.contains_key(&raw) {
        return TypeKind::UserType(raw);
    }
    if enum_defs.contains_key(&raw) {
        return TypeKind::Enum(raw);
    }
    if raw == "Json" {
        return TypeKind::Json;
    }
    if raw == "JsonBuilder" {
        return TypeKind::JsonBuilder;
    }
    if raw.starts_with("List<") && raw.ends_with('>') {
        let inner = &raw[5..raw.len() - 1];
        return TypeKind::List(Box::new(resolve_type_ref(
            &TypeRef {
                raw: inner.to_string(),
            },
            type_defs,
            enum_defs,
        )));
    }
    if let Some((ok, err)) = split_generic_pair(&raw, "Result") {
        return TypeKind::Result(
            Box::new(resolve_type_ref(&TypeRef { raw: ok }, type_defs, enum_defs)),
            Box::new(resolve_type_ref(
                &TypeRef { raw: err },
                type_defs,
                enum_defs,
            )),
        );
    }
    if let Some((key, value)) = split_generic_pair(&raw, "Map") {
        return TypeKind::Map(
            Box::new(resolve_type_ref(
                &TypeRef { raw: key },
                type_defs,
                enum_defs,
            )),
            Box::new(resolve_type_ref(
                &TypeRef { raw: value },
                type_defs,
                enum_defs,
            )),
        );
    }
    if raw.starts_with("Chan<") && raw.ends_with('>') {
        let inner = &raw[5..raw.len() - 1];
        return TypeKind::Chan(Box::new(resolve_type_ref(
            &TypeRef {
                raw: inner.to_string(),
            },
            type_defs,
            enum_defs,
        )));
    }
    parse_type_ref(t)
}

fn parse_type_ref(t: &TypeRef) -> TypeKind {
    let raw = t.raw.replace(' ', "");
    if raw.is_empty() {
        return TypeKind::Unknown;
    }
    if raw == "Int" {
        return TypeKind::Int;
    }
    if raw == "Float" {
        return TypeKind::Float;
    }
    if raw == "Bool" {
        return TypeKind::Bool;
    }
    if raw == "Str" {
        return TypeKind::Str;
    }
    if raw == "Json" {
        return TypeKind::Json;
    }
    if raw == "JsonBuilder" {
        return TypeKind::JsonBuilder;
    }
    if raw == "Void" {
        return TypeKind::Void;
    }
    if raw.starts_with("List<") && raw.ends_with('>') {
        let inner = &raw[5..raw.len() - 1];
        return TypeKind::List(Box::new(parse_type_ref(&TypeRef {
            raw: inner.to_string(),
        })));
    }
    if let Some((ok, err)) = split_generic_pair(&raw, "Result") {
        return TypeKind::Result(
            Box::new(parse_type_ref(&TypeRef { raw: ok })),
            Box::new(parse_type_ref(&TypeRef { raw: err })),
        );
    }
    if let Some((key, value)) = split_generic_pair(&raw, "Map") {
        return TypeKind::Map(
            Box::new(parse_type_ref(&TypeRef { raw: key })),
            Box::new(parse_type_ref(&TypeRef { raw: value })),
        );
    }
    if raw.starts_with("Chan<") && raw.ends_with('>') {
        let inner = &raw[5..raw.len() - 1];
        return TypeKind::Chan(Box::new(parse_type_ref(&TypeRef {
            raw: inner.to_string(),
        })));
    }
    if raw.chars().next().map_or(false, |c| c.is_ascii_uppercase())
        && raw.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return TypeKind::UserType(raw);
    }
    TypeKind::Unknown
}

fn split_generic_pair(raw: &str, outer: &str) -> Option<(String, String)> {
    let prefix = format!("{outer}<");
    if !raw.starts_with(&prefix) || !raw.ends_with('>') {
        return None;
    }
    let inner = &raw[prefix.len()..raw.len() - 1];
    let mut depth = 0i32;
    let mut split_at = None;
    for (idx, ch) in inner.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                split_at = Some(idx);
                break;
            }
            _ => {}
        }
    }
    let idx = split_at?;
    let left = inner[..idx].trim();
    let right = inner[idx + 1..].trim();
    if left.is_empty() || right.is_empty() {
        return None;
    }
    Some((left.to_string(), right.to_string()))
}

fn type_compatible(a: &TypeKind, b: &TypeKind) -> bool {
    if matches!(a, TypeKind::Unknown) || matches!(b, TypeKind::Unknown) || a == b {
        return true;
    }
    match (a, b) {
        (TypeKind::List(a_inner), TypeKind::List(b_inner)) => type_compatible(a_inner, b_inner),
        (TypeKind::Map(a_key, a_value), TypeKind::Map(b_key, b_value)) => {
            type_compatible(a_key, b_key) && type_compatible(a_value, b_value)
        }
        (TypeKind::Result(a_ok, a_err), TypeKind::Result(b_ok, b_err)) => {
            type_compatible(a_ok, b_ok) && type_compatible(a_err, b_err)
        }
        (TypeKind::Chan(a_inner), TypeKind::Chan(b_inner)) => type_compatible(a_inner, b_inner),
        (TypeKind::UserType(a), TypeKind::UserType(b)) => a == b,
        (TypeKind::Enum(a), TypeKind::Enum(b)) => a == b,
        (TypeKind::Int, TypeKind::Float) | (TypeKind::Float, TypeKind::Int) => true,
        _ => false,
    }
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
    }
}

fn unify_return_types(types: &[TypeKind]) -> TypeKind {
    if types.is_empty() {
        return TypeKind::Void;
    }
    let mut current = types[0].clone();
    for t in &types[1..] {
        if !type_compatible(&current, t) {
            return TypeKind::Unknown;
        }
        if matches!(current, TypeKind::Unknown) {
            current = t.clone();
        }
    }
    current
}

fn is_container_member_api(field: &str) -> bool {
    matches!(
        field,
        "append"
            | "get"
            | "set"
            | "contains"
            | "remove"
            | "len"
            | "sort_desc"
            | "take"
            | "recv"
            | "send"
            | "close"
            | "listen"
            | "warn"
    )
}

fn is_known_effect(e: &str) -> bool {
    matches!(
        e,
        "alloc" | "mut_state" | "io" | "net" | "concurrency" | "nondet"
    )
}

fn collect_member_chain(expr: &Expr, parts: &mut Vec<String>) -> bool {
    match expr {
        Expr::Ident { name, .. } => {
            parts.push(name.clone());
            true
        }
        Expr::Member { object, field, .. } => {
            if collect_member_chain(object, parts) {
                parts.push(field.clone());
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn extract_stdlib_call_target(callee: &Expr) -> Option<(String, String)> {
    let Expr::Member { .. } = callee else {
        return None;
    };
    let mut parts = Vec::new();
    if !collect_member_chain(callee, &mut parts) || parts.len() < 2 {
        return None;
    }
    if !is_builtin_ident(&parts[0]) {
        return None;
    }
    let field = parts.pop()?;
    let namespace = parts.join(".");
    Some((namespace, field))
}

fn stdlib_namespace_return_hint(namespace: &str, field: &str) -> Option<TypeKind> {
    match (namespace, field) {
        ("time", "now_ms") | ("time", "monotonic_now_ms") | ("time", "duration_ms") => {
            Some(TypeKind::Int)
        }
        ("time", "sleep_ms") => Some(TypeKind::Void),
        ("path", "join") | ("path", "parent") | ("path", "basename") => Some(TypeKind::Str),
        ("path", "is_absolute") => Some(TypeKind::Bool),
        ("fs", "exists") | ("fs", "write_text") | ("fs", "create_dir") => Some(TypeKind::Bool),
        ("fs", "read_text") => Some(TypeKind::Str),
        ("net", "listen")
        | ("net", "listener_port")
        | ("net", "accept")
        | ("net", "connect")
        | ("net", "write") => Some(TypeKind::Int),
        ("net", "read") | ("net", "resolve") => Some(TypeKind::Str),
        ("net", "close") => Some(TypeKind::Bool),
        ("convert", "to_int") | ("convert", "parse_i64") => Some(TypeKind::Int),
        ("convert", "to_float")
        | ("convert", "parse_f64")
        | ("convert", "i64_to_f64")
        | ("convert", "f64_from_bits") => Some(TypeKind::Float),
        ("convert", "to_str") | ("convert", "to_str_f64") | ("convert", "format_f64") => {
            Some(TypeKind::Str)
        }
        ("convert", "f64_to_bits") => Some(TypeKind::Int),
        ("math", "sqrt") => Some(TypeKind::Float),
        ("str_builder", "new") | ("str_builder", "append") | ("str_builder", "append_char") => {
            Some(TypeKind::Int)
        }
        ("str_builder", "finish") => Some(TypeKind::Str),
        ("json.builder", "new")
        | ("json.builder", "begin_object")
        | ("json.builder", "end_object")
        | ("json.builder", "begin_array")
        | ("json.builder", "end_array")
        | ("json.builder", "key")
        | ("json.builder", "value_null")
        | ("json.builder", "value_bool")
        | ("json.builder", "value_i64")
        | ("json.builder", "value_f64")
        | ("json.builder", "value_str")
        | ("json.builder", "value_json") => Some(TypeKind::JsonBuilder),
        ("json.builder", "finish") => Some(TypeKind::Str),
        ("json", "parse")
        | ("json", "null")
        | ("json", "bool")
        | ("json", "i64")
        | ("json", "f64")
        | ("json", "str") => Some(TypeKind::Json),
        ("json", "stringify") | ("json", "stringify_pretty") => Some(TypeKind::Str),
        ("simd", "f64x2_splat")
        | ("simd", "f64x2_make")
        | ("simd", "f64x2_add")
        | ("simd", "f64x2_sub")
        | ("simd", "f64x2_mul") => Some(TypeKind::Int),
        ("simd", "f64x2_gt") => Some(TypeKind::Int),
        ("simd", "f64x2_extract") => Some(TypeKind::Float),
        ("text", "trim")
        | ("text", "replace")
        | ("text", "to_lower")
        | ("text", "to_upper")
        | ("text", "split_part") => Some(TypeKind::Str),
        ("text", "contains") | ("text", "starts_with") | ("text", "ends_with") => {
            Some(TypeKind::Bool)
        }
        ("text", "byte_len") | ("text", "index_of") => Some(TypeKind::Int),
        ("encoding", "hex_encode")
        | ("encoding", "hex_decode")
        | ("encoding", "base64_encode")
        | ("encoding", "base64_decode")
        | ("encoding", "url_encode")
        | ("encoding", "url_decode") => Some(TypeKind::Str),
        ("log", "info") | ("log", "warn") | ("log", "error") => Some(TypeKind::Void),
        ("env", "get") | ("env", "get_required") => Some(TypeKind::Str),
        ("env", "has") => Some(TypeKind::Bool),
        ("cli", "args_len") => Some(TypeKind::Int),
        ("cli", "arg") => Some(TypeKind::Str),
        ("json", "is_valid") => Some(TypeKind::Bool),
        ("json", "parse_i64") => Some(TypeKind::Int),
        ("json", "stringify_i64") | ("json", "minify") => Some(TypeKind::Str),
        ("regex", "count") => Some(TypeKind::Int),
        ("regex", "replace_all") => Some(TypeKind::Str),
        ("http", "status_text")
        | ("http", "build_request_line")
        | ("http", "get")
        | ("http", "post")
        | ("http", "request") => Some(TypeKind::Str),
        ("http", "default_port") | ("http", "request_status") => Some(TypeKind::Int),
        #[cfg(feature = "bench-runtime")]
        ("bench", "md5_hex")
        | ("bench", "json_canonical")
        | ("bench", "json_repeat_array")
        | ("bench", "secp256k1")
        | ("bench", "edigits") => Some(TypeKind::Str),
        #[cfg(feature = "bench-runtime")]
        ("bench", "http_server_bench")
        | ("bench", "net_listen")
        | ("bench", "net_listener_port")
        | ("bench", "net_accept")
        | ("bench", "net_connect")
        | ("bench", "net_write") => Some(TypeKind::Int),
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_read") => Some(TypeKind::Str),
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_close") => Some(TypeKind::Bool),
        _ => None,
    }
}

fn infer_stdlib_namespace_call(
    namespace: &str,
    field: &str,
    args: &[Expr],
    arg_types: &[TypeKind],
    type_defs: &BTreeMap<String, Vec<(String, TypeKind)>>,
    diagnostics: &mut Diagnostics,
    observed_effects: &mut BTreeSet<String>,
    call_span: Span,
) -> Option<TypeKind> {
    if namespace == "json" {
        if let Some(target_type_name) = field.strip_prefix("encode_") {
            if !type_defs.contains_key(target_type_name) {
                diagnostics.push(Diagnostic::new(
                    "E2239",
                    Severity::Error,
                    format!("unknown json codec target type `{target_type_name}`"),
                    call_span,
                ));
                return Some(TypeKind::Unknown);
            }
            if args.len() != 1 {
                diagnostics.push(Diagnostic::new(
                    "E2237",
                    Severity::Error,
                    format!("`json.{field}` expects 1 argument(s), got {}", args.len()),
                    call_span,
                ));
                return Some(TypeKind::Unknown);
            }
            if let Some(actual) = arg_types.first() {
                if !matches!(actual, TypeKind::Unknown)
                    && !type_compatible(actual, &TypeKind::UserType(target_type_name.to_string()))
                {
                    diagnostics.push(Diagnostic::new(
                        "E2238",
                        Severity::Error,
                        format!(
                            "`json.{field}` argument 1 expects `{}`, got `{}`",
                            target_type_name,
                            type_name(actual)
                        ),
                        args.first().map(|arg| arg.span()).unwrap_or(call_span),
                    ));
                }
            }
            return Some(TypeKind::Str);
        }
        if let Some(target_type_name) = field.strip_prefix("decode_") {
            if !type_defs.contains_key(target_type_name) {
                diagnostics.push(Diagnostic::new(
                    "E2239",
                    Severity::Error,
                    format!("unknown json codec target type `{target_type_name}`"),
                    call_span,
                ));
                return Some(TypeKind::Unknown);
            }
            if args.len() != 2 {
                diagnostics.push(Diagnostic::new(
                    "E2237",
                    Severity::Error,
                    format!("`json.{field}` expects 2 argument(s), got {}", args.len()),
                    call_span,
                ));
                return Some(TypeKind::Unknown);
            }
            if let Some(actual) = arg_types.first() {
                if !matches!(actual, TypeKind::Str | TypeKind::Unknown) {
                    diagnostics.push(Diagnostic::new(
                        "E2238",
                        Severity::Error,
                        format!(
                            "`json.{field}` argument 1 expects `Str`, got `{}`",
                            type_name(actual)
                        ),
                        args.first().map(|arg| arg.span()).unwrap_or(call_span),
                    ));
                }
            }
            if let Some(actual) = arg_types.get(1) {
                if !matches!(actual, TypeKind::Unknown)
                    && !type_compatible(actual, &TypeKind::UserType(target_type_name.to_string()))
                {
                    diagnostics.push(Diagnostic::new(
                        "E2238",
                        Severity::Error,
                        format!(
                            "`json.{field}` argument 2 expects `{}`, got `{}`",
                            target_type_name,
                            type_name(actual)
                        ),
                        args.get(1).map(|arg| arg.span()).unwrap_or(call_span),
                    ));
                }
            }
            return Some(TypeKind::UserType(target_type_name.to_string()));
        }
        if field == "from_map" {
            if args.len() != 1 {
                diagnostics.push(Diagnostic::new(
                    "E2237",
                    Severity::Error,
                    format!("`json.from_map` expects 1 argument(s), got {}", args.len()),
                    call_span,
                ));
                return Some(TypeKind::Unknown);
            }
            return Some(TypeKind::Str);
        }
        if field == "parse" {
            if args.len() != 1 {
                diagnostics.push(Diagnostic::new(
                    "E2237",
                    Severity::Error,
                    format!("`json.parse` expects 1 argument(s), got {}", args.len()),
                    call_span,
                ));
                return Some(TypeKind::Unknown);
            }
            if let Some(actual) = arg_types.first() {
                if !matches!(actual, TypeKind::Str | TypeKind::Unknown) {
                    diagnostics.push(Diagnostic::new(
                        "E2238",
                        Severity::Error,
                        format!("`json.parse` argument 1 expects `Str`, got `{}`", type_name(actual)),
                        args.first().map(|arg| arg.span()).unwrap_or(call_span),
                    ));
                }
            }
            return Some(TypeKind::Json);
        }
        if field == "stringify" || field == "stringify_pretty" {
            if args.len() != 1 {
                diagnostics.push(Diagnostic::new(
                    "E2237",
                    Severity::Error,
                    format!("`json.{field}` expects 1 argument(s), got {}", args.len()),
                    call_span,
                ));
                return Some(TypeKind::Unknown);
            }
            if let Some(actual) = arg_types.first() {
                if !matches!(actual, TypeKind::Json | TypeKind::Unknown) {
                    diagnostics.push(Diagnostic::new(
                        "E2238",
                        Severity::Error,
                        format!("`json.{field}` argument 1 expects `Json`, got `{}`", type_name(actual)),
                        args.first().map(|arg| arg.span()).unwrap_or(call_span),
                    ));
                }
            }
            return Some(TypeKind::Str);
        }
        if matches!(field, "null" | "bool" | "i64" | "f64" | "str") {
            let expected = match field {
                "null" => Some((&[][..], "")),
                "bool" => Some((&["Bool"][..], "")),
                "i64" => Some((&["Int"][..], "")),
                "f64" => Some((&["Float"][..], "")),
                "str" => Some((&["Str"][..], "")),
                _ => None,
            }?;
            if args.len() != expected.0.len() {
                diagnostics.push(Diagnostic::new(
                    "E2237",
                    Severity::Error,
                    format!("`json.{field}` expects {} argument(s), got {}", expected.0.len(), args.len()),
                    call_span,
                ));
                return Some(TypeKind::Unknown);
            }
            for (idx, expect_name) in expected.0.iter().enumerate() {
                let actual = arg_types.get(idx).cloned().unwrap_or(TypeKind::Unknown);
                let ok = match (*expect_name, &actual) {
                    ("Bool", TypeKind::Bool | TypeKind::Unknown) => true,
                    ("Int", TypeKind::Int | TypeKind::Unknown) => true,
                    ("Float", TypeKind::Float | TypeKind::Unknown) => true,
                    ("Str", TypeKind::Str | TypeKind::Unknown) => true,
                    _ => false,
                };
                if !ok {
                    diagnostics.push(Diagnostic::new(
                        "E2238",
                        Severity::Error,
                        format!(
                            "`json.{field}` argument {} expects `{}`, got `{}`",
                            idx + 1,
                            expect_name,
                            type_name(&actual)
                        ),
                        args.get(idx).map(|arg| arg.span()).unwrap_or(call_span),
                    ));
                }
            }
            return Some(TypeKind::Json);
        }
    }
    if namespace == "json.builder" {
        let (expected, ret) = match field {
            "new" => Some((&["Int"][..], TypeKind::JsonBuilder)),
            "begin_object" | "end_object" | "begin_array" | "end_array" | "value_null" => {
                Some((&["JsonBuilder"][..], TypeKind::JsonBuilder))
            }
            "key" => Some((&["JsonBuilder", "Str"][..], TypeKind::JsonBuilder)),
            "value_bool" => Some((&["JsonBuilder", "Bool"][..], TypeKind::JsonBuilder)),
            "value_i64" => Some((&["JsonBuilder", "Int"][..], TypeKind::JsonBuilder)),
            "value_f64" => Some((&["JsonBuilder", "Float"][..], TypeKind::JsonBuilder)),
            "value_str" => Some((&["JsonBuilder", "Str"][..], TypeKind::JsonBuilder)),
            "value_json" => Some((&["JsonBuilder", "Json"][..], TypeKind::JsonBuilder)),
            "finish" => Some((&["JsonBuilder"][..], TypeKind::Str)),
            _ => None,
        }?;
        if args.len() != expected.len() {
            diagnostics.push(Diagnostic::new(
                "E2237",
                Severity::Error,
                format!("`{}.{} ` expects {} argument(s), got {}", namespace, field, expected.len(), args.len()),
                call_span,
            ));
            return Some(TypeKind::Unknown);
        }
        for (idx, expect_name) in expected.iter().enumerate() {
            let actual = arg_types.get(idx).cloned().unwrap_or(TypeKind::Unknown);
            let ok = match (*expect_name, &actual) {
                ("JsonBuilder", TypeKind::JsonBuilder | TypeKind::Unknown) => true,
                ("Json", TypeKind::Json | TypeKind::Unknown) => true,
                ("Str", TypeKind::Str | TypeKind::Unknown) => true,
                ("Bool", TypeKind::Bool | TypeKind::Unknown) => true,
                ("Int", TypeKind::Int | TypeKind::Unknown) => true,
                ("Float", TypeKind::Float | TypeKind::Unknown) => true,
                _ => false,
            };
            if !ok {
                diagnostics.push(Diagnostic::new(
                    "E2238",
                    Severity::Error,
                    format!(
                        "`{}.{} ` argument {} expects `{}`, got `{}`",
                        namespace,
                        field,
                        idx + 1,
                        expect_name,
                        type_name(&actual)
                    ),
                    args.get(idx).map(|arg| arg.span()).unwrap_or(call_span),
                ));
            }
        }
        return Some(ret);
    }

    let ret = stdlib_namespace_return_hint(namespace, field)?;
    let expected = match (namespace, field) {
        ("time", "now_ms") | ("time", "monotonic_now_ms") => Some((&[][..], "nondet")),
        ("time", "duration_ms") => Some((&["Int"][..], "")),
        ("time", "sleep_ms") => Some((&["Int"][..], "io")),
        ("path", "join") => Some((&["Str", "Str"][..], "")),
        ("path", "parent") | ("path", "basename") | ("path", "is_absolute") => {
            Some((&["Str"][..], ""))
        }
        ("fs", "exists") | ("fs", "read_text") | ("fs", "create_dir") => Some((&["Str"][..], "io")),
        ("fs", "write_text") => Some((&["Str", "Str"][..], "io")),
        ("net", "listen") | ("net", "connect") => Some((&["Str", "Int"][..], "net")),
        ("net", "listener_port") | ("net", "accept") | ("net", "close") => {
            Some((&["Int"][..], "net"))
        }
        ("net", "read") => Some((&["Int", "Int"][..], "net")),
        ("net", "write") => Some((&["Int", "Str"][..], "net")),
        ("net", "resolve") => Some((&["Str"][..], "net")),
        ("convert", "to_int") | ("convert", "parse_i64") => Some((&["Str"][..], "")),
        ("convert", "to_float") | ("convert", "parse_f64") => Some((&["Str"][..], "")),
        ("convert", "i64_to_f64") => Some((&["Int"][..], "")),
        ("convert", "f64_to_bits") => Some((&["Float"][..], "")),
        ("convert", "f64_from_bits") => Some((&["Int"][..], "")),
        ("convert", "to_str") => Some((&["Int"][..], "")),
        ("convert", "to_str_f64") => Some((&["Float"][..], "")),
        ("convert", "format_f64") => Some((&["Float", "Int"][..], "")),
        ("math", "sqrt") => Some((&["Float"][..], "")),
        ("str_builder", "new") => Some((&["Int"][..], "")),
        ("str_builder", "append") => Some((&["Int", "Str"][..], "")),
        ("str_builder", "append_char") => Some((&["Int", "Int"][..], "")),
        ("str_builder", "finish") => Some((&["Int"][..], "")),
        ("simd", "f64x2_splat") => Some((&["Float"][..], "")),
        ("simd", "f64x2_make") => Some((&["Float", "Float"][..], "")),
        ("simd", "f64x2_add")
        | ("simd", "f64x2_sub")
        | ("simd", "f64x2_mul")
        | ("simd", "f64x2_gt") => Some((&["Int", "Int"][..], "")),
        ("simd", "f64x2_extract") => Some((&["Int", "Int"][..], "")),
        ("text", "trim")
        | ("text", "to_lower")
        | ("text", "to_upper")
        | ("encoding", "hex_encode")
        | ("encoding", "hex_decode")
        | ("encoding", "base64_encode")
        | ("encoding", "base64_decode")
        | ("encoding", "url_encode")
        | ("encoding", "url_decode")
        | ("env", "get")
        | ("env", "get_required") => Some((&["Str"][..], "")),
        ("text", "contains")
        | ("text", "starts_with")
        | ("text", "ends_with")
        | ("text", "index_of") => Some((&["Str", "Str"][..], "")),
        ("text", "replace") => Some((&["Str", "Str", "Str"][..], "")),
        ("text", "byte_len") => Some((&["Str"][..], "")),
        ("text", "split_part") => Some((&["Str", "Str", "Int"][..], "")),
        ("log", "info") | ("log", "warn") | ("log", "error") => Some((&["Str"][..], "io")),
        ("env", "has") => Some((&["Str"][..], "nondet")),
        ("cli", "args_len") => Some((&[][..], "nondet")),
        ("cli", "arg") => Some((&["Int"][..], "nondet")),
        ("json", "is_valid") | ("json", "parse_i64") | ("json", "minify") => {
            Some((&["Str"][..], ""))
        }
        ("json", "stringify_i64") => Some((&["Int"][..], "")),
        ("regex", "count") => Some((&["Str", "Str"][..], "")),
        ("regex", "replace_all") => Some((&["Str", "Str", "Str"][..], "")),
        ("http", "status_text") => Some((&["Int"][..], "")),
        ("http", "default_port") => Some((&["Str"][..], "")),
        ("http", "build_request_line") => Some((&["Str", "Str"][..], "")),
        ("http", "get") => Some((&["Str", "Int"][..], "net")),
        ("http", "post") => Some((&["Str", "Str", "Int"][..], "net")),
        ("http", "request") | ("http", "request_status") => {
            Some((&["Str", "Str", "Str", "Int"][..], "net"))
        }
        #[cfg(feature = "bench-runtime")]
        ("bench", "md5_hex") | ("bench", "json_canonical") => Some((&["Str"][..], "")),
        #[cfg(feature = "bench-runtime")]
        ("bench", "json_repeat_array") => Some((&["Str", "Int"][..], "")),
        #[cfg(feature = "bench-runtime")]
        ("bench", "http_server_bench") => Some((&["Int"][..], "net")),
        #[cfg(feature = "bench-runtime")]
        ("bench", "secp256k1") | ("bench", "edigits") => Some((&["Int"][..], "")),
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_listen") => Some((&["Str", "Int"][..], "net")),
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_listener_port") | ("bench", "net_accept") => Some((&["Int"][..], "net")),
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_connect") => Some((&["Str", "Int"][..], "net")),
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_read") => Some((&["Int", "Int"][..], "net")),
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_write") => Some((&["Int", "Str"][..], "net")),
        #[cfg(feature = "bench-runtime")]
        ("bench", "net_close") => Some((&["Int"][..], "net")),
        _ => None,
    };
    if let Some((expected_args, effect)) = expected {
        if args.len() != expected_args.len() {
            diagnostics.push(Diagnostic::new(
                "E2237",
                Severity::Error,
                format!(
                    "`{namespace}.{field}` expects {} argument(s), got {}",
                    expected_args.len(),
                    args.len()
                ),
                call_span,
            ));
            return Some(TypeKind::Unknown);
        }
        for (idx, expected_ty) in expected_args.iter().enumerate() {
            let Some(actual) = arg_types.get(idx) else {
                continue;
            };
            let expect_match = match *expected_ty {
                "Int" => matches!(actual, TypeKind::Int | TypeKind::Unknown),
                "Str" => matches!(actual, TypeKind::Str | TypeKind::Unknown),
                "Float" => matches!(actual, TypeKind::Float | TypeKind::Unknown),
                _ => true,
            };
            if !expect_match {
                diagnostics.push(Diagnostic::new(
                    "E2238",
                    Severity::Error,
                    format!(
                        "`{namespace}.{field}` argument {} expects `{expected_ty}`, got `{}`",
                        idx + 1,
                        type_name(actual)
                    ),
                    args[idx].span(),
                ));
                return Some(TypeKind::Unknown);
            }
        }
        if !effect.is_empty() {
            observed_effects.insert(effect.to_string());
        }
        return Some(ret);
    }
    diagnostics.push(Diagnostic::new(
        "E2239",
        Severity::Error,
        format!("unknown stdlib API `{namespace}.{field}`"),
        call_span,
    ));
    Some(TypeKind::Unknown)
}

fn is_builtin_ident(name: &str) -> bool {
    if matches!(
        name,
        "len"
            | "min"
            | "max"
            | "sorted_desc"
            | "cpu_count"
            | "chan"
            | "ok"
            | "err"
            | "print"
            | "println"
            | "time"
            | "path"
            | "fs"
            | "net"
            | "convert"
            | "math"
            | "str_builder"
            | "simd"
            | "text"
            | "encoding"
            | "log"
            | "env"
            | "cli"
            | "json"
            | "regex"
            | "http"
            | "true"
            | "false"
    ) {
        return true;
    }
    #[cfg(feature = "bench-runtime")]
    {
        if name == "bench" {
            return true;
        }
    }
    false
}
