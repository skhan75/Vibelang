// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use vibe_ast::{Contract, Declaration, Expr, FileAst, SelectPattern, Stmt};
use vibe_diagnostics::{Diagnostic, Diagnostics, Severity, Span};
use vibe_parser::parse_source;
#[allow(clippy::single_component_path_imports)] // Ensures `vibe_pkg` is linked; required by project conventions.
use vibe_pkg;

pub struct CompilationUnit {
    pub source: String,
    pub ast: FileAst,
    pub diagnostics: Diagnostics,
    pub namespace_map: BTreeMap<(String, String), String>,
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

    let root_dir = crate::find_project_root(entry_path).unwrap_or_else(|| {
        entry_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });
    let root_dir = if root_dir.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        root_dir
    };
    if let Some(d) = check_vibelang_version(&root_dir) {
        diagnostics.push(d);
    }

    if entry_parsed.ast.module.is_none() && entry_parsed.ast.imports.is_empty() {
        let (ns_decls, namespace_map) = load_stdlib_namespace_functions();
        let mut decls = entry_parsed.ast.declarations;
        decls.extend(ns_decls);
        return Ok(CompilationUnit {
            source: entry_source,
            ast: FileAst {
                module: None,
                imports: Vec::new(),
                declarations: decls,
            },
            diagnostics,
            namespace_map,
        });
    }

    let canonical_root = root_dir.canonicalize().unwrap_or(root_dir.clone());
    let mut files = crate::collect_vibe_files(&root_dir)?;

    if let Some(stdlib_root) = find_stdlib_root() {
        if let Ok(stdlib_files) = crate::collect_vibe_files(&stdlib_root) {
            files.extend(stdlib_files);
        }
    }

    let manifest_opt = load_project_manifest_optional(&canonical_root);
    let lock_opt: Option<vibe_pkg::Lockfile> = match &manifest_opt {
        Some(_) => try_load_lockfile(&canonical_root)?,
        None => None,
    };
    let foreign_package_names: BTreeSet<String> = manifest_opt
        .as_ref()
        .map(|m| dependency_package_names(m, lock_opt.as_ref()))
        .unwrap_or_default();
    if let (Some(manifest), Some(store_root)) = (
        manifest_opt.as_ref(),
        find_package_store_root(&canonical_root),
    ) {
        let allowed = resolved_dependency_store_dirs(&store_root, manifest, lock_opt.as_ref())?;
        if !allowed.is_empty() {
            let pkg_files_all = collect_package_files(&store_root)?;
            let filtered: Vec<PathBuf> = pkg_files_all
                .into_iter()
                .filter(|p| {
                    package_name_version_under_store(&store_root, p)
                        .map(|nv| allowed.contains(&nv))
                        .unwrap_or(false)
                })
                .collect();
            files.extend(filtered);
        }
    }

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

    let stdlib_parent = find_stdlib_root()
        .and_then(|p| p.parent().map(|pp| pp.to_path_buf()))
        .and_then(|p| p.canonicalize().ok());

    let package_store_canonical = find_package_store_root(&canonical_root)
        .and_then(|p| p.canonicalize().ok());

    let mut module_index = BTreeMap::<String, usize>::new();
    for (idx, (_path, parsed)) in docs.iter().enumerate() {
        let Some(module_name) = &parsed.ast.module else {
            continue;
        };
        let expected =
            expected_module_name_from_path(&canonical_root, &docs[idx].0).or_else(|| {
                stdlib_parent
                    .as_ref()
                    .and_then(|sp| expected_module_name_from_path(sp, &docs[idx].0))
            }).or_else(|| {
                package_store_canonical
                    .as_ref()
                    .and_then(|ps| expected_package_module_name(ps, &docs[idx].0))
            });
        if let Some(expected_module) = expected {
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
            namespace_map: BTreeMap::new(),
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
            namespace_map: BTreeMap::new(),
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
        &foreign_package_names,
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
    let mut visible_types = BTreeSet::<String>::new();
    for module in &order {
        let Some(idx) = module_index.get(module).copied() else {
            continue;
        };
        let parsed = &docs[idx].1;
        for decl in &parsed.ast.declarations {
            match decl {
                Declaration::Function(func) => {
                    merged_decls.push(decl.clone());
                    if module == &entry_module || func.is_public {
                        visible_functions.insert(func.name.clone());
                    } else {
                        private_functions.insert(func.name.clone(), module.clone());
                    }
                }
                Declaration::Type(t) => {
                    if module == &entry_module || t.is_public {
                        if !visible_types.insert(t.name.clone()) {
                            diagnostics.push(Diagnostic::new(
                                "E2317",
                                Severity::Error,
                                format!(
                                    "duplicate type `{}` — already defined in another imported module",
                                    t.name
                                ),
                                t.span,
                            ));
                        }
                        merged_decls.push(decl.clone());
                    }
                }
                Declaration::Enum(e) => {
                    if module == &entry_module || e.is_public {
                        if !visible_types.insert(e.name.clone()) {
                            diagnostics.push(Diagnostic::new(
                                "E2317",
                                Severity::Error,
                                format!(
                                    "duplicate enum `{}` — already defined in another imported module",
                                    e.name
                                ),
                                e.span,
                            ));
                        }
                        merged_decls.push(decl.clone());
                    }
                }
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

    let (ns_decls, namespace_map) = load_stdlib_namespace_functions();
    let existing_fns: BTreeSet<String> = merged_decls
        .iter()
        .filter_map(|d| match d {
            Declaration::Function(f) => Some(f.name.clone()),
            _ => None,
        })
        .collect();
    for decl in ns_decls {
        if let Declaration::Function(ref f) = decl {
            if existing_fns.contains(&f.name) {
                continue;
            }
        }
        merged_decls.push(decl);
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
        namespace_map,
    })
}

#[allow(clippy::too_many_arguments)]
fn visit_module(
    module_name: &str,
    root_package: &str,
    foreign_packages: &BTreeSet<String>,
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
        if imported_package != root_package
            && imported_package != "std"
            && !foreign_packages.contains(imported_package)
        {
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
            foreign_packages,
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
        Contract::Intent { .. } | Contract::Effect { .. } | Contract::Native { .. } => {}
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

/// Returns `.yb/pkg/store/` under the project root when it exists as a directory.
pub(crate) fn find_package_store_root(project_root: &Path) -> Option<PathBuf> {
    let store = project_root.join(".yb").join("pkg").join("store");
    store.is_dir().then_some(store)
}

/// Walks `<store_root>/<name>/<version>/` trees and collects supported Vibe source files (e.g. `.yb`).
pub(crate) fn collect_package_files(store_root: &Path) -> Result<Vec<PathBuf>, String> {
    if !store_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<PathBuf> = fs::read_dir(store_root)
        .map_err(|e| format!("failed to read `{}`: {e}", store_root.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    entries.sort();
    let mut out = Vec::new();
    for name_path in entries {
        if !name_path.is_dir() {
            continue;
        }
        let mut vers: Vec<PathBuf> = fs::read_dir(&name_path)
            .map_err(|e| format!("failed to read `{}`: {e}", name_path.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        vers.sort();
        for ver_path in vers {
            if !ver_path.is_dir() {
                continue;
            }
            out.extend(crate::collect_vibe_files(&ver_path)?);
        }
    }
    out.sort();
    Ok(out)
}

/// Validates `[vibelang].version` from `vibe.toml` against this compiler build.
pub(crate) fn check_vibelang_version(project_root: &Path) -> Option<Diagnostic> {
    let manifest_path = project_root.join(vibe_pkg::MANIFEST_FILENAME);
    if !manifest_path.is_file() {
        return None;
    }
    let manifest = match vibe_pkg::load_manifest(&manifest_path) {
        Ok(m) => m,
        Err(msg) => {
            return Some(Diagnostic::new(
                "E2318",
                Severity::Error,
                msg,
                Span::new(1, 1, 1, 1),
            ));
        }
    };
    match manifest.check_compiler_version(env!("CARGO_PKG_VERSION")) {
        Ok(()) => None,
        Err(msg) => Some(Diagnostic::new(
            "E2318",
            Severity::Error,
            msg,
            Span::new(1, 1, 1, 1),
        )),
    }
}

fn load_project_manifest_optional(project_root: &Path) -> Option<vibe_pkg::Manifest> {
    let path = project_root.join(vibe_pkg::MANIFEST_FILENAME);
    vibe_pkg::load_manifest(&path).ok()
}

fn try_load_lockfile(project_root: &Path) -> Result<Option<vibe_pkg::Lockfile>, String> {
    let path = project_root.join(vibe_pkg::LOCK_FILENAME);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read `{}`: {e}", path.display()))?;
    toml::from_str(&raw).map_err(|e| format!("failed to parse `{}`: {e}", path.display()))
}

fn dependency_package_names(
    manifest: &vibe_pkg::Manifest,
    lock: Option<&vibe_pkg::Lockfile>,
) -> BTreeSet<String> {
    let root = &manifest.package.name;
    if let Some(lock) = lock {
        lock.package
            .iter()
            .filter(|p| &p.name != root)
            .map(|p| p.name.clone())
            .collect()
    } else {
        manifest.dependencies.keys().cloned().collect()
    }
}

fn resolved_dependency_store_dirs(
    store_root: &Path,
    manifest: &vibe_pkg::Manifest,
    lock: Option<&vibe_pkg::Lockfile>,
) -> Result<BTreeSet<(String, String)>, String> {
    let root = manifest.package.name.clone();
    if let Some(lock) = lock {
        return Ok(lock
            .package
            .iter()
            .filter(|p| p.name != root)
            .map(|p| (p.name.clone(), p.version.clone()))
            .collect());
    }
    let mut set = BTreeSet::new();
    for name in manifest.dependencies.keys() {
        let dir = store_root.join(name);
        if !dir.is_dir() {
            continue;
        }
        let mut vers: Vec<String> = fs::read_dir(&dir)
            .map_err(|e| format!("failed to read `{}`: {e}", dir.display()))?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let p = e.path();
                p.is_dir()
                    .then_some(e.file_name().to_str().map(str::to_string))
                    .flatten()
            })
            .collect();
        vers.sort();
        match vers.len() {
            0 => {}
            1 => {
                set.insert((name.clone(), vers[0].clone()));
            }
            _ => {
                return Err(format!(
                    "multiple installed versions of dependency `{name}` under `{}` without a lockfile; run `vibe install`",
                    dir.display()
                ));
            }
        }
    }
    Ok(set)
}

fn package_name_version_under_store(store_root: &Path, file: &Path) -> Option<(String, String)> {
    let rel = file.strip_prefix(store_root).ok()?;
    let mut c = rel
        .components()
        .filter_map(|comp| comp.as_os_str().to_str().map(str::to_string));
    let name = c.next()?;
    let version = c.next()?;
    Some((name, version))
}

fn expected_package_module_name(store_root: &Path, file: &Path) -> Option<String> {
    let rel = file.strip_prefix(store_root).ok()?;
    let parts: Vec<String> = rel
        .components()
        .filter_map(|c| c.as_os_str().to_str().map(str::to_string))
        .collect();
    if parts.len() < 3 {
        return None;
    }
    let pkg_name = parts[0].clone();
    let content_root = store_root.join(&parts[0]).join(&parts[1]);
    let inner = expected_module_name_from_path(&content_root, file)?;
    Some(format!("{pkg_name}.{inner}"))
}

fn is_native_only(func: &vibe_ast::FunctionDecl) -> bool {
    func.contracts
        .iter()
        .any(|c| matches!(c, Contract::Native { .. }))
        && func.body.is_empty()
        && func.tail_expr.is_none()
}

fn load_stdlib_namespace_functions() -> (Vec<Declaration>, BTreeMap<(String, String), String>) {
    let mut ns_decls = Vec::new();
    let mut ns_map = BTreeMap::new();

    let Some(stdlib_root) = find_stdlib_root() else {
        return (ns_decls, ns_map);
    };
    let Ok(files) = crate::collect_vibe_files(&stdlib_root) else {
        return (ns_decls, ns_map);
    };

    for file in files {
        let Ok(source) = fs::read_to_string(&file) else {
            continue;
        };
        let parsed = parse_source(&source);
        let Some(module_name) = &parsed.ast.module else {
            continue;
        };
        if !module_name.starts_with("std.") {
            continue;
        }
        let namespace = &module_name["std.".len()..];
        for decl in &parsed.ast.declarations {
            if let Declaration::Type(t) = decl {
                if t.is_public {
                    ns_decls.push(decl.clone());
                }
            }
            if let Declaration::Function(func) = decl {
                if is_native_only(func) {
                    let mangled = format!("__stdlib_{namespace}__{}", func.name);
                    let mut mangled_func = func.clone();
                    mangled_func.name = mangled.clone();
                    ns_decls.push(Declaration::Function(mangled_func));
                    if func.is_public {
                        ns_map.insert((namespace.to_string(), func.name.clone()), mangled);
                    }
                } else {
                    ns_decls.push(decl.clone());
                    if func.is_public {
                        ns_map.insert(
                            (namespace.to_string(), func.name.clone()),
                            func.name.clone(),
                        );
                    }
                }
            }
        }
    }
    (ns_decls, ns_map)
}

include!(concat!(env!("OUT_DIR"), "/embedded_stdlib.rs"));

fn find_stdlib_root() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("VIBE_STDLIB_PATH") {
        let path = PathBuf::from(p);
        if path.is_dir() {
            return Some(path);
        }
    }

    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;
    let mut cur = Some(exe_dir);
    for _ in 0..10 {
        let Some(dir) = cur else {
            break;
        };
        let candidate = dir.join("stdlib").join("std");
        if candidate.is_dir() {
            return Some(candidate);
        }
        cur = dir.parent();
    }

    if !EMBEDDED_STDLIB.is_empty() {
        if let Some(dir) = extract_embedded_stdlib() {
            return Some(dir);
        }
    }

    None
}

fn extract_embedded_stdlib() -> Option<PathBuf> {
    let cache_dir = std::env::temp_dir()
        .join("vibe-stdlib")
        .join(env!("CARGO_PKG_VERSION"));
    let cache_has_all_embedded = EMBEDDED_STDLIB
        .iter()
        .all(|(name, _)| cache_dir.join(name).is_file());
    if cache_has_all_embedded {
        return Some(cache_dir);
    }
    fs::create_dir_all(&cache_dir).ok()?;
    for (name, content) in EMBEDDED_STDLIB {
        fs::write(cache_dir.join(name), content).ok()?;
    }
    Some(cache_dir)
}
