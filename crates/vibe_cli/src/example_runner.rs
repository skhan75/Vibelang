// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use vibe_ast::{BinaryOp, Contract, Declaration, Expr, FileAst, FunctionDecl, Stmt, UnaryOp};

use crate::deterministic_utils::{
    deterministic_len, deterministic_max, deterministic_min, deterministic_sort_desc,
    deterministic_sorted_desc, deterministic_take, deterministic_value_name,
    deterministic_value_to_string, DeterministicValue,
};

const MAX_CALL_DEPTH: usize = 32;
const MAX_LOOP_ITERS: usize = 20_000;

#[derive(Debug, Clone, Default)]
pub struct ExampleRunSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<String>,
}

#[allow(dead_code)]
pub fn run_examples(ast: &FileAst) -> ExampleRunSummary {
    run_examples_with_policy(ast, true)
}

pub fn run_examples_with_policy(ast: &FileAst, enforce_contracts: bool) -> ExampleRunSummary {
    let mut summary = ExampleRunSummary::default();
    let mut functions = BTreeMap::new();
    for decl in &ast.declarations {
        let Declaration::Function(func) = decl else {
            continue;
        };
        functions.insert(func.name.clone(), func);
    }

    for decl in &ast.declarations {
        let Declaration::Function(func) = decl else {
            continue;
        };
        for contract in &func.contracts {
            let Contract::Examples { cases, .. } = contract else {
                continue;
            };
            for case in cases {
                summary.total += 1;
                match run_example_case(case, &functions, enforce_contracts) {
                    Ok(()) => summary.passed += 1,
                    Err(err) => {
                        summary.failed += 1;
                        summary
                            .failures
                            .push(format!("{} example failed: {err}", func.name));
                    }
                }
            }
        }
    }
    summary
}

fn run_example_case(
    case: &vibe_ast::ExampleCase,
    functions: &BTreeMap<String, &FunctionDecl>,
    enforce_contracts: bool,
) -> Result<(), String> {
    let mut env = BTreeMap::new();
    let got = eval_expr_with_ctx(
        &case.call,
        &mut env,
        functions,
        0,
        None,
        None,
        enforce_contracts,
    )?;
    let mut env = BTreeMap::new();
    let expected = eval_expr_with_ctx(
        &case.expected,
        &mut env,
        functions,
        0,
        None,
        None,
        enforce_contracts,
    )?;
    if got == expected {
        Ok(())
    } else {
        Err(format!(
            "expected {}, got {}",
            deterministic_value_to_string(&expected),
            deterministic_value_to_string(&got)
        ))
    }
}

#[allow(dead_code)]
fn eval_expr(
    expr: &Expr,
    env: &mut BTreeMap<String, DeterministicValue>,
    functions: &BTreeMap<String, &FunctionDecl>,
    depth: usize,
) -> Result<DeterministicValue, String> {
    eval_expr_with_ctx(expr, env, functions, depth, None, None, true)
}

