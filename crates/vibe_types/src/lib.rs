use std::collections::{BTreeMap, BTreeSet};

mod effect_diagnostics;
mod effect_propagation;
mod ownership;

use vibe_ast::{
    BinaryOp, Contract, Declaration, Expr, FileAst, SelectPattern, Stmt, TypeRef, UnaryOp,
};
use vibe_diagnostics::{Diagnostic, Diagnostics, Severity, Span};
use vibe_hir::{
    verify_hir, HirContractKind, HirExpr, HirExprKind, HirFunction, HirParam, HirProgram,
    HirSelectCase, HirSelectPattern, HirStmt,
};

use crate::effect_diagnostics::emit_effect_diagnostics;
use crate::effect_propagation::{
    collect_direct_calls, compute_transitive_effects, FunctionEffectSummary,
};
use crate::ownership::{check_go_sendability, check_shared_mutation_in_concurrent_context};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Int,
    Float,
    Bool,
    Str,
    List(Box<TypeKind>),
    Map(Box<TypeKind>, Box<TypeKind>),
    Result(Box<TypeKind>, Box<TypeKind>),
    Chan(Box<TypeKind>),
    Void,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct CheckOutput {
    pub diagnostics: Diagnostics,
    pub hir: HirProgram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContractContext {
    Require,
    Ensure,
    Other,
}

pub fn check_and_lower(ast: &FileAst) -> CheckOutput {
    let mut diagnostics = Diagnostics::default();
    let mut signatures: BTreeMap<String, Option<TypeKind>> = BTreeMap::new();
    let mut hir = HirProgram::default();
    let mut effect_summaries: Vec<FunctionEffectSummary> = Vec::new();

    for decl in &ast.declarations {
        let Declaration::Function(f) = decl;
        if signatures.contains_key(&f.name) {
            diagnostics.push(Diagnostic::new(
                "E2002",
                Severity::Error,
                format!("duplicate function `{}`", f.name),
                f.span,
            ));
        }
        signatures.insert(f.name.clone(), f.return_type.as_ref().map(parse_type_ref));
    }

    for decl in &ast.declarations {
        let Declaration::Function(func) = decl;
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
                    .map(parse_type_ref)
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
                        &signatures,
                        ContractContext::Require,
                        &mut diagnostics,
                        &mut observed_effects,
                    );
                    if !matches!(
                        infer_expr(
                            expr,
                            &env,
                            &signatures,
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
                            &signatures,
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
                            &signatures,
                            ContractContext::Other,
                            &mut diagnostics,
                            &mut observed_effects,
                        );
                        infer_expr(
                            &case.expected,
                            &env,
                            &signatures,
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
                expr: lower_contract_expr(expr, &env, None),
            });
        }
        for stmt in &func.body {
            check_stmt(
                stmt,
                &mut env,
                &signatures,
                &mut diagnostics,
                &mut observed_effects,
                &mut inferred_returns,
                &mut hir_body,
                &ensure_contract_exprs,
            );
        }
        let mut hir_tail_expr = None;
        if let Some(expr) = &func.tail_expr {
            let t = infer_expr(
                expr,
                &env,
                &signatures,
                ContractContext::Other,
                &mut diagnostics,
                &mut observed_effects,
            );
            inferred_returns.push(t);
            let lowered_tail = lower_expr(expr, &env);
            for contract_expr in &ensure_contract_exprs {
                hir_body.push(HirStmt::ContractCheck {
                    kind: HirContractKind::Ensure,
                    expr: lower_contract_expr(contract_expr, &env, Some(&lowered_tail)),
                });
            }
            hir_tail_expr = Some(lowered_tail);
        }

        let inferred_return = unify_return_types(&inferred_returns);
        if let Some(declared) = func.return_type.as_ref() {
            let declared = parse_type_ref(declared);
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

    CheckOutput { diagnostics, hir }
}

fn check_stmt(
    stmt: &Stmt,
    env: &mut BTreeMap<String, TypeKind>,
    sigs: &BTreeMap<String, Option<TypeKind>>,
    diagnostics: &mut Diagnostics,
    observed_effects: &mut BTreeSet<String>,
    inferred_returns: &mut Vec<TypeKind>,
    hir_out: &mut Vec<HirStmt>,
    ensure_contract_exprs: &[Expr],
) {
    match stmt {
        Stmt::Binding { name, expr, .. } => {
            let t = infer_expr(
                expr,
                env,
                sigs,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            let lowered_expr = lower_expr(expr, env);
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
                sigs,
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
                Expr::Member { .. } => {
                    observed_effects.insert("mut_state".to_string());
                }
                _ => {}
            }
            hir_out.push(HirStmt::Assignment {
                target: lower_expr(target, env),
                expr: lower_expr(expr, env),
            });
        }
        Stmt::Return { expr, .. } => {
            let ret_ty = infer_expr(
                expr,
                env,
                sigs,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            inferred_returns.push(ret_ty);
            let lowered_return_expr = lower_expr(expr, env);
            for ensure_expr in ensure_contract_exprs {
                hir_out.push(HirStmt::ContractCheck {
                    kind: HirContractKind::Ensure,
                    expr: lower_contract_expr(ensure_expr, env, Some(&lowered_return_expr)),
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
                sigs,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            hir_out.push(HirStmt::Expr {
                expr: lower_expr(expr, env),
            });
        }
        Stmt::For {
            var, iter, body, ..
        } => {
            let iter_ty = infer_expr(
                iter,
                env,
                sigs,
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
                    sigs,
                    diagnostics,
                    observed_effects,
                    inferred_returns,
                    &mut child_hir,
                    ensure_contract_exprs,
                );
            }
            hir_out.push(HirStmt::For {
                var: var.clone(),
                iter: lower_expr(iter, env),
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
                sigs,
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
                    sigs,
                    diagnostics,
                    observed_effects,
                    inferred_returns,
                    &mut then_hir,
                    ensure_contract_exprs,
                );
            }
            let mut else_hir = Vec::new();
            for s in else_body {
                check_stmt(
                    s,
                    env,
                    sigs,
                    diagnostics,
                    observed_effects,
                    inferred_returns,
                    &mut else_hir,
                    ensure_contract_exprs,
                );
            }
            hir_out.push(HirStmt::If {
                cond: lower_expr(cond, env),
                then_body: then_hir,
                else_body: else_hir,
            });
        }
        Stmt::While { cond, body, .. } => {
            let cond_ty = infer_expr(
                cond,
                env,
                sigs,
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
                    sigs,
                    diagnostics,
                    observed_effects,
                    inferred_returns,
                    &mut child_hir,
                    ensure_contract_exprs,
                );
            }
            hir_out.push(HirStmt::While {
                cond: lower_expr(cond, env),
                body: child_hir,
            });
        }
        Stmt::Repeat { count, body, .. } => {
            let count_ty = infer_expr(
                count,
                env,
                sigs,
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
                    sigs,
                    diagnostics,
                    observed_effects,
                    inferred_returns,
                    &mut child_hir,
                    ensure_contract_exprs,
                );
            }
            hir_out.push(HirStmt::Repeat {
                count: lower_expr(count, env),
                body: child_hir,
            });
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
                            sigs,
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
                    sigs,
                    ContractContext::Other,
                    diagnostics,
                    observed_effects,
                );
                lowered_cases.push(HirSelectCase {
                    pattern: lower_select_pattern(&c.pattern, env),
                    action: lower_expr(&c.action, env),
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
                sigs,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            check_go_sendability(expr, env, expr_type_hint, diagnostics);
            hir_out.push(HirStmt::Go {
                expr: lower_expr(expr, env),
            });
        }
        Stmt::Thread { expr, .. } => {
            observed_effects.insert("concurrency".to_string());
            let _ = infer_expr(
                expr,
                env,
                sigs,
                ContractContext::Other,
                diagnostics,
                observed_effects,
            );
            check_go_sendability(expr, env, expr_type_hint, diagnostics);
            hir_out.push(HirStmt::Thread {
                expr: lower_expr(expr, env),
            });
        }
    }
}

fn lower_select_pattern(
    pattern: &SelectPattern,
    env: &BTreeMap<String, TypeKind>,
) -> HirSelectPattern {
    match pattern {
        SelectPattern::Receive { binding, expr } => HirSelectPattern::Receive {
            binding: binding.clone(),
            expr: lower_expr(expr, env),
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
                .map(|e| lower_contract_expr(e, env, dot_result))
                .collect(),
        ),
        Expr::Map { entries, .. } => HirExprKind::Map(
            entries
                .iter()
                .map(|(k, v)| {
                    (
                        lower_contract_expr(k, env, dot_result),
                        lower_contract_expr(v, env, dot_result),
                    )
                })
                .collect(),
        ),
        Expr::Member { object, field, .. } => HirExprKind::Member {
            object: Box::new(lower_contract_expr(object, env, dot_result)),
            field: field.clone(),
        },
        Expr::Index { object, index, .. } => HirExprKind::Index {
            object: Box::new(lower_contract_expr(object, env, dot_result)),
            index: Box::new(lower_contract_expr(index, env, dot_result)),
        },
        Expr::Slice {
            object, start, end, ..
        } => HirExprKind::Slice {
            object: Box::new(lower_contract_expr(object, env, dot_result)),
            start: start
                .as_ref()
                .map(|expr| Box::new(lower_contract_expr(expr, env, dot_result))),
            end: end
                .as_ref()
                .map(|expr| Box::new(lower_contract_expr(expr, env, dot_result))),
        },
        Expr::Call { callee, args, .. } => HirExprKind::Call {
            callee: Box::new(lower_contract_expr(callee, env, dot_result)),
            args: args
                .iter()
                .map(|a| lower_contract_expr(a, env, dot_result))
                .collect(),
        },
        Expr::Binary {
            left, op, right, ..
        } => HirExprKind::Binary {
            left: Box::new(lower_contract_expr(left, env, dot_result)),
            op: *op,
            right: Box::new(lower_contract_expr(right, env, dot_result)),
        },
        Expr::Unary { op, expr, .. } => HirExprKind::Unary {
            op: *op,
            expr: Box::new(lower_contract_expr(expr, env, dot_result)),
        },
        Expr::Async { expr, .. } => HirExprKind::Async {
            expr: Box::new(lower_contract_expr(expr, env, dot_result)),
        },
        Expr::Await { expr, .. } => HirExprKind::Await {
            expr: Box::new(lower_contract_expr(expr, env, dot_result)),
        },
        Expr::Question { expr, .. } => HirExprKind::Question {
            expr: Box::new(lower_contract_expr(expr, env, dot_result)),
        },
        Expr::DotResult { .. } => HirExprKind::DotResult,
        Expr::Old { expr, .. } => HirExprKind::Old {
            expr: Box::new(lower_contract_expr(expr, env, dot_result)),
        },
    };
    HirExpr::new(kind, ty)
}

fn lower_expr(expr: &Expr, env: &BTreeMap<String, TypeKind>) -> HirExpr {
    let ty = type_name(&expr_type_hint(expr, env));
    let kind = match expr {
        Expr::Ident { name, .. } => HirExprKind::Ident(name.clone()),
        Expr::Int { value, .. } => HirExprKind::Int(*value),
        Expr::Float { value, .. } => HirExprKind::Float(*value),
        Expr::Bool { value, .. } => HirExprKind::Bool(*value),
        Expr::String { value, .. } => HirExprKind::String(value.clone()),
        Expr::List { items, .. } => {
            HirExprKind::List(items.iter().map(|e| lower_expr(e, env)).collect())
        }
        Expr::Map { entries, .. } => HirExprKind::Map(
            entries
                .iter()
                .map(|(k, v)| (lower_expr(k, env), lower_expr(v, env)))
                .collect(),
        ),
        Expr::Member { object, field, .. } => HirExprKind::Member {
            object: Box::new(lower_expr(object, env)),
            field: field.clone(),
        },
        Expr::Index { object, index, .. } => HirExprKind::Index {
            object: Box::new(lower_expr(object, env)),
            index: Box::new(lower_expr(index, env)),
        },
        Expr::Slice {
            object, start, end, ..
        } => HirExprKind::Slice {
            object: Box::new(lower_expr(object, env)),
            start: start.as_ref().map(|expr| Box::new(lower_expr(expr, env))),
            end: end.as_ref().map(|expr| Box::new(lower_expr(expr, env))),
        },
        Expr::Call { callee, args, .. } => HirExprKind::Call {
            callee: Box::new(lower_expr(callee, env)),
            args: args.iter().map(|a| lower_expr(a, env)).collect(),
        },
        Expr::Binary {
            left, op, right, ..
        } => HirExprKind::Binary {
            left: Box::new(lower_expr(left, env)),
            op: *op,
            right: Box::new(lower_expr(right, env)),
        },
        Expr::Unary { op, expr, .. } => HirExprKind::Unary {
            op: *op,
            expr: Box::new(lower_expr(expr, env)),
        },
        Expr::Async { expr, .. } => HirExprKind::Async {
            expr: Box::new(lower_expr(expr, env)),
        },
        Expr::Await { expr, .. } => HirExprKind::Await {
            expr: Box::new(lower_expr(expr, env)),
        },
        Expr::Question { expr, .. } => HirExprKind::Question {
            expr: Box::new(lower_expr(expr, env)),
        },
        Expr::DotResult { .. } => HirExprKind::DotResult,
        Expr::Old { expr, .. } => HirExprKind::Old {
            expr: Box::new(lower_expr(expr, env)),
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
            if let Expr::Member { object, field, .. } = &**callee {
                if let Expr::Ident { name, .. } = &**object {
                    if let Some(ty) = stdlib_namespace_return_hint(name, field) {
                        return ty;
                    }
                }
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
    }
}

fn infer_expr(
    expr: &Expr,
    env: &BTreeMap<String, TypeKind>,
    sigs: &BTreeMap<String, Option<TypeKind>>,
    context: ContractContext,
    diagnostics: &mut Diagnostics,
    observed_effects: &mut BTreeSet<String>,
) -> TypeKind {
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
                    sigs,
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
                    infer_expr(key, env, sigs, context, diagnostics, observed_effects);
                let inferred_value =
                    infer_expr(value, env, sigs, context, diagnostics, observed_effects);
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
        Expr::Member { object, field, .. } => {
            let base = infer_expr(object, env, sigs, context, diagnostics, observed_effects);
            match field.as_str() {
                "len" => TypeKind::Int,
                "balance" => TypeKind::Int,
                _ => match base {
                    TypeKind::Map(key_ty, value_ty)
                        if matches!(*key_ty, TypeKind::Str) && !is_container_member_api(field) =>
                    {
                        *value_ty
                    }
                    other => other,
                },
            }
        }
        Expr::Index {
            object,
            index,
            span,
        } => {
            let object_ty = infer_expr(object, env, sigs, context, diagnostics, observed_effects);
            let index_ty = infer_expr(index, env, sigs, context, diagnostics, observed_effects);
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
            let object_ty = infer_expr(object, env, sigs, context, diagnostics, observed_effects);
            if let Some(start) = start {
                let start_ty = infer_expr(start, env, sigs, context, diagnostics, observed_effects);
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
                let end_ty = infer_expr(end, env, sigs, context, diagnostics, observed_effects);
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
                        format!("slicing is only supported for List<T> and Str; got `{}`", type_name(&other)),
                        *span,
                    ));
                    TypeKind::Unknown
                }
            }
        }
        Expr::Call { callee, args, .. } => {
            let callee_ty = infer_expr(callee, env, sigs, context, diagnostics, observed_effects);
            let mut arg_types = Vec::with_capacity(args.len());
            for arg in args {
                arg_types.push(infer_expr(
                    arg,
                    env,
                    sigs,
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
                if let Expr::Member { object, field, .. } = &**callee {
                    if let Expr::Ident { name: namespace, .. } = &**object {
                        if let Some(ret) = infer_stdlib_namespace_call(
                            namespace,
                            field,
                            args,
                            &arg_types,
                            diagnostics,
                            observed_effects,
                            callee.span(),
                        ) {
                            return ret;
                        }
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
                    "send" | "close" => {
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
            let lt = infer_expr(left, env, sigs, context, diagnostics, observed_effects);
            let rt = infer_expr(right, env, sigs, context, diagnostics, observed_effects);
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
            let t = infer_expr(expr, env, sigs, context, diagnostics, observed_effects);
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
            infer_expr(expr, env, sigs, context, diagnostics, observed_effects)
        }
        Expr::Await { expr, .. } => {
            observed_effects.insert("concurrency".to_string());
            infer_expr(expr, env, sigs, context, diagnostics, observed_effects)
        }
        Expr::Question { expr, span } => {
            let inner = infer_expr(expr, env, sigs, context, diagnostics, observed_effects);
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
            infer_expr(expr, env, sigs, context, diagnostics, observed_effects)
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
        TypeKind::List(inner) => format!("List<{}>", type_name(inner)),
        TypeKind::Map(key, value) => format!("Map<{}, {}>", type_name(key), type_name(value)),
        TypeKind::Result(ok, err) => format!("Result<{}, {}>", type_name(ok), type_name(err)),
        TypeKind::Chan(inner) => format!("Chan<{}>", type_name(inner)),
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

fn stdlib_namespace_return_hint(namespace: &str, field: &str) -> Option<TypeKind> {
    match (namespace, field) {
        ("time", "now_ms") | ("time", "duration_ms") => Some(TypeKind::Int),
        ("time", "sleep_ms") => Some(TypeKind::Void),
        ("path", "join") | ("path", "parent") | ("path", "basename") => Some(TypeKind::Str),
        ("path", "is_absolute") => Some(TypeKind::Bool),
        ("fs", "exists") | ("fs", "write_text") | ("fs", "create_dir") => Some(TypeKind::Bool),
        ("fs", "read_text") => Some(TypeKind::Str),
        ("json", "is_valid") => Some(TypeKind::Bool),
        ("json", "parse_i64") => Some(TypeKind::Int),
        ("json", "stringify_i64") | ("json", "minify") => Some(TypeKind::Str),
        ("http", "status_text") | ("http", "build_request_line") => Some(TypeKind::Str),
        ("http", "default_port") => Some(TypeKind::Int),
        _ => None,
    }
}

fn infer_stdlib_namespace_call(
    namespace: &str,
    field: &str,
    args: &[Expr],
    arg_types: &[TypeKind],
    diagnostics: &mut Diagnostics,
    observed_effects: &mut BTreeSet<String>,
    call_span: Span,
) -> Option<TypeKind> {
    let ret = stdlib_namespace_return_hint(namespace, field)?;
    let expected = match (namespace, field) {
        ("time", "now_ms") => Some((&[][..], "nondet")),
        ("time", "duration_ms") => Some((&["Int"][..], "")),
        ("time", "sleep_ms") => Some((&["Int"][..], "io")),
        ("path", "join") => Some((&["Str", "Str"][..], "")),
        ("path", "parent") | ("path", "basename") | ("path", "is_absolute") => {
            Some((&["Str"][..], ""))
        }
        ("fs", "exists") | ("fs", "read_text") | ("fs", "create_dir") => Some((&["Str"][..], "io")),
        ("fs", "write_text") => Some((&["Str", "Str"][..], "io")),
        ("json", "is_valid") | ("json", "parse_i64") | ("json", "minify") => {
            Some((&["Str"][..], ""))
        }
        ("json", "stringify_i64") => Some((&["Int"][..], "")),
        ("http", "status_text") => Some((&["Int"][..], "")),
        ("http", "default_port") => Some((&["Str"][..], "")),
        ("http", "build_request_line") => Some((&["Str", "Str"][..], "")),
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
    matches!(
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
            | "json"
            | "http"
            | "true"
            | "false"
    )
}
