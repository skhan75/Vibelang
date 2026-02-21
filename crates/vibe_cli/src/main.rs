mod deterministic_utils;
mod example_runner;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::{env, fs};

use vibe_codegen::{emit_object, CodegenOptions};
use vibe_diagnostics::Diagnostic;
use vibe_diagnostics::Diagnostics;
use vibe_doc::{extract_docs, render_markdown};
use vibe_fmt::{format_source, needs_formatting};
use vibe_indexer::build_file_index;
use vibe_indexer::{
    default_metadata_root, is_supported_source_file, prepare_index_root, IncrementalIndexer,
    IncrementalTelemetry, IndexStats, IndexStore, SUPPORTED_SOURCE_EXTS,
};
use vibe_lsp::run_line_stdio;
use vibe_mir::MirProgram;
use vibe_mir::{lower_hir_to_mir, mir_debug_dump};
use vibe_parser::parse_source;
use vibe_pkg::{
    default_mirror_root, install_project, resolve_project, write_lockfile, LOCK_FILENAME,
    MANIFEST_FILENAME,
};
use vibe_runtime::{compile_runtime_object, link_executable, RuntimeBuildOptions};
use vibe_sidecar::models::FindingSeverity;
use vibe_sidecar::SidecarMode;
use vibe_sidecar::{BudgetPolicy, IntentLintRequest, SidecarService};
use vibe_types::check_and_lower;

use crate::example_runner::{run_examples_with_policy, ExampleRunSummary};

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
    if is_help_token(&args[0]) {
        if args.len() > 2 {
            return Err("`vibe --help` accepts at most one command argument".to_string());
        }
        let cmd = args.get(1).map(String::as_str);
        print_help(cmd)?;
        return Ok(ExitCode::SUCCESS);
    }
    if args[0] == "help" {
        if args.len() > 2 {
            return Err("usage: vibe help [command]".to_string());
        }
        let cmd = args.get(1).map(String::as_str);
        print_help(cmd)?;
        return Ok(ExitCode::SUCCESS);
    }
    if args[0] == "--version" || args[0] == "version" {
        let json = matches!(args.get(1).map(String::as_str), Some("--json"));
        if args.len() > 2 || (args.len() == 2 && !json) {
            return Err("usage: vibe --version [--json]".to_string());
        }
        println!("{}", render_version(json));
        return Ok(ExitCode::SUCCESS);
    }
    let cmd = args.remove(0);
    if args.len() == 1 && is_help_token(&args[0]) {
        print_help(Some(&cmd))?;
        return Ok(ExitCode::SUCCESS);
    }
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
            if build_args.emit_obj_only {
                return Err("`--emit-obj-only` is not valid for `vibe run`".to_string());
            }
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
            if test_args.emit_obj_only {
                return Err("`--emit-obj-only` is not valid for `vibe test`".to_string());
            }
            run_test(&test_args)
        }
        "index" => {
            let index_args = parse_index_args(&args)?;
            run_index(&index_args)
        }
        "lsp" => run_lsp(&args),
        "fmt" => {
            let fmt_args = parse_fmt_args(&args)?;
            run_fmt(&fmt_args)
        }
        "doc" => {
            let doc_args = parse_doc_args(&args)?;
            run_doc(&doc_args)
        }
        "new" => {
            let new_args = parse_new_args(&args)?;
            run_new(&new_args)
        }
        "pkg" => {
            let pkg_args = parse_pkg_args(&args)?;
            run_pkg(&pkg_args)
        }
        "lint" => {
            let lint_args = parse_lint_args(&args)?;
            run_lint(&lint_args)
        }
        _ => Err(format!("unknown command `{cmd}`\n{}", usage())),
    }
}

fn usage() -> String {
    "usage: vibe <command> [options]\nrun `vibe --help` for the full command manual.".to_string()
}

fn is_help_token(raw: &str) -> bool {
    matches!(raw, "--help" | "-h")
}

fn print_help(command: Option<&str>) -> Result<(), String> {
    let text = match command {
        None => root_help(),
        Some(cmd) => command_help(cmd)
            .ok_or_else(|| format!("unknown command `{cmd}`\n{}", usage()))?
            .to_string(),
    };
    println!("{text}");
    Ok(())
}

fn root_help() -> String {
    r#"VibeLang CLI Manual

USAGE
  vibe <command> [options]

COMMANDS
  check <path>              Parse + type-check source and print diagnostics
  ast <path>                Print parsed AST
  hir <path>                Print typed HIR
  mir <path>                Print lowered MIR
  build <path> [flags]      Build native artifact(s)
  run <path> [flags] [-- <args...>]
                            Build and execute compiled program
  test <path|dir> [flags]   Run fixture-aware test flow
  index [path] [flags]      Build/update semantic index
  lsp [--index-root <dir>]  Start line-stdio LSP server
  fmt [path] [flags]        Format source files
  doc [path] [flags]        Generate markdown API docs
  new <name> [flags]        Scaffold new app/library project
  pkg <resolve|lock|install> [flags]
                            Dependency resolution + lock + install
  lint [path] --intent [flags]
                            Run intent-aware lint checks

GLOBAL OPTIONS
  --help, -h                Show this manual or command help
  help [command]            Show command help
  --version                 Print CLI version metadata
  --version --json          Print version metadata as JSON

EXIT CODES
  0                         Success
  1                         Command/runtime failure
  2                         CLI usage/argument error

EXAMPLES
  vibe --help
  vibe --version --json
  vibe help build
  vibe build main.yb --profile release --target x86_64-unknown-linux-gnu
  vibe run main.yb -- --arg1 value
  vibe lint . --intent --changed --mode local
"#
    .to_string()
}

