// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

use crate::{MirExpr, MirFunction, MirProgram, MirStmt};

pub fn optimize_mir(program: &mut MirProgram, level: u8) {
    if level == 0 {
        return;
    }
    constant_fold_program(program);
    dead_code_eliminate_program(program);
    if level >= 2 {
        inline_small_functions(program);
        constant_fold_program(program);
        dead_code_eliminate_program(program);
        licm_program(program);
    }
}

fn constant_fold_expr(expr: &MirExpr) -> MirExpr {
    match expr {
        MirExpr::Binary { left, op, right } => {
            let left_folded = constant_fold_expr(left);
            let right_folded = constant_fold_expr(right);

            if let (MirExpr::Int(l), MirExpr::Int(r)) = (&left_folded, &right_folded) {
                match op.as_str() {
                    "Add" => return MirExpr::Int(l.wrapping_add(*r)),
                    "Sub" => return MirExpr::Int(l.wrapping_sub(*r)),
                    "Mul" => return MirExpr::Int(l.wrapping_mul(*r)),
                    "Div" if *r != 0 => return MirExpr::Int(l / r),
                    "Mod" if *r != 0 => return MirExpr::Int(l % r),
                    "Lt" => return MirExpr::Int(if l < r { 1 } else { 0 }),
                    "Le" => return MirExpr::Int(if l <= r { 1 } else { 0 }),
                    "Gt" => return MirExpr::Int(if l > r { 1 } else { 0 }),
                    "Ge" => return MirExpr::Int(if l >= r { 1 } else { 0 }),
                    "Eq" => return MirExpr::Int(if l == r { 1 } else { 0 }),
                    "Ne" => return MirExpr::Int(if l != r { 1 } else { 0 }),
                    _ => {}
                }
            }

            if let (MirExpr::Float(l), MirExpr::Float(r)) = (&left_folded, &right_folded) {
                match op.as_str() {
                    "Add" => return MirExpr::Float(l + r),
                    "Sub" => return MirExpr::Float(l - r),
                    "Mul" => return MirExpr::Float(l * r),
                    "Div" if *r != 0.0 => return MirExpr::Float(l / r),
                    _ => {}
                }
            }

            if let (MirExpr::Bool(l), MirExpr::Bool(r)) = (&left_folded, &right_folded) {
                match op.as_str() {
                    "And" => return MirExpr::Bool(*l && *r),
                    "Or" => return MirExpr::Bool(*l || *r),
                    _ => {}
                }
            }

            if let (MirExpr::Str(l), MirExpr::Str(r)) = (&left_folded, &right_folded) {
                if op == "Add" {
                    return MirExpr::Str(format!("{l}{r}"));
                }
            }

            MirExpr::Binary {
                left: Box::new(left_folded),
                op: op.clone(),
                right: Box::new(right_folded),
            }
        }
        MirExpr::Unary { op, expr } => {
            let folded = constant_fold_expr(expr);
            match (&folded, op.as_str()) {
                (MirExpr::Int(v), "Neg") => MirExpr::Int(-v),
                (MirExpr::Float(v), "Neg") => MirExpr::Float(-v),
                (MirExpr::Bool(v), "Not") => MirExpr::Bool(!v),
                _ => MirExpr::Unary {
                    op: op.clone(),
                    expr: Box::new(folded),
                },
            }
        }
        MirExpr::Call { callee, args } => MirExpr::Call {
            callee: Box::new(constant_fold_expr(callee)),
            args: args.iter().map(constant_fold_expr).collect(),
        },
        MirExpr::Index {
            object,
            index,
            object_is_str,
        } => MirExpr::Index {
            object: Box::new(constant_fold_expr(object)),
            index: Box::new(constant_fold_expr(index)),
            object_is_str: *object_is_str,
        },
        MirExpr::Member {
            object,
            field,
            object_type,
        } => MirExpr::Member {
            object: Box::new(constant_fold_expr(object)),
            field: field.clone(),
            object_type: object_type.clone(),
        },
        MirExpr::List(items) => MirExpr::List(items.iter().map(constant_fold_expr).collect()),
        MirExpr::Map(pairs) => MirExpr::Map(
            pairs
                .iter()
                .map(|(k, v)| (constant_fold_expr(k), constant_fold_expr(v)))
                .collect(),
        ),
        MirExpr::Slice {
            object,
            start,
            end,
            object_is_str,
        } => MirExpr::Slice {
            object: Box::new(constant_fold_expr(object)),
            start: start.as_ref().map(|e| Box::new(constant_fold_expr(e))),
            end: end.as_ref().map(|e| Box::new(constant_fold_expr(e))),
            object_is_str: *object_is_str,
        },
        MirExpr::Constructor { type_name, fields } => MirExpr::Constructor {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(n, e)| (n.clone(), constant_fold_expr(e)))
                .collect(),
        },
        MirExpr::Async { expr } => MirExpr::Async {
            expr: Box::new(constant_fold_expr(expr)),
        },
        MirExpr::Await { expr } => MirExpr::Await {
            expr: Box::new(constant_fold_expr(expr)),
        },
        MirExpr::Question { expr } => MirExpr::Question {
            expr: Box::new(constant_fold_expr(expr)),
        },
        MirExpr::Old { expr } => MirExpr::Old {
            expr: Box::new(constant_fold_expr(expr)),
        },
        other => other.clone(),
    }
}