fn eval_expr_with_ctx(
    expr: &Expr,
    env: &mut BTreeMap<String, DeterministicValue>,
    functions: &BTreeMap<String, &FunctionDecl>,
    depth: usize,
    dot_result: Option<&DeterministicValue>,
    old_env: Option<&BTreeMap<String, DeterministicValue>>,
    enforce_contracts: bool,
) -> Result<DeterministicValue, String> {
    if depth > MAX_CALL_DEPTH {
        return Err("example evaluation exceeded max call depth".to_string());
    }
    match expr {
        Expr::Ident { name, .. } => env
            .get(name)
            .cloned()
            .ok_or_else(|| format!("unknown identifier `{name}` in example evaluation")),
        Expr::Int { value, .. } => Ok(DeterministicValue::Int(*value)),
        Expr::Float { value, .. } => Ok(DeterministicValue::Float(*value)),
        Expr::Bool { value, .. } => Ok(DeterministicValue::Bool(*value)),
        Expr::String { value, .. } => Ok(DeterministicValue::Str(value.clone())),
        Expr::List { items, .. } => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(eval_expr_with_ctx(
                    item,
                    env,
                    functions,
                    depth + 1,
                    dot_result,
                    old_env,
                    enforce_contracts,
                )?);
            }
            Ok(DeterministicValue::List(out))
        }
        Expr::Map { .. } => Err("map evaluation is not supported in phase 2 examples".to_string()),
        Expr::Member { object, field, .. } => {
            let obj = eval_expr_with_ctx(
                object,
                env,
                functions,
                depth + 1,
                dot_result,
                old_env,
                enforce_contracts,
            )?;
            if field == "len" {
                return deterministic_len(&obj);
            }
            Err(format!(
                "member access `.{field}` is not supported in phase 2 examples"
            ))
        }
        Expr::Index { .. } => Err("indexing is not supported in phase 2 examples".to_string()),
        Expr::Slice { .. } => Err("slicing is not supported in phase 2 examples".to_string()),
        Expr::Call { callee, args, .. } => {
            let mut eval_args = Vec::with_capacity(args.len());
            for arg in args {
                eval_args.push(eval_expr_with_ctx(
                    arg,
                    env,
                    functions,
                    depth + 1,
                    dot_result,
                    old_env,
                    enforce_contracts,
                )?);
            }
            match &**callee {
                Expr::Ident { name, .. } => {
                    eval_ident_call(name, eval_args, functions, depth + 1, enforce_contracts)
                }
                Expr::Member { object, field, .. } => {
                    let object_value = eval_expr_with_ctx(
                        object,
                        env,
                        functions,
                        depth + 1,
                        dot_result,
                        old_env,
                        enforce_contracts,
                    )?;
                    eval_member_call(&object_value, field, &eval_args)
                }
                _ => Err("dynamic call target is not supported in phase 2 examples".to_string()),
            }
        }
        Expr::Binary {
            left, op, right, ..
        } => {
            let l = eval_expr_with_ctx(
                left,
                env,
                functions,
                depth + 1,
                dot_result,
                old_env,
                enforce_contracts,
            )?;
            let r = eval_expr_with_ctx(
                right,
                env,
                functions,
                depth + 1,
                dot_result,
                old_env,
                enforce_contracts,
            )?;
            eval_binary(op, &l, &r)
        }
        Expr::Unary { op, expr, .. } => {
            let v = eval_expr_with_ctx(
                expr,
                env,
                functions,
                depth + 1,
                dot_result,
                old_env,
                enforce_contracts,
            )?;
            eval_unary(op, &v)
        }
        Expr::Async { expr, .. } | Expr::Await { expr, .. } => eval_expr_with_ctx(
            expr,
            env,
            functions,
            depth + 1,
            dot_result,
            old_env,
            enforce_contracts,
        ),
        Expr::Question { expr, .. } => eval_expr_with_ctx(
            expr,
            env,
            functions,
            depth + 1,
            dot_result,
            old_env,
            enforce_contracts,
        ),
        Expr::DotResult { .. } => dot_result
            .cloned()
            .ok_or_else(|| "`.` placeholder is not valid in this context".to_string()),
        Expr::Old { expr, .. } => {
            let Some(snapshot) = old_env else {
                return Err("`old(...)` is not valid in this context".to_string());
            };
            let mut snapshot_env = snapshot.clone();
            eval_expr_with_ctx(
                expr,
                &mut snapshot_env,
                functions,
                depth + 1,
                dot_result,
                old_env,
                enforce_contracts,
            )
        }
        Expr::Constructor { .. } => {
            Err("type constructors are not supported in phase 2 examples".to_string())
        }
        Expr::EnumVariant { .. } => {
            Err("enum variants are not supported in phase 2 examples".to_string())
        }
    }
}

