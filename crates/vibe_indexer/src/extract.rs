use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use sha2::{Digest, Sha256};
use vibe_ast::{Contract, Declaration, Expr, FileAst, FunctionDecl, SelectPattern, Stmt};
use vibe_diagnostics::Diagnostic;
use vibe_hir::HirProgram;

use crate::model::{
    EffectMismatch, FileIndex, FunctionMeta, IndexSpan, IndexedDiagnostic, IndexedSeverity,
    Reference, Symbol, SymbolId, SymbolKind,
};

pub fn build_file_index(
    file_path: &Path,
    source: &str,
    ast: &FileAst,
    hir: &HirProgram,
    diagnostics: &[Diagnostic],
) -> FileIndex {
    let file = file_path.to_string_lossy().to_string();
    let module = ast.module.clone();
    let mut symbols = Vec::new();
    let mut references = Vec::new();
    let mut function_meta = Vec::new();
    let mut effect_mismatches = Vec::new();
    let mut dependencies = BTreeSet::new();
    dependencies.extend(ast.imports.iter().cloned());

    let mut function_symbol_ids = BTreeMap::new();
    let mut declared_functions = Vec::new();
    for (idx, decl) in ast.declarations.iter().enumerate() {
        let Declaration::Function(func) = decl else {
            continue;
        };
        let symbol_id = SymbolId(stable_symbol_id(
            &file,
            &func.name,
            SymbolKind::Function,
            "decl",
            idx as u64,
        ));
        function_symbol_ids.insert(func.name.clone(), symbol_id);
        declared_functions.push(func.clone());
        symbols.push(Symbol {
            id: symbol_id,
            name: func.name.clone(),
            kind: SymbolKind::Function,
            module: module.clone(),
            file: file.clone(),
            span: IndexSpan::from(func.span),
        });
    }

    for func in declared_functions {
        let function_symbol_id = *function_symbol_ids
            .get(&func.name)
            .expect("function symbol id should exist");
        index_function(
            &file,
            &module,
            &func,
            &function_symbol_ids,
            &mut symbols,
            &mut references,
            &mut dependencies,
        );
        function_meta.push(build_function_meta(&file, &func, function_symbol_id, hir));
        if let Some(hir_fn) = hir.functions.iter().find(|f| f.name == func.name) {
            let declared_only = hir_fn
                .effects_declared
                .difference(&hir_fn.effects_observed)
                .cloned()
                .collect::<Vec<_>>();
            let observed_only = hir_fn
                .effects_observed
                .difference(&hir_fn.effects_declared)
                .cloned()
                .collect::<Vec<_>>();
            if !declared_only.is_empty() || !observed_only.is_empty() {
                effect_mismatches.push(EffectMismatch {
                    function_name: func.name.clone(),
                    file: file.clone(),
                    declared_only,
                    observed_only,
                });
            }
        }
    }

    let mut file_index = FileIndex {
        file: file.clone(),
        file_hash: stable_hash_hex(source),
        symbols,
        references,
        function_meta,
        effect_mismatches,
        diagnostics: diagnostics
            .iter()
            .map(|d| IndexedDiagnostic {
                code: d.code.clone(),
                severity: IndexedSeverity::from(d.severity),
                message: d.message.clone(),
                span: IndexSpan::from(d.span),
            })
            .collect(),
        dependencies: dependencies.into_iter().collect(),
    };
    file_index.normalize();
    file_index
}

pub fn stable_hash_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn stable_symbol_id(file: &str, name: &str, kind: SymbolKind, group: &str, ordinal: u64) -> u64 {
    let text = format!("{file}|{name}|{kind:?}|{group}|{ordinal}");
    let hash = stable_hash_hex(&text);
    u64::from_str_radix(&hash[..16], 16).unwrap_or(0)
}