fn constant_fold_stmts(stmts: &mut Vec<MirStmt>) {
    for stmt in stmts.iter_mut() {
        match stmt {
            MirStmt::Let { expr, .. } | MirStmt::Assign { expr, .. } => {
                *expr = constant_fold_expr(expr);
            }
            MirStmt::Expr(expr)
            | MirStmt::Return(expr)
            | MirStmt::Go(expr)
            | MirStmt::Thread(expr) => {
                *expr = constant_fold_expr(expr);
            }
            MirStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                *cond = constant_fold_expr(cond);
                constant_fold_stmts(then_body);
                constant_fold_stmts(else_body);
            }
            MirStmt::While { cond, body } => {
                *cond = constant_fold_expr(cond);
                constant_fold_stmts(body);
            }
            MirStmt::For { iter, body, .. } => {
                *iter = constant_fold_expr(iter);
                constant_fold_stmts(body);
            }
            MirStmt::Repeat { count, body } => {
                *count = constant_fold_expr(count);
                constant_fold_stmts(body);
            }
            MirStmt::ContractCheck { expr, .. } => {
                *expr = constant_fold_expr(expr);
            }
            MirStmt::Select { cases } => {
                for case in cases {
                    case.action = constant_fold_expr(&case.action);
                }
            }
            MirStmt::Match {
                scrutinee,
                arms,
                default_action,
            } => {
                *scrutinee = constant_fold_expr(scrutinee);
                for arm in arms {
                    arm.pattern = constant_fold_expr(&arm.pattern);
                    arm.action = constant_fold_expr(&arm.action);
                }
                if let Some(def) = default_action {
                    *def = constant_fold_expr(def);
                }
            }
            MirStmt::Break | MirStmt::Continue => {}
        }
    }
}

fn constant_fold_program(program: &mut MirProgram) {
    for func in &mut program.functions {
        constant_fold_stmts(&mut func.body);
    }
}

fn collect_used_vars_expr(expr: &MirExpr, used: &mut BTreeSet<String>) {
    match expr {
        MirExpr::Var(name) => {
            used.insert(name.clone());
        }
        MirExpr::Binary { left, right, .. } => {
            collect_used_vars_expr(left, used);
            collect_used_vars_expr(right, used);
        }
        MirExpr::Unary { expr, .. } => collect_used_vars_expr(expr, used),
        MirExpr::Call { callee, args } => {
            collect_used_vars_expr(callee, used);
            for arg in args {
                collect_used_vars_expr(arg, used);
            }
        }
        MirExpr::Index { object, index, .. } => {
            collect_used_vars_expr(object, used);
            collect_used_vars_expr(index, used);
        }
        MirExpr::Slice {
            object, start, end, ..
        } => {
            collect_used_vars_expr(object, used);
            if let Some(s) = start {
                collect_used_vars_expr(s, used);
            }
            if let Some(e) = end {
                collect_used_vars_expr(e, used);
            }
        }
        MirExpr::Member { object, .. } => collect_used_vars_expr(object, used),
        MirExpr::List(items) => {
            for item in items {
                collect_used_vars_expr(item, used);
            }
        }
        MirExpr::Map(pairs) => {
            for (k, v) in pairs {
                collect_used_vars_expr(k, used);
                collect_used_vars_expr(v, used);
            }
        }
        MirExpr::Constructor { fields, .. } => {
            for (_, e) in fields {
                collect_used_vars_expr(e, used);
            }
        }
        MirExpr::Async { expr }
        | MirExpr::Await { expr }
        | MirExpr::Question { expr }
        | MirExpr::Old { expr } => {
            collect_used_vars_expr(expr, used);
        }
        _ => {}
    }
}

