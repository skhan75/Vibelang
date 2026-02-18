use std::collections::{BTreeMap, BTreeSet};

use vibe_ast::{
    BinaryOp, Contract, Declaration, Expr, FileAst, SelectPattern, Stmt, TypeRef, UnaryOp,
};
use vibe_diagnostics::{Diagnostic, Diagnostics, Severity};
use vibe_hir::{
    verify_hir, HirExpr, HirExprKind, HirFunction, HirParam, HirProgram, HirSelectCase,
    HirSelectPattern, HirStmt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Int,
    Float,
    Bool,
    Str,
    List(Box<TypeKind>),
    Result(Box<TypeKind>, Box<TypeKind>),
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
        for stmt in &func.body {
            check_stmt(
                stmt,
                &mut env,
                &signatures,
                &mut diagnostics,
                &mut observed_effects,
                &mut inferred_returns,
                &mut hir_body,
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
            hir_tail_expr = Some(lower_expr(expr, &env));
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

        for observed in &observed_effects {
            if !declared_effects.contains(observed) {
                diagnostics.push(Diagnostic::new(
                    "E3002",
                    Severity::Warning,
                    format!("observed effect `{observed}` is not declared in `@effect`"),
                    func.span,
                ));
            }
        }
        for declared in &declared_effects {
            if !observed_effects.contains(declared) {
                diagnostics.push(Diagnostic::new(
                    "E3003",
                    Severity::Info,
                    format!("declared effect `{declared}` was not observed"),
                    func.span,
                ));
            }
        }

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
            hir_out.push(HirStmt::Return {
                expr: lower_expr(expr, env),
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
            hir_out.push(HirStmt::Go {
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
    }
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
        Expr::Map { .. } => TypeKind::Unknown,
        Expr::Member { object, field, .. } => match field.as_str() {
            "len" | "balance" => TypeKind::Int,
            _ => expr_type_hint(object, env),
        },
        Expr::Call { callee, .. } => {
            if let Expr::Ident { name, .. } = &**callee {
                return match name.as_str() {
                    "len" | "min" | "cpu_count" => TypeKind::Int,
                    "sorted_desc" => TypeKind::Bool,
                    "print" | "println" => TypeKind::Void,
                    _ => TypeKind::Unknown,
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
        Expr::Map { .. } => {
            observed_effects.insert("alloc".to_string());
            TypeKind::Unknown
        }
        Expr::Member { object, field, .. } => {
            let base = infer_expr(object, env, sigs, context, diagnostics, observed_effects);
            match field.as_str() {
                "len" => TypeKind::Int,
                "balance" => TypeKind::Int,
                _ => base,
            }
        }
        Expr::Call { callee, args, .. } => {
            let _ = infer_expr(callee, env, sigs, context, diagnostics, observed_effects);
            for arg in args {
                let _ = infer_expr(arg, env, sigs, context, diagnostics, observed_effects);
            }
            if let Expr::Ident { name, .. } = &**callee {
                match name.as_str() {
                    "chan" => {
                        observed_effects.insert("alloc".to_string());
                        observed_effects.insert("concurrency".to_string());
                        return TypeKind::Unknown;
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
                match field.as_str() {
                    "sort_desc" | "take" | "recv" | "send" | "close" => {
                        observed_effects.insert("concurrency".to_string());
                        return TypeKind::Unknown;
                    }
                    "warn" | "listen" => {
                        observed_effects.insert("io".to_string());
                        return TypeKind::Unknown;
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
        | Expr::Unary { expr: object, .. } => {
            validate_contract_expr(object, context, diagnostics);
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
    if raw.starts_with("Result<") && raw.ends_with('>') {
        let inner = &raw[7..raw.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 2 {
            return TypeKind::Result(
                Box::new(parse_type_ref(&TypeRef {
                    raw: parts[0].trim().to_string(),
                })),
                Box::new(parse_type_ref(&TypeRef {
                    raw: parts[1].trim().to_string(),
                })),
            );
        }
    }
    TypeKind::Unknown
}

fn type_compatible(a: &TypeKind, b: &TypeKind) -> bool {
    matches!(a, TypeKind::Unknown)
        || matches!(b, TypeKind::Unknown)
        || a == b
        || (matches!(a, TypeKind::Int) && matches!(b, TypeKind::Float))
        || (matches!(a, TypeKind::Float) && matches!(b, TypeKind::Int))
}

fn type_name(t: &TypeKind) -> String {
    match t {
        TypeKind::Int => "Int".to_string(),
        TypeKind::Float => "Float".to_string(),
        TypeKind::Bool => "Bool".to_string(),
        TypeKind::Str => "Str".to_string(),
        TypeKind::List(inner) => format!("List<{}>", type_name(inner)),
        TypeKind::Result(ok, err) => format!("Result<{}, {}>", type_name(ok), type_name(err)),
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

fn is_known_effect(e: &str) -> bool {
    matches!(
        e,
        "alloc" | "mut_state" | "io" | "net" | "concurrency" | "nondet"
    )
}

fn is_builtin_ident(name: &str) -> bool {
    matches!(
        name,
        "len"
            | "min"
            | "max"
            | "sorted_desc"
            | "cpu_count"
            | "ok"
            | "err"
            | "print"
            | "println"
            | "true"
            | "false"
    )
}