#[allow(clippy::too_many_arguments)]
fn index_function(
    file: &str,
    module: &Option<String>,
    func: &FunctionDecl,
    function_symbol_ids: &BTreeMap<String, SymbolId>,
    symbols: &mut Vec<Symbol>,
    references: &mut Vec<Reference>,
    dependencies: &mut BTreeSet<String>,
) {
    let mut locals = BTreeMap::<String, SymbolId>::new();
    for (idx, param) in func.params.iter().enumerate() {
        let symbol_id = SymbolId(stable_symbol_id(
            file,
            &format!("{}::{}", func.name, param.name),
            SymbolKind::Param,
            "param",
            idx as u64,
        ));
        locals.insert(param.name.clone(), symbol_id);
        symbols.push(Symbol {
            id: symbol_id,
            name: param.name.clone(),
            kind: SymbolKind::Param,
            module: module.clone(),
            file: file.to_string(),
            span: IndexSpan::from(func.span),
        });
    }

    let mut local_seq = 0u64;
    for (idx, contract) in func.contracts.iter().enumerate() {
        match contract {
            Contract::Intent { text, span } => {
                symbols.push(Symbol {
                    id: SymbolId(stable_symbol_id(
                        file,
                        &format!("{}::@intent", func.name),
                        SymbolKind::Contract,
                        "intent",
                        idx as u64,
                    )),
                    name: text.clone(),
                    kind: SymbolKind::Contract,
                    module: module.clone(),
                    file: file.to_string(),
                    span: IndexSpan::from(*span),
                });
            }
            Contract::Examples { cases, span } => {
                symbols.push(Symbol {
                    id: SymbolId(stable_symbol_id(
                        file,
                        &format!("{}::@examples", func.name),
                        SymbolKind::Contract,
                        "examples",
                        idx as u64,
                    )),
                    name: "@examples".to_string(),
                    kind: SymbolKind::Contract,
                    module: module.clone(),
                    file: file.to_string(),
                    span: IndexSpan::from(*span),
                });
                for case in cases {
                    collect_expr_refs(
                        &case.call,
                        file,
                        &locals,
                        function_symbol_ids,
                        references,
                        dependencies,
                    );
                    collect_expr_refs(
                        &case.expected,
                        file,
                        &locals,
                        function_symbol_ids,
                        references,
                        dependencies,
                    );
                }
            }
            Contract::Require { expr, span } | Contract::Ensure { expr, span } => {
                symbols.push(Symbol {
                    id: SymbolId(stable_symbol_id(
                        file,
                        &format!("{}::@contract", func.name),
                        SymbolKind::Contract,
                        "prepost",
                        idx as u64,
                    )),
                    name: "@contract".to_string(),
                    kind: SymbolKind::Contract,
                    module: module.clone(),
                    file: file.to_string(),
                    span: IndexSpan::from(*span),
                });
                collect_expr_refs(
                    expr,
                    file,
                    &locals,
                    function_symbol_ids,
                    references,
                    dependencies,
                );
            }
            Contract::Effect { name, span } => {
                symbols.push(Symbol {
                    id: SymbolId(stable_symbol_id(
                        file,
                        &format!("{}::@effect::{name}", func.name),
                        SymbolKind::Effect,
                        "effect",
                        idx as u64,
                    )),
                    name: name.clone(),
                    kind: SymbolKind::Effect,
                    module: module.clone(),
                    file: file.to_string(),
                    span: IndexSpan::from(*span),
                });
            }
        }
    }

    for stmt in &func.body {
        collect_stmt_refs(
            stmt,
            file,
            module,
            func,
            &mut locals,
            &mut local_seq,
            function_symbol_ids,
            symbols,
            references,
            dependencies,
        );
    }
    if let Some(tail) = &func.tail_expr {
        collect_expr_refs(
            tail,
            file,
            &locals,
            function_symbol_ids,
            references,
            dependencies,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_stmt_refs(
    stmt: &Stmt,
    file: &str,
    module: &Option<String>,
    func: &FunctionDecl,
    locals: &mut BTreeMap<String, SymbolId>,
    local_seq: &mut u64,
    function_symbol_ids: &BTreeMap<String, SymbolId>,
    symbols: &mut Vec<Symbol>,
    references: &mut Vec<Reference>,
    dependencies: &mut BTreeSet<String>,
) {
    match stmt {
        Stmt::Binding { name, expr, span } => {
            collect_expr_refs(
                expr,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
            let symbol_id = SymbolId(stable_symbol_id(
                file,
                &format!("{}::{}", func.name, name),
                SymbolKind::Local,
                "binding",
                *local_seq,
            ));
            *local_seq += 1;
            locals.insert(name.clone(), symbol_id);
            symbols.push(Symbol {
                id: symbol_id,
                name: name.clone(),
                kind: SymbolKind::Local,
                module: module.clone(),
                file: file.to_string(),
                span: IndexSpan::from(*span),
            });
        }
        Stmt::Assignment { target, expr, .. } => {
            collect_expr_refs(
                target,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
            collect_expr_refs(
                expr,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
        }
        Stmt::Return { expr, .. }
        | Stmt::ExprStmt { expr, .. }
        | Stmt::Go { expr, .. }
        | Stmt::Thread { expr, .. } => {
            collect_expr_refs(
                expr,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
        }
        Stmt::For {
            var,
            iter,
            body,
            span,
            ..
        } => {
            collect_expr_refs(
                iter,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
            let symbol_id = SymbolId(stable_symbol_id(
                file,
                &format!("{}::{}", func.name, var),
                SymbolKind::Local,
                "for-var",
                *local_seq,
            ));
            *local_seq += 1;
            locals.insert(var.clone(), symbol_id);
            symbols.push(Symbol {
                id: symbol_id,
                name: var.clone(),
                kind: SymbolKind::Local,
                module: module.clone(),
                file: file.to_string(),
                span: IndexSpan::from(*span),
            });
            for child in body {
                collect_stmt_refs(
                    child,
                    file,
                    module,
                    func,
                    locals,
                    local_seq,
                    function_symbol_ids,
                    symbols,
                    references,
                    dependencies,
                );
            }
        }
        Stmt::If {
            cond,
            then_body,
            else_body,
            ..
        } => {
            collect_expr_refs(
                cond,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
            for child in then_body {
                collect_stmt_refs(
                    child,
                    file,
                    module,
                    func,
                    locals,
                    local_seq,
                    function_symbol_ids,
                    symbols,
                    references,
                    dependencies,
                );
            }
            for child in else_body {
                collect_stmt_refs(
                    child,
                    file,
                    module,
                    func,
                    locals,
                    local_seq,
                    function_symbol_ids,
                    symbols,
                    references,
                    dependencies,
                );
            }
        }
        Stmt::While { cond, body, .. } => {
            collect_expr_refs(
                cond,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
            for child in body {
                collect_stmt_refs(
                    child,
                    file,
                    module,
                    func,
                    locals,
                    local_seq,
                    function_symbol_ids,
                    symbols,
                    references,
                    dependencies,
                );
            }
        }
        Stmt::Repeat { count, body, .. } => {
            collect_expr_refs(
                count,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
            for child in body {
                collect_stmt_refs(
                    child,
                    file,
                    module,
                    func,
                    locals,
                    local_seq,
                    function_symbol_ids,
                    symbols,
                    references,
                    dependencies,
                );
            }
        }
        Stmt::Select { cases, .. } => {
            for case in cases {
                match &case.pattern {
                    SelectPattern::Receive { binding, expr } => {
                        collect_expr_refs(
                            expr,
                            file,
                            locals,
                            function_symbol_ids,
                            references,
                            dependencies,
                        );
                        let symbol_id = SymbolId(stable_symbol_id(
                            file,
                            &format!("{}::{}", func.name, binding),
                            SymbolKind::Local,
                            "select-binding",
                            *local_seq,
                        ));
                        *local_seq += 1;
                        locals.insert(binding.clone(), symbol_id);
                        symbols.push(Symbol {
                            id: symbol_id,
                            name: binding.clone(),
                            kind: SymbolKind::Local,
                            module: module.clone(),
                            file: file.to_string(),
                            span: IndexSpan::from(case.span),
                        });
                    }
                    SelectPattern::After { .. } | SelectPattern::Default => {}
                    SelectPattern::Closed { ident } => {
                        if let Some(symbol_id) = locals.get(ident) {
                            references.push(Reference {
                                symbol_id: *symbol_id,
                                file: file.to_string(),
                                span: IndexSpan::from(case.span),
                            });
                        }
                    }
                }
                collect_expr_refs(
                    &case.action,
                    file,
                    locals,
                    function_symbol_ids,
                    references,
                    dependencies,
                );
            }
        }
        Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::Match {
            scrutinee, arms, default_action, ..
        } => {
            collect_expr_refs(scrutinee, file, locals, function_symbol_ids, references, dependencies);
            for arm in arms {
                collect_expr_refs(
                    &arm.pattern,
                    file,
                    locals,
                    function_symbol_ids,
                    references,
                    dependencies,
                );
                collect_expr_refs(
                    &arm.action,
                    file,
                    locals,
                    function_symbol_ids,
                    references,
                    dependencies,
                );
            }
            if let Some(e) = default_action {
                collect_expr_refs(
                    e,
                    file,
                    locals,
                    function_symbol_ids,
                    references,
                    dependencies,
                );
            }
        }
    }
}

fn collect_expr_refs(
    expr: &Expr,
    file: &str,
    locals: &BTreeMap<String, SymbolId>,
    function_symbol_ids: &BTreeMap<String, SymbolId>,
    references: &mut Vec<Reference>,
    dependencies: &mut BTreeSet<String>,
) {
    match expr {
        Expr::Ident { name, span } => {
            if let Some(symbol_id) = locals.get(name).or_else(|| function_symbol_ids.get(name)) {
                references.push(Reference {
                    symbol_id: *symbol_id,
                    file: file.to_string(),
                    span: IndexSpan::from(*span),
                });
            } else if !is_builtin_ident(name) {
                dependencies.insert(name.clone());
            }
        }
        Expr::Call { callee, args, .. } => {
            collect_expr_refs(
                callee,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
            for arg in args {
                collect_expr_refs(
                    arg,
                    file,
                    locals,
                    function_symbol_ids,
                    references,
                    dependencies,
                );
            }
        }
        Expr::Member { object, .. }
        | Expr::Unary { expr: object, .. }
        | Expr::Async { expr: object, .. }
        | Expr::Await { expr: object, .. }
        | Expr::Question { expr: object, .. }
        | Expr::Old { expr: object, .. } => {
            collect_expr_refs(
                object,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
        }
        Expr::Index { object, index, .. } => {
            collect_expr_refs(
                object,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
            collect_expr_refs(
                index,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
        }
        Expr::Slice {
            object, start, end, ..
        } => {
            collect_expr_refs(
                object,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
            if let Some(start) = start {
                collect_expr_refs(
                    start,
                    file,
                    locals,
                    function_symbol_ids,
                    references,
                    dependencies,
                );
            }
            if let Some(end) = end {
                collect_expr_refs(
                    end,
                    file,
                    locals,
                    function_symbol_ids,
                    references,
                    dependencies,
                );
            }
        }
        Expr::Binary { left, right, .. } => {
            collect_expr_refs(
                left,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
            collect_expr_refs(
                right,
                file,
                locals,
                function_symbol_ids,
                references,
                dependencies,
            );
        }
        Expr::List { items, .. } => {
            for item in items {
                collect_expr_refs(
                    item,
                    file,
                    locals,
                    function_symbol_ids,
                    references,
                    dependencies,
                );
            }
        }
        Expr::Map { entries, .. } => {
            for (key, value) in entries {
                collect_expr_refs(
                    key,
                    file,
                    locals,
                    function_symbol_ids,
                    references,
                    dependencies,
                );
                collect_expr_refs(
                    value,
                    file,
                    locals,
                    function_symbol_ids,
                    references,
                    dependencies,
                );
            }
        }
        Expr::Constructor { fields, .. } => {
            for (_, e) in fields {
                collect_expr_refs(
                    e,
                    file,
                    locals,
                    function_symbol_ids,
                    references,
                    dependencies,
                );
            }
        }
        Expr::Int { .. }
        | Expr::Float { .. }
        | Expr::Bool { .. }
        | Expr::String { .. }
        | Expr::DotResult { .. }
        | Expr::EnumVariant { .. } => {}
    }
}

fn build_function_meta(
    file: &str,
    func: &FunctionDecl,
    symbol_id: SymbolId,
    hir: &HirProgram,
) -> FunctionMeta {
    let signature = format!(
        "{}({})->{}",
        func.name,
        func.params
            .iter()
            .map(|p| format!(
                "{}:{}",
                p.name,
                p.ty.as_ref().map(|t| t.raw.as_str()).unwrap_or("Unknown")
            ))
            .collect::<Vec<_>>()
            .join(","),
        func.return_type
            .as_ref()
            .map(|t| t.raw.as_str())
            .unwrap_or("Unknown")
    );
    let signature_hash = stable_hash_hex(&signature);

    let intent_text = func.contracts.iter().find_map(|c| match c {
        Contract::Intent { text, .. } => Some(text.clone()),
        _ => None,
    });
    let has_examples = func
        .contracts
        .iter()
        .any(|c| matches!(c, Contract::Examples { .. }));

    let (effects_declared, effects_observed) = hir
        .functions
        .iter()
        .find(|f| f.name == func.name)
        .map(|f| {
            (
                f.effects_declared.iter().cloned().collect::<Vec<_>>(),
                f.effects_observed.iter().cloned().collect::<Vec<_>>(),
            )
        })
        .unwrap_or_default();

    FunctionMeta {
        symbol_id,
        function_name: func.name.clone(),
        file: file.to_string(),
        signature_hash,
        effects_declared,
        effects_observed,
        intent_text,
        has_examples,
        is_public: func.is_public,
    }
}

fn is_builtin_ident(name: &str) -> bool {
    matches!(
        name,
        "len"
            | "min"
            | "max"
            | "sorted_desc"
            | "sort_desc"
            | "take"
            | "cpu_count"
            | "chan"
            | "ok"
            | "err"
            | "print"
            | "println"
            | "true"
            | "false"
    )
}
