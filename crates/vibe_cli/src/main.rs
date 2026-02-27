mod deterministic_utils;
mod example_runner;
mod module_resolver;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::{env, fs};

use std::collections::BTreeMap;

use vibe_codegen::{emit_object_with_types, CodegenOptions};
use vibe_diagnostics::Diagnostic;
use vibe_diagnostics::Diagnostics;
use vibe_doc::{extract_docs, render_markdown};
use vibe_fmt::{format_source, needs_formatting};
use vibe_indexer::build_file_index;
use vibe_indexer::{
    default_metadata_root, is_supported_source_file, prepare_index_root, IncrementalIndexer,
    IncrementalTelemetry, IndexStats, IndexStore, SUPPORTED_SOURCE_EXTS,
};
use vibe_lsp::{run_lsp_stdio, TransportMode};
use vibe_mir::MirProgram;
use vibe_mir::{lower_hir_to_mir, mir_debug_dump};
use vibe_parser::parse_source;
use vibe_pkg::{
    audit_project, default_mirror_root, default_registry_root, install_project, publish_project,
    resolve_project, semver_delta, upgrade_plan, write_lockfile, SemverDelta, LOCK_FILENAME,
    MANIFEST_FILENAME,
};
use vibe_runtime::{compile_runtime_object, link_executable, RuntimeBuildOptions};
use vibe_sidecar::models::FindingSeverity;
use vibe_sidecar::SidecarMode;
use vibe_sidecar::{BudgetPolicy, IntentLintRequest, SidecarService};
use vibe_types::{check_and_lower, type_kind_to_codegen_str};

