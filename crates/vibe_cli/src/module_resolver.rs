use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use vibe_ast::{Contract, Declaration, Expr, FileAst, SelectPattern, Stmt};
use vibe_diagnostics::{Diagnostic, Diagnostics, Severity, Span};
use vibe_parser::parse_source;

pub struct CompilationUnit {
    pub source: String,
    pub ast: FileAst,
    pub diagnostics: Diagnostics,
}

struct ParsedSource {
    source: String,
    ast: FileAst,
}

pub fn resolve_compilation_unit(entry_path: &Path) -> Result<CompilationUnit, String> {
    let entry_source = fs::read_to_string(entry_path)
        .map_err(|e| format!("failed to read `{}`: {e}", entry_path.display()))?;
    let entry_parsed = parse_source(&entry_source);
    let mut diagnostics = Diagnostics::default();
    diagnostics.extend(entry_parsed.diagnostics.clone().into_sorted());

    if entry_parsed.ast.module.is_none() && entry_parsed.ast.imports.is_empty() {
        return Ok(CompilationUnit {
            source: entry_source,
            ast: entry_parsed.ast,
            diagnostics,
        });
    }

    let root_dir = crate::find_project_root(entry_path).unwrap_or_else(|| {
        entry_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });
    let canonical_root = root_dir.canonicalize().unwrap_or(root_dir.clone());
    let files = crate::collect_vibe_files(&root_dir)?;

    let mut docs = Vec::new();
    for file in files {
        let source = fs::read_to_string(&file)
            .map_err(|e| format!("failed to read `{}`: {e}", file.display()))?;
        let parsed = parse_source(&source);
        diagnostics.extend(parsed.diagnostics.into_sorted());
        docs.push((
            file.canonicalize().unwrap_or(file.clone()),
            ParsedSource {
                source,
                ast: parsed.ast,
            },
        ));
    }
    docs.sort_by(|a, b| a.0.cmp(&b.0));

    let mut module_index = BTreeMap::<String, usize>::new();
    for (idx, (_path, parsed)) in docs.iter().enumerate() {
        let Some(module_name) = &parsed.ast.module else {
            continue;
        };
        if let Some(expected_module) = expected_module_name_from_path(&canonical_root, &docs[idx].0)
        {
            if module_name != &expected_module {
                diagnostics.push(Diagnostic::new(
                    "E2316",
                    Severity::Error,
                    format!(
                        "module declaration `{module_name}` does not match file layout `{expected_module}` for `{}`",
                        docs[idx].0.display()
                    ),
                    module_span(&parsed.ast),
                ));
            }
        }
        if let Some(prev_idx) = module_index.get(module_name).copied() {
            let prev_path = &docs[prev_idx].0;
            diagnostics.push(Diagnostic::new(
                "E2310",
                Severity::Error,
                format!(
                    "duplicate module declaration `{module_name}` in `{}` and `{}`",
                    prev_path.display(),
                    docs[idx].0.display()
                ),
                module_span(&parsed.ast),
            ));
            continue;
        }
        module_index.insert(module_name.clone(), idx);
    }

    let Some(entry_module) = entry_parsed.ast.module.clone() else {
        diagnostics.push(Diagnostic::new(
            "E2315",
            Severity::Error,
            "imports require an explicit `module` declaration",
            module_span(&entry_parsed.ast),
        ));
        return Ok(CompilationUnit {
            source: entry_source,
            ast: entry_parsed.ast,
            diagnostics,
        });
    };

    let Some(entry_idx) = module_index.get(&entry_module).copied() else {
        diagnostics.push(Diagnostic::new(
            "E2311",
            Severity::Error,
            format!("entry module `{entry_module}` could not be resolved"),
            module_span(&entry_parsed.ast),
        ));
        return Ok(CompilationUnit {
            source: entry_source,
            ast: entry_parsed.ast,
            diagnostics,
        });
    };

    let root_package = entry_module
        .split('.')
        .next()
        .unwrap_or(entry_module.as_str())
        .to_string();
    let mut visited = BTreeSet::new();
    let mut stack = Vec::<String>::new();
    let mut order = Vec::<String>::new();
    visit_module(
        &entry_module,
        &root_package,
        &module_index,
        &docs,
        &mut visited,
        &mut stack,
        &mut order,
        &mut diagnostics,
    );

    let mut merged_decls = Vec::new();
    let mut visible_functions = BTreeSet::<String>::new();
    let mut private_functions = BTreeMap::<String, String>::new();
    for module in &order {
        let Some(idx) = module_index.get(module).copied() else {
            continue;
        };
        let parsed = &docs[idx].1;
        for decl in &parsed.ast.declarations {
            let Declaration::Function(func) = decl else {
                continue;
            };
            if module == &entry_module || func.is_public {
                merged_decls.push(decl.clone());
                visible_functions.insert(func.name.clone());
            } else {
                private_functions.insert(func.name.clone(), module.clone());
            }
        }
    }

    let entry_ast = &docs[entry_idx].1.ast;
    let mut called_functions = BTreeSet::new();
    collect_called_functions(entry_ast, &mut called_functions);
    for name in called_functions {
        if visible_functions.contains(&name) {
            continue;
        }
        if let Some(owner) = private_functions.get(&name) {
            diagnostics.push(Diagnostic::new(
                "E2314",
                Severity::Error,
                format!(
                    "function `{name}` from module `{owner}` is private; mark it `pub` to import"
                ),
                module_span(entry_ast),
            ));
        }
    }

    let mut merged_source = String::new();
    for module in &order {
        if let Some(idx) = module_index.get(module).copied() {
            merged_source.push_str(&format!("// module: {module}\n"));
            merged_source.push_str(&docs[idx].1.source);
            merged_source.push('\n');
        }
    }

    Ok(CompilationUnit {
        source: merged_source,
        ast: FileAst {
            module: Some(entry_module),
            imports: entry_ast.imports.clone(),
            declarations: merged_decls,
        },
        diagnostics,
    })
}

