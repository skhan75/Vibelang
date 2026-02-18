use std::collections::{BTreeMap, BTreeSet};

use vibe_hir::{HirExpr, HirExprKind, HirProgram, HirSelectPattern, HirStmt};

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
    Select {
        cases: Vec<MirSelectCase>,
    },
    Go(MirExpr),
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
    Question {
        expr: Box<MirExpr>,
    },
    DotResult,
    Old {
        expr: Box<MirExpr>,
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
            HirStmt::For { var, iter, body } => {
                out.push(MirStmt::Let {
                    name: var.clone(),
                    expr: lower_expr(iter)?,
                });
                out.extend(lower_stmt_list(body)?);
            }
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
                            },
                            action: lower_expr(&c.action)?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            }),
            HirStmt::Go { expr } => out.push(MirStmt::Go(lower_expr(expr)?)),
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
        HirExprKind::Member { object, field } => MirExpr::Member {
            object: Box::new(lower_expr(object)?),
            field: field.clone(),
        },
        HirExprKind::Call { callee, args } => MirExpr::Call {
            callee: Box::new(lower_expr(callee)?),
            args: args
                .iter()
                .map(lower_expr)
                .collect::<Result<Vec<_>, String>>()?,
        },
        HirExprKind::Binary { left, op, right } => MirExpr::Binary {
            left: Box::new(lower_expr(left)?),
            op: format!("{op:?}"),
            right: Box::new(lower_expr(right)?),
        },
        HirExprKind::Unary { op, expr } => MirExpr::Unary {
            op: format!("{op:?}"),
            expr: Box::new(lower_expr(expr)?),
        },
        HirExprKind::Question { expr } => MirExpr::Question {
            expr: Box::new(lower_expr(expr)?),
        },
        HirExprKind::DotResult => MirExpr::DotResult,
        HirExprKind::Old { expr } => MirExpr::Old {
            expr: Box::new(lower_expr(expr)?),
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
            MirStmt::Expr(expr) | MirStmt::Return(expr) | MirStmt::Go(expr) => {
                verify_expr(expr)?;
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
                    }
                    verify_expr(&case.action)?;
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
        MirExpr::Member { object, field } => {
            verify_expr(object)?;
            if field.trim().is_empty() {
                return Err("empty member field in MIR".to_string());
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
        MirExpr::Question { expr } | MirExpr::Old { expr } => {
            verify_expr(expr)?;
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

pub fn parse_type_name(raw: &str) -> MirType {
    let normalized = raw.replace(' ', "");
    match normalized.as_str() {
        "Int" => MirType::I64,
        "Float" => MirType::F64,
        "Bool" => MirType::Bool,
        "Str" => MirType::Str,
        "Void" => MirType::Void,
        _ => MirType::Unknown,
    }
}

pub fn mir_type_name(ty: &MirType) -> &'static str {
    match ty {
        MirType::I64 => "I64",
        MirType::F64 => "F64",
        MirType::Bool => "Bool",
        MirType::Str => "Str",
        MirType::Void => "Void",
        MirType::Unknown => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use vibe_hir::{HirExpr, HirExprKind, HirFunction, HirProgram, HirStmt};

    use super::{lower_hir_to_mir, mir_debug_dump, verify_mir, MirStmt};

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
            }],
        };
        let first = lower_hir_to_mir(&hir).expect("first lowering");
        let second = lower_hir_to_mir(&hir).expect("second lowering");
        assert_eq!(mir_debug_dump(&first), mir_debug_dump(&second));
        assert!(verify_mir(&first).is_ok());
    }
}