use crate::example_runner::{run_examples_with_policy, ExampleRunSummary};
use crate::module_resolver::resolve_compilation_unit;

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
            if let Some(compile_phase_report_path) = &artifacts.compile_phase_report_path {
                println!(
                    "built {} (object: {}, runtime: {}, debug-map: {}, unsafe-audit: {}, alloc-profile: {}, compile-phases: {})",
                    artifacts.binary_path.display(),
                    artifacts.object_path.display(),
                    artifacts.runtime_object_path.display(),
                    artifacts.debug_map_path.display(),
                    artifacts.unsafe_audit_path.display(),
                    artifacts.alloc_profile_path.display(),
                    compile_phase_report_path.display()
                );
            } else {
                println!(
                    "built {} (object: {}, runtime: {}, debug-map: {}, unsafe-audit: {}, alloc-profile: {})",
                    artifacts.binary_path.display(),
                    artifacts.object_path.display(),
                    artifacts.runtime_object_path.display(),
                    artifacts.debug_map_path.display(),
                    artifacts.unsafe_audit_path.display(),
                    artifacts.alloc_profile_path.display()
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        "run" => {
            let build_args = parse_build_like_args(&args, true)?;
            if build_args.emit_obj_only {
                return Err("`--emit-obj-only` is not valid for `vibe run`".to_string());
            }
            ensure_runnable_entry_has_main(&build_args.source_path)?;
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
            let test_args = parse_test_args(&args)?;
            if test_args.build.emit_obj_only {
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
  test <path|dir> [flags]   Run fixture-aware test flow (filter/shard/report)
  index [path] [flags]      Build/update semantic index
  lsp [--index-root <dir>] [--transport legacy|jsonrpc]
                            Start LSP server over selected stdio transport
  fmt [path] [flags]        Format source files
  doc [path] [flags]        Generate markdown API docs
  new <name> [flags]        Scaffold app/service/cli/library project
  pkg <resolve|lock|install|publish|audit|upgrade-plan|semver-check> [flags]
                            Dependency lifecycle + registry + audit + semver tools
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
  vibe build <path> [--profile dev|release] [--target <triple>] [--debuginfo none|line|full] [--offline] [--locked] [--emit-obj-only] [--emit-compile-phases]

FLAGS
  --profile <name>          Build profile (dev|release)
  --target <triple>         Target triple for codegen/runtime
  --debuginfo <mode>        Debug info level (none|line|full)
  --offline                 Informational offline mode flag
  --locked                  Enforce lockfile/manifest locked-mode checks
  --emit-obj-only           Skip runtime compile/link and emit object-only artifacts
  --emit-compile-phases     Emit compile phase timing report artifact (`*.compile_phases.json`)
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
  `--emit-compile-phases` is valid and opt-in.
"#,
        ),
        "test" => Some(
            r#"vibe test

USAGE
  vibe test <path|dir> [--profile dev|release] [--target <triple>] [--debuginfo none|line|full] [--offline] [--locked] [--emit-compile-phases] [--filter <substr>] [--shard <index>/<total>] [--report text|json|--json]

DESCRIPTION
  Runs file/directory tests, including contract/example checks where applicable.
  Supports large-suite filtering/sharding and structured JSON reports.

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
  vibe lsp [--index-root <dir>] [--transport legacy|jsonrpc]

DESCRIPTION
  Starts language-server protocol endpoint.

FLAGS
  --transport <mode>        Transport mode (legacy|jsonrpc), default: jsonrpc
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
  vibe new <name> [--path <dir>] [--app|--service|--cli|--lib] [--ext yb|vibe]

FLAGS
  --app                     Scaffold application template (default)
  --service                 Scaffold multi-module service template
  --cli                     Scaffold multi-module CLI template
  --lib, --library          Scaffold library template
  --ext <ext>               Source extension for generated template
"#,
        ),
        "pkg" => Some(
            r#"vibe pkg

USAGE
  vibe pkg <resolve|lock|install|publish|audit|upgrade-plan|semver-check> [flags]

SUBCOMMANDS
  resolve                   Resolve dependency graph
  lock                      Resolve and write lockfile
  install                   Resolve and install dependencies
  publish                   Publish local package into registry layout/index
  audit                     Run vulnerability/license policy checks
  upgrade-plan              Show dependency upgrade guidance from mirror
  semver-check              Classify version delta (patch/minor/major)

COMMON FLAGS
  --path <dir>              Project root (default `.`)
  --mirror <dir>            Mirror root for resolve/install/audit/upgrade-plan
  --registry <dir>          Registry root for publish
  --policy <file>           Audit policy TOML (license denylist)
  --advisory-db <file>      Advisory DB TOML (vulnerability entries)
  --current <ver>           Current version for semver-check
  --next <ver>              Next version for semver-check
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
    let unit = resolve_compilation_unit(Path::new(path))?;
    let checked = check_and_lower(&unit.ast);
    let mut merged_diags = unit.diagnostics.clone().into_sorted();
    merged_diags.extend(checked.diagnostics.clone().into_sorted());
    let mut all = Diagnostics::default();
    all.extend(merged_diags.clone());
    if let Err(err) = best_effort_refresh_index(
        Path::new(path),
        &unit.source,
        &unit.ast,
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
    let unit = resolve_compilation_unit(Path::new(path))?;
    println!("{:#?}", unit.ast);
    let out = unit.diagnostics.to_golden();
    if !out.trim().is_empty() {
        eprintln!("{out}");
    }
    Ok(if unit.diagnostics.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn run_hir(path: &str) -> Result<ExitCode, String> {
    let unit = resolve_compilation_unit(Path::new(path))?;
    let checked = check_and_lower(&unit.ast);
    println!("{:#?}", checked.hir);
    let mut all = Diagnostics::default();
    all.extend(unit.diagnostics.into_sorted());
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
    let unit = resolve_compilation_unit(Path::new(path))?;
    let checked = check_and_lower(&unit.ast);
    let mut all = Diagnostics::default();
    all.extend(unit.diagnostics.clone().into_sorted());
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
    emit_compile_phases: bool,
    exec_args: Vec<String>,
}

#[derive(Debug, Clone)]
struct BuildArtifacts {
    object_path: PathBuf,
    runtime_object_path: PathBuf,
    debug_map_path: PathBuf,
    unsafe_audit_path: PathBuf,
    alloc_profile_path: PathBuf,
    compile_phase_report_path: Option<PathBuf>,
    binary_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestReportFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TestShard {
    index: usize,
    total: usize,
}

#[derive(Debug, Clone)]
struct TestArgs {
    build: BuildArgs,
    filter: Option<String>,
    shard: Option<TestShard>,
    report: TestReportFormat,
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
    Service,
    Cli,
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
    Publish,
    Audit,
    UpgradePlan,
    SemverCheck,
}

#[derive(Debug, Clone)]
struct PkgArgs {
    command: PkgCommand,
    project_root: PathBuf,
    mirror_root: Option<PathBuf>,
    registry_root: Option<PathBuf>,
    policy_path: Option<PathBuf>,
    advisory_db_path: Option<PathBuf>,
    current_version: Option<String>,
    next_version: Option<String>,
}

fn parse_build_like_args(args: &[String], allow_exec_args: bool) -> Result<BuildArgs, String> {
    if args.is_empty() {
        return Err("missing source path".to_string());
    }
    let mut idx = 0usize;
    let source_path = PathBuf::from(&args[idx]);
    idx += 1;

    let mut profile = "dev".to_string();
    let mut target = default_build_target().to_string();
    let mut debuginfo = "line".to_string();
    let mut offline = false;
    let mut locked = false;
    let mut emit_obj_only = false;
    let mut emit_compile_phases = false;
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
            "--emit-compile-phases" => {
                emit_compile_phases = true;
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
        emit_compile_phases,
        exec_args,
    })
}

fn parse_test_args(args: &[String]) -> Result<TestArgs, String> {
    if args.is_empty() {
        return Err("missing source path".to_string());
    }
    let mut idx = 0usize;
    let source_path = PathBuf::from(&args[idx]);
    idx += 1;

    let mut profile = "dev".to_string();
    let mut target = default_build_target().to_string();
    let mut debuginfo = "line".to_string();
    let mut offline = false;
    let mut locked = false;
    let mut emit_obj_only = false;
    let mut emit_compile_phases = false;
    let mut filter = None;
    let mut shard = None;
    let mut report = TestReportFormat::Text;

    while idx < args.len() {
        match args[idx].as_str() {
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
            "--emit-compile-phases" => {
                emit_compile_phases = true;
            }
            "--filter" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--filter`".to_string())?;
                if val.trim().is_empty() {
                    return Err("`--filter` value cannot be empty".to_string());
                }
                filter = Some(val.clone());
            }
            "--shard" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--shard`".to_string())?;
                shard = Some(parse_shard_spec(val)?);
            }
            "--report" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--report`".to_string())?;
                report = match val.as_str() {
                    "text" => TestReportFormat::Text,
                    "json" => TestReportFormat::Json,
                    _ => {
                        return Err(format!(
                            "unsupported report format `{val}` (expected text|json)"
                        ))
                    }
                };
            }
            "--json" => {
                report = TestReportFormat::Json;
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
        idx += 1;
    }

    Ok(TestArgs {
        build: BuildArgs {
            source_path,
            profile,
            target,
            debuginfo,
            offline,
            locked,
            emit_obj_only,
            emit_compile_phases,
            exec_args: Vec::new(),
        },
        filter,
        shard,
        report,
    })
}

fn parse_shard_spec(raw: &str) -> Result<TestShard, String> {
    let parts = raw.split('/').collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err(format!(
            "invalid shard spec `{raw}` (expected <index>/<total>, e.g. 1/3)"
        ));
    }
    let index = parts[0]
        .parse::<usize>()
        .map_err(|e| format!("invalid shard index `{}`: {e}", parts[0]))?;
    let total = parts[1]
        .parse::<usize>()
        .map_err(|e| format!("invalid shard total `{}`: {e}", parts[1]))?;
    if total == 0 {
        return Err("shard total must be greater than 0".to_string());
    }
    if index == 0 || index > total {
        return Err(format!(
            "shard index must be in range 1..={total}, got {index}"
        ));
    }
    Ok(TestShard {
        index: index - 1,
        total,
    })
}

fn default_build_target() -> &'static str {
    match (env::consts::ARCH, env::consts::OS) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        _ => "x86_64-unknown-linux-gnu",
    }
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
            "--lib" | "--library" => template = NewTemplate::Lib,
            "--app" => template = NewTemplate::App,
            "--service" => template = NewTemplate::Service,
            "--cli" => template = NewTemplate::Cli,
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
        return Err(
            "missing pkg subcommand (expected resolve|lock|install|publish|audit|upgrade-plan|semver-check)"
                .to_string(),
        );
    };
    let command = match first.as_str() {
        "resolve" => PkgCommand::Resolve,
        "lock" => PkgCommand::Lock,
        "install" => PkgCommand::Install,
        "publish" => PkgCommand::Publish,
        "audit" => PkgCommand::Audit,
        "upgrade-plan" => PkgCommand::UpgradePlan,
        "semver-check" => PkgCommand::SemverCheck,
        other => return Err(format!("unknown pkg subcommand `{other}`")),
    };

    let mut idx = 1usize;
    let mut project_root = PathBuf::from(".");
    let mut mirror_root = None;
    let mut registry_root = None;
    let mut policy_path = None;
    let mut advisory_db_path = None;
    let mut current_version = None;
    let mut next_version = None;
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
            "--registry" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--registry`".to_string())?;
                registry_root = Some(PathBuf::from(val));
            }
            "--policy" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--policy`".to_string())?;
                policy_path = Some(PathBuf::from(val));
            }
            "--advisory-db" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--advisory-db`".to_string())?;
                advisory_db_path = Some(PathBuf::from(val));
            }
            "--current" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--current`".to_string())?;
                current_version = Some(val.clone());
            }
            "--next" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--next`".to_string())?;
                next_version = Some(val.clone());
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
        idx += 1;
    }

    Ok(PkgArgs {
        command,
        project_root,
        mirror_root,
        registry_root,
        policy_path,
        advisory_db_path,
        current_version,
        next_version,
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
        telemetry.incremental_update_latency_ms = report.incremental_update_latency_ms;
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
    println!("{}", telemetry.summary_line());
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
    let mut transport = TransportMode::JsonRpc;
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
            "--transport" => {
                idx += 1;
                let val = args
                    .get(idx)
                    .ok_or_else(|| "missing value for `--transport`".to_string())?;
                transport = match val.as_str() {
                    "legacy" => TransportMode::Legacy,
                    "jsonrpc" => TransportMode::JsonRpc,
                    _ => {
                        return Err(format!(
                            "invalid `--transport` value `{val}` (expected legacy|jsonrpc)"
                        ))
                    }
                };
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
        idx += 1;
    }
    run_lsp_stdio(index_root, transport)?;
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
    let module_ns = sanitize_module_ident(&args.name);
    let mut sources = Vec::<(PathBuf, String)>::new();
    let (primary_source, readme_header, readme_hint) = match args.template {
        NewTemplate::App => {
            let main = project_dir.join(format!("main.{}", args.extension));
            sources.push((
                main.clone(),
                r#"pub main() -> Int {
  @effect io
  println("hello from vibelang")
  0
}
"#
                .to_string(),
            ));
            (
                main,
                "Application template.",
                format!("Run app: `vibe run main.{}`", args.extension),
            )
        }
        NewTemplate::Lib => {
            let lib = project_dir.join(format!("lib.{}", args.extension));
            sources.push((
                lib.clone(),
                r#"pub add(a: Int, b: Int) -> Int {
  a + b
}
"#
                .to_string(),
            ));
            (
                lib,
                "Library template.",
                format!("Build checks: `vibe check lib.{}`", args.extension),
            )
        }
        NewTemplate::Service => {
            let module_dir = project_dir.join(&module_ns);
            let main = module_dir.join(format!("main.{}", args.extension));
            let router = module_dir.join(format!("router.{}", args.extension));
            sources.push((
                main.clone(),
                format!(
                    r#"module {module}.main
import {module}.router

pub main() -> Int {{
  @effect io
  println("service starting")
  route()
}}
"#,
                    module = module_ns
                ),
            ));
            sources.push((
                router,
                format!(
                    r#"module {module}.router

pub route() -> Int {{
  0
}}
"#,
                    module = module_ns
                ),
            ));
            let hint_path = format!("{module_ns}/main.{}", args.extension);
            (
                main,
                "Service template (multi-module).",
                format!("Run service: `vibe run {hint_path}`"),
            )
        }
        NewTemplate::Cli => {
            let module_dir = project_dir.join(&module_ns);
            let main = module_dir.join(format!("main.{}", args.extension));
            let commands = module_dir.join(format!("commands.{}", args.extension));
            sources.push((
                main.clone(),
                format!(
                    r#"module {module}.main
import {module}.commands

pub main() -> Int {{
  @effect io
  println("cli ready")
  run_command()
}}
"#,
                    module = module_ns
                ),
            ));
            sources.push((
                commands,
                format!(
                    r#"module {module}.commands

pub run_command() -> Int {{
  0
}}
"#,
                    module = module_ns
                ),
            ));
            let hint_path = format!("{module_ns}/main.{}", args.extension);
            (
                main,
                "CLI template (multi-module).",
                format!("Run CLI: `vibe run {hint_path}`"),
            )
        }
    };
    for (path, source) in &sources {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "failed to create source directory `{}`: {e}",
                    parent.display()
                )
            })?;
        }
        fs::write(path, source)
            .map_err(|e| format!("failed to write source template `{}`: {e}", path.display()))?;
    }

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
    let source_hint = primary_source
        .strip_prefix(&project_dir)
        .unwrap_or(primary_source.as_path())
        .to_string_lossy()
        .replace('\\', "/");
    let readme = format!(
        "# {}\n\n{}\n\n- Primary source: `{}`\n- {}\n",
        args.name, readme_header, source_hint, readme_hint
    );
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

fn sanitize_module_ident(raw: &str) -> String {
    let mut out = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out = out.trim_matches('_').to_string();
    if out.is_empty() {
        return "app".to_string();
    }
    if out.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        return format!("pkg_{out}");
    }
    out
}