fn collect_used_vars_stmts(stmts: &[MirStmt], used: &mut BTreeSet<String>) {
    for stmt in stmts {
        match stmt {
            MirStmt::Let { expr, .. } | MirStmt::Assign { expr, .. } => {
                collect_used_vars_expr(expr, used);
            }
            MirStmt::Expr(expr)
            | MirStmt::Return(expr)
            | MirStmt::Go(expr)
            | MirStmt::Thread(expr) => {
                collect_used_vars_expr(expr, used);
            }
            MirStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                collect_used_vars_expr(cond, used);
                collect_used_vars_stmts(then_body, used);
                collect_used_vars_stmts(else_body, used);
            }
            MirStmt::While { cond, body } => {
                collect_used_vars_expr(cond, used);
                collect_used_vars_stmts(body, used);
            }
            MirStmt::For {
                var, iter, body, ..
            } => {
                used.insert(var.clone());
                collect_used_vars_expr(iter, used);
                collect_used_vars_stmts(body, used);
            }
            MirStmt::Repeat { count, body } => {
                collect_used_vars_expr(count, used);
                collect_used_vars_stmts(body, used);
            }
            MirStmt::ContractCheck { expr, .. } => {
                collect_used_vars_expr(expr, used);
            }
            MirStmt::Select { cases } => {
                for case in cases {
                    collect_used_vars_expr(&case.action, used);
                }
            }
            MirStmt::Match {
                scrutinee,
                arms,
                default_action,
            } => {
                collect_used_vars_expr(scrutinee, used);
                for arm in arms {
                    collect_used_vars_expr(&arm.action, used);
                }
                if let Some(def) = default_action {
                    collect_used_vars_expr(def, used);
                }
            }
            MirStmt::Break | MirStmt::Continue => {}
        }
    }
}

fn expr_has_side_effects(expr: &MirExpr) -> bool {
    match expr {
        MirExpr::Int(_)
        | MirExpr::Float(_)
        | MirExpr::Bool(_)
        | MirExpr::Str(_)
        | MirExpr::Var(_)
        | MirExpr::DotResult
        | MirExpr::EnumVariant { .. } => false,
        MirExpr::Binary { left, right, .. } => {
            expr_has_side_effects(left) || expr_has_side_effects(right)
        }
        MirExpr::Unary { expr, .. } => expr_has_side_effects(expr),
        MirExpr::List(items) => items.iter().any(expr_has_side_effects),
        MirExpr::Map(pairs) => pairs
            .iter()
            .any(|(k, v)| expr_has_side_effects(k) || expr_has_side_effects(v)),
        MirExpr::Constructor { fields, .. } => fields.iter().any(|(_, e)| expr_has_side_effects(e)),
        MirExpr::Call { .. }
        | MirExpr::Index { .. }
        | MirExpr::Slice { .. }
        | MirExpr::Member { .. }
        | MirExpr::Async { .. }
        | MirExpr::Await { .. }
        | MirExpr::Question { .. }
        | MirExpr::Old { .. } => true,
    }
}

fn dce_stmts(stmts: &mut Vec<MirStmt>) {
    let mut used = BTreeSet::new();
    collect_used_vars_stmts(stmts, &mut used);

    stmts.retain(|stmt| match stmt {
        MirStmt::Let { name, expr } => used.contains(name) || expr_has_side_effects(expr),
        _ => true,
    });

    for stmt in stmts.iter_mut() {
        match stmt {
            MirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                dce_stmts(then_body);
                dce_stmts(else_body);
            }
            MirStmt::While { body, .. }
            | MirStmt::For { body, .. }
            | MirStmt::Repeat { body, .. } => {
                dce_stmts(body);
            }
            _ => {}
        }
    }
}

fn dead_code_eliminate_program(program: &mut MirProgram) {
    for func in &mut program.functions {
        dce_stmts(&mut func.body);
    }
}

fn count_stmts(stmts: &[MirStmt]) -> usize {
    let mut count = 0;
    for stmt in stmts {
        count += 1;
        match stmt {
            MirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                count += count_stmts(then_body);
                count += count_stmts(else_body);
            }
            MirStmt::While { body, .. }
            | MirStmt::For { body, .. }
            | MirStmt::Repeat { body, .. } => {
                count += count_stmts(body);
            }
            _ => {}
        }
    }
    count
}