fn eval_ident_call(
    name: &str,
    args: Vec<DeterministicValue>,
    functions: &BTreeMap<String, &FunctionDecl>,
    depth: usize,
    enforce_contracts: bool,
) -> Result<DeterministicValue, String> {
    match name {
        "len" => {
            if args.len() != 1 {
                return Err("len expects one argument".to_string());
            }
            deterministic_len(&args[0])
        }
        "min" => {
            if args.len() != 2 {
                return Err("min expects two arguments".to_string());
            }
            deterministic_min(&args[0], &args[1])
        }
        "max" => {
            if args.len() != 2 {
                return Err("max expects two arguments".to_string());
            }
            deterministic_max(&args[0], &args[1])
        }
        "sorted_desc" => {
            if args.len() != 1 {
                return Err("sorted_desc expects one argument".to_string());
            }
            deterministic_sorted_desc(&args[0])
        }
        "cpu_count" => Ok(DeterministicValue::Int(1)),
        "ok" | "err" | "print" | "println" => Ok(DeterministicValue::Void),
        other => eval_function_call(other, args, functions, depth + 1, enforce_contracts),
    }
}

fn eval_member_call(
    object: &DeterministicValue,
    field: &str,
    args: &[DeterministicValue],
) -> Result<DeterministicValue, String> {
    match field {
        "sort_desc" => {
            if !args.is_empty() {
                return Err("sort_desc expects no arguments".to_string());
            }
            deterministic_sort_desc(object)
        }
        "take" => {
            if args.len() != 1 {
                return Err("take expects one argument".to_string());
            }
            deterministic_take(object, &args[0])
        }
        _ => Err(format!(
            "method `.{field}()` is not supported in phase 2 examples"
        )),
    }
}

fn eval_function_call(
    name: &str,
    args: Vec<DeterministicValue>,
    functions: &BTreeMap<String, &FunctionDecl>,
    depth: usize,
    enforce_contracts: bool,
) -> Result<DeterministicValue, String> {
    if depth > MAX_CALL_DEPTH {
        return Err("example function call exceeded max depth".to_string());
    }
    let func = functions
        .get(name)
        .ok_or_else(|| format!("unknown function `{name}` in example call"))?;
    if func.params.len() != args.len() {
        return Err(format!(
            "function `{name}` expects {} args, got {}",
            func.params.len(),
            args.len()
        ));
    }
    let mut env = BTreeMap::new();
    for (param, arg) in func.params.iter().zip(args) {
        env.insert(param.name.clone(), arg);
    }
    let entry_snapshot = env.clone();
    if enforce_contracts {
        for contract in &func.contracts {
            let Contract::Require { expr, .. } = contract else {
                continue;
            };
            let mut require_env = env.clone();
            let value = eval_expr_with_ctx(
                expr,
                &mut require_env,
                functions,
                depth + 1,
                None,
                Some(&entry_snapshot),
                enforce_contracts,
            )?;
            if !value.as_bool()? {
                return Err(format!("contract @require failed in `{}`", func.name));
            }
        }
    }

    let flow = eval_stmt_list(
        &func.body,
        &mut env,
        functions,
        depth + 1,
        enforce_contracts,
    )?;
    let result = match flow {
        EvalControl::Return(ret) => ret,
        EvalControl::Next => {
            if let Some(expr) = &func.tail_expr {
                eval_expr_with_ctx(
                    expr,
                    &mut env,
                    functions,
                    depth + 1,
                    None,
                    Some(&entry_snapshot),
                    enforce_contracts,
                )?
            } else {
                DeterministicValue::Void
            }
        }
        EvalControl::Break | EvalControl::Continue => {
            return Err("`break`/`continue` used outside a loop".to_string());
        }
    };

    if enforce_contracts {
        for contract in &func.contracts {
            let Contract::Ensure { expr, .. } = contract else {
                continue;
            };
            let mut ensure_env = env.clone();
            let value = eval_expr_with_ctx(
                expr,
                &mut ensure_env,
                functions,
                depth + 1,
                Some(&result),
                Some(&entry_snapshot),
                enforce_contracts,
            )?;
            if !value.as_bool()? {
                return Err(format!("contract @ensure failed in `{}`", func.name));
            }
        }
    }

    Ok(result)
}

enum EvalControl {
    Next,
    Return(DeterministicValue),
    Break,
    Continue,
}

