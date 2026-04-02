// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod optimize;

use std::collections::{BTreeMap, BTreeSet};

use vibe_hir::{HirContractKind, HirExpr, HirExprKind, HirProgram, HirSelectPattern, HirStmt};

#[derive(Debug, Clone, Default)]
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
}

#[derive(Debug, Clone, Default)]
pub struct MirFunction {
    pub name: String,
    pub is_public: bool,
    pub params: Vec<MirParam>,
    pub return_type: MirType,
    pub body: Vec<MirStmt>,
    pub native_symbol: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MirParam {
    pub name: String,
    pub ty: MirType,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MirType {
    I64,
    F64,
    Bool,
    Str,
    Json,
    JsonBuilder,
    Result,
    Void,
    #[default]
    Unknown,
    /// Opaque pointer (closure env, closure value, etc.).
    Ptr,
}

#[derive(Debug, Clone)]
pub enum MirStmt {
    Let {
        name: String,
        expr: MirExpr,
    },
    Assign {
        name: String,
        expr: MirExpr,
    },
    Expr(MirExpr),
    Return(MirExpr),
    For {
        var: String,
        iter: MirExpr,
        iter_kind: MirForIterKind,
        body: Vec<MirStmt>,
    },
    If {
        cond: MirExpr,
        then_body: Vec<MirStmt>,
        else_body: Vec<MirStmt>,
    },
    While {
        cond: MirExpr,
        body: Vec<MirStmt>,
    },
    Repeat {
        count: MirExpr,
        body: Vec<MirStmt>,
    },
    Break,
    Continue,
    Select {
        cases: Vec<MirSelectCase>,
    },
    Go(MirExpr),
    Thread(MirExpr),
    ContractCheck {
        kind: MirContractKind,
        expr: MirExpr,
    },
    Match {
        scrutinee: MirExpr,
        arms: Vec<MirMatchArm>,
        default_action: Option<MirExpr>,
    },
}

#[derive(Debug, Clone)]
pub struct MirMatchArm {
    pub pattern: MirExpr,
    pub action: MirExpr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirForIterKind {
    List,
    MapInt,
    MapStr,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirContractKind {
    Require,
    Ensure,
}

#[derive(Debug, Clone)]
pub struct MirSelectCase {
    pub pattern: MirSelectPattern,
    pub action: MirExpr,
}

#[derive(Debug, Clone)]
pub enum MirSelectPattern {
    Receive { binding: String, source: MirExpr },
    After { duration_literal: String },
    Closed { ident: String },
    Default,
}

#[derive(Debug, Clone)]
pub enum MirExpr {
    Var(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    List(Vec<MirExpr>),
    Map(Vec<(MirExpr, MirExpr)>),
    Member {
        object: Box<MirExpr>,
        field: String,
        object_type: Option<String>,
    },
    Index {
        object: Box<MirExpr>,
        index: Box<MirExpr>,
        object_is_str: bool,
    },
    Slice {
        object: Box<MirExpr>,
        start: Option<Box<MirExpr>>,
        end: Option<Box<MirExpr>>,
        object_is_str: bool,
    },
    Call {
        callee: Box<MirExpr>,
        args: Vec<MirExpr>,
    },
    Binary {
        left: Box<MirExpr>,
        op: String,
        right: Box<MirExpr>,
    },
    Unary {
        op: String,
        expr: Box<MirExpr>,
    },
    Async {
        expr: Box<MirExpr>,
    },
    Await {
        expr: Box<MirExpr>,
    },
    Question {
        expr: Box<MirExpr>,
    },
    ResultOk {
        expr: Box<MirExpr>,
    },
    ResultErr {
        expr: Box<MirExpr>,
    },
    DotResult,
    Old {
        expr: Box<MirExpr>,
    },
    Constructor {
        type_name: String,
        fields: Vec<(String, MirExpr)>,
    },
    /// Tagged enum value: slot0 = tag, slots 1.. = `payload` field values (expression position).
    EnumVariant {
        enum_name: String,
        variant: String,
        payload: Vec<(String, MirExpr)>,
    },
    /// Introduced only inside `EnumVariant` payloads on `match` arms: bind scrutinee field to a local.
    PatternBind {
        bind_as: Option<String>,
    },
    MakeClosure {
        closure_fn: String,
        captures: Vec<MirExpr>,
    },
    EnvLoad {
        slot: u32,
        ty: MirType,
    },
    ClosureCall {
        closure: Box<MirExpr>,
        args: Vec<MirExpr>,
        /// User parameter MIR types (excluding hidden env); parsed from callee HIR `fn(...)->R`.
        user_param_tys: Vec<MirType>,
        /// Return type of the callee function value.
        ret_ty: MirType,
    },
}

pub fn lower_hir_to_mir(hir: &HirProgram) -> Result<MirProgram, String> {
    let mut out = MirProgram::default();
    let globals: BTreeSet<String> = hir.functions.iter().map(|f| f.name.clone()).collect();
    for f in &hir.functions {
        let params: BTreeSet<String> = f.params.iter().map(|p| p.name.clone()).collect();
        let mut locals = BTreeSet::new();
        let mut body = lower_stmt_list(&f.body, &globals, &params, &mut locals)?;
        if let Some(tail) = &f.tail_expr {
            body.push(MirStmt::Return(lower_expr(
                tail, &globals, &params, &locals,
            )?));
        }
        out.functions.push(MirFunction {
            name: f.name.clone(),
            is_public: f.is_public,
            params: f
                .params
                .iter()
                .map(|p| MirParam {
                    name: p.name.clone(),
                    ty: p
                        .ty
                        .as_ref()
                        .map(|t| mir_param_ty_from_hir_raw(&t.raw))
                        .unwrap_or(MirType::Unknown),
                })
                .collect(),
            return_type: f
                .return_type
                .as_ref()
                .map(|t| mir_param_ty_from_hir_raw(&t.raw))
                .unwrap_or_else(|| {
                    mir_param_ty_from_hir_raw(f.inferred_return_type.as_deref().unwrap_or("Unknown"))
                }),
            body,
            native_symbol: f.native_symbol.clone(),
        });
    }
    verify_mir(&out)?;
    Ok(out)
}

fn hir_expr_is_fn_type(ty: &str) -> bool {
    let s: String = ty.chars().filter(|c| !c.is_whitespace()).collect();
    s.starts_with("fn(")
}

/// True when a call `callee(...)` must use the closure calling convention (env + indirect).
fn callee_needs_closure_call(
    callee: &HirExpr,
    globals: &BTreeSet<String>,
    params: &BTreeSet<String>,
    locals: &BTreeSet<String>,
) -> bool {
    if !hir_expr_is_fn_type(&callee.ty) {
        return false;
    }
    let HirExprKind::Ident(name) = &callee.kind else {
        return true;
    };
    params.contains(name)
        || locals.contains(name)
        || !globals.contains(name)
}

fn lower_stmt_list(
    stmts: &[HirStmt],
    globals: &BTreeSet<String>,
    params: &BTreeSet<String>,
    locals: &mut BTreeSet<String>,
) -> Result<Vec<MirStmt>, String> {
    let mut out = Vec::new();
    for stmt in stmts {
        match stmt {
            HirStmt::Binding { name, expr } => {
                let e = lower_expr(expr, globals, params, locals)?;
                locals.insert(name.clone());
                out.push(MirStmt::Let {
                    name: name.clone(),
                    expr: e,
                });
            }
            HirStmt::Assignment { target, expr } => match &target.kind {
                HirExprKind::Ident(name) => out.push(MirStmt::Assign {
                    name: name.clone(),
                    expr: lower_expr(expr, globals, params, locals)?,
                }),
                _ => {
                    out.push(MirStmt::Expr(MirExpr::Call {
                        callee: Box::new(MirExpr::Var("__assign".to_string())),
                        args: vec![
                            lower_expr(target, globals, params, locals)?,
                            lower_expr(expr, globals, params, locals)?,
                        ],
                    }));
                }
            },
            HirStmt::Return { expr } => {
                out.push(MirStmt::Return(lower_expr(
                    expr, globals, params, locals,
                )?))
            }
            HirStmt::Expr { expr } => {
                out.push(MirStmt::Expr(lower_expr(expr, globals, params, locals)?))
            }
            HirStmt::For { var, iter, body } => {
                let iter_e = lower_expr(iter, globals, params, locals)?;
                let mut inner_locals = locals.clone();
                inner_locals.insert(var.clone());
                out.push(MirStmt::For {
                    var: var.clone(),
                    iter: iter_e,
                    iter_kind: classify_for_iter_kind(&iter.ty),
                    body: lower_stmt_list(body, globals, params, &mut inner_locals)?,
                });
            }
            HirStmt::If {
                cond,
                then_body,
                else_body,
            } => out.push(MirStmt::If {
                cond: lower_expr(cond, globals, params, locals)?,
                then_body: lower_stmt_list(then_body, globals, params, locals)?,
                else_body: lower_stmt_list(else_body, globals, params, locals)?,
            }),
            HirStmt::While { cond, body } => out.push(MirStmt::While {
                cond: lower_expr(cond, globals, params, locals)?,
                body: lower_stmt_list(body, globals, params, locals)?,
            }),
            HirStmt::Repeat { count, body } => out.push(MirStmt::Repeat {
                count: lower_expr(count, globals, params, locals)?,
                body: lower_stmt_list(body, globals, params, locals)?,
            }),
            HirStmt::Break => out.push(MirStmt::Break),
            HirStmt::Continue => out.push(MirStmt::Continue),
            HirStmt::Select { cases } => out.push(MirStmt::Select {
                cases: cases
                    .iter()
                    .map(|c| {
                        Ok(MirSelectCase {
                            pattern: match &c.pattern {
                                HirSelectPattern::Receive { binding, expr } => {
                                    MirSelectPattern::Receive {
                                        binding: binding.clone(),
                                        source: lower_expr(expr, globals, params, locals)?,
                                    }
                                }
                                HirSelectPattern::After { duration_literal } => {
                                    MirSelectPattern::After {
                                        duration_literal: duration_literal.clone(),
                                    }
                                }
                                HirSelectPattern::Closed { ident } => MirSelectPattern::Closed {
                                    ident: ident.clone(),
                                },
                                HirSelectPattern::Default => MirSelectPattern::Default,
                            },
                            action: lower_expr(&c.action, globals, params, locals)?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            }),
            HirStmt::Go { expr } => {
                out.push(MirStmt::Go(lower_expr(expr, globals, params, locals)?))
            }
            HirStmt::Thread { expr } => {
                out.push(MirStmt::Thread(lower_expr(expr, globals, params, locals)?))
            }
            HirStmt::ContractCheck { kind, expr } => out.push(MirStmt::ContractCheck {
                kind: match kind {
                    HirContractKind::Require => MirContractKind::Require,
                    HirContractKind::Ensure => MirContractKind::Ensure,
                },
                expr: lower_expr(expr, globals, params, locals)?,
            }),
            HirStmt::Match {
                scrutinee,
                arms,
                default_action,
            } => out.push(MirStmt::Match {
                scrutinee: lower_expr(scrutinee, globals, params, locals)?,
                arms: arms
                    .iter()
                    .map(|a| {
                        Ok(MirMatchArm {
                            pattern: lower_match_arm_pattern(&a.pattern, globals, params, locals)?,
                            action: lower_expr(&a.action, globals, params, locals)?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
                default_action: default_action
                    .as_ref()
                    .map(|e| lower_expr(e, globals, params, locals))
                    .transpose()?,
            }),
        }
    }
    Ok(out)
}

fn lower_match_arm_pattern(
    expr: &HirExpr,
    _globals: &BTreeSet<String>,
    _params: &BTreeSet<String>,
    _locals: &BTreeSet<String>,
) -> Result<MirExpr, String> {
    match &expr.kind {
        HirExprKind::EnumVariant {
            enum_name,
            variant,
            fields,
        } => {
            let mut payload = Vec::with_capacity(fields.len());
            for (fname, e) in fields {
                let bind = match &e.kind {
                    HirExprKind::Ident(name) if name == "_" => MirExpr::PatternBind { bind_as: None },
                    HirExprKind::Ident(name) => MirExpr::PatternBind {
                        bind_as: Some(name.clone()),
                    },
                    _ => {
                        return Err(format!(
                            "E3499: match pattern field `{fname}` must be an identifier or `_`"
                        ));
                    }
                };
                payload.push((fname.clone(), bind));
            }
            Ok(MirExpr::EnumVariant {
                enum_name: enum_name.clone(),
                variant: variant.clone(),
                payload,
            })
        }
        _ => Err("E3499: match arm pattern must be enum variant".to_string()),
    }
}

fn lower_expr(
    expr: &HirExpr,
    globals: &BTreeSet<String>,
    params: &BTreeSet<String>,
    locals: &BTreeSet<String>,
) -> Result<MirExpr, String> {
    Ok(match &expr.kind {
        HirExprKind::Ident(name) => MirExpr::Var(name.clone()),
        HirExprKind::Int(v) => MirExpr::Int(*v),
        HirExprKind::Float(v) => MirExpr::Float(*v),
        HirExprKind::Bool(v) => MirExpr::Bool(*v),
        HirExprKind::String(v) => MirExpr::Str(v.clone()),
        HirExprKind::List(items) => MirExpr::List(
            items
                .iter()
                .map(|e| lower_expr(e, globals, params, locals))
                .collect::<Result<Vec<_>, String>>()?,
        ),
        HirExprKind::Map(entries) => MirExpr::Map(
            entries
                .iter()
                .map(|(k, v)| {
                    Ok((
                        lower_expr(k, globals, params, locals)?,
                        lower_expr(v, globals, params, locals)?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?,
        ),
        HirExprKind::Member { object, field } => {
            let ot = (!object.ty.is_empty()
                && object.ty != "Int"
                && object.ty != "Float"
                && object.ty != "Bool"
                && object.ty != "Str"
                && !object.ty.starts_with("List")
                && !object.ty.starts_with("Map"))
            .then(|| object.ty.clone());
            MirExpr::Member {
                object: Box::new(lower_expr(object, globals, params, locals)?),
                field: field.clone(),
                object_type: ot,
            }
        },
        HirExprKind::Index { object, index } => MirExpr::Index {
            object: Box::new(lower_expr(object, globals, params, locals)?),
            index: Box::new(lower_expr(index, globals, params, locals)?),
            object_is_str: object.ty == "Str",
        },
        HirExprKind::Slice { object, start, end } => MirExpr::Slice {
            object: Box::new(lower_expr(object, globals, params, locals)?),
            start: start
                .as_ref()
                .map(|e| lower_expr(e, globals, params, locals))
                .transpose()?
                .map(Box::new),
            end: end
                .as_ref()
                .map(|e| lower_expr(e, globals, params, locals))
                .transpose()?
                .map(Box::new),
            object_is_str: object.ty == "Str",
        },
        HirExprKind::Call { callee, args } => {
            if callee_needs_closure_call(callee, globals, params, locals) {
                let (user_param_tys, ret_ty) = parse_fn_mir_sig(&callee.ty).ok_or_else(|| {
                    format!(
                        "could not parse function type `{}` for closure call",
                        callee.ty
                    )
                })?;
                return Ok(MirExpr::ClosureCall {
                    closure: Box::new(lower_expr(callee, globals, params, locals)?),
                    args: args
                        .iter()
                        .map(|a| lower_expr(a, globals, params, locals))
                        .collect::<Result<Vec<_>, String>>()?,
                    user_param_tys,
                    ret_ty,
                });
            }
            if let HirExprKind::Ident(name) = &callee.kind {
                if name == "type_of" {
                    if args.len() != 1 {
                        return Err("`type_of` expects exactly one argument".to_string());
                    }
                    let raw = args[0].ty.trim();
                    let label = if raw.is_empty() || raw == "Unknown" {
                        "Unknown".to_string()
                    } else {
                        raw.to_string()
                    };
                    return Ok(MirExpr::Str(label));
                }
            }
            let mut lowered_callee = lower_expr(callee, globals, params, locals)?;
            if let MirExpr::Member {
                ref object,
                ref mut field,
                ..
            } = lowered_callee
            {
                if let MirExpr::Var(ns) = object.as_ref() {
                    if ns == "json" && field == "encode" && args.len() == 1 {
                        let arg_ty = &args[0].ty;
                        if !arg_ty.is_empty()
                            && arg_ty != "Unknown"
                            && arg_ty
                                .chars()
                                .next()
                                .is_some_and(|c| c.is_ascii_uppercase())
                        {
                            *field = format!("encode_{arg_ty}");
                        }
                    } else if ns == "json" && field == "decode" && args.len() == 2 {
                        let fallback_ty = &args[1].ty;
                        if !fallback_ty.is_empty()
                            && fallback_ty != "Unknown"
                            && fallback_ty
                                .chars()
                                .next()
                                .is_some_and(|c| c.is_ascii_uppercase())
                        {
                            *field = format!("decode_{fallback_ty}");
                        }
                    }
                }
            }
            if let MirExpr::Var(ref name) = lowered_callee {
                if name == "ok" && args.len() == 1 {
                    return Ok(MirExpr::ResultOk {
                        expr: Box::new(lower_expr(&args[0], globals, params, locals)?),
                    });
                }
                if name == "err" && args.len() == 1 {
                    return Ok(MirExpr::ResultErr {
                        expr: Box::new(lower_expr(&args[0], globals, params, locals)?),
                    });
                }
            }
            MirExpr::Call {
                callee: Box::new(lowered_callee),
                args: args
                    .iter()
                    .map(|a| lower_expr(a, globals, params, locals))
                    .collect::<Result<Vec<_>, String>>()?,
            }
        }
        HirExprKind::Binary { left, op, right } => MirExpr::Binary {
            left: Box::new(lower_expr(left, globals, params, locals)?),
            op: format!("{op:?}"),
            right: Box::new(lower_expr(right, globals, params, locals)?),
        },
        HirExprKind::Unary { op, expr } => MirExpr::Unary {
            op: format!("{op:?}"),
            expr: Box::new(lower_expr(expr, globals, params, locals)?),
        },
        HirExprKind::Async { expr } => MirExpr::Async {
            expr: Box::new(lower_expr(expr, globals, params, locals)?),
        },
        HirExprKind::Await { expr } => MirExpr::Await {
            expr: Box::new(lower_expr(expr, globals, params, locals)?),
        },
        HirExprKind::Question { expr } => MirExpr::Question {
            expr: Box::new(lower_expr(expr, globals, params, locals)?),
        },
        HirExprKind::DotResult => MirExpr::DotResult,
        HirExprKind::Old { expr } => MirExpr::Old {
            expr: Box::new(lower_expr(expr, globals, params, locals)?),
        },
        HirExprKind::Constructor { type_name, fields } => MirExpr::Constructor {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(n, e)| Ok((n.clone(), lower_expr(e, globals, params, locals)?)))
                .collect::<Result<Vec<_>, String>>()?,
        },
        HirExprKind::EnumVariant {
            enum_name,
            variant,
            fields,
        } => MirExpr::EnumVariant {
            enum_name: enum_name.clone(),
            variant: variant.clone(),
            payload: fields
                .iter()
                .map(|(n, e)| Ok((n.clone(), lower_expr(e, globals, params, locals)?)))
                .collect::<Result<Vec<_>, String>>()?,
        },
        HirExprKind::MakeClosure {
            closure_fn,
            captures,
        } => MirExpr::MakeClosure {
            closure_fn: closure_fn.clone(),
            captures: captures
                .iter()
                .map(|c| lower_expr(c, globals, params, locals))
                .collect::<Result<Vec<_>, String>>()?,
        },
        HirExprKind::EnvLoad { slot } => MirExpr::EnvLoad {
            slot: *slot,
            ty: parse_type_name(&expr.ty),
        },
    })
}

pub fn verify_mir(program: &MirProgram) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for f in &program.functions {
        if f.name.trim().is_empty() {
            return Err("empty function name in MIR".to_string());
        }
        if !seen.insert(f.name.clone()) {
            return Err(format!("duplicate function `{}` in MIR", f.name));
        }
        verify_stmt_list(&f.body, &mut BTreeMap::new())?;
    }
    Ok(())
}

fn verify_stmt_list(
    stmts: &[MirStmt],
    locals: &mut BTreeMap<String, MirType>,
) -> Result<(), String> {
    for stmt in stmts {
        match stmt {
            MirStmt::Let { name, expr } => {
                if name.trim().is_empty() {
                    return Err("empty binding name in MIR".to_string());
                }
                verify_expr(expr)?;
                locals.insert(name.clone(), MirType::Unknown);
            }
            MirStmt::Assign { name, expr } => {
                if name.trim().is_empty() {
                    return Err("empty assignment target in MIR".to_string());
                }
                verify_expr(expr)?;
            }
            MirStmt::Expr(expr)
            | MirStmt::Return(expr)
            | MirStmt::Go(expr)
            | MirStmt::Thread(expr) => {
                verify_expr(expr)?;
            }
            MirStmt::ContractCheck { expr, .. } => {
                verify_expr(expr)?;
            }
            MirStmt::For {
                var,
                iter,
                iter_kind: _,
                body,
            } => {
                if var.trim().is_empty() {
                    return Err("empty for-loop variable in MIR".to_string());
                }
                verify_expr(iter)?;
                let mut child = locals.clone();
                child.insert(var.clone(), MirType::Unknown);
                verify_stmt_list(body, &mut child)?;
            }
            MirStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                verify_expr(cond)?;
                let mut then_scope = locals.clone();
                verify_stmt_list(then_body, &mut then_scope)?;
                let mut else_scope = locals.clone();
                verify_stmt_list(else_body, &mut else_scope)?;
            }
            MirStmt::While { cond, body } => {
                verify_expr(cond)?;
                let mut child = locals.clone();
                verify_stmt_list(body, &mut child)?;
            }
            MirStmt::Repeat { count, body } => {
                verify_expr(count)?;
                let mut child = locals.clone();
                verify_stmt_list(body, &mut child)?;
            }
            MirStmt::Break | MirStmt::Continue => {}
            MirStmt::Select { cases } => {
                for case in cases {
                    match &case.pattern {
                        MirSelectPattern::Receive { binding, source } => {
                            if binding.trim().is_empty() {
                                return Err("empty select receive binding in MIR".to_string());
                            }
                            verify_expr(source)?;
                        }
                        MirSelectPattern::After { duration_literal } => {
                            if duration_literal.trim().is_empty() {
                                return Err("empty select after duration in MIR".to_string());
                            }
                        }
                        MirSelectPattern::Closed { ident } => {
                            if ident.trim().is_empty() {
                                return Err("empty select closed identifier in MIR".to_string());
                            }
                        }
                        MirSelectPattern::Default => {}
                    }
                    verify_expr(&case.action)?;
                }
            }
            MirStmt::Match {
                scrutinee,
                arms,
                default_action,
            } => {
                verify_expr(scrutinee)?;
                for arm in arms {
                    verify_expr(&arm.pattern)?;
                    verify_expr(&arm.action)?;
                }
                if let Some(e) = default_action {
                    verify_expr(e)?;
                }
            }
        }
    }
    Ok(())
}

fn verify_expr(expr: &MirExpr) -> Result<(), String> {
    match expr {
        MirExpr::Var(name) => {
            if name.trim().is_empty() {
                return Err("empty variable expression in MIR".to_string());
            }
        }
        MirExpr::List(items) => {
            for item in items {
                verify_expr(item)?;
            }
        }
        MirExpr::Map(entries) => {
            for (k, v) in entries {
                verify_expr(k)?;
                verify_expr(v)?;
            }
        }
        MirExpr::Member {
            object,
            field,
            object_type: _,
        } => {
            verify_expr(object)?;
            if field.trim().is_empty() {
                return Err("empty member field in MIR".to_string());
            }
        }
        MirExpr::Index { object, index, .. } => {
            verify_expr(object)?;
            verify_expr(index)?;
        }
        MirExpr::Slice {
            object, start, end, ..
        } => {
            verify_expr(object)?;
            if let Some(start) = start {
                verify_expr(start)?;
            }
            if let Some(end) = end {
                verify_expr(end)?;
            }
        }
        MirExpr::Call { callee, args } => {
            verify_expr(callee)?;
            for arg in args {
                verify_expr(arg)?;
            }
        }
        MirExpr::Binary { left, right, .. } => {
            verify_expr(left)?;
            verify_expr(right)?;
        }
        MirExpr::Unary { expr, .. } => {
            verify_expr(expr)?;
        }
        MirExpr::Async { expr } | MirExpr::Await { expr } => {
            verify_expr(expr)?;
        }
        MirExpr::Question { expr }
        | MirExpr::ResultOk { expr }
        | MirExpr::ResultErr { expr }
        | MirExpr::Old { expr } => {
            verify_expr(expr)?;
        }
        MirExpr::Constructor {
            type_name: _,
            fields,
        } => {
            for (_, e) in fields {
                verify_expr(e)?;
            }
        }
        MirExpr::EnumVariant { payload, .. } => {
            for (_, e) in payload {
                verify_expr(e)?;
            }
        }
        MirExpr::PatternBind { .. } => {}
        MirExpr::MakeClosure { captures, .. } => {
            for c in captures {
                verify_expr(c)?;
            }
        }
        MirExpr::EnvLoad { .. } => {}
        MirExpr::ClosureCall {
            closure,
            args,
            user_param_tys,
            ret_ty: _,
        } => {
            verify_expr(closure)?;
            for a in args {
                verify_expr(a)?;
            }
            if user_param_tys.len() != args.len() {
                return Err("closure call arg count mismatch vs signature".to_string());
            }
        }
        MirExpr::Int(_)
        | MirExpr::Float(_)
        | MirExpr::Bool(_)
        | MirExpr::Str(_)
        | MirExpr::DotResult => {}
    }
    Ok(())
}

pub fn mir_debug_dump(program: &MirProgram) -> String {
    let mut out = String::new();
    for f in &program.functions {
        out.push_str(&format!("fn {}(", f.name));
        for (idx, p) in f.params.iter().enumerate() {
            if idx > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("{}: {}", p.name, mir_type_name(&p.ty)));
        }
        out.push_str(&format!(") -> {} {{\n", mir_type_name(&f.return_type)));
        for stmt in &f.body {
            out.push_str(&format!("  {:?}\n", stmt));
        }
        out.push_str("}\n");
    }
    out
}

fn classify_for_iter_kind(raw_ty: &str) -> MirForIterKind {
    let normalized = raw_ty.replace(' ', "");
    if normalized.starts_with("List<") {
        return MirForIterKind::List;
    }
    if normalized.starts_with("Map<Int,") {
        return MirForIterKind::MapInt;
    }
    if normalized.starts_with("Map<Str,") {
        return MirForIterKind::MapStr;
    }
    MirForIterKind::Unknown
}

fn mir_param_ty_from_hir_raw(raw: &str) -> MirType {
    let compact: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.starts_with("fn(") {
        MirType::Ptr
    } else {
        parse_type_name(&compact)
    }
}

/// Parses `fn(T1,T2,...)->R` (whitespace-tolerant) into user parameter MIR types and return type.
pub fn parse_fn_mir_sig(ty: &str) -> Option<(Vec<MirType>, MirType)> {
    let compact: String = ty.chars().filter(|c| !c.is_whitespace()).collect();
    let rest = compact.strip_prefix("fn(")?;
    let mut depth = 0i32;
    for (i, c) in rest.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                if depth > 0 {
                    depth -= 1;
                } else {
                    let params_slice = &rest[..i];
                    let after = &rest[i + 1..];
                    let ret_slice = after.strip_prefix("->")?;
                    let user_param_tys = if params_slice.is_empty() {
                        vec![]
                    } else {
                        split_fn_param_list(params_slice)?
                            .into_iter()
                            .map(|s| parse_type_name(&s.chars().filter(|c| !c.is_whitespace()).collect::<String>()))
                            .collect()
                    };
                    let ret_ty = parse_type_name(
                        &ret_slice
                            .chars()
                            .filter(|c| !c.is_whitespace())
                            .collect::<String>(),
                    );
                    return Some((user_param_tys, ret_ty));
                }
            }
            _ => {}
        }
    }
    None
}

fn split_fn_param_list(s: &str) -> Option<Vec<String>> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(s[start..i].trim().to_string());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(s[start..].trim().to_string());
    Some(out)
}

pub fn parse_type_name(raw: &str) -> MirType {
    let normalized = raw.replace(' ', "");
    match normalized.as_str() {
        "Int" => MirType::I64,
        "Float" => MirType::F64,
        "Bool" => MirType::Bool,
        "Str" => MirType::Str,
        "Json" => MirType::Json,
        "JsonBuilder" => MirType::JsonBuilder,
        "Result" => MirType::Result,
        "Void" => MirType::Void,
        "Ptr" => MirType::Ptr,
        _ if normalized.starts_with("Result<") => MirType::Result,
        _ => MirType::Unknown,
    }
}

pub fn mir_type_name(ty: &MirType) -> &'static str {
    match ty {
        MirType::I64 => "I64",
        MirType::F64 => "F64",
        MirType::Bool => "Bool",
        MirType::Str => "Str",
        MirType::Json => "Json",
        MirType::JsonBuilder => "JsonBuilder",
        MirType::Result => "Result",
        MirType::Void => "Void",
        MirType::Unknown => "Unknown",
        MirType::Ptr => "Ptr",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use vibe_hir::{HirExpr, HirExprKind, HirFunction, HirParam, HirProgram, HirStmt};

    use super::{lower_hir_to_mir, mir_debug_dump, verify_mir, MirExpr, MirStmt};

    #[test]
    fn fn_param_call_lowers_to_closure_call() {
        let hir = HirProgram {
            functions: vec![
                HirFunction {
                    name: "run_with".to_string(),
                    is_public: true,
                    params: vec![
                        HirParam {
                            name: "x".to_string(),
                            ty: None,
                        },
                        HirParam {
                            name: "cb".to_string(),
                            ty: None,
                        },
                    ],
                    return_type: None,
                    inferred_return_type: Some("Int".to_string()),
                    effects_declared: BTreeSet::new(),
                    effects_observed: BTreeSet::new(),
                    body: vec![],
                    tail_expr: Some(HirExpr::new(
                        HirExprKind::Call {
                            callee: Box::new(HirExpr::new(
                                HirExprKind::Ident("cb".to_string()),
                                "fn(Int)->Int",
                            )),
                            args: vec![HirExpr::new(HirExprKind::Ident("x".to_string()), "Int")],
                        },
                        "Int",
                    )),
                    native_symbol: None,
                },
                HirFunction {
                    name: "main".to_string(),
                    is_public: true,
                    params: vec![],
                    return_type: None,
                    inferred_return_type: Some("Int".to_string()),
                    effects_declared: BTreeSet::new(),
                    effects_observed: BTreeSet::new(),
                    body: vec![],
                    tail_expr: Some(HirExpr::new(HirExprKind::Int(0), "Int")),
                    native_symbol: None,
                },
            ],
        };
        let mir = lower_hir_to_mir(&hir).expect("lower");
        let run = mir.functions.iter().find(|f| f.name == "run_with").unwrap();
        assert!(
            matches!(
                run.body.as_slice(),
                [MirStmt::Return(MirExpr::ClosureCall { .. })]
            ),
            "expected ClosureCall for `cb(x)`, got {:?}",
            run.body
        );
    }

    #[test]
    fn type_of_call_lowers_to_string_of_hir_type() {
        let hir = HirProgram {
            functions: vec![HirFunction {
                name: "main".to_string(),
                is_public: true,
                params: vec![],
                return_type: None,
                inferred_return_type: Some("Int".to_string()),
                effects_declared: BTreeSet::new(),
                effects_observed: BTreeSet::new(),
                body: vec![],
                tail_expr: Some(HirExpr::new(
                    HirExprKind::Call {
                        callee: Box::new(HirExpr::new(
                            HirExprKind::Ident("type_of".to_string()),
                            "Unknown",
                        )),
                        args: vec![HirExpr::new(HirExprKind::Int(1), "Int")],
                    },
                    "Str",
                )),
                native_symbol: None,
            }],
        };
        let mir = lower_hir_to_mir(&hir).expect("lowering should succeed");
        assert!(matches!(
            mir.functions[0].body.as_slice(),
            [MirStmt::Return(MirExpr::Str(s))] if s == "Int"
        ));
    }

    #[test]
    fn lower_hir_program_to_mir_program() {
        let hir = HirProgram {
            functions: vec![HirFunction {
                name: "main".to_string(),
                is_public: true,
                params: vec![],
                return_type: None,
                inferred_return_type: Some("Int".to_string()),
                effects_declared: BTreeSet::new(),
                effects_observed: BTreeSet::new(),
                body: vec![HirStmt::Expr {
                    expr: HirExpr::new(
                        HirExprKind::Call {
                            callee: Box::new(HirExpr::new(
                                HirExprKind::Ident("println".to_string()),
                                "Unknown",
                            )),
                            args: vec![HirExpr::new(
                                HirExprKind::String("hello".to_string()),
                                "Str",
                            )],
                        },
                        "Void",
                    ),
                }],
                tail_expr: Some(HirExpr::new(HirExprKind::Int(0), "Int")),
                native_symbol: None,
            }],
        };
        let mir = lower_hir_to_mir(&hir).expect("lowering should succeed");
        assert_eq!(mir.functions.len(), 1);
        assert!(matches!(
            mir.functions[0].body.last(),
            Some(MirStmt::Return(_))
        ));
    }

    #[test]
    fn mir_dump_is_stable_for_same_input() {
        let hir = HirProgram {
            functions: vec![HirFunction {
                name: "main".to_string(),
                is_public: true,
                params: vec![],
                return_type: None,
                inferred_return_type: Some("Int".to_string()),
                effects_declared: BTreeSet::new(),
                effects_observed: BTreeSet::new(),
                body: vec![],
                tail_expr: Some(HirExpr::new(HirExprKind::Int(7), "Int")),
                native_symbol: None,
            }],
        };
        let first = lower_hir_to_mir(&hir).expect("first lowering");
        let second = lower_hir_to_mir(&hir).expect("second lowering");
        assert_eq!(mir_debug_dump(&first), mir_debug_dump(&second));
        assert!(verify_mir(&first).is_ok());
    }
}
