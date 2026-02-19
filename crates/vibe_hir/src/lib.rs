use std::collections::{BTreeMap, BTreeSet};

use vibe_ast::{BinaryOp, TypeRef, UnaryOp};

#[derive(Debug, Clone, Default)]
pub struct HirProgram {
    pub functions: Vec<HirFunction>,
}

#[derive(Debug, Clone, Default)]
pub struct HirFunction {
    pub name: String,
    pub is_public: bool,
    pub params: Vec<HirParam>,
    pub return_type: Option<TypeRef>,
    pub inferred_return_type: Option<String>,
    pub effects_declared: BTreeSet<String>,
    pub effects_observed: BTreeSet<String>,
    pub body: Vec<HirStmt>,
    pub tail_expr: Option<HirExpr>,
}

#[derive(Debug, Clone, Default)]
pub struct HirParam {
    pub name: String,
    pub ty: Option<TypeRef>,
}

#[derive(Debug, Clone)]
pub enum HirStmt {
    Binding {
        name: String,
        expr: HirExpr,
    },
    Assignment {
        target: HirExpr,
        expr: HirExpr,
    },
    Return {
        expr: HirExpr,
    },
    Expr {
        expr: HirExpr,
    },
    For {
        var: String,
        iter: HirExpr,
        body: Vec<HirStmt>,
    },
    If {
        cond: HirExpr,
        then_body: Vec<HirStmt>,
        else_body: Vec<HirStmt>,
    },
    While {
        cond: HirExpr,
        body: Vec<HirStmt>,
    },
    Repeat {
        count: HirExpr,
        body: Vec<HirStmt>,
    },
    Select {
        cases: Vec<HirSelectCase>,
    },
    Go {
        expr: HirExpr,
    },
}

#[derive(Debug, Clone)]
pub struct HirSelectCase {
    pub pattern: HirSelectPattern,
    pub action: HirExpr,
}

#[derive(Debug, Clone)]
pub enum HirSelectPattern {
    Receive { binding: String, expr: HirExpr },
    After { duration_literal: String },
    Closed { ident: String },
    Default,
}

#[derive(Debug, Clone)]
pub enum HirExprKind {
    Ident(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    List(Vec<HirExpr>),
    Map(Vec<(HirExpr, HirExpr)>),
    Member {
        object: Box<HirExpr>,
        field: String,
    },
    Call {
        callee: Box<HirExpr>,
        args: Vec<HirExpr>,
    },
    Binary {
        left: Box<HirExpr>,
        op: BinaryOp,
        right: Box<HirExpr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<HirExpr>,
    },
    Question {
        expr: Box<HirExpr>,
    },
    DotResult,
    Old {
        expr: Box<HirExpr>,
    },
}

#[derive(Debug, Clone)]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub ty: String,
}

impl HirExpr {
    pub fn new(kind: HirExprKind, ty: impl Into<String>) -> Self {
        Self {
            kind,
            ty: ty.into(),
        }
    }
}

pub fn verify_hir(program: &HirProgram) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for f in &program.functions {
        if f.name.trim().is_empty() {
            return Err("empty function name in HIR".to_string());
        }
        if !seen.insert(f.name.clone()) {
            return Err(format!("duplicate function `{}` in HIR", f.name));
        }

        let mut scope = BTreeMap::new();
        for p in &f.params {
            scope.insert(
                p.name.clone(),
                p.ty.as_ref()
                    .map(|t| t.raw.clone())
                    .unwrap_or_else(|| "Unknown".to_string()),
            );
        }

        verify_stmt_list(&f.body, &mut scope)?;
        if let Some(tail) = &f.tail_expr {
            verify_expr(tail)?;
        }
    }
    Ok(())
}