fn eval_stmt_list(
    stmts: &[Stmt],
    env: &mut BTreeMap<String, DeterministicValue>,
    functions: &BTreeMap<String, &FunctionDecl>,
    depth: usize,
    enforce_contracts: bool,
) -> Result<EvalControl, String> {
    for stmt in stmts {
        let flow = eval_stmt(stmt, env, functions, depth + 1, enforce_contracts)?;
        if !matches!(flow, EvalControl::Next) {
            return Ok(flow);
        }
    }
    Ok(EvalControl::Next)
}

fn try_eval_mutating_member_stmt(
    expr: &Expr,
    env: &mut BTreeMap<String, DeterministicValue>,
    functions: &BTreeMap<String, &FunctionDecl>,
    depth: usize,
    enforce_contracts: bool,
) -> Result<bool, String> {
    let Expr::Call { callee, args, .. } = expr else {
        return Ok(false);
    };
    let Expr::Member { object, field, .. } = &**callee else {
        return Ok(false);
    };
    let Expr::Ident { name, .. } = &**object else {
        return Ok(false);
    };
    match field.as_str() {
        "append" => {
            if args.len() != 1 {
                return Err("append expects one argument".to_string());
            }
            let value = eval_expr_with_ctx(
                &args[0],
                env,
                functions,
                depth + 1,
                None,
                None,
                enforce_contracts,
            )?;
            let target = env
                .get_mut(name)
                .ok_or_else(|| format!("unknown identifier `{name}` for append"))?;
            match target {
                DeterministicValue::List(items) => {
                    items.push(value);
                    Ok(true)
                }
                other => Err(format!(
                    "append target must be List, got `{}`",
                    deterministic_value_name(other)
                )),
            }
        }
        "set" => {
            if args.len() != 2 {
                return Err("set expects index and value arguments".to_string());
            }
            let index = eval_expr_with_ctx(
                &args[0],
                env,
                functions,
                depth + 1,
                None,
                None,
                enforce_contracts,
            )?
            .as_int()?;
            let value = eval_expr_with_ctx(
                &args[1],
                env,
                functions,
                depth + 1,
                None,
                None,
                enforce_contracts,
            )?;
            let target = env
                .get_mut(name)
                .ok_or_else(|| format!("unknown identifier `{name}` for set"))?;
            match target {
                DeterministicValue::List(items) => {
                    if index < 0 || index as usize >= items.len() {
                        return Err(
                            "list.set index out of bounds in example evaluation".to_string()
                        );
                    }
                    items[index as usize] = value;
                    Ok(true)
                }
                other => Err(format!(
                    "set target must be List, got `{}`",
                    deterministic_value_name(other)
                )),
            }
        }
        _ => Ok(false),
    }
}

