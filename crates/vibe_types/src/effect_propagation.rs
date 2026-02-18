use std::collections::{BTreeMap, BTreeSet};

use vibe_ast::{Contract, Expr, FunctionDecl, Stmt};
use vibe_diagnostics::Span;

#[derive(Debug, Clone)]
pub struct FunctionEffectSummary {
    pub name: String,
    pub span: Span,
    pub declared_effects: BTreeSet<String>,
    pub direct_observed_effects: BTreeSet<String>,
    pub direct_calls: BTreeSet<String>,
}

pub fn collect_direct_calls(func: &FunctionDecl) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for contract in &func.contracts {
        collect_calls_from_contract(contract, &mut out);
    }
    for stmt in &func.body {
        collect_calls_from_stmt(stmt, &mut out);
    }
    if let Some(expr) = &func.tail_expr {
        collect_calls_from_expr(expr, &mut out);
    }
    out.remove(&func.name);
    out
}

pub fn compute_transitive_effects(
    summaries: &[FunctionEffectSummary],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut effect_map = BTreeMap::new();
    for summary in summaries {
        effect_map.insert(
            summary.name.clone(),
            summary.direct_observed_effects.clone(),
        );
    }

    let mut changed = true;
    while changed {
        changed = false;
        for summary in summaries {
            let mut merged = effect_map
                .get(&summary.name)
                .cloned()
                .unwrap_or_else(BTreeSet::new);
            for callee in &summary.direct_calls {
                if let Some(callee_effects) = effect_map.get(callee) {
                    let before = merged.len();
                    merged.extend(callee_effects.iter().cloned());
                    if merged.len() != before {
                        changed = true;
                    }
                }
            }
            effect_map.insert(summary.name.clone(), merged);
        }
    }

    effect_map
}

fn collect_calls_from_contract(contract: &Contract, out: &mut BTreeSet<String>) {
    match contract {
        Contract::Require { expr, .. } | Contract::Ensure { expr, .. } => {
            collect_calls_from_expr(expr, out);
        }
        Contract::Examples { cases, .. } => {
            for case in cases {
                collect_calls_from_expr(&case.call, out);
                collect_calls_from_expr(&case.expected, out);
            }
        }
        Contract::Intent { .. } | Contract::Effect { .. } => {}
    }
}

fn collect_calls_from_stmt(stmt: &Stmt, out: &mut BTreeSet<String>) {
    match stmt {
        Stmt::Binding { expr, .. } | Stmt::ExprStmt { expr, .. } | Stmt::Go { expr, .. } => {
            collect_calls_from_expr(expr, out);
        }
        Stmt::Assignment { target, expr, .. } => {
            collect_calls_from_expr(target, out);
            collect_calls_from_expr(expr, out);
        }
        Stmt::Return { expr, .. } => {
            collect_calls_from_expr(expr, out);
        }
        Stmt::For { iter, body, .. }
        | Stmt::While {
            cond: iter, body, ..
        } => {
            collect_calls_from_expr(iter, out);
            for s in body {
                collect_calls_from_stmt(s, out);
            }
        }
        Stmt::If {
            cond,
            then_body,
            else_body,
            ..
        } => {
            collect_calls_from_expr(cond, out);
            for s in then_body {
                collect_calls_from_stmt(s, out);
            }
            for s in else_body {
                collect_calls_from_stmt(s, out);
            }
        }
        Stmt::Repeat { count, body, .. } => {
            collect_calls_from_expr(count, out);
            for s in body {
                collect_calls_from_stmt(s, out);
            }
        }
        Stmt::Select { cases, .. } => {
            for case in cases {
                match &case.pattern {
                    vibe_ast::SelectPattern::Receive { expr, .. } => {
                        collect_calls_from_expr(expr, out);
                    }
                    vibe_ast::SelectPattern::After { .. }
                    | vibe_ast::SelectPattern::Closed { .. } => {}
                }
                collect_calls_from_expr(&case.action, out);
            }
        }
    }
}

fn collect_calls_from_expr(expr: &Expr, out: &mut BTreeSet<String>) {
    match expr {
        Expr::Call { callee, args, .. } => {
            if let Expr::Ident { name, .. } = &**callee {
                out.insert(name.clone());
            }
            collect_calls_from_expr(callee, out);
            for arg in args {
                collect_calls_from_expr(arg, out);
            }
        }
        Expr::Member { object, .. }
        | Expr::Unary { expr: object, .. }
        | Expr::Question { expr: object, .. }
        | Expr::Old { expr: object, .. } => {
            collect_calls_from_expr(object, out);
        }
        Expr::Binary { left, right, .. } => {
            collect_calls_from_expr(left, out);
            collect_calls_from_expr(right, out);
        }
        Expr::List { items, .. } => {
            for item in items {
                collect_calls_from_expr(item, out);
            }
        }
        Expr::Map { entries, .. } => {
            for (k, v) in entries {
                collect_calls_from_expr(k, out);
                collect_calls_from_expr(v, out);
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