fn substitute_expr(expr: &MirExpr, bindings: &BTreeMap<String, MirExpr>) -> MirExpr {
    match expr {
        MirExpr::Var(name) => {
            if let Some(replacement) = bindings.get(name) {
                replacement.clone()
            } else {
                expr.clone()
            }
        }
        MirExpr::Binary { left, op, right } => MirExpr::Binary {
            left: Box::new(substitute_expr(left, bindings)),
            op: op.clone(),
            right: Box::new(substitute_expr(right, bindings)),
        },
        MirExpr::Unary { op, expr } => MirExpr::Unary {
            op: op.clone(),
            expr: Box::new(substitute_expr(expr, bindings)),
        },
        MirExpr::Call { callee, args } => MirExpr::Call {
            callee: Box::new(substitute_expr(callee, bindings)),
            args: args.iter().map(|a| substitute_expr(a, bindings)).collect(),
        },
        MirExpr::Index {
            object,
            index,
            object_is_str,
        } => MirExpr::Index {
            object: Box::new(substitute_expr(object, bindings)),
            index: Box::new(substitute_expr(index, bindings)),
            object_is_str: *object_is_str,
        },
        MirExpr::Member {
            object,
            field,
            object_type,
        } => MirExpr::Member {
            object: Box::new(substitute_expr(object, bindings)),
            field: field.clone(),
            object_type: object_type.clone(),
        },
        MirExpr::List(items) => {
            MirExpr::List(items.iter().map(|i| substitute_expr(i, bindings)).collect())
        }
        MirExpr::Map(pairs) => MirExpr::Map(
            pairs
                .iter()
                .map(|(k, v)| (substitute_expr(k, bindings), substitute_expr(v, bindings)))
                .collect(),
        ),
        MirExpr::Slice {
            object,
            start,
            end,
            object_is_str,
        } => MirExpr::Slice {
            object: Box::new(substitute_expr(object, bindings)),
            start: start
                .as_ref()
                .map(|e| Box::new(substitute_expr(e, bindings))),
            end: end.as_ref().map(|e| Box::new(substitute_expr(e, bindings))),
            object_is_str: *object_is_str,
        },
        MirExpr::Constructor { type_name, fields } => MirExpr::Constructor {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(n, e)| (n.clone(), substitute_expr(e, bindings)))
                .collect(),
        },
        MirExpr::Async { expr } => MirExpr::Async {
            expr: Box::new(substitute_expr(expr, bindings)),
        },
        MirExpr::Await { expr } => MirExpr::Await {
            expr: Box::new(substitute_expr(expr, bindings)),
        },
        MirExpr::Question { expr } => MirExpr::Question {
            expr: Box::new(substitute_expr(expr, bindings)),
        },
        MirExpr::Old { expr } => MirExpr::Old {
            expr: Box::new(substitute_expr(expr, bindings)),
        },
        other => other.clone(),
    }
}

fn substitute_stmts(stmts: &[MirStmt], bindings: &BTreeMap<String, MirExpr>) -> Vec<MirStmt> {
    stmts
        .iter()
        .map(|stmt| match stmt {
            MirStmt::Let { name, expr } => MirStmt::Let {
                name: name.clone(),
                expr: substitute_expr(expr, bindings),
            },
            MirStmt::Assign { name, expr } => MirStmt::Assign {
                name: name.clone(),
                expr: substitute_expr(expr, bindings),
            },
            MirStmt::Expr(expr) => MirStmt::Expr(substitute_expr(expr, bindings)),
            MirStmt::Return(expr) => MirStmt::Return(substitute_expr(expr, bindings)),
            MirStmt::Go(expr) => MirStmt::Go(substitute_expr(expr, bindings)),
            MirStmt::Thread(expr) => MirStmt::Thread(substitute_expr(expr, bindings)),
            MirStmt::If {
                cond,
                then_body,
                else_body,
            } => MirStmt::If {
                cond: substitute_expr(cond, bindings),
                then_body: substitute_stmts(then_body, bindings),
                else_body: substitute_stmts(else_body, bindings),
            },
            MirStmt::While { cond, body } => MirStmt::While {
                cond: substitute_expr(cond, bindings),
                body: substitute_stmts(body, bindings),
            },
            MirStmt::For {
                var,
                iter,
                iter_kind,
                body,
            } => MirStmt::For {
                var: var.clone(),
                iter: substitute_expr(iter, bindings),
                iter_kind: *iter_kind,
                body: substitute_stmts(body, bindings),
            },
            MirStmt::Repeat { count, body } => MirStmt::Repeat {
                count: substitute_expr(count, bindings),
                body: substitute_stmts(body, bindings),
            },
            other => other.clone(),
        })
        .collect()
}