fn eval_stmt(
    stmt: &Stmt,
    env: &mut BTreeMap<String, DeterministicValue>,
    functions: &BTreeMap<String, &FunctionDecl>,
    depth: usize,
    enforce_contracts: bool,
) -> Result<EvalControl, String> {
    match stmt {
        Stmt::Binding { name, expr, .. } => {
            let value = eval_expr_with_ctx(
                expr,
                env,
                functions,
                depth + 1,
                None,
                None,
                enforce_contracts,
            )?;
            env.insert(name.clone(), value);
            Ok(EvalControl::Next)
        }
        Stmt::Assignment { target, expr, .. } => match target {
            Expr::Ident { name, .. } => {
                let value = eval_expr_with_ctx(
                    expr,
                    env,
                    functions,
                    depth + 1,
                    None,
                    None,
                    enforce_contracts,
                )?;
                env.insert(name.clone(), value);
                Ok(EvalControl::Next)
            }
            _ => Err("only identifier assignment is supported in phase 2 examples".to_string()),
        },
        Stmt::Return { expr, .. } => Ok(EvalControl::Return(eval_expr_with_ctx(
            expr,
            env,
            functions,
            depth + 1,
            None,
            None,
            enforce_contracts,
        )?)),
        Stmt::ExprStmt { expr, .. } => {
            if try_eval_mutating_member_stmt(expr, env, functions, depth + 1, enforce_contracts)? {
                return Ok(EvalControl::Next);
            }
            let _ = eval_expr_with_ctx(
                expr,
                env,
                functions,
                depth + 1,
                None,
                None,
                enforce_contracts,
            )?;
            Ok(EvalControl::Next)
        }
        Stmt::For {
            var, iter, body, ..
        } => {
            let iterable = eval_expr_with_ctx(
                iter,
                env,
                functions,
                depth + 1,
                None,
                None,
                enforce_contracts,
            )?;
            let items = iterable.as_list()?.to_vec();
            for item in items {
                env.insert(var.clone(), item);
                match eval_stmt_list(body, env, functions, depth + 1, enforce_contracts)? {
                    EvalControl::Next => {}
                    EvalControl::Continue => continue,
                    EvalControl::Break => return Ok(EvalControl::Next),
                    EvalControl::Return(ret) => return Ok(EvalControl::Return(ret)),
                }
            }
            Ok(EvalControl::Next)
        }
        Stmt::If {
            cond,
            then_body,
            else_body,
            ..
        } => {
            let cond_value = eval_expr_with_ctx(
                cond,
                env,
                functions,
                depth + 1,
                None,
                None,
                enforce_contracts,
            )?;
            let branch = if cond_value.as_bool()? {
                then_body
            } else {
                else_body
            };
            eval_stmt_list(branch, env, functions, depth + 1, enforce_contracts)
        }
        Stmt::While { cond, body, .. } => {
            for _ in 0..MAX_LOOP_ITERS {
                let cond_value = eval_expr_with_ctx(
                    cond,
                    env,
                    functions,
                    depth + 1,
                    None,
                    None,
                    enforce_contracts,
                )?;
                if !cond_value.as_bool()? {
                    return Ok(EvalControl::Next);
                }
                match eval_stmt_list(body, env, functions, depth + 1, enforce_contracts)? {
                    EvalControl::Next => {}
                    EvalControl::Continue => continue,
                    EvalControl::Break => return Ok(EvalControl::Next),
                    EvalControl::Return(ret) => return Ok(EvalControl::Return(ret)),
                }
            }
            Err("while loop exceeded deterministic iteration budget".to_string())
        }
        Stmt::Repeat { count, body, .. } => {
            let times = eval_expr_with_ctx(
                count,
                env,
                functions,
                depth + 1,
                None,
                None,
                enforce_contracts,
            )?
            .as_int()?
            .max(0) as usize;
            for _ in 0..times.min(MAX_LOOP_ITERS) {
                match eval_stmt_list(body, env, functions, depth + 1, enforce_contracts)? {
                    EvalControl::Next => {}
                    EvalControl::Continue => continue,
                    EvalControl::Break => return Ok(EvalControl::Next),
                    EvalControl::Return(ret) => return Ok(EvalControl::Return(ret)),
                }
            }
            if times > MAX_LOOP_ITERS {
                return Err("repeat loop exceeded deterministic iteration budget".to_string());
            }
            Ok(EvalControl::Next)
        }
        Stmt::Select { .. } | Stmt::Go { .. } | Stmt::Thread { .. } => {
            Err("select/go are not supported in phase 2 deterministic example runner".to_string())
        }
        Stmt::Break { .. } => Ok(EvalControl::Break),
        Stmt::Continue { .. } => Ok(EvalControl::Continue),
        Stmt::Match { .. } => {
            Err("match is not supported in phase 2 deterministic example runner".to_string())
        }
    }
}