fn verify_stmt_list(stmts: &[HirStmt], scope: &mut BTreeMap<String, String>) -> Result<(), String> {
    for stmt in stmts {
        match stmt {
            HirStmt::Binding { name, expr } => {
                if name.trim().is_empty() {
                    return Err("empty binding name in HIR".to_string());
                }
                verify_expr(expr)?;
                scope.insert(name.clone(), expr.ty.clone());
            }
            HirStmt::Assignment { target, expr } => {
                verify_expr(target)?;
                verify_expr(expr)?;
            }
            HirStmt::Return { expr } | HirStmt::Expr { expr } | HirStmt::Go { expr } => {
                verify_expr(expr)?;
            }
            HirStmt::For { var, iter, body } => {
                if var.trim().is_empty() {
                    return Err("empty for-loop variable in HIR".to_string());
                }
                verify_expr(iter)?;
                let mut child = scope.clone();
                child.insert(var.clone(), "Unknown".to_string());
                verify_stmt_list(body, &mut child)?;
            }
            HirStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                verify_expr(cond)?;
                let mut then_scope = scope.clone();
                verify_stmt_list(then_body, &mut then_scope)?;
                let mut else_scope = scope.clone();
                verify_stmt_list(else_body, &mut else_scope)?;
            }
            HirStmt::While { cond, body } => {
                verify_expr(cond)?;
                let mut child = scope.clone();
                verify_stmt_list(body, &mut child)?;
            }
            HirStmt::Repeat { count, body } => {
                verify_expr(count)?;
                let mut child = scope.clone();
                verify_stmt_list(body, &mut child)?;
            }
            HirStmt::Select { cases } => {
                for case in cases {
                    match &case.pattern {
                        HirSelectPattern::Receive { binding, expr } => {
                            if binding.trim().is_empty() {
                                return Err("empty select receive binding in HIR".to_string());
                            }
                            verify_expr(expr)?;
                        }
                        HirSelectPattern::After { duration_literal } => {
                            if duration_literal.trim().is_empty() {
                                return Err("empty duration literal in select after".to_string());
                            }
                        }
                        HirSelectPattern::Closed { ident } => {
                            if ident.trim().is_empty() {
                                return Err("empty identifier in select closed pattern".to_string());
                            }
                        }
                        HirSelectPattern::Default => {}
                    }
                    verify_expr(&case.action)?;
                }
            }
        }
    }
    Ok(())
}

fn verify_expr(expr: &HirExpr) -> Result<(), String> {
    if expr.ty.trim().is_empty() {
        return Err("empty expression type in HIR".to_string());
    }
    match &expr.kind {
        HirExprKind::Ident(name) => {
            if name.trim().is_empty() {
                return Err("empty identifier expression in HIR".to_string());
            }
        }
        HirExprKind::List(items) => {
            for item in items {
                verify_expr(item)?;
            }
        }
        HirExprKind::Map(entries) => {
            for (k, v) in entries {
                verify_expr(k)?;
                verify_expr(v)?;
            }
        }
        HirExprKind::Member { object, field } => {
            verify_expr(object)?;
            if field.trim().is_empty() {
                return Err("empty member field name in HIR".to_string());
            }
        }
        HirExprKind::Call { callee, args } => {
            verify_expr(callee)?;
            for arg in args {
                verify_expr(arg)?;
            }
        }
        HirExprKind::Binary { left, right, .. } => {
            verify_expr(left)?;
            verify_expr(right)?;
        }
        HirExprKind::Unary { expr, .. } => {
            verify_expr(expr)?;
        }
        HirExprKind::Question { expr } => {
            verify_expr(expr)?;
        }
        HirExprKind::Old { expr } => {
            verify_expr(expr)?;
        }
        HirExprKind::Int(_)
        | HirExprKind::Float(_)
        | HirExprKind::Bool(_)
        | HirExprKind::String(_)
        | HirExprKind::DotResult => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_hir_accepts_function_with_body() {
        let program = HirProgram {
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
                            args: vec![HirExpr::new(HirExprKind::String("hi".to_string()), "Str")],
                        },
                        "Void",
                    ),
                }],
                tail_expr: Some(HirExpr::new(HirExprKind::Int(0), "Int")),
            }],
        };
        assert!(verify_hir(&program).is_ok());
    }

    #[test]
    fn verify_hir_rejects_empty_expr_type() {
        let program = HirProgram {
            functions: vec![HirFunction {
                name: "main".to_string(),
                is_public: true,
                params: vec![],
                return_type: None,
                inferred_return_type: Some("Int".to_string()),
                effects_declared: BTreeSet::new(),
                effects_observed: BTreeSet::new(),
                body: vec![HirStmt::Expr {
                    expr: HirExpr::new(HirExprKind::Int(1), ""),
                }],
                tail_expr: None,
            }],
        };
        let err = verify_hir(&program).expect_err("expected verifier failure");
        assert!(err.contains("empty expression type"));
    }
}