fn command_help(command: &str) -> Option<&'static str> {
    match command {
        "check" => Some(
            r#"vibe check

USAGE
  vibe check <path>

DESCRIPTION
  Parses and type-checks a source file, printing deterministic diagnostics.
"#,
        ),
        "ast" => Some(
            r#"vibe ast

USAGE
  vibe ast <path>

DESCRIPTION
  Parses source and prints the AST plus parser diagnostics.
"#,
        ),
        "hir" => Some(
            r#"vibe hir

USAGE
  vibe hir <path>

DESCRIPTION
  Type-checks source and prints lowered typed HIR.
"#,
        ),
        "mir" => Some(
            r#"vibe mir

USAGE
  vibe mir <path>

DESCRIPTION
  Lowers HIR to MIR and prints deterministic MIR debug output.
"#,
        ),
        "build" => Some(
            r#"vibe build

USAGE
  vibe build <path> [--profile dev|release] [--target <triple>] [--debuginfo none|line|full] [--offline] [--locked] [--emit-obj-only]

FLAGS
  --profile <name>          Build profile (dev|release)
  --target <triple>         Target triple for codegen/runtime
  --debuginfo <mode>        Debug info level (none|line|full)
  --offline                 Informational offline mode flag
  --locked                  Enforce lockfile/manifest locked-mode checks
  --emit-obj-only           Skip runtime compile/link and emit object-only artifacts
"#,
        ),
        "run" => Some(
            r#"vibe run

USAGE
  vibe run <path> [build flags] [-- <program args...>]

DESCRIPTION
  Builds source and executes produced native binary.

NOTES
  `--emit-obj-only` is not valid for `vibe run`.
"#,
        ),
        "test" => Some(
            r#"vibe test

USAGE
  vibe test <path|dir> [--profile dev|release] [--target <triple>] [--debuginfo none|line|full] [--offline] [--locked]

DESCRIPTION
  Runs file/directory tests, including contract/example checks where applicable.

NOTES
  `--emit-obj-only` is not valid for `vibe test`.
"#,
        ),
        "index" => Some(
            r#"vibe index

USAGE
  vibe index [path] [--path <dir>] [--rebuild] [--stats]

FLAGS
  --path <dir>              Explicit target root
  --rebuild                 Force full index rebuild
  --stats                   Print telemetry/statistics snapshot
"#,
        ),
        "lsp" => Some(
            r#"vibe lsp

USAGE
  vibe lsp [--index-root <dir>]

DESCRIPTION
  Starts line-stdio language-server protocol endpoint.
"#,
        ),
        "fmt" => Some(
            r#"vibe fmt

USAGE
  vibe fmt [path] [--path <dir>] [--check]

FLAGS
  --check                   Do not rewrite files; fail if formatting is needed
"#,
        ),
        "doc" => Some(
            r#"vibe doc

USAGE
  vibe doc [path] [--path <dir>] [--out <file>]

DESCRIPTION
  Generates markdown API docs for VibeLang sources.
"#,
        ),
        "new" => Some(
            r#"vibe new

USAGE
  vibe new <name> [--path <dir>] [--app|--lib] [--ext yb|vibe]

FLAGS
  --app                     Scaffold application template (default)
  --lib                     Scaffold library template
  --ext <ext>               Source extension for generated template
"#,
        ),
        "pkg" => Some(
            r#"vibe pkg

USAGE
  vibe pkg <resolve|lock|install> [--path <dir>] [--mirror <dir>]

SUBCOMMANDS
  resolve                   Resolve dependency graph
  lock                      Resolve and write lockfile
  install                   Resolve and install dependencies
"#,
        ),
        "lint" => Some(
            r#"vibe lint

USAGE
  vibe lint [path] --intent [--changed] [--suggest] [--mode local|hybrid|cloud] [--telemetry-out <file>] [--max-local-ms <n>] [--max-cloud-ms <n>] [--max-requests-per-day <n>] [--path <dir>]

DESCRIPTION
  Runs intent-aware lint checks and emits deterministic diagnostics.

NOTES
  Current lint mode requires `--intent`.
"#,
        ),
        _ => None,
    }
}

fn render_version(json: bool) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let commit_raw = option_env!("VIBE_GIT_SHA")
        .or(option_env!("GITHUB_SHA"))
        .or(option_env!("VERGEN_GIT_SHA"))
        .unwrap_or("unknown");
    let commit_short = if commit_raw.len() > 12 {
        &commit_raw[..12]
    } else {
        commit_raw
    };
    let profile = if cfg!(debug_assertions) {
        "dev"
    } else {
        "release"
    };
    let os = if env::consts::OS == "macos" {
        "darwin"
    } else {
        env::consts::OS
    };
    let target = format!("{}-{os}", env::consts::ARCH);

    if json {
        format!(
            "{{\"name\":\"vibe\",\"version\":\"{}\",\"commit\":\"{}\",\"target\":\"{}\",\"profile\":\"{}\"}}",
            json_escape(version),
            json_escape(commit_short),
            json_escape(&target),
            json_escape(profile),
        )
    } else {
        format!("vibe {version} (commit={commit_short}, target={target}, profile={profile})")
    }
}