#[allow(clippy::too_many_arguments)]
fn visit_module(
    module_name: &str,
    root_package: &str,
    module_index: &BTreeMap<String, usize>,
    docs: &[(PathBuf, ParsedSource)],
    visited: &mut BTreeSet<String>,
    stack: &mut Vec<String>,
    order: &mut Vec<String>,
    diagnostics: &mut Diagnostics,
) {
    if visited.contains(module_name) {
        return;
    }
    if let Some(pos) = stack.iter().position(|m| m == module_name) {
        let mut cycle = stack[pos..].to_vec();
        cycle.push(module_name.to_string());
        diagnostics.push(Diagnostic::new(
            "E2312",
            Severity::Error,
            format!("import cycle detected: {}", cycle.join(" -> ")),
            Span::new(1, 1, 1, 1),
        ));
        return;
    }
    let Some(idx) = module_index.get(module_name).copied() else {
        diagnostics.push(Diagnostic::new(
            "E2311",
            Severity::Error,
            format!("module `{module_name}` was not found in project sources"),
            Span::new(1, 1, 1, 1),
        ));
        return;
    };

    stack.push(module_name.to_string());
    let imports = docs[idx].1.ast.imports.clone();
    for import in imports {
        let imported_package = import.split('.').next().unwrap_or(import.as_str());
        if imported_package != root_package {
            diagnostics.push(Diagnostic::new(
                "E2313",
                Severity::Error,
                format!(
                    "cross-package import `{import}` is not allowed from package `{root_package}`"
                ),
                module_span(&docs[idx].1.ast),
            ));
            continue;
        }
        if !module_index.contains_key(&import) {
            diagnostics.push(Diagnostic::new(
                "E2311",
                Severity::Error,
                format!("imported module `{import}` was not found"),
                module_span(&docs[idx].1.ast),
            ));
            continue;
        }
        visit_module(
            &import,
            root_package,
            module_index,
            docs,
            visited,
            stack,
            order,
            diagnostics,
        );
    }
    stack.pop();
    visited.insert(module_name.to_string());
    order.push(module_name.to_string());
}

fn collect_called_functions(ast: &FileAst, out: &mut BTreeSet<String>) {
    for decl in &ast.declarations {
        let Declaration::Function(func) = decl else {
            continue;
        };
        for contract in &func.contracts {
            collect_calls_from_contract(contract, out);
        }
        for stmt in &func.body {
            collect_calls_from_stmt(stmt, out);
        }
        if let Some(expr) = &func.tail_expr {
            collect_calls_from_expr(expr, out);
        }
    }
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
        Stmt::Binding { expr, .. }
        | Stmt::ExprStmt { expr, .. }
        | Stmt::Go { expr, .. }
        | Stmt::Thread { expr, .. } => collect_calls_from_expr(expr, out),
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
                if let SelectPattern::Receive { expr, .. } = &case.pattern {
                    collect_calls_from_expr(expr, out);
                }
                collect_calls_from_expr(&case.action, out);
            }
        }
        Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::Match {
            scrutinee,
            arms,
            default_action,
            ..
        } => {
            collect_calls_from_expr(scrutinee, out);
            for arm in arms {
                collect_calls_from_expr(&arm.pattern, out);
                collect_calls_from_expr(&arm.action, out);
            }
            if let Some(e) = default_action {
                collect_calls_from_expr(e, out);
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
        | Expr::Async { expr: object, .. }
        | Expr::Await { expr: object, .. }
        | Expr::Question { expr: object, .. }
        | Expr::Old { expr: object, .. } => {
            collect_calls_from_expr(object, out);
        }
        Expr::Index { object, index, .. } => {
            collect_calls_from_expr(object, out);
            collect_calls_from_expr(index, out);
        }
        Expr::Slice {
            object, start, end, ..
        } => {
            collect_calls_from_expr(object, out);
            if let Some(start) = start {
                collect_calls_from_expr(start, out);
            }
            if let Some(end) = end {
                collect_calls_from_expr(end, out);
            }
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
        Expr::Constructor { fields, .. } => {
            for (_, e) in fields {
                collect_calls_from_expr(e, out);
            }
        }
        Expr::Ident { .. }
        | Expr::Int { .. }
        | Expr::Float { .. }
        | Expr::Bool { .. }
        | Expr::String { .. }
        | Expr::DotResult { .. }
        | Expr::EnumVariant { .. } => {}
    }
}

fn module_span(ast: &FileAst) -> Span {
    ast.declarations
        .first()
        .map(|decl| match decl {
            Declaration::Function(func) => func.span,
            Declaration::Type(t) => t.span,
            Declaration::Enum(e) => e.span,
        })
        .unwrap_or_else(|| Span::new(1, 1, 1, 1))
}

fn expected_module_name_from_path(root: &Path, source: &Path) -> Option<String> {
    let rel = source.strip_prefix(root).ok()?;
    let without_ext = rel.with_extension("");
    let parts = without_ext
        .components()
        .filter_map(|c| {
            let raw = c.as_os_str().to_str()?;
            if raw.is_empty() {
                return None;
            }
            Some(raw.to_string())
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("."))
}