const MAX_INLINE_STMTS: usize = 12;

fn inline_small_functions(program: &mut MirProgram) {
    let candidates: BTreeMap<String, &MirFunction> = program
        .functions
        .iter()
        .filter(|f| {
            !f.is_public
                && f.name != "main"
                && count_stmts(&f.body) <= MAX_INLINE_STMTS
                && !contains_recursion(&f.body, &f.name)
        })
        .map(|f| (f.name.clone(), f))
        .collect();

    if candidates.is_empty() {
        return;
    }

    let inline_bodies: BTreeMap<String, (Vec<String>, Vec<MirStmt>)> = candidates
        .iter()
        .map(|(name, f)| {
            let param_names: Vec<String> = f.params.iter().map(|p| p.name.clone()).collect();
            (name.clone(), (param_names, f.body.clone()))
        })
        .collect();

    for func in &mut program.functions {
        inline_calls_in_stmts(&mut func.body, &inline_bodies);
    }
}

fn contains_recursion(stmts: &[MirStmt], fn_name: &str) -> bool {
    for stmt in stmts {
        match stmt {
            MirStmt::Let { expr, .. }
            | MirStmt::Assign { expr, .. }
            | MirStmt::Expr(expr)
            | MirStmt::Return(expr)
            | MirStmt::Go(expr)
            | MirStmt::Thread(expr) => {
                if expr_calls_fn(expr, fn_name) {
                    return true;
                }
            }
            MirStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                if expr_calls_fn(cond, fn_name)
                    || contains_recursion(then_body, fn_name)
                    || contains_recursion(else_body, fn_name)
                {
                    return true;
                }
            }
            MirStmt::While { cond, body } => {
                if expr_calls_fn(cond, fn_name) || contains_recursion(body, fn_name) {
                    return true;
                }
            }
            MirStmt::For { iter, body, .. } => {
                if expr_calls_fn(iter, fn_name) || contains_recursion(body, fn_name) {
                    return true;
                }
            }
            MirStmt::Repeat { count, body } => {
                if expr_calls_fn(count, fn_name) || contains_recursion(body, fn_name) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn expr_calls_fn(expr: &MirExpr, fn_name: &str) -> bool {
    match expr {
        MirExpr::Call { callee, args } => {
            if matches!(&**callee, MirExpr::Var(name) if name == fn_name) {
                return true;
            }
            expr_calls_fn(callee, fn_name) || args.iter().any(|a| expr_calls_fn(a, fn_name))
        }
        MirExpr::Binary { left, right, .. } => {
            expr_calls_fn(left, fn_name) || expr_calls_fn(right, fn_name)
        }
        MirExpr::Unary { expr, .. } => expr_calls_fn(expr, fn_name),
        MirExpr::Index { object, index, .. } => {
            expr_calls_fn(object, fn_name) || expr_calls_fn(index, fn_name)
        }
        MirExpr::Member { object, .. } => expr_calls_fn(object, fn_name),
        MirExpr::List(items) => items.iter().any(|i| expr_calls_fn(i, fn_name)),
        MirExpr::Map(pairs) => pairs
            .iter()
            .any(|(k, v)| expr_calls_fn(k, fn_name) || expr_calls_fn(v, fn_name)),
        _ => false,
    }
}

fn inline_calls_in_stmts(
    stmts: &mut Vec<MirStmt>,
    inline_bodies: &BTreeMap<String, (Vec<String>, Vec<MirStmt>)>,
) {
    let mut i = 0;
    while i < stmts.len() {
        match &mut stmts[i] {
            MirStmt::Let { name, expr } => {
                if let Some((result_name, expanded)) =
                    try_inline_call_expr(expr, inline_bodies, name)
                {
                    let _ = result_name;
                    stmts.splice(i..=i, expanded);
                    continue;
                }
            }
            MirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                inline_calls_in_stmts(then_body, inline_bodies);
                inline_calls_in_stmts(else_body, inline_bodies);
            }
            MirStmt::While { body, .. }
            | MirStmt::For { body, .. }
            | MirStmt::Repeat { body, .. } => {
                inline_calls_in_stmts(body, inline_bodies);
            }
            _ => {}
        }
        i += 1;
    }
}

fn try_inline_call_expr(
    expr: &MirExpr,
    inline_bodies: &BTreeMap<String, (Vec<String>, Vec<MirStmt>)>,
    result_var: &str,
) -> Option<(String, Vec<MirStmt>)> {
    if let MirExpr::Call { callee, args } = expr {
        if let MirExpr::Var(fn_name) = &**callee {
            if let Some((param_names, body)) = inline_bodies.get(fn_name) {
                if args.len() != param_names.len() {
                    return None;
                }
                let mut bindings = BTreeMap::new();
                for (pname, arg) in param_names.iter().zip(args.iter()) {
                    bindings.insert(pname.clone(), arg.clone());
                }
                let mut inlined = substitute_stmts(body, &bindings);
                rewrite_returns_to_assign(&mut inlined, result_var);
                return Some((result_var.to_string(), inlined));
            }
        }
    }
    None
}

fn rewrite_returns_to_assign(stmts: &mut Vec<MirStmt>, target_var: &str) {
    for stmt in stmts.iter_mut() {
        match stmt {
            MirStmt::Return(expr) => {
                *stmt = MirStmt::Let {
                    name: target_var.to_string(),
                    expr: expr.clone(),
                };
            }
            MirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                rewrite_returns_to_assign(then_body, target_var);
                rewrite_returns_to_assign(else_body, target_var);
            }
            MirStmt::While { body, .. }
            | MirStmt::For { body, .. }
            | MirStmt::Repeat { body, .. } => {
                rewrite_returns_to_assign(body, target_var);
            }
            _ => {}
        }
    }
}

fn collect_modified_vars(stmts: &[MirStmt], modified: &mut BTreeSet<String>) {
    for stmt in stmts {
        match stmt {
            MirStmt::Assign { name, .. } => {
                modified.insert(name.clone());
            }
            MirStmt::Let { name, .. } => {
                modified.insert(name.clone());
            }
            MirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_modified_vars(then_body, modified);
                collect_modified_vars(else_body, modified);
            }
            MirStmt::While { body, .. }
            | MirStmt::For { body, .. }
            | MirStmt::Repeat { body, .. } => {
                collect_modified_vars(body, modified);
            }
            _ => {}
        }
    }
}