fn run_pkg(args: &PkgArgs) -> Result<ExitCode, String> {
    match args.command {
        PkgCommand::Resolve => {
            let mirror_root = args
                .mirror_root
                .clone()
                .unwrap_or_else(|| default_mirror_root(&args.project_root));
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
            let mirror_root = args
                .mirror_root
                .clone()
                .unwrap_or_else(|| default_mirror_root(&args.project_root));
            let resolution = resolve_project(&args.project_root, &mirror_root)?;
            let lock = write_lockfile(&args.project_root, &resolution)?;
            println!(
                "wrote deterministic lockfile `{}` (packages={})",
                lock.display(),
                resolution.packages.len()
            );
        }
        PkgCommand::Install => {
            let mirror_root = args
                .mirror_root
                .clone()
                .unwrap_or_else(|| default_mirror_root(&args.project_root));
            let report = install_project(&args.project_root, &mirror_root)?;
            println!(
                "install complete: packages={} lock={} store={}",
                report.installed,
                report.lock_path.display(),
                report.store_root.display()
            );
        }
        PkgCommand::Publish => {
            let registry_root = args
                .registry_root
                .clone()
                .unwrap_or_else(|| default_registry_root(&args.project_root));
            let report = publish_project(&args.project_root, &registry_root)?;
            println!(
                "published {} v{} to {} (index={})",
                report.package,
                report.version,
                report.published_dir.display(),
                report.index_path.display()
            );
        }
        PkgCommand::Audit => {
            let mirror_root = args
                .mirror_root
                .clone()
                .unwrap_or_else(|| default_mirror_root(&args.project_root));
            let report = audit_project(
                &args.project_root,
                &mirror_root,
                args.policy_path.as_deref(),
                args.advisory_db_path.as_deref(),
            )?;
            println!(
                "audit summary: scanned={} findings={}",
                report.scanned,
                report.findings.len()
            );
            for finding in &report.findings {
                println!(
                    "  - {}: {} {}: {}",
                    finding.kind, finding.package, finding.version, finding.detail
                );
            }
            if !report.findings.is_empty() {
                return Ok(ExitCode::from(1));
            }
        }
        PkgCommand::UpgradePlan => {
            let mirror_root = args
                .mirror_root
                .clone()
                .unwrap_or_else(|| default_mirror_root(&args.project_root));
            let plan = upgrade_plan(&args.project_root, &mirror_root)?;
            println!("upgrade plan: entries={}", plan.entries.len());
            for entry in plan.entries {
                println!(
                    "  - {} current={} latest_compatible={} latest_available={} manifest_change={}",
                    entry.package,
                    entry.current,
                    entry.latest_compatible,
                    entry.latest_available,
                    if entry.requires_manifest_change {
                        "yes"
                    } else {
                        "no"
                    }
                );
            }
        }
        PkgCommand::SemverCheck => {
            let current = args.current_version.as_deref().ok_or_else(|| {
                "`vibe pkg semver-check` requires `--current <version>`".to_string()
            })?;
            let next = args
                .next_version
                .as_deref()
                .ok_or_else(|| "`vibe pkg semver-check` requires `--next <version>`".to_string())?;
            let delta = semver_delta(current, next)?;
            let class = match delta {
                SemverDelta::Patch => "patch",
                SemverDelta::Minor => "minor",
                SemverDelta::Major => "major",
                SemverDelta::Unchanged => "unchanged",
            };
            println!("semver delta: {current} -> {next} ({class})");
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn build_source(args: &BuildArgs) -> Result<BuildArtifacts, String> {
    let timing_enabled = args.emit_compile_phases;
    let total_start = if timing_enabled {
        Some(std::time::Instant::now())
    } else {
        None
    };
    let locked_start = if timing_enabled {
        Some(std::time::Instant::now())
    } else {
        None
    };
    if args.locked {
        enforce_locked_mode(&args.source_path)?;
    }
    let locked_ms = locked_start
        .as_ref()
        .map(|start| start.elapsed().as_millis())
        .unwrap_or(0);

    let resolve_start = if timing_enabled {
        Some(std::time::Instant::now())
    } else {
        None
    };
    let unit = resolve_compilation_unit(&args.source_path)?;
    let resolve_ms = resolve_start
        .as_ref()
        .map(|start| start.elapsed().as_millis())
        .unwrap_or(0);

    let check_start = if timing_enabled {
        Some(std::time::Instant::now())
    } else {
        None
    };
    let checked = check_and_lower(&unit.ast);
    let check_ms = check_start
        .as_ref()
        .map(|start| start.elapsed().as_millis())
        .unwrap_or(0);

    let diagnostics_start = if timing_enabled {
        Some(std::time::Instant::now())
    } else {
        None
    };
    let mut all = Diagnostics::default();
    all.extend(unit.diagnostics.clone().into_sorted());
    all.extend(checked.diagnostics.into_sorted());
    let diags = all.to_golden();
    let diagnostics_ms = diagnostics_start
        .as_ref()
        .map(|start| start.elapsed().as_millis())
        .unwrap_or(0);
    if !diags.trim().is_empty() {
        eprintln!("{diags}");
    }
    if all.has_errors() {
        return Err("build failed due to errors".to_string());
    }

    let contract_preflight_start = if timing_enabled {
        Some(std::time::Instant::now())
    } else {
        None
    };
    enforce_contract_preflight(&unit.ast, &args.source_path, &args.profile)?;
    let contract_preflight_ms = contract_preflight_start
        .as_ref()
        .map(|start| start.elapsed().as_millis())
        .unwrap_or(0);

    let mir_lower_start = if timing_enabled {
        Some(std::time::Instant::now())
    } else {
        None
    };
    let mir =
        lower_hir_to_mir(&checked.hir).map_err(|e| format!("HIR->MIR lowering failed: {e}"))?;
    let mir_lower_ms = mir_lower_start
        .as_ref()
        .map(|start| start.elapsed().as_millis())
        .unwrap_or(0);

    let codegen_start = if timing_enabled {
        Some(std::time::Instant::now())
    } else {
        None
    };
    let type_defs: BTreeMap<String, Vec<(String, String)>> = checked
        .type_defs
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                v.iter()
                    .map(|(f, t)| (f.clone(), type_kind_to_codegen_str(t)))
                    .collect(),
            )
        })
        .collect();
    let object_bytes = emit_object_with_types(
        &mir,
        &CodegenOptions {
            target: args.target.clone(),
            profile: args.profile.clone(),
            debuginfo: args.debuginfo.clone(),
        },
        &type_defs,
        &checked.enum_defs,
    )
    .map_err(|e| format!("codegen failed: {e}"))?;
    let codegen_ms = codegen_start
        .as_ref()
        .map(|start| start.elapsed().as_millis())
        .unwrap_or(0);

    if args.offline {
        // Phase 2 keeps AI and network out of the compile path. This flag is currently informational.
    }

    let io_prepare_start = if timing_enabled {
        Some(std::time::Instant::now())
    } else {
        None
    };
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
    let io_prepare_ms = io_prepare_start
        .as_ref()
        .map(|start| start.elapsed().as_millis())
        .unwrap_or(0);

    let object_write_start = if timing_enabled {
        Some(std::time::Instant::now())
    } else {
        None
    };
    fs::write(&object_path, object_bytes)
        .map_err(|e| format!("failed to write object `{}`: {e}", object_path.display()))?;
    let object_write_ms = object_write_start
        .as_ref()
        .map(|start| start.elapsed().as_millis())
        .unwrap_or(0);

    let runtime_options = RuntimeBuildOptions {
        target: args.target.clone(),
        profile: args.profile.clone(),
        debuginfo: args.debuginfo.clone(),
    };
    let mut runtime_compile_ms = 0u128;
    let mut link_ms = 0u128;
    let mut obj_only_stub_write_ms = 0u128;
    let (runtime_object_path, binary_path) = if args.emit_obj_only {
        let stub_write_start = if timing_enabled {
            Some(std::time::Instant::now())
        } else {
            None
        };
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
        obj_only_stub_write_ms = stub_write_start
            .as_ref()
            .map(|start| start.elapsed().as_millis())
            .unwrap_or(0);
        (runtime_stub, binary_stub)
    } else {
        let runtime_compile_start = if timing_enabled {
            Some(std::time::Instant::now())
        } else {
            None
        };
        let runtime_object_path = compile_runtime_object(&artifacts_dir, &runtime_options)?;
        runtime_compile_ms = runtime_compile_start
            .as_ref()
            .map(|start| start.elapsed().as_millis())
            .unwrap_or(0);
        let link_start = if timing_enabled {
            Some(std::time::Instant::now())
        } else {
            None
        };
        link_executable(
            &object_path,
            &runtime_object_path,
            &binary_path,
            &runtime_options,
        )?;
        link_ms = link_start
            .as_ref()
            .map(|start| start.elapsed().as_millis())
            .unwrap_or(0);
        (runtime_object_path, binary_path)
    };
    let aux_artifacts_start = if timing_enabled {
        Some(std::time::Instant::now())
    } else {
        None
    };
    let debug_map_path = write_debug_map(&artifacts_dir, &args.source_path, args, &mir, stem)
        .map_err(|e| {
            format!(
                "failed to write debug map for `{}`: {e}",
                args.source_path.display()
            )
        })?;
    let unsafe_audit_path = write_unsafe_audit_report(&artifacts_dir, &args.source_path, stem)
        .map_err(|e| {
            format!(
                "failed to write unsafe audit report for `{}`: {e}",
                args.source_path.display()
            )
        })?;
    let alloc_profile_path =
        write_alloc_profile(&artifacts_dir, &args.source_path, &checked.hir, stem).map_err(
            |e| {
                format!(
                    "failed to write allocation profile for `{}`: {e}",
                    args.source_path.display()
                )
            },
        )?;
    let aux_artifacts_ms = aux_artifacts_start
        .as_ref()
        .map(|start| start.elapsed().as_millis())
        .unwrap_or(0);

    let compile_phase_report_path = if timing_enabled {
        let path = artifacts_dir.join(format!("{stem}.compile_phases.json"));
        let total_ms = total_start
            .as_ref()
            .map(|start| start.elapsed().as_millis())
            .unwrap_or(0);
        let report_json = format!(
            concat!(
                "{{\n",
                "  \"format\": \"vibe-compiler-phase-timing-v1\",\n",
                "  \"source\": \"{}\",\n",
                "  \"profile\": \"{}\",\n",
                "  \"target\": \"{}\",\n",
                "  \"debuginfo\": \"{}\",\n",
                "  \"emit_obj_only\": {},\n",
                "  \"phases_ms\": {{\n",
                "    \"locked_mode\": {},\n",
                "    \"resolve_compilation_unit\": {},\n",
                "    \"check_and_lower\": {},\n",
                "    \"diagnostics_merge\": {},\n",
                "    \"contract_preflight\": {},\n",
                "    \"hir_to_mir_lower\": {},\n",
                "    \"codegen_emit_object\": {},\n",
                "    \"artifact_io_prepare\": {},\n",
                "    \"object_write\": {},\n",
                "    \"runtime_compile\": {},\n",
                "    \"link\": {},\n",
                "    \"obj_only_stub_write\": {},\n",
                "    \"auxiliary_reports\": {},\n",
                "    \"total\": {}\n",
                "  }}\n",
                "}}\n"
            ),
            json_escape(&args.source_path.display().to_string()),
            json_escape(&args.profile),
            json_escape(&args.target),
            json_escape(&args.debuginfo),
            args.emit_obj_only,
            locked_ms,
            resolve_ms,
            check_ms,
            diagnostics_ms,
            contract_preflight_ms,
            mir_lower_ms,
            codegen_ms,
            io_prepare_ms,
            object_write_ms,
            runtime_compile_ms,
            link_ms,
            obj_only_stub_write_ms,
            aux_artifacts_ms,
            total_ms
        );
        fs::write(&path, report_json).map_err(|e| {
            format!(
                "failed to write compile phase report `{}`: {e}",
                path.display()
            )
        })?;
        Some(path)
    } else {
        None
    };

    Ok(BuildArtifacts {
        object_path,
        runtime_object_path,
        debug_map_path,
        unsafe_audit_path,
        alloc_profile_path,
        compile_phase_report_path,
        binary_path,
    })
}

