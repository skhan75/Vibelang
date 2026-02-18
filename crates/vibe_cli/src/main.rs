use std::{env, fs, process::ExitCode};

use vibe_diagnostics::Diagnostics;
use vibe_parser::parse_source;
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
            let src =
                fs::read_to_string(path).map_err(|e| format!("failed to read `{path}`: {e}"))?;
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
        "ast" => {
            let path = args.first().ok_or_else(usage)?;
            let src =
                fs::read_to_string(path).map_err(|e| format!("failed to read `{path}`: {e}"))?;
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
        "hir" => {
            let path = args.first().ok_or_else(usage)?;
            let src =
                fs::read_to_string(path).map_err(|e| format!("failed to read `{path}`: {e}"))?;
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
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage: vibe <check|ast|hir> <path>".to_string()
}
