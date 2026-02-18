mod deterministic_utils;
mod example_runner;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::{env, fs};

use vibe_codegen::{emit_object, CodegenOptions};
use vibe_diagnostics::Diagnostic;
use vibe_diagnostics::Diagnostics;
use vibe_indexer::build_file_index;
use vibe_indexer::{default_index_root, IncrementalIndexer, IncrementalTelemetry, IndexStats, IndexStore};
use vibe_lsp::run_line_stdio;
use vibe_mir::MirProgram;
use vibe_mir::{lower_hir_to_mir, mir_debug_dump};
use vibe_parser::parse_source;
use vibe_runtime::{compile_runtime_object, link_executable, RuntimeBuildOptions};
use vibe_types::check_and_lower;

use crate::example_runner::{run_examples, ExampleRunSummary};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return Err(usage());
    }
    let cmd = args.remove(0);
    match cmd.as_str() {
        "check" => {
            let path = args.first().ok_or_else(usage)?;
            run_check(path)
        }
        "ast" => {
            let path = args.first().ok_or_else(usage)?;
            run_ast(path)
        }
        "hir" => {
            let path = args.first().ok_or_else(usage)?;
            run_hir(path)
        }
        "mir" => {
            let path = args.first().ok_or_else(usage)?;
            run_mir(path)
        }
        "build" => {
            let build_args = parse_build_like_args(&args, false)?;
            let artifacts = build_source(&build_args)?;
            println!(
                "built {} (object: {}, runtime: {}, debug-map: {})",
                artifacts.binary_path.display(),
                artifacts.object_path.display(),
                artifacts.runtime_object_path.display(),
                artifacts.debug_map_path.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        "run" => {
            let build_args = parse_build_like_args(&args, true)?;
            let artifacts = build_source(&build_args)?;
            let status = Command::new(&artifacts.binary_path)
                .args(&build_args.exec_args)
                .status()
                .map_err(|e| {
                    format!(
                        "failed to execute binary `{}`: {e}",
                        artifacts.binary_path.display()
                    )
                })?;
            Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
        }
        "test" => {
            let test_args = parse_build_like_args(&args, false)?;
            run_test(&test_args)
        }
        "index" => {
            let index_args = parse_index_args(&args)?;
            run_index(&index_args)
        }
        "lsp" => run_lsp(&args),
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage: vibe <check|ast|hir|mir|build|run|test|index|lsp> <path> [--profile dev|release] [--target x86_64-unknown-linux-gnu] [--offline] [--debuginfo none|line|full]".to_string()
}

fn run_check(path: &str) -> Result<ExitCode, String> {
    let src = fs::read_to_string(path).map_err(|e| format!("failed to read `{path}`: {e}"))?;
    let parsed = parse_source(&src);
    let checked = check_and_lower(&parsed.ast);
    let mut merged_diags = parsed.diagnostics.clone().into_sorted();
    merged_diags.extend(checked.diagnostics.clone().into_sorted());
    let mut all = Diagnostics::default();
    all.extend(merged_diags.clone());
    if let Err(err) = best_effort_refresh_index(
        Path::new(path),
        &src,
        &parsed.ast,
        &checked.hir,
        &merged_diags,
    ) {
        eprintln!("index refresh skipped: {err}");
    }
    let out = all.to_golden();
    if out.trim().is_empty() {
        println!("OK");
    } else {
        print!("{out}");
    }
    Ok(if all.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn run_ast(path: &str) -> Result<ExitCode, String> {
    let src = fs::read_to_string(path).map_err(|e| format!("failed to read `{path}`: {e}"))?;
    let parsed = parse_source(&src);
    println!("{:#?}", parsed.ast);
    let out = parsed.diagnostics.to_golden();
    if !out.trim().is_empty() {
        eprintln!("{out}");
    }
    Ok(if parsed.diagnostics.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn run_hir(path: &str) -> Result<ExitCode, String> {
    let src = fs::read_to_string(path).map_err(|e| format!("failed to read `{path}`: {e}"))?;
    let parsed = parse_source(&src);
    let checked = check_and_lower(&parsed.ast);
    println!("{:#?}", checked.hir);
    let mut all = Diagnostics::default();
    all.extend(parsed.diagnostics.into_sorted());
    all.extend(checked.diagnostics.into_sorted());
    let out = all.to_golden();
    if !out.trim().is_empty() {
        eprintln!("{out}");
    }
    Ok(if all.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn run_mir(path: &str) -> Result<ExitCode, String> {
    let src = fs::read_to_string(path).map_err(|e| format!("failed to read `{path}`: {e}"))?;
    let parsed = parse_source(&src);
    let checked = check_and_lower(&parsed.ast);
    let mut all = Diagnostics::default();
    all.extend(parsed.diagnostics.clone().into_sorted());
    all.extend(checked.diagnostics.clone().into_sorted());
    if all.has_errors() {
        let out = all.to_golden();
        if !out.trim().is_empty() {
            eprintln!("{out}");
        }
        return Ok(ExitCode::from(1));
    }
    let mir =
        lower_hir_to_mir(&checked.hir).map_err(|e| format!("HIR->MIR lowering failed: {e}"))?;
    print!("{}", mir_debug_dump(&mir));
    Ok(ExitCode::SUCCESS)
}

#[derive(Debug, Clone)]
struct BuildArgs {
    source_path: PathBuf,
    profile: String,
    target: String,
    debuginfo: String,
    offline: bool,
    exec_args: Vec<String>,
}

#[derive(Debug, Clone)]
struct BuildArtifacts {
    object_path: PathBuf,
    runtime_object_path: PathBuf,
    debug_map_path: PathBuf,
    binary_path: PathBuf,
}

#[derive(Debug, Clone)]
struct IndexArgs {
    target_path: PathBuf,
    rebuild: bool,
    stats: bool,
}

fn parse_build_like_args(args: &[String], allow_exec_args: bool) -> Result<BuildArgs, String> {
    if args.is_empty() {
        return Err("missing source path".to_string());
    }
    let mut idx = 0usize;
    let source_path = PathBuf::from(&args[idx]);
    idx += 1;

    let mut profile = "dev".to_string();
    let mut target = "x86_64-unknown-linux-gnu".to_string();
    let mut debuginfo = "line".to_string();
    let mut offline = false;
    let mut exec_args = Vec::new();

    while idx < args.len() {
        let arg = &args[idx];
        if arg == "--" {
            if allow_exec_args {
                exec_args.extend_from_slice(&args[idx + 1..]);
                break;
            }
            return Err("`--` is only valid for `vibe run`".to_string());
        }
        match arg.as_str() {
            "--profile" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--profile`".to_string())?;
                if val != "dev" && val != "release" {
                    return Err(format!(
                        "unsupported profile `{val}` (expected dev|release)"
                    ));
                }
                profile = val.clone();
            }
            "--target" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--target`".to_string())?;
                target = val.clone();
            }
            "--debuginfo" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--debuginfo`".to_string())?;
                if !matches!(val.as_str(), "none" | "line" | "full") {
                    return Err(format!(
                        "unsupported debuginfo `{val}` (expected none|line|full)"
                    ));
                }
                debuginfo = val.clone();
            }
            "--offline" => {
                offline = true;
            }
            other => {
                return Err(format!("unknown argument `{other}`"));
            }
        }
        idx += 1;
    }

    Ok(BuildArgs {
        source_path,
        profile,
        target,
        debuginfo,
        offline,
        exec_args,
    })
}

fn parse_index_args(args: &[String]) -> Result<IndexArgs, String> {
    let mut idx = 0usize;
    let mut target_path = PathBuf::from(".");
    let mut rebuild = false;
    let mut stats = false;

    if let Some(first) = args.first() {
        if !first.starts_with("--") {
            target_path = PathBuf::from(first);
            idx = 1;
        }
    }

    while idx < args.len() {
        match args[idx].as_str() {
            "--rebuild" => rebuild = true,
            "--stats" => stats = true,
            "--path" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--path`".to_string())?;
                target_path = PathBuf::from(val);
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
        idx += 1;
    }

    Ok(IndexArgs {
        target_path,
        rebuild,
        stats,
    })
}

fn run_index(args: &IndexArgs) -> Result<ExitCode, String> {
    let files = collect_vibe_files(&args.target_path)?;
    if files.is_empty() {
        return Err(format!(
            "no .vibe files found under `{}`",
            args.target_path.display()
        ));
    }

    let index_root = default_index_root(&args.target_path);
    let mut store = IndexStore::open_or_create(&index_root)?;
    if args.rebuild {
        store.clear();
    }
    let mut incremental = IncrementalIndexer::new(store);
    let cold_start = std::time::Instant::now();
    let mut telemetry = IncrementalTelemetry::default();

    for file in &files {
        let file_index = build_index_for_file(file)?;
        incremental.record_file_index(file_index, &mut telemetry);
    }

    let mut single_file_incremental_ms = 0u128;
    if let Some(first) = files.first() {
        let changed = first
            .canonicalize()
            .unwrap_or_else(|_| first.clone())
            .to_string_lossy()
            .to_string();
        let report = incremental.update_changed_files_with_loader(&changed, |file_path| {
            let path = PathBuf::from(file_path);
            if !path.exists() {
                return Ok(None);
            }
            Ok(Some(build_index_for_file(&path)?))
        })?;
        single_file_incremental_ms = report.incremental_update_latency_ms;
    }

    incremental.store().save()?;
    let stats = incremental.store().stats();
    let cold_ms = cold_start.elapsed().as_millis();
    println!(
        "indexed {} files into {}",
        files.len(),
        index_root.display()
    );
    if args.stats {
        print_index_stats(
            &stats,
            cold_ms,
            single_file_incremental_ms,
            &index_root,
            &args.target_path,
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn run_lsp(args: &[String]) -> Result<ExitCode, String> {
    let mut index_root = env::current_dir()
        .map_err(|e| format!("failed to resolve current directory: {e}"))?
        .join(".vibe")
        .join("index");
    let mut idx = 0usize;
    while idx < args.len() {
        match args[idx].as_str() {
            "--index-root" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--index-root`".to_string())?;
                index_root = PathBuf::from(val);
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
        idx += 1;
    }
    run_line_stdio(index_root)?;
    Ok(ExitCode::SUCCESS)
}

fn build_source(args: &BuildArgs) -> Result<BuildArtifacts, String> {
    let src = fs::read_to_string(&args.source_path).map_err(|e| {
        format!(
            "failed to read `{}`: {e}",
            args.source_path.as_path().display()
        )
    })?;
    let parsed = parse_source(&src);
    let checked = check_and_lower(&parsed.ast);
    let mut all = Diagnostics::default();
    all.extend(parsed.diagnostics.into_sorted());
    all.extend(checked.diagnostics.into_sorted());
    let diags = all.to_golden();
    if !diags.trim().is_empty() {
        eprintln!("{diags}");
    }
    if all.has_errors() {
        return Err("build failed due to errors".to_string());
    }

    let mir =
        lower_hir_to_mir(&checked.hir).map_err(|e| format!("HIR->MIR lowering failed: {e}"))?;
    let object_bytes = emit_object(
        &mir,
        &CodegenOptions {
            target: args.target.clone(),
            profile: args.profile.clone(),
            debuginfo: args.debuginfo.clone(),
        },
    )
    .map_err(|e| format!("codegen failed: {e}"))?;

    if args.offline {
        // Phase 2 keeps AI and network out of the compile path. This flag is currently informational.
    }

    let artifacts_dir = artifact_directory(&args.source_path, &args.profile, &args.target);
    fs::create_dir_all(&artifacts_dir)
        .map_err(|e| format!("failed to create artifacts directory: {e}"))?;

    let stem = args
        .source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "invalid source filename".to_string())?;
    let object_path = artifacts_dir.join(format!("{stem}.o"));
    let binary_path = artifacts_dir.join(stem);
    fs::write(&object_path, object_bytes)
        .map_err(|e| format!("failed to write object `{}`: {e}", object_path.display()))?;

    let runtime_options = RuntimeBuildOptions {
        target: args.target.clone(),
        profile: args.profile.clone(),
        debuginfo: args.debuginfo.clone(),
    };
    let runtime_object_path = compile_runtime_object(&artifacts_dir, &runtime_options)?;
    link_executable(
        &object_path,
        &runtime_object_path,
        &binary_path,
        &runtime_options,
    )?;
    let debug_map_path = write_debug_map(&artifacts_dir, &args.source_path, args, &mir, stem)
        .map_err(|e| {
            format!(
                "failed to write debug map for `{}`: {e}",
                args.source_path.display()
            )
        })?;

    Ok(BuildArtifacts {
        object_path,
        runtime_object_path,
        debug_map_path,
        binary_path,
    })
}

fn build_index_for_file(file: &Path) -> Result<vibe_indexer::FileIndex, String> {
    let canonical = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
    let src = fs::read_to_string(&canonical)
        .map_err(|e| format!("failed to read `{}`: {e}", canonical.display()))?;
    let parsed = parse_source(&src);
    let checked = check_and_lower(&parsed.ast);
    let mut diagnostics = parsed.diagnostics.into_sorted();
    diagnostics.extend(checked.diagnostics.into_sorted());
    Ok(build_file_index(
        &canonical,
        &src,
        &parsed.ast,
        &checked.hir,
        &diagnostics,
    ))
}

fn best_effort_refresh_index(
    path: &Path,
    source: &str,
    ast: &vibe_ast::FileAst,
    hir: &vibe_hir::HirProgram,
    diagnostics: &[Diagnostic],
) -> Result<(), String> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let file_index = build_file_index(&canonical, source, ast, hir, diagnostics);
    let index_root = default_index_root(&canonical);
    let store = IndexStore::open_or_create(&index_root)?;
    let mut incremental = IncrementalIndexer::new(store);
    let mut telemetry = IncrementalTelemetry::default();
    incremental.record_file_index(file_index, &mut telemetry);
    incremental.store().save()?;
    Ok(())
}

fn print_index_stats(
    stats: &IndexStats,
    cold_ms: u128,
    incremental_ms: u128,
    index_root: &Path,
    target_path: &Path,
) {
    let total_source_bytes = collect_total_source_bytes(target_path).unwrap_or(0);
    let memory_ratio = if total_source_bytes == 0 {
        0.0
    } else {
        stats.memory_estimate_bytes as f64 / total_source_bytes as f64
    };
    println!(
        "index stats: files={} symbols={} references={} function_meta={} diagnostics={} cold_ms={} incremental_ms={} memory_bytes={} memory_ratio={:.4} root={}",
        stats.files,
        stats.symbols,
        stats.references,
        stats.function_meta,
        stats.diagnostics,
        cold_ms,
        incremental_ms,
        stats.memory_estimate_bytes,
        memory_ratio,
        index_root.display()
    );
}

fn collect_total_source_bytes(target_path: &Path) -> Result<usize, String> {
    let files = collect_vibe_files(target_path)?;
    let mut total = 0usize;
    for file in files {
        let metadata = fs::metadata(&file)
            .map_err(|e| format!("failed to read metadata `{}`: {e}", file.display()))?;
        total += usize::try_from(metadata.len()).unwrap_or(0);
    }
    Ok(total)
}

fn artifact_directory(source_path: &Path, profile: &str, target: &str) -> PathBuf {
    let base = source_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".vibe")
        .join("artifacts")
        .join(profile)
        .join(target)
}

fn run_test(args: &BuildArgs) -> Result<ExitCode, String> {
    let files = collect_vibe_files(&args.source_path)?;
    if files.is_empty() {
        return Err(format!(
            "no .vibe files found under `{}`",
            args.source_path.display()
        ));
    }
    let total_files = files.len();

    let mut compile_failures = 0usize;
    let mut examples = ExampleRunSummary::default();
    let mut main_run_total = 0usize;
    let mut main_run_failures = 0usize;

    for file in files {
        let src = fs::read_to_string(&file)
            .map_err(|e| format!("failed to read `{}`: {e}", file.display()))?;
        let parsed = parse_source(&src);
        let checked = check_and_lower(&parsed.ast);
        let mut all = Diagnostics::default();
        all.extend(parsed.diagnostics.clone().into_sorted());
        all.extend(checked.diagnostics.clone().into_sorted());

        let diag_out = all.to_golden();
        if !diag_out.trim().is_empty() {
            eprintln!("{}:\n{diag_out}", file.display());
        }
        if all.has_errors() {
            compile_failures += 1;
            continue;
        }

        let current_examples = run_examples(&parsed.ast);
        examples.total += current_examples.total;
        examples.passed += current_examples.passed;
        examples.failed += current_examples.failed;
        examples.failures.extend(current_examples.failures);

        if has_main_function(&parsed.ast) {
            main_run_total += 1;
            let single_file_args = BuildArgs {
                source_path: file.clone(),
                profile: args.profile.clone(),
                target: args.target.clone(),
                debuginfo: args.debuginfo.clone(),
                offline: args.offline,
                exec_args: Vec::new(),
            };
            let artifacts = build_source(&single_file_args)?;
            let status = Command::new(&artifacts.binary_path).status().map_err(|e| {
                format!(
                    "failed to execute test binary `{}`: {e}",
                    artifacts.binary_path.display()
                )
            })?;
            if !status.success() {
                main_run_failures += 1;
                eprintln!(
                    "{}: main returned non-zero exit code {:?}",
                    file.display(),
                    status.code()
                );
            }
        }
    }

    if !examples.failures.is_empty() {
        eprintln!("example failures:");
        for failure in &examples.failures {
            eprintln!("  - {failure}");
        }
    }
    println!(
        "test summary: files={}, compile_failures={}, examples={} passed={} failed={}, mains={} main_failures={}",
        total_files,
        compile_failures,
        examples.total,
        examples.passed,
        examples.failed,
        main_run_total,
        main_run_failures
    );

    if compile_failures > 0 || examples.failed > 0 || main_run_failures > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn collect_vibe_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        if path.extension().and_then(|x| x.to_str()) == Some("vibe") {
            return Ok(vec![path.to_path_buf()]);
        }
        return Err(format!("expected a .vibe file, got `{}`", path.display()));
    }
    if !path.is_dir() {
        return Err(format!("path does not exist: `{}`", path.display()));
    }
    let mut out = Vec::new();
    collect_vibe_files_recursive(path, &mut out)?;
    out.sort();
    Ok(out)
}

fn collect_vibe_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut entries = fs::read_dir(dir)
        .map_err(|e| format!("failed to read directory `{}`: {e}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_vibe_files_recursive(&path, out)?;
            continue;
        }
        if path.extension().and_then(|x| x.to_str()) == Some("vibe") {
            out.push(path);
        }
    }
    Ok(())
}

fn has_main_function(ast: &vibe_ast::FileAst) -> bool {
    ast.declarations.iter().any(|decl| {
        let vibe_ast::Declaration::Function(func) = decl;
        func.name == "main"
    })
}

fn write_debug_map(
    artifacts_dir: &Path,
    source_path: &Path,
    args: &BuildArgs,
    mir: &MirProgram,
    stem: &str,
) -> Result<PathBuf, String> {
    let mut functions = mir
        .functions
        .iter()
        .map(|f| {
            format!(
                "{}({} params) -> {:?}",
                f.name,
                f.params.len(),
                f.return_type
            )
        })
        .collect::<Vec<_>>();
    functions.sort();

    let mut out = String::new();
    out.push_str("vibe-debug-map-v0\n");
    out.push_str(&format!("source={}\n", source_path.display()));
    out.push_str(&format!("profile={}\n", args.profile));
    out.push_str(&format!("target={}\n", args.target));
    out.push_str(&format!("debuginfo={}\n", args.debuginfo));
    out.push_str("functions:\n");
    for function in functions {
        out.push_str(&format!("  - {function}\n"));
    }

    let debug_map_path = artifacts_dir.join(format!("{stem}.debug.map"));
    fs::write(&debug_map_path, out)
        .map_err(|e| format!("failed to write `{}`: {e}", debug_map_path.display()))?;
    Ok(debug_map_path)
}