fn expr_is_loop_invariant(expr: &MirExpr, modified: &BTreeSet<String>) -> bool {
    match expr {
        MirExpr::Int(_)
        | MirExpr::Float(_)
        | MirExpr::Bool(_)
        | MirExpr::Str(_)
        | MirExpr::DotResult
        | MirExpr::EnumVariant { .. } => true,
        MirExpr::Var(name) => !modified.contains(name),
        MirExpr::Binary { left, right, .. } => {
            expr_is_loop_invariant(left, modified) && expr_is_loop_invariant(right, modified)
        }
        MirExpr::Unary { expr, .. } => expr_is_loop_invariant(expr, modified),
        _ => false,
    }
}

fn hoist_from_loop_body(body: &mut Vec<MirStmt>) -> Vec<MirStmt> {
    let mut modified = BTreeSet::new();
    collect_modified_vars(body, &mut modified);

    let mut hoisted = Vec::new();
    body.retain(|stmt| {
        if let MirStmt::Let { name, expr } = stmt {
            if !modified.contains(name) && expr_is_loop_invariant(expr, &modified) {
                hoisted.push(stmt.clone());
                return false;
            }
        }
        true
    });
    hoisted
}

fn licm_stmts(stmts: &mut Vec<MirStmt>) {
    let mut insertions: Vec<(usize, Vec<MirStmt>)> = Vec::new();

    for (i, stmt) in stmts.iter_mut().enumerate() {
        match stmt {
            MirStmt::While { body, .. } => {
                let hoisted = hoist_from_loop_body(body);
                if !hoisted.is_empty() {
                    insertions.push((i, hoisted));
                }
                licm_stmts(body);
            }
            MirStmt::For { body, .. } | MirStmt::Repeat { body, .. } => {
                let hoisted = hoist_from_loop_body(body);
                if !hoisted.is_empty() {
                    insertions.push((i, hoisted));
                }
                licm_stmts(body);
            }
            MirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                licm_stmts(then_body);
                licm_stmts(else_body);
            }
            _ => {}
        }
    }

    let mut offset = 0;
    for (pos, hoisted) in insertions {
        let insert_at = pos + offset;
        let count = hoisted.len();
        for (j, h) in hoisted.into_iter().enumerate() {
            stmts.insert(insert_at + j, h);
        }
        offset += count;
    }
}

fn licm_program(program: &mut MirProgram) {
    for func in &mut program.functions {
        licm_stmts(&mut func.body);
    }
}