fn build_index_for_file(file: &Path) -> Result<vibe_indexer::FileIndex, String> {
    let canonical = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
    let unit = resolve_compilation_unit(&canonical)?;
    let checked = check_and_lower(&unit.ast);
    let mut diagnostics = unit.diagnostics.into_sorted();
    diagnostics.extend(checked.diagnostics.into_sorted());
    Ok(build_file_index(
        &canonical,
        &unit.source,
        &unit.ast,
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
    if current.as_os_str().is_empty() {
        current = PathBuf::from(".");
    }
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
    let mut candidates = vec![source_path.to_path_buf()];
    if let Ok(canonical) = source_path.canonicalize() {
        if canonical != source_path {
            candidates.push(canonical);
        }
    }

    for candidate in &candidates {
        if let Some(project_root) = find_project_root(candidate) {
            if let Ok(rel) = candidate.strip_prefix(&project_root) {
                let normalized = rel.to_string_lossy().replace('\\', "/");
                if !normalized.is_empty() {
                    return format!("project://{normalized}");
                }
            }
        }
    }

    let file_name = candidates
        .iter()
        .filter_map(|path| path.file_name().and_then(|s| s.to_str()))
        .find(|name| !name.is_empty())
        .unwrap_or("source");
    format!("external://{file_name}")
}

fn run_test(args: &TestArgs) -> Result<ExitCode, String> {
    if args.build.locked {
        enforce_locked_mode(&args.build.source_path)?;
    }
    let start = std::time::Instant::now();
    let mut files = collect_vibe_files(&args.build.source_path)?;
    if files.is_empty() {
        return Err(format!(
            "no source files ({}) found under `{}`",
            supported_source_ext_display(),
            args.build.source_path.display()
        ));
    }
    let discovered_files = files.len();
    if let Some(filter) = &args.filter {
        files.retain(|file| file.to_string_lossy().contains(filter));
    }
    if let Some(shard) = args.shard {
        files = files
            .into_iter()
            .enumerate()
            .filter_map(|(idx, file)| {
                if idx % shard.total == shard.index {
                    Some(file)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
    }
    if files.is_empty() {
        return Err(format!(
            "no source files selected after applying filter/shard on `{}`",
            args.build.source_path.display()
        ));
    }
    let selected_files = files.len();

    let mut compile_failures = 0usize;
    let mut examples = ExampleRunSummary::default();
    let mut main_run_total = 0usize;
    let mut main_run_failures = 0usize;
    let enforce_contract_checks = contract_checks_enabled(&args.build.profile);

    for file in files {
        let unit = resolve_compilation_unit(&file)?;
        let checked = check_and_lower(&unit.ast);
        let mut all = Diagnostics::default();
        all.extend(unit.diagnostics.clone().into_sorted());
        all.extend(checked.diagnostics.clone().into_sorted());

        let diag_out = all.to_golden();
        if !diag_out.trim().is_empty() {
            eprintln!("{}:\n{diag_out}", file.display());
        }
        if all.has_errors() {
            compile_failures += 1;
            continue;
        }

        let current_examples = run_examples_with_policy(&unit.ast, enforce_contract_checks);
        examples.total += current_examples.total;
        examples.passed += current_examples.passed;
        examples.failed += current_examples.failed;
        examples.failures.extend(current_examples.failures);

        if has_main_function(&unit.ast) {
            main_run_total += 1;
            let single_file_args = BuildArgs {
                source_path: file.clone(),
                profile: args.build.profile.clone(),
                target: args.build.target.clone(),
                debuginfo: args.build.debuginfo.clone(),
                offline: args.build.offline,
                locked: args.build.locked,
                emit_obj_only: false,
                emit_compile_phases: args.build.emit_compile_phases,
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
    let duration_ms = start.elapsed().as_millis();
    match args.report {
        TestReportFormat::Text => {
            println!(
                "test summary: files_discovered={} files_selected={} compile_failures={} examples={} passed={} failed={} mains={} main_failures={} contract_checks={} filter={} shard={} duration_ms={}",
                discovered_files,
                selected_files,
                compile_failures,
                examples.total,
                examples.passed,
                examples.failed,
                main_run_total,
                main_run_failures,
                if enforce_contract_checks { "on" } else { "off" },
                args.filter.as_deref().unwrap_or("-"),
                args.shard
                    .map(|shard| format!("{}/{}", shard.index + 1, shard.total))
                    .unwrap_or_else(|| "-".to_string()),
                duration_ms
            );
        }
        TestReportFormat::Json => {
            println!(
                "{}",
                render_test_summary_json(
                    discovered_files,
                    selected_files,
                    compile_failures,
                    &examples,
                    main_run_total,
                    main_run_failures,
                    enforce_contract_checks,
                    args.filter.as_deref(),
                    args.shard,
                    duration_ms,
                )
            );
        }
    }

    if compile_failures > 0 || examples.failed > 0 || main_run_failures > 0 {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

#[allow(clippy::too_many_arguments)]
fn render_test_summary_json(
    discovered_files: usize,
    selected_files: usize,
    compile_failures: usize,
    examples: &ExampleRunSummary,
    main_run_total: usize,
    main_run_failures: usize,
    contract_checks: bool,
    filter: Option<&str>,
    shard: Option<TestShard>,
    duration_ms: u128,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"files_discovered\": {},\n", discovered_files));
    out.push_str(&format!("  \"files_selected\": {},\n", selected_files));
    out.push_str(&format!("  \"compile_failures\": {},\n", compile_failures));
    out.push_str(&format!("  \"examples_total\": {},\n", examples.total));
    out.push_str(&format!("  \"examples_passed\": {},\n", examples.passed));
    out.push_str(&format!("  \"examples_failed\": {},\n", examples.failed));
    out.push_str(&format!("  \"mains_total\": {},\n", main_run_total));
    out.push_str(&format!("  \"mains_failed\": {},\n", main_run_failures));
    out.push_str(&format!(
        "  \"contract_checks\": \"{}\",\n",
        if contract_checks { "on" } else { "off" }
    ));
    match filter {
        Some(value) => out.push_str(&format!("  \"filter\": \"{}\",\n", json_escape(value))),
        None => out.push_str("  \"filter\": null,\n"),
    }
    match shard {
        Some(shard) => out.push_str(&format!(
            "  \"shard\": \"{}/{}\",\n",
            shard.index + 1,
            shard.total
        )),
        None => out.push_str("  \"shard\": null,\n"),
    }
    out.push_str("  \"example_failures\": [");
    if !examples.failures.is_empty() {
        out.push('\n');
        for (idx, failure) in examples.failures.iter().enumerate() {
            let comma = if idx + 1 == examples.failures.len() {
                ""
            } else {
                ","
            };
            out.push_str(&format!("    \"{}\"{comma}\n", json_escape(failure)));
        }
        out.push_str("  ],\n");
    } else {
        out.push_str("],\n");
    }
    out.push_str(&format!("  \"duration_ms\": {}\n", duration_ms));
    out.push('}');
    out
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

fn ensure_runnable_entry_has_main(source_path: &Path) -> Result<(), String> {
    let unit = resolve_compilation_unit(source_path)?;
    if has_main_function(&unit.ast) {
        return Ok(());
    }
    Err(format!(
        "`vibe run` requires an entry source file that defines `main`; `{}` has no main function. \
run an entry file (for example `.../main.yb`) or use `vibe check {}` for module validation.",
        source_path.display(),
        source_path.display()
    ))
}

fn has_main_function(ast: &vibe_ast::FileAst) -> bool {
    ast.declarations.iter().any(|decl| {
        let vibe_ast::Declaration::Function(func) = decl else {
            return false;
        };
        func.name == "main"
    })
}

#[derive(Debug, Clone)]
struct UnsafeAuditBlock {
    file: String,
    begin_line: usize,
    end_line: usize,
    reason: String,
    review: String,
}

#[derive(Debug, Clone)]
struct PendingUnsafeBlock {
    begin_line: usize,
    reason: String,
    review: Option<String>,
}

fn comment_payload<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    let trimmed = line.trim();
    let comment = trimmed.strip_prefix("//")?.trim();
    let payload = comment.strip_prefix(marker)?.trim();
    Some(payload)
}

fn comment_contains_unsafe(line: &str) -> bool {
    let trimmed = line.trim();
    let Some(comment) = trimmed.strip_prefix("//") else {
        return false;
    };
    comment.contains("@unsafe")
}

fn source_files_for_unsafe_audit(source_path: &Path) -> Result<Vec<PathBuf>, String> {
    let target = find_project_root(source_path).unwrap_or_else(|| source_path.to_path_buf());
    let mut files = collect_vibe_files(&target)?;
    files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    files.dedup();
    Ok(files)
}

fn collect_unsafe_blocks_for_file(
    file: &Path,
    source: &str,
    blocks: &mut Vec<UnsafeAuditBlock>,
    violations: &mut Vec<String>,
) {
    let mut pending: Option<PendingUnsafeBlock> = None;
    for (idx, line) in source.lines().enumerate() {
        let line_no = idx + 1;
        if let Some(payload) = comment_payload(line, "@unsafe begin:") {
            if pending.is_some() {
                violations.push(format!(
                    "{}:{} nested `@unsafe begin` is not allowed",
                    file.display(),
                    line_no
                ));
                continue;
            }
            let reason = if payload.is_empty() {
                violations.push(format!(
                    "{}:{} `@unsafe begin:` requires a non-empty reason",
                    file.display(),
                    line_no
                ));
                "unspecified".to_string()
            } else {
                payload.to_string()
            };
            pending = Some(PendingUnsafeBlock {
                begin_line: line_no,
                reason,
                review: None,
            });
            continue;
        }

        if let Some(payload) = comment_payload(line, "@unsafe review:") {
            let Some(active) = pending.as_mut() else {
                violations.push(format!(
                    "{}:{} `@unsafe review:` appears outside an unsafe block",
                    file.display(),
                    line_no
                ));
                continue;
            };
            if payload.is_empty() {
                violations.push(format!(
                    "{}:{} `@unsafe review:` requires a non-empty review ticket/reference",
                    file.display(),
                    line_no
                ));
                continue;
            }
            active.review = Some(payload.to_string());
            continue;
        }

        if comment_payload(line, "@unsafe end").is_some() {
            let Some(active) = pending.take() else {
                violations.push(format!(
                    "{}:{} `@unsafe end` appears without matching `@unsafe begin:`",
                    file.display(),
                    line_no
                ));
                continue;
            };
            let Some(review) = active.review else {
                violations.push(format!(
                    "{}:{} unsafe block missing `@unsafe review:` between begin/end",
                    file.display(),
                    active.begin_line
                ));
                continue;
            };
            blocks.push(UnsafeAuditBlock {
                file: file.display().to_string(),
                begin_line: active.begin_line,
                end_line: line_no,
                reason: active.reason,
                review,
            });
            continue;
        }

        if comment_contains_unsafe(line) {
            violations.push(format!(
                "{}:{} unsupported unsafe marker; expected one of `@unsafe begin:`, `@unsafe review:`, or `@unsafe end`",
                file.display(),
                line_no
            ));
        }
    }
    if let Some(active) = pending.take() {
        violations.push(format!(
            "{}:{} unclosed unsafe block (missing `@unsafe end`)",
            file.display(),
            active.begin_line
        ));
    }
}

fn write_unsafe_audit_report(
    artifacts_dir: &Path,
    source_path: &Path,
    stem: &str,
) -> Result<PathBuf, String> {
    let files = source_files_for_unsafe_audit(source_path)?;
    let mut blocks = Vec::new();
    let mut violations = Vec::new();
    for file in files {
        let source = fs::read_to_string(&file)
            .map_err(|e| format!("failed to read `{}` for unsafe audit: {e}", file.display()))?;
        collect_unsafe_blocks_for_file(&file, &source, &mut blocks, &mut violations);
    }
    blocks.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.begin_line.cmp(&b.begin_line))
            .then(a.end_line.cmp(&b.end_line))
    });

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"format\": \"vibe-unsafe-audit-v1\",\n");
    out.push_str(&format!(
        "  \"source\": \"{}\",\n",
        json_escape(&normalize_source_for_debug_map(source_path))
    ));
    out.push_str("  \"blocks\": [\n");
    for (idx, block) in blocks.iter().enumerate() {
        let comma = if idx + 1 == blocks.len() { "" } else { "," };
        out.push_str(&format!(
            "    {{\"file\":\"{}\",\"begin_line\":{},\"end_line\":{},\"reason\":\"{}\",\"review\":\"{}\"}}{comma}\n",
            json_escape(&block.file),
            block.begin_line,
            block.end_line,
            json_escape(&block.reason),
            json_escape(&block.review),
        ));
    }
    out.push_str("  ],\n");
    out.push_str("  \"violations\": [\n");
    for (idx, violation) in violations.iter().enumerate() {
        let comma = if idx + 1 == violations.len() { "" } else { "," };
        out.push_str(&format!("    \"{}\"{comma}\n", json_escape(violation)));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");

    let report_path = artifacts_dir.join(format!("{stem}.unsafe.audit.json"));
    fs::write(&report_path, out)
        .map_err(|e| format!("failed to write `{}`: {e}", report_path.display()))?;

    if !violations.is_empty() {
        let mut details = String::new();
        for violation in &violations {
            details.push_str(&format!("  - {violation}\n"));
        }
        return Err(format!(
            "unsafe audit found {} violation(s); see `{}`\n{}",
            violations.len(),
            report_path.display(),
            details
        ));
    }
    Ok(report_path)
}

fn write_alloc_profile(
    artifacts_dir: &Path,
    source_path: &Path,
    hir: &vibe_hir::HirProgram,
    stem: &str,
) -> Result<PathBuf, String> {
    let mut functions = hir
        .functions
        .iter()
        .map(|function| {
            let effects_declared = function
                .effects_declared
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            let effects_observed = function
                .effects_observed
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            (
                function.name.clone(),
                effects_declared,
                effects_observed,
                function.effects_declared.contains("alloc"),
                function.effects_observed.contains("alloc"),
            )
        })
        .collect::<Vec<_>>();
    functions.sort_by(|a, b| a.0.cmp(&b.0));

    let alloc_declared_count = functions.iter().filter(|entry| entry.3).count();
    let alloc_observed_count = functions.iter().filter(|entry| entry.4).count();

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"format\": \"vibe-alloc-profile-v1\",\n");
    out.push_str(&format!(
        "  \"source\": \"{}\",\n",
        json_escape(&normalize_source_for_debug_map(source_path))
    ));
    out.push_str("  \"functions\": [\n");
    for (idx, function) in functions.iter().enumerate() {
        let comma = if idx + 1 == functions.len() { "" } else { "," };
        out.push_str("    {\n");
        out.push_str(&format!(
            "      \"name\": \"{}\",\n",
            json_escape(&function.0)
        ));
        out.push_str(&format!("      \"alloc_declared\": {},\n", function.3));
        out.push_str(&format!("      \"alloc_observed\": {},\n", function.4));
        out.push_str("      \"effects_declared\": [");
        for (eff_idx, effect) in function.1.iter().enumerate() {
            let eff_comma = if eff_idx + 1 == function.1.len() {
                ""
            } else {
                ","
            };
            out.push_str(&format!("\"{}\"{eff_comma}", json_escape(effect)));
        }
        out.push_str("],\n");
        out.push_str("      \"effects_observed\": [");
        for (eff_idx, effect) in function.2.iter().enumerate() {
            let eff_comma = if eff_idx + 1 == function.2.len() {
                ""
            } else {
                ","
            };
            out.push_str(&format!("\"{}\"{eff_comma}", json_escape(effect)));
        }
        out.push_str("]\n");
        out.push_str(&format!("    }}{comma}\n"));
    }
    out.push_str("  ],\n");
    out.push_str("  \"summary\": {\n");
    out.push_str(&format!("    \"functions_total\": {},\n", functions.len()));
    out.push_str(&format!(
        "    \"alloc_declared_count\": {},\n",
        alloc_declared_count
    ));
    out.push_str(&format!(
        "    \"alloc_observed_count\": {}\n",
        alloc_observed_count
    ));
    out.push_str("  }\n");
    out.push_str("}\n");

    let profile_path = artifacts_dir.join(format!("{stem}.alloc.profile.json"));
    fs::write(&profile_path, out)
        .map_err(|e| format!("failed to write `{}`: {e}", profile_path.display()))?;
    Ok(profile_path)
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
