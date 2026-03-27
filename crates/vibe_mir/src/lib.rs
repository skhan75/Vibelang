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
    EnumVariant {
        enum_name: String,
        variant: String,
    },
}

pub fn lower_hir_to_mir(hir: &HirProgram) -> Result<MirProgram, String> {
    let mut out = MirProgram::default();
    for f in &hir.functions {
        let mut body = lower_stmt_list(&f.body)?;
        if let Some(tail) = &f.tail_expr {
            body.push(MirStmt::Return(lower_expr(tail)?));
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
                        .map(|t| parse_type_name(&t.raw))
                        .unwrap_or(MirType::Unknown),
                })
                .collect(),
            return_type: f
                .return_type
                .as_ref()
                .map(|t| parse_type_name(&t.raw))
                .unwrap_or_else(|| {
                    parse_type_name(f.inferred_return_type.as_deref().unwrap_or("Unknown"))
                }),
            body,
            native_symbol: f.native_symbol.clone(),
        });
    }
    verify_mir(&out)?;
    Ok(out)
}

fn lower_stmt_list(stmts: &[HirStmt]) -> Result<Vec<MirStmt>, String> {
    let mut out = Vec::new();
    for stmt in stmts {
        match stmt {
            HirStmt::Binding { name, expr } => out.push(MirStmt::Let {
                name: name.clone(),
                expr: lower_expr(expr)?,
            }),
            HirStmt::Assignment { target, expr } => match &target.kind {
                HirExprKind::Ident(name) => out.push(MirStmt::Assign {
                    name: name.clone(),
                    expr: lower_expr(expr)?,
                }),
                _ => {
                    out.push(MirStmt::Expr(MirExpr::Call {
                        callee: Box::new(MirExpr::Var("__assign".to_string())),
                        args: vec![lower_expr(target)?, lower_expr(expr)?],
                    }));
                }
            },
            HirStmt::Return { expr } => out.push(MirStmt::Return(lower_expr(expr)?)),
            HirStmt::Expr { expr } => out.push(MirStmt::Expr(lower_expr(expr)?)),
            HirStmt::For { var, iter, body } => out.push(MirStmt::For {
                var: var.clone(),
                iter: lower_expr(iter)?,
                iter_kind: classify_for_iter_kind(&iter.ty),
                body: lower_stmt_list(body)?,
            }),
            HirStmt::If {
                cond,
                then_body,
                else_body,
            } => out.push(MirStmt::If {
                cond: lower_expr(cond)?,
                then_body: lower_stmt_list(then_body)?,
                else_body: lower_stmt_list(else_body)?,
            }),
            HirStmt::While { cond, body } => out.push(MirStmt::While {
                cond: lower_expr(cond)?,
                body: lower_stmt_list(body)?,
            }),
            HirStmt::Repeat { count, body } => out.push(MirStmt::Repeat {
                count: lower_expr(count)?,
                body: lower_stmt_list(body)?,
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
                                        source: lower_expr(expr)?,
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
                            action: lower_expr(&c.action)?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            }),
            HirStmt::Go { expr } => out.push(MirStmt::Go(lower_expr(expr)?)),
            HirStmt::Thread { expr } => out.push(MirStmt::Thread(lower_expr(expr)?)),
            HirStmt::ContractCheck { kind, expr } => out.push(MirStmt::ContractCheck {
                kind: match kind {
                    HirContractKind::Require => MirContractKind::Require,
                    HirContractKind::Ensure => MirContractKind::Ensure,
                },
                expr: lower_expr(expr)?,
            }),
            HirStmt::Match {
                scrutinee,
                arms,
                default_action,
            } => out.push(MirStmt::Match {
                scrutinee: lower_expr(scrutinee)?,
                arms: arms
                    .iter()
                    .map(|a| {
                        Ok(MirMatchArm {
                            pattern: lower_expr(&a.pattern)?,
                            action: lower_expr(&a.action)?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
                default_action: default_action.as_ref().map(lower_expr).transpose()?,
            }),
        }
    }
    Ok(out)
}

fn lower_expr(expr: &HirExpr) -> Result<MirExpr, String> {
    Ok(match &expr.kind {
        HirExprKind::Ident(name) => MirExpr::Var(name.clone()),
        HirExprKind::Int(v) => MirExpr::Int(*v),
        HirExprKind::Float(v) => MirExpr::Float(*v),
        HirExprKind::Bool(v) => MirExpr::Bool(*v),
        HirExprKind::String(v) => MirExpr::Str(v.clone()),
        HirExprKind::List(items) => MirExpr::List(
            items
                .iter()
                .map(lower_expr)
                .collect::<Result<Vec<_>, String>>()?,
        ),
        HirExprKind::Map(entries) => MirExpr::Map(
            entries
                .iter()
                .map(|(k, v)| Ok((lower_expr(k)?, lower_expr(v)?)))
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
                object: Box::new(lower_expr(object)?),
                field: field.clone(),
                object_type: ot,
            }
        }
        HirExprKind::Index { object, index } => MirExpr::Index {
            object: Box::new(lower_expr(object)?),
            index: Box::new(lower_expr(index)?),
            object_is_str: object.ty == "Str",
        },
        HirExprKind::Slice { object, start, end } => MirExpr::Slice {
            object: Box::new(lower_expr(object)?),
            start: start
                .as_ref()
                .map(|expr| lower_expr(expr))
                .transpose()?
                .map(Box::new),
            end: end
                .as_ref()
                .map(|expr| lower_expr(expr))
                .transpose()?
                .map(Box::new),
            object_is_str: object.ty == "Str",
        },
        HirExprKind::Call { callee, args } => {
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
            let mut lowered_callee = lower_expr(callee)?;
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
                        expr: Box::new(lower_expr(&args[0])?),
                    });
                }
                if name == "err" && args.len() == 1 {
                    return Ok(MirExpr::ResultErr {
                        expr: Box::new(lower_expr(&args[0])?),
                    });
                }
            }
            MirExpr::Call {
                callee: Box::new(lowered_callee),
                args: args
                    .iter()
                    .map(lower_expr)
                    .collect::<Result<Vec<_>, String>>()?,
            }
        }
        HirExprKind::Binary { left, op, right } => MirExpr::Binary {
            left: Box::new(lower_expr(left)?),
            op: format!("{op:?}"),
            right: Box::new(lower_expr(right)?),
        },
        HirExprKind::Unary { op, expr } => MirExpr::Unary {
            op: format!("{op:?}"),
            expr: Box::new(lower_expr(expr)?),
        },
        HirExprKind::Async { expr } => MirExpr::Async {
            expr: Box::new(lower_expr(expr)?),
        },
        HirExprKind::Await { expr } => MirExpr::Await {
            expr: Box::new(lower_expr(expr)?),
        },
        HirExprKind::Question { expr } => MirExpr::Question {
            expr: Box::new(lower_expr(expr)?),
        },
        HirExprKind::DotResult => MirExpr::DotResult,
        HirExprKind::Old { expr } => MirExpr::Old {
            expr: Box::new(lower_expr(expr)?),
        },
        HirExprKind::Constructor { type_name, fields } => MirExpr::Constructor {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(n, e)| Ok((n.clone(), lower_expr(e)?)))
                .collect::<Result<Vec<_>, String>>()?,
        },
        HirExprKind::EnumVariant { enum_name, variant } => MirExpr::EnumVariant {
            enum_name: enum_name.clone(),
            variant: variant.clone(),
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
        MirExpr::EnumVariant { .. } => {}
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
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use vibe_hir::{HirExpr, HirExprKind, HirFunction, HirProgram, HirStmt};

    use super::{lower_hir_to_mir, mir_debug_dump, verify_mir, MirExpr, MirStmt};

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
