use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::{env, fs};

use vibe_codegen::{emit_object, CodegenOptions};
use vibe_diagnostics::Diagnostics;
use vibe_mir::{lower_hir_to_mir, mir_debug_dump};
use vibe_parser::parse_source;
use vibe_runtime::{compile_runtime_object, link_executable, RuntimeBuildOptions};
use vibe_types::check_and_lower;

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
                "built {} (object: {}, runtime: {})",
                artifacts.binary_path.display(),
                artifacts.object_path.display(),
                artifacts.runtime_object_path.display()
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
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage: vibe <check|ast|hir|mir|build|run> <path> [--profile dev|release] [--target x86_64-unknown-linux-gnu] [--offline]".to_string()
}

fn run_check(path: &str) -> Result<ExitCode, String> {
    let src = fs::read_to_string(path).map_err(|e| format!("failed to read `{path}`: {e}"))?;
    let parsed = parse_source(&src);
    let checked = check_and_lower(&parsed.ast);
    let mut all = Diagnostics::default();
    all.extend(parsed.diagnostics.into_sorted());
    all.extend(checked.diagnostics.into_sorted());
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
    offline: bool,
    exec_args: Vec<String>,
}

#[derive(Debug, Clone)]
struct BuildArtifacts {
    object_path: PathBuf,
    runtime_object_path: PathBuf,
    binary_path: PathBuf,
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
        offline,
        exec_args,
    })
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
    };
    let runtime_object_path = compile_runtime_object(&artifacts_dir, &runtime_options)?;
    link_executable(
        &object_path,
        &runtime_object_path,
        &binary_path,
        &runtime_options,
    )?;

    Ok(BuildArtifacts {
        object_path,
        runtime_object_path,
        binary_path,
    })
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