fn eval_binary(
    op: &BinaryOp,
    left: &DeterministicValue,
    right: &DeterministicValue,
) -> Result<DeterministicValue, String> {
    match op {
        BinaryOp::Add => match (left, right) {
            (DeterministicValue::Int(a), DeterministicValue::Int(b)) => {
                Ok(DeterministicValue::Int(a + b))
            }
            (DeterministicValue::Float(a), DeterministicValue::Float(b)) => {
                Ok(DeterministicValue::Float(a + b))
            }
            (DeterministicValue::Str(a), DeterministicValue::Str(b)) => {
                Ok(DeterministicValue::Str(format!("{a}{b}")))
            }
            _ => Err(format!(
                "unsupported Add operands `{}` and `{}`",
                deterministic_value_name(left),
                deterministic_value_name(right)
            )),
        },
        BinaryOp::Sub => match (left, right) {
            (DeterministicValue::Int(a), DeterministicValue::Int(b)) => {
                Ok(DeterministicValue::Int(a - b))
            }
            (DeterministicValue::Float(a), DeterministicValue::Float(b)) => {
                Ok(DeterministicValue::Float(a - b))
            }
            _ => Err("Sub expects numeric operands".to_string()),
        },
        BinaryOp::Mul => match (left, right) {
            (DeterministicValue::Int(a), DeterministicValue::Int(b)) => {
                Ok(DeterministicValue::Int(a * b))
            }
            (DeterministicValue::Float(a), DeterministicValue::Float(b)) => {
                Ok(DeterministicValue::Float(a * b))
            }
            _ => Err("Mul expects numeric operands".to_string()),
        },
        BinaryOp::Div => match (left, right) {
            (DeterministicValue::Int(a), DeterministicValue::Int(b)) => {
                Ok(DeterministicValue::Int(a / b))
            }
            (DeterministicValue::Float(a), DeterministicValue::Float(b)) => {
                Ok(DeterministicValue::Float(a / b))
            }
            _ => Err("Div expects numeric operands".to_string()),
        },
        BinaryOp::Eq => Ok(DeterministicValue::Bool(left == right)),
        BinaryOp::Ne => Ok(DeterministicValue::Bool(left != right)),
        BinaryOp::Lt => compare_numeric(left, right, |a, b| a < b),
        BinaryOp::Le => compare_numeric(left, right, |a, b| a <= b),
        BinaryOp::Gt => compare_numeric(left, right, |a, b| a > b),
        BinaryOp::Ge => compare_numeric(left, right, |a, b| a >= b),
        BinaryOp::And => Ok(DeterministicValue::Bool(left.as_bool()? && right.as_bool()?)),
        BinaryOp::Or => Ok(DeterministicValue::Bool(left.as_bool()? || right.as_bool()?)),
    }
}

fn compare_numeric(
    left: &DeterministicValue,
    right: &DeterministicValue,
    cmp: impl Fn(f64, f64) -> bool,
) -> Result<DeterministicValue, String> {
    let l = as_f64(left)?;
    let r = as_f64(right)?;
    Ok(DeterministicValue::Bool(cmp(l, r)))
}

fn as_f64(value: &DeterministicValue) -> Result<f64, String> {
    match value {
        DeterministicValue::Int(v) => Ok(*v as f64),
        DeterministicValue::Float(v) => Ok(*v),
        _ => Err(format!(
            "expected numeric value, got `{}`",
            deterministic_value_name(value)
        )),
    }
}

fn eval_unary(op: &UnaryOp, value: &DeterministicValue) -> Result<DeterministicValue, String> {
    match op {
        UnaryOp::Neg => match value {
            DeterministicValue::Int(v) => Ok(DeterministicValue::Int(-v)),
            DeterministicValue::Float(v) => Ok(DeterministicValue::Float(-v)),
            _ => Err("Neg expects Int or Float".to_string()),
        },
        UnaryOp::Not => Ok(DeterministicValue::Bool(!value.as_bool()?)),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use vibe_parser::parse_source;

    use super::run_examples;

    #[test]
    fn runs_topk_examples_from_fixture() {
        let src = fs::read_to_string(
            workspace_root().join("compiler/tests/fixtures/contract_ok/topk_contracts.vibe"),
        )
        .expect("read topk fixture");
        let parsed = parse_source(&src);
        assert!(
            !parsed.diagnostics.has_errors(),
            "parse should succeed: {}",
            parsed.diagnostics.to_golden()
        );
        let summary = run_examples(&parsed.ast);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 0);
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("resolve workspace root")
    }
}