fn json_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
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
    locked: bool,
    emit_obj_only: bool,
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

#[derive(Debug, Clone)]
struct LintArgs {
    target_path: PathBuf,
    intent: bool,
    changed: bool,
    include_suggestions: bool,
    mode: SidecarMode,
    telemetry_out: Option<PathBuf>,
    max_local_ms: Option<u64>,
    max_cloud_ms: Option<u64>,
    max_requests_per_day: Option<u64>,
}

#[derive(Debug, Clone)]
struct FmtArgs {
    target_path: PathBuf,
    check: bool,
}

#[derive(Debug, Clone)]
struct DocArgs {
    target_path: PathBuf,
    out: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NewTemplate {
    App,
    Lib,
}

#[derive(Debug, Clone)]
struct NewArgs {
    name: String,
    base_dir: PathBuf,
    template: NewTemplate,
    extension: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PkgCommand {
    Resolve,
    Lock,
    Install,
}

#[derive(Debug, Clone)]
struct PkgArgs {
    command: PkgCommand,
    project_root: PathBuf,
    mirror_root: Option<PathBuf>,
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
    let mut locked = false;
    let mut emit_obj_only = false;
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
            "--locked" => {
                locked = true;
            }
            "--emit-obj-only" => {
                emit_obj_only = true;
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
        locked,
        emit_obj_only,
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

fn parse_lint_args(args: &[String]) -> Result<LintArgs, String> {
    let mut idx = 0usize;
    let mut target_path = PathBuf::from(".");
    let mut intent = false;
    let mut changed = false;
    let mut include_suggestions = false;
    let mut mode = SidecarMode::LocalOnly;
    let mut telemetry_out = None;
    let mut max_local_ms = None;
    let mut max_cloud_ms = None;
    let mut max_requests_per_day = None;

    if let Some(first) = args.first() {
        if !first.starts_with("--") {
            target_path = PathBuf::from(first);
            idx = 1;
        }
    }

    while idx < args.len() {
        match args[idx].as_str() {
            "--intent" => intent = true,
            "--changed" => changed = true,
            "--suggest" => include_suggestions = true,
            "--mode" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--mode`".to_string())?;
                mode = parse_sidecar_mode(val)?;
            }
            "--telemetry-out" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--telemetry-out`".to_string())?;
                telemetry_out = Some(PathBuf::from(val));
            }
            "--max-local-ms" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--max-local-ms`".to_string())?;
                max_local_ms = Some(
                    val.parse::<u64>()
                        .map_err(|_| "invalid value for `--max-local-ms`".to_string())?,
                );
            }
            "--max-cloud-ms" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--max-cloud-ms`".to_string())?;
                max_cloud_ms = Some(
                    val.parse::<u64>()
                        .map_err(|_| "invalid value for `--max-cloud-ms`".to_string())?,
                );
            }
            "--max-requests-per-day" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--max-requests-per-day`".to_string())?;
                max_requests_per_day = Some(
                    val.parse::<u64>()
                        .map_err(|_| "invalid value for `--max-requests-per-day`".to_string())?,
                );
            }
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

    if !intent {
        return Err("`vibe lint` currently supports only `--intent` mode".to_string());
    }

    Ok(LintArgs {
        target_path,
        intent,
        changed,
        include_suggestions,
        mode,
        telemetry_out,
        max_local_ms,
        max_cloud_ms,
        max_requests_per_day,
    })
}

fn parse_fmt_args(args: &[String]) -> Result<FmtArgs, String> {
    let mut idx = 0usize;
    let mut target_path = PathBuf::from(".");
    let mut check = false;

    if let Some(first) = args.first() {
        if !first.starts_with("--") {
            target_path = PathBuf::from(first);
            idx = 1;
        }
    }

    while idx < args.len() {
        match args[idx].as_str() {
            "--check" => check = true,
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
    Ok(FmtArgs { target_path, check })
}

fn parse_doc_args(args: &[String]) -> Result<DocArgs, String> {
    let mut idx = 0usize;
    let mut target_path = PathBuf::from(".");
    let mut out = None;

    if let Some(first) = args.first() {
        if !first.starts_with("--") {
            target_path = PathBuf::from(first);
            idx = 1;
        }
    }

    while idx < args.len() {
        match args[idx].as_str() {
            "--out" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--out`".to_string())?;
                out = Some(PathBuf::from(val));
            }
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

    Ok(DocArgs { target_path, out })
}

fn parse_new_args(args: &[String]) -> Result<NewArgs, String> {
    if args.is_empty() {
        return Err("missing project name".to_string());
    }

    let name = args[0].clone();
    if name.contains('/') || name.contains('\\') {
        return Err("project name must not contain path separators".to_string());
    }

    let mut idx = 1usize;
    let mut base_dir = PathBuf::from(".");
    let mut template = NewTemplate::App;
    let mut extension = "yb".to_string();
    while idx < args.len() {
        match args[idx].as_str() {
            "--path" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--path`".to_string())?;
                base_dir = PathBuf::from(val);
            }
            "--lib" => template = NewTemplate::Lib,
            "--app" => template = NewTemplate::App,
            "--ext" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--ext`".to_string())?;
                if !SUPPORTED_SOURCE_EXTS.iter().any(|ext| ext == val) {
                    return Err(format!(
                        "unsupported extension `{val}` (expected {})",
                        supported_source_ext_display()
                    ));
                }
                extension = val.clone();
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
        idx += 1;
    }

    Ok(NewArgs {
        name,
        base_dir,
        template,
        extension,
    })
}

fn parse_pkg_args(args: &[String]) -> Result<PkgArgs, String> {
    let Some(first) = args.first() else {
        return Err("missing pkg subcommand (expected resolve|lock|install)".to_string());
    };
    let command = match first.as_str() {
        "resolve" => PkgCommand::Resolve,
        "lock" => PkgCommand::Lock,
        "install" => PkgCommand::Install,
        other => return Err(format!("unknown pkg subcommand `{other}`")),
    };

    let mut idx = 1usize;
    let mut project_root = PathBuf::from(".");
    let mut mirror_root = None;
    while idx < args.len() {
        match args[idx].as_str() {
            "--path" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--path`".to_string())?;
                project_root = PathBuf::from(val);
            }
            "--mirror" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--mirror`".to_string())?;
                mirror_root = Some(PathBuf::from(val));
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
        idx += 1;
    }

    Ok(PkgArgs {
        command,
        project_root,
        mirror_root,
    })
}

fn parse_sidecar_mode(raw: &str) -> Result<SidecarMode, String> {
    match raw {
        "local" | "local-only" => Ok(SidecarMode::LocalOnly),
        "hybrid" => Ok(SidecarMode::Hybrid),
        "cloud" => Ok(SidecarMode::Cloud),
        other => Err(format!(
            "unsupported mode `{other}` (expected local|hybrid|cloud)"
        )),
    }
}

fn run_index(args: &IndexArgs) -> Result<ExitCode, String> {
    let files = collect_vibe_files(&args.target_path)?;
    if files.is_empty() {
        return Err(format!(
            "no source files ({}) found under `{}`",
            supported_source_ext_display(),
            args.target_path.display()
        ));
    }

    let index_root = prepare_index_root(&args.target_path)?;
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
        telemetry.files_reindexed += report.files_reindexed;
        telemetry.invalidation_fanout += report.invalidation_fanout;
        telemetry.cache_hits += report.cache_hits;
        telemetry.cache_misses += report.cache_misses;
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
            telemetry.cache_hits,
            telemetry.cache_misses,
            &index_root,
            &args.target_path,
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn run_lint(args: &LintArgs) -> Result<ExitCode, String> {
    if !args.intent {
        return Err("`vibe lint` currently supports only `--intent`".to_string());
    }

    let files = collect_vibe_files(&args.target_path)?;
    if files.is_empty() {
        return Err(format!(
            "no source files ({}) found under `{}`",
            supported_source_ext_display(),
            args.target_path.display()
        ));
    }

    let index_root = prepare_index_root(&args.target_path)?;
    let store = IndexStore::open_or_create(&index_root)?;
    let mut incremental = IncrementalIndexer::new(store);
    let mut telemetry = IncrementalTelemetry::default();
    let mut changed_files = Vec::new();

    let git_changed = if args.changed {
        git_changed_source_files(&args.target_path)?
    } else {
        None
    };

    for file in &files {
        let file_index = build_index_for_file(file)?;
        let key = file_index.file.clone();
        let old_hash = incremental
            .store()
            .snapshot()
            .files
            .get(&key)
            .map(|existing| existing.file_hash.as_str())
            .unwrap_or("");
        let changed_by_hash = old_hash != file_index.file_hash;
        let changed_by_git = git_changed.as_ref().is_some_and(|set| set.contains(&key));
        let should_include = if args.changed {
            if git_changed.is_some() {
                changed_by_git
            } else {
                changed_by_hash
            }
        } else {
            true
        };
        if should_include {
            changed_files.push(key);
        }
        incremental.record_file_index(file_index, &mut telemetry);
    }
    changed_files.sort();
    changed_files.dedup();
    incremental.store().save()?;

    if args.changed && changed_files.is_empty() {
        println!(
            "no changed source files ({}) detected",
            supported_source_ext_display()
        );
        return Ok(ExitCode::SUCCESS);
    }

    let mut policy = BudgetPolicy {
        mode: args.mode,
        ..BudgetPolicy::default()
    };
    if let Some(max_local_ms) = args.max_local_ms {
        policy.max_local_latency_ms = max_local_ms;
    }
    if let Some(max_cloud_ms) = args.max_cloud_ms {
        policy.max_cloud_latency_ms = max_cloud_ms;
    }
    if let Some(max_requests_per_day) = args.max_requests_per_day {
        policy.max_requests_per_day = max_requests_per_day;
    }

    let telemetry_out = args.telemetry_out.clone();
    let mut sidecar = SidecarService::new(&index_root, policy, telemetry_out.is_some())?;
    let request = IntentLintRequest {
        query: None,
        changed_only: args.changed,
        changed_files,
        include_suggestions: args.include_suggestions,
    };
    let mut response = sidecar.lint_intent(&request);
    let (accepted_suggestions, rejected_suggestions) =
        revalidate_and_gate_suggestions(response.suggestions)?;
    response.suggestions = accepted_suggestions;
    if rejected_suggestions > 0 {
        response.findings.push(vibe_sidecar::IntentFinding {
            code: "I6001".to_string(),
            severity: FindingSeverity::Warning,
            message: format!(
                "{rejected_suggestions} suggestion(s) were rejected by compiler revalidation"
            ),
            confidence: 1.0,
            evidence: Vec::new(),
            incomplete: false,
        });
    }
    response.findings.sort_by(|a, b| {
        let a_file = a
            .evidence
            .first()
            .map(|e| e.file.as_str())
            .unwrap_or_default();
        let b_file = b
            .evidence
            .first()
            .map(|e| e.file.as_str())
            .unwrap_or_default();
        (a_file, a.code.as_str(), a.message.as_str()).cmp(&(
            b_file,
            b.code.as_str(),
            b.message.as_str(),
        ))
    });
    response
        .suggestions
        .sort_by(|a, b| (a.id.as_str(), a.title.as_str()).cmp(&(b.id.as_str(), b.title.as_str())));

    if response.findings.is_empty() {
        println!("OK");
    } else {
        for finding in &response.findings {
            let severity = match finding.severity {
                FindingSeverity::Info => "info",
                FindingSeverity::Warning => "warning",
                FindingSeverity::Error => "error",
            };
            let evidence = finding
                .evidence
                .iter()
                .map(|e| {
                    if let Some(symbol) = &e.symbol {
                        format!("{}::{symbol}", e.file)
                    } else {
                        e.file.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                "{}: {}: {} (confidence={:.2}{})",
                finding.code,
                severity,
                finding.message,
                finding.confidence,
                if finding.incomplete {
                    ", incomplete=true"
                } else {
                    ""
                }
            );
            if !evidence.is_empty() {
                println!("  evidence: {evidence}");
            }
        }
    }

    if args.include_suggestions && !response.suggestions.is_empty() {
        println!("suggestions:");
        for suggestion in &response.suggestions {
            println!(
                "- {} [{}] confidence={:.2} verified={}",
                suggestion.title, suggestion.id, suggestion.confidence, suggestion.verified
            );
        }
    }

    if response.incomplete {
        println!("intent lint returned partial results (latency/budget guard).");
    }
    if let Some(path) = telemetry_out {
        sidecar.telemetry().write_json(&path)?;
        println!("telemetry written to {}", path.display());
    }
    Ok(ExitCode::SUCCESS)
}

fn revalidate_and_gate_suggestions(
    suggestions: Vec<vibe_sidecar::CandidateSuggestion>,
) -> Result<(Vec<vibe_sidecar::CandidateSuggestion>, usize), String> {
    let mut accepted = Vec::new();
    let mut rejected = 0usize;
    for mut suggestion in suggestions {
        let Some(evidence_file) = suggestion
            .evidence
            .iter()
            .find(|e| !e.file.trim().is_empty())
            .map(|e| PathBuf::from(&e.file))
        else {
            rejected += 1;
            continue;
        };
        let verified = compiler_revalidate_file(&evidence_file)?;
        if verified {
            suggestion.verified = true;
            accepted.push(suggestion);
        } else {
            rejected += 1;
        }
    }
    Ok((accepted, rejected))
}

fn compiler_revalidate_file(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }
    let src = fs::read_to_string(path).map_err(|e| {
        format!(
            "failed to read suggestion evidence file `{}`: {e}",
            path.display()
        )
    })?;
    let parsed = parse_source(&src);
    let checked = check_and_lower(&parsed.ast);
    let mut diags = Diagnostics::default();
    diags.extend(parsed.diagnostics.into_sorted());
    diags.extend(checked.diagnostics.into_sorted());
    Ok(!diags.has_errors())
}

fn git_changed_source_files(target_path: &Path) -> Result<Option<BTreeSet<String>>, String> {
    let base_dir = if target_path.is_dir() {
        target_path.to_path_buf()
    } else {
        target_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    };

    let probe = Command::new("git")
        .arg("-C")
        .arg(&base_dir)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output();
    let Ok(probe_out) = probe else {
        return Ok(None);
    };
    if !probe_out.status.success() {
        return Ok(None);
    }

    let mut out = BTreeSet::new();
    for ext in SUPPORTED_SOURCE_EXTS {
        let glob = format!("*.{ext}");
        for args in [
            vec![
                "diff".to_string(),
                "--name-only".to_string(),
                "--".to_string(),
                glob.clone(),
            ],
            vec![
                "diff".to_string(),
                "--cached".to_string(),
                "--name-only".to_string(),
                "--".to_string(),
                glob.clone(),
            ],
        ] {
            let output = Command::new("git")
                .arg("-C")
                .arg(&base_dir)
                .args(args)
                .output()
                .map_err(|e| format!("failed to query git changed files: {e}"))?;
            if !output.status.success() {
                continue;
            }
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines().filter(|line| !line.trim().is_empty()) {
                let absolute = base_dir.join(line.trim());
                let normalized = absolute
                    .canonicalize()
                    .unwrap_or_else(|_| absolute.clone())
                    .to_string_lossy()
                    .to_string();
                out.insert(normalized);
            }
        }
    }
    Ok(Some(out))
}

fn run_lsp(args: &[String]) -> Result<ExitCode, String> {
    let cwd =
        env::current_dir().map_err(|e| format!("failed to resolve current directory: {e}"))?;
    let mut index_root = prepare_index_root(&cwd)?;
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

fn run_fmt(args: &FmtArgs) -> Result<ExitCode, String> {
    let files = collect_vibe_files(&args.target_path)?;
    if files.is_empty() {
        return Err(format!(
            "no source files ({}) found under `{}`",
            supported_source_ext_display(),
            args.target_path.display()
        ));
    }
    let mut changed = Vec::new();
    let mut total_scanned = 0usize;
    let mut rewritten = 0usize;
    for file in files {
        total_scanned += 1;
        let src = fs::read_to_string(&file)
            .map_err(|e| format!("failed to read `{}`: {e}", file.display()))?;
        if needs_formatting(&src) {
            let formatted = format_source(&src);
            changed.push(file.clone());
            if !args.check {
                fs::write(&file, formatted)
                    .map_err(|e| format!("failed to write `{}`: {e}", file.display()))?;
                rewritten += 1;
            }
        }
    }

    if args.check {
        if changed.is_empty() {
            println!("format check: clean");
            return Ok(ExitCode::SUCCESS);
        }
        eprintln!("format check failed; files requiring formatting:");
        for path in &changed {
            eprintln!("  - {}", path.display());
        }
        return Ok(ExitCode::from(1));
    }

    println!(
        "format complete: scanned={} rewritten={}",
        total_scanned, rewritten
    );
    Ok(ExitCode::SUCCESS)
}

fn run_doc(args: &DocArgs) -> Result<ExitCode, String> {
    let files = collect_vibe_files(&args.target_path)?;
    if files.is_empty() {
        return Err(format!(
            "no source files ({}) found under `{}`",
            supported_source_ext_display(),
            args.target_path.display()
        ));
    }
    let default_out = if args.target_path.is_file() {
        args.target_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("api.md")
    } else {
        args.target_path.join("docs").join("api.md")
    };
    let out_path = args.out.clone().unwrap_or(default_out);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create doc output directory: {e}"))?;
    }

    let mut out = String::new();
    out.push_str("# VibeLang Generated Docs\n\n");
    out.push_str("Generated by `vibe doc`.\n\n");
    let mut total_items = 0usize;

    for file in files {
        let src = fs::read_to_string(&file)
            .map_err(|e| format!("failed to read `{}`: {e}", file.display()))?;
        let items = extract_docs(&src);
        total_items += items.len();
        out.push_str(&format!("## Module: `{}`\n\n", file.display()));
        out.push_str(&render_markdown(
            file.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("module"),
            &items,
        ));
        out.push('\n');
    }

    fs::write(&out_path, out)
        .map_err(|e| format!("failed to write docs `{}`: {e}", out_path.display()))?;
    println!(
        "doc generation complete: functions={} output={}",
        total_items,
        out_path.display()
    );
    Ok(ExitCode::SUCCESS)
}

fn run_new(args: &NewArgs) -> Result<ExitCode, String> {
    let project_dir = args.base_dir.join(&args.name);
    if project_dir.exists() {
        return Err(format!(
            "target project path already exists: `{}`",
            project_dir.display()
        ));
    }
    fs::create_dir_all(&project_dir).map_err(|e| {
        format!(
            "failed to create project directory `{}`: {e}",
            project_dir.display()
        )
    })?;
    let source_name = if args.template == NewTemplate::Lib {
        format!("lib.{}", args.extension)
    } else {
        format!("main.{}", args.extension)
    };
    let source_path = project_dir.join(source_name);
    let source = if args.template == NewTemplate::Lib {
        r#"pub add(a: Int, b: Int) -> Int {
  a + b
}
"#
    } else {
        r#"pub main() -> Int {
  @effect io
  println("hello from vibelang")
  0
}
"#
    };
    fs::write(&source_path, source).map_err(|e| {
        format!(
            "failed to write source template `{}`: {e}",
            source_path.display()
        )
    })?;

    let manifest = format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\n\n[dependencies]\n",
        args.name
    );
    fs::write(project_dir.join("vibe.toml"), manifest).map_err(|e| {
        format!(
            "failed to write project manifest `{}`: {e}",
            project_dir.join("vibe.toml").display()
        )
    })?;
    fs::write(project_dir.join(".gitignore"), ".yb/\n")
        .map_err(|e| format!("failed to write .gitignore: {e}"))?;
    let readme = if args.template == NewTemplate::Lib {
        format!(
            "# {}\n\nLibrary template.\n\n- Build checks: `vibe check {}`\n",
            args.name,
            source_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("lib.yb")
        )
    } else {
        format!(
            "# {}\n\nApplication template.\n\n- Run app: `vibe run {}`\n",
            args.name,
            source_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("main.yb")
        )
    };
    fs::write(project_dir.join("README.md"), readme)
        .map_err(|e| format!("failed to write README.md: {e}"))?;

    if args.extension == "vibe" && legacy_warning_enabled() {
        eprintln!(
            "warning: `.vibe` extension is legacy; prefer `.yb` for new projects (see docs/policy/source_extension_policy_v1x.md)"
        );
    }
    println!(
        "created project `{}` ({:?} template, extension .{})",
        project_dir.display(),
        args.template,
        args.extension
    );
    Ok(ExitCode::SUCCESS)
}

fn run_pkg(args: &PkgArgs) -> Result<ExitCode, String> {
    let mirror_root = args
        .mirror_root
        .clone()
        .unwrap_or_else(|| default_mirror_root(&args.project_root));
    match args.command {
        PkgCommand::Resolve => {
            let resolution = resolve_project(&args.project_root, &mirror_root)?;
            println!(
                "resolved root package `{}` v{} with {} transitive package(s) from mirror `{}`",
                resolution.root.name,
                resolution.root.version,
                resolution.packages.len(),
                mirror_root.display()
            );
            for pkg in resolution.packages {
                println!("  - {} {} [{}]", pkg.name, pkg.version, pkg.source);
            }
        }
        PkgCommand::Lock => {
            let resolution = resolve_project(&args.project_root, &mirror_root)?;
            let lock = write_lockfile(&args.project_root, &resolution)?;
            println!(
                "wrote deterministic lockfile `{}` (packages={})",
                lock.display(),
                resolution.packages.len()
            );
        }
        PkgCommand::Install => {
            let report = install_project(&args.project_root, &mirror_root)?;
            println!(
                "install complete: packages={} lock={} store={}",
                report.installed,
                report.lock_path.display(),
                report.store_root.display()
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn build_source(args: &BuildArgs) -> Result<BuildArtifacts, String> {
    let src = fs::read_to_string(&args.source_path).map_err(|e| {
        format!(
            "failed to read `{}`: {e}",
            args.source_path.as_path().display()
        )
    })?;
    if args.locked {
        enforce_locked_mode(&args.source_path)?;
    }
    let parsed = parse_source(&src);
    let checked = check_and_lower(&parsed.ast);
    let mut all = Diagnostics::default();
    all.extend(parsed.diagnostics.clone().into_sorted());
    all.extend(checked.diagnostics.into_sorted());
    let diags = all.to_golden();
    if !diags.trim().is_empty() {
        eprintln!("{diags}");
    }
    if all.has_errors() {
        return Err("build failed due to errors".to_string());
    }
    enforce_contract_preflight(&parsed.ast, &args.source_path, &args.profile)?;

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
    let (runtime_object_path, binary_path) = if args.emit_obj_only {
        let runtime_stub = artifacts_dir.join("vibe_runtime.obj_only");
        let binary_stub = artifacts_dir.join(format!("{stem}.obj_only"));
        fs::write(&runtime_stub, "obj-only build; runtime compile skipped\n").map_err(|e| {
            format!(
                "failed to write runtime stub `{}`: {e}",
                runtime_stub.display()
            )
        })?;
        fs::write(&binary_stub, "obj-only build; binary link skipped\n").map_err(|e| {
            format!(
                "failed to write binary stub `{}`: {e}",
                binary_stub.display()
            )
        })?;
        (runtime_stub, binary_stub)
    } else {
        let runtime_object_path = compile_runtime_object(&artifacts_dir, &runtime_options)?;
        link_executable(
            &object_path,
            &runtime_object_path,
            &binary_path,
            &runtime_options,
        )?;
        (runtime_object_path, binary_path)
    };
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
    let index_root = prepare_index_root(&canonical)?;
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
    cache_hits: usize,
    cache_misses: usize,
    index_root: &Path,
    target_path: &Path,
) {
    let total_source_bytes = collect_total_source_bytes(target_path).unwrap_or(0);
    let memory_ratio = if total_source_bytes == 0 {
        0.0
    } else {
        stats.memory_estimate_bytes as f64 / total_source_bytes as f64
    };
    let cache_total = cache_hits + cache_misses;
    let cache_hit_rate = if cache_total == 0 {
        0.0
    } else {
        cache_hits as f64 / cache_total as f64
    };
    println!(
        "index stats: files={} symbols={} references={} function_meta={} diagnostics={} cold_ms={} incremental_ms={} cache_hits={} cache_misses={} cache_hit_rate={:.4} memory_bytes={} memory_ratio={:.4} root={}",
        stats.files,
        stats.symbols,
        stats.references,
        stats.function_meta,
        stats.diagnostics,
        cold_ms,
        incremental_ms,
        cache_hits,
        cache_misses,
        cache_hit_rate,
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
    default_metadata_root(source_path)
        .join("artifacts")
        .join(profile)
        .join(target)
}

fn find_project_root(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };
    loop {
        if current.join(MANIFEST_FILENAME).is_file() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

fn enforce_locked_mode(source_path: &Path) -> Result<(), String> {
    let Some(project_root) = find_project_root(source_path) else {
        return Err(
            "`--locked` requires a project manifest (`vibe.toml`) in the source path hierarchy"
                .to_string(),
        );
    };
    let lock_path = project_root.join(LOCK_FILENAME);
    if !lock_path.is_file() {
        return Err(format!(
            "`--locked` requires lockfile `{}`; run `vibe pkg lock --path {}`",
            lock_path.display(),
            project_root.display()
        ));
    }
    Ok(())
}

fn normalize_source_for_debug_map(source_path: &Path) -> String {
    let canonical = source_path
        .canonicalize()
        .unwrap_or_else(|_| source_path.to_path_buf());
    if let Some(project_root) = find_project_root(&canonical) {
        if let Ok(rel) = canonical.strip_prefix(&project_root) {
            return format!("project://{}", rel.to_string_lossy().replace('\\', "/"));
        }
    }
    if let Ok(cwd) = env::current_dir() {
        if let Ok(rel) = canonical.strip_prefix(cwd) {
            return format!("./{}", rel.to_string_lossy().replace('\\', "/"));
        }
    }
    canonical.to_string_lossy().replace('\\', "/")
}

fn run_test(args: &BuildArgs) -> Result<ExitCode, String> {
    let start = std::time::Instant::now();
    let files = collect_vibe_files(&args.source_path)?;
    if files.is_empty() {
        return Err(format!(
            "no source files ({}) found under `{}`",
            supported_source_ext_display(),
            args.source_path.display()
        ));
    }
    let total_files = files.len();

    let mut compile_failures = 0usize;
    let mut examples = ExampleRunSummary::default();
    let mut main_run_total = 0usize;
    let mut main_run_failures = 0usize;
    let enforce_contract_checks = contract_checks_enabled(&args.profile);

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

        let current_examples = run_examples_with_policy(&parsed.ast, enforce_contract_checks);
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
                locked: args.locked,
                emit_obj_only: false,
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
        "test summary: files={}, compile_failures={}, examples={} passed={} failed={}, mains={} main_failures={} contract_checks={} duration_ms={}",
        total_files,
        compile_failures,
        examples.total,
        examples.passed,
        examples.failed,
        main_run_total,
        main_run_failures,
        if enforce_contract_checks { "on" } else { "off" },
        start.elapsed().as_millis()
    );

    if compile_failures > 0 || examples.failed > 0 || main_run_failures > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn contract_checks_enabled(profile: &str) -> bool {
    if let Ok(raw) = env::var("VIBE_CONTRACT_CHECKS") {
        let normalized = raw.trim().to_ascii_lowercase();
        if normalized == "0" || normalized == "false" || normalized == "off" {
            return false;
        }
        if normalized == "1" || normalized == "true" || normalized == "on" {
            return true;
        }
    }
    profile != "release"
}

fn enforce_contract_preflight(
    ast: &vibe_ast::FileAst,
    source_path: &Path,
    profile: &str,
) -> Result<(), String> {
    if !contract_checks_enabled(profile) {
        return Ok(());
    }
    let summary = run_examples_with_policy(ast, true);
    if summary.failed == 0 {
        return Ok(());
    }
    let mut details = String::new();
    for failure in &summary.failures {
        details.push_str(&format!("  - {failure}\n"));
    }
    Err(format!(
        "contract/example preflight failed for `{}`: {} of {} example(s) failed\n{}",
        source_path.display(),
        summary.failed,
        summary.total,
        details
    ))
}

fn collect_vibe_files(path: &Path) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        if is_supported_source_file(path) {
            let single = vec![path.to_path_buf()];
            maybe_warn_legacy_sources(&single);
            return Ok(single);
        }
        return Err(format!(
            "expected a source file ({}), got `{}`",
            supported_source_ext_display(),
            path.display()
        ));
    }
    if !path.is_dir() {
        return Err(format!("path does not exist: `{}`", path.display()));
    }
    let mut out = Vec::new();
    collect_vibe_files_recursive(path, &mut out)?;
    out.sort();
    detect_mixed_extension_stem_conflicts(&out)?;
    maybe_warn_legacy_sources(&out);
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
            let skip = path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|name| name == ".yb" || name == ".vibe");
            if skip {
                continue;
            }
            collect_vibe_files_recursive(&path, out)?;
            continue;
        }
        if is_supported_source_file(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn detect_mixed_extension_stem_conflicts(files: &[PathBuf]) -> Result<(), String> {
    let mut stems = std::collections::BTreeMap::<String, Vec<PathBuf>>::new();
    for file in files {
        let parent = file
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let stem = file
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("invalid source filename `{}`", file.display()))?;
        let key = format!("{}::{}", parent.display(), stem);
        stems.entry(key).or_default().push(file.clone());
    }

    for (_key, mut same_stem_files) in stems {
        same_stem_files.sort();
        let mut exts = same_stem_files
            .iter()
            .filter_map(|p| p.extension().and_then(|e| e.to_str()).map(str::to_string))
            .collect::<Vec<_>>();
        exts.sort();
        exts.dedup();
        if exts.len() > 1 {
            let listed = same_stem_files
                .iter()
                .map(|p| format!("`{}`", p.display()))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "conflicting source files share a stem across extensions: {listed}. \
rename one file or keep a single extension per stem to avoid artifact collisions."
            ));
        }
    }
    Ok(())
}

fn supported_source_ext_display() -> String {
    SUPPORTED_SOURCE_EXTS
        .iter()
        .map(|ext| format!(".{ext}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn legacy_warning_enabled() -> bool {
    if let Ok(raw) = env::var("VIBE_WARN_LEGACY_EXT") {
        let normalized = raw.trim().to_ascii_lowercase();
        return normalized == "1" || normalized == "true" || normalized == "on";
    }
    false
}

fn maybe_warn_legacy_sources(files: &[PathBuf]) {
    if !legacy_warning_enabled() {
        return;
    }
    let mut warned = BTreeSet::new();
    for file in files {
        let Some(ext) = file.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        if ext == "vibe" {
            warned.insert(file.display().to_string());
        }
    }
    if !warned.is_empty() {
        eprintln!("warning: legacy `.vibe` source detected in canonical `.yb` mode:");
        for path in warned {
            eprintln!("  - {path}");
        }
        eprintln!("set extension to `.yb` for new source files; `.vibe` remains supported during migration.");
    }
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
    out.push_str(&format!(
        "source={}\n",
        normalize_source_for_debug_map(source_path)
    ));
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
