use std::process::{Command, ExitStatus};

#[test]
fn root_help_has_manual_sections() {
    let out = run_vibe(&["--help"]);
    assert!(
        out.status.success(),
        "--help should succeed\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(out.stdout.contains("VibeLang CLI Manual"));
    assert!(out.stdout.contains("USAGE"));
    assert!(out.stdout.contains("COMMANDS"));
    assert!(out.stdout.contains("GLOBAL OPTIONS"));
    assert!(out.stdout.contains("vibe --version --json"));
}

#[test]
fn per_command_help_matches_help_alias() {
    let via_alias = run_vibe(&["help", "build"]);
    let via_flag = run_vibe(&["build", "--help"]);
    assert!(
        via_alias.status.success() && via_flag.status.success(),
        "command help should succeed\nalias stdout:\n{}\nalias stderr:\n{}\nflag stdout:\n{}\nflag stderr:\n{}",
        via_alias.stdout,
        via_alias.stderr,
        via_flag.stdout,
        via_flag.stderr
    );
    assert_eq!(via_alias.stdout, via_flag.stdout);
    assert!(via_flag.stdout.contains("vibe build"));
    assert!(via_flag.stdout.contains("--locked"));
}

#[test]
fn unknown_command_help_returns_usage_error() {
    let out = run_vibe(&["help", "unknown_cmd"]);
    assert!(
        !out.status.success(),
        "unknown command help should fail\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(out.stderr.contains("unknown command `unknown_cmd`"));
    assert!(out.stderr.contains("usage: vibe <command> [options]"));
}

#[test]
fn lint_help_mentions_intent_mode_requirement() {
    let out = run_vibe(&["lint", "--help"]);
    assert!(
        out.status.success(),
        "lint --help should succeed\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(out.stdout.contains("--intent"));
    assert!(out
        .stdout
        .contains("Current lint mode requires `--intent`."));
}

fn run_vibe(args: &[&str]) -> CmdOutput {
    let output = Command::new(vibe_bin())
        .args(args)
        .output()
        .expect("run vibe command");
    CmdOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn vibe_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vibe")
}

struct CmdOutput {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}
