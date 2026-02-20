use std::fs;
use std::path::PathBuf;
use std::process::Command;

use vibe_fmt::format_source;

#[test]
fn host_formatter_matches_selfhost_fixture_outputs() {
    for fixture in fixture_names() {
        let input = fs::read_to_string(fixtures_root().join(format!("{fixture}.input")))
            .expect("read input");
        let selfhost = fs::read_to_string(fixtures_root().join(format!("{fixture}.selfhost.out")))
            .expect("read selfhost output");
        let host = format_source(&input);
        assert_eq!(
            host, selfhost,
            "fixture `{fixture}` diverged between host formatter and selfhost prototype output"
        );
    }
}

#[test]
fn host_formatter_repeat_runs_are_deterministic() {
    for fixture in fixture_names() {
        let input = fs::read_to_string(fixtures_root().join(format!("{fixture}.input")))
            .expect("read input");
        let first = format_source(&input);
        let second = format_source(&first);
        let third = format_source(&second);
        assert_eq!(first, second, "first/second outputs differ for `{fixture}`");
        assert_eq!(second, third, "second/third outputs differ for `{fixture}`");
    }
}

#[test]
fn selfhost_formatter_examples_execute_via_vibe_test() {
    let out = run_vibe_test_for_selfhost();
    assert!(
        out.status.success(),
        "selfhost formatter execution failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("examples=2 passed=2 failed=0"),
        "unexpected selfhost summary:\n{}",
        out.stdout
    );
}

#[test]
fn selfhost_formatter_repeat_runs_are_deterministic() {
    let first = run_vibe_test_for_selfhost();
    let second = run_vibe_test_for_selfhost();
    assert!(
        first.status.success() && second.status.success(),
        "repeat selfhost runs failed:\nfirst stdout:\n{}\nfirst stderr:\n{}\nsecond stdout:\n{}\nsecond stderr:\n{}",
        first.stdout,
        first.stderr,
        second.stdout,
        second.stderr
    );
    assert_eq!(
        normalize_summary(&first.stdout),
        normalize_summary(&second.stdout),
        "selfhost execution summary should be deterministic except timing fields"
    );
}

fn fixture_names() -> Vec<&'static str> {
    vec!["basic", "nested"]
}

fn run_vibe_test_for_selfhost() -> CmdOutput {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "vibe_cli", "--", "test", "selfhost/formatter_core.yb"])
        .current_dir(workspace_root())
        .output()
        .expect("run vibe test for selfhost formatter");
    CmdOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn normalize_summary(stdout: &str) -> String {
    stdout
        .lines()
        .map(|line| {
            if let Some(idx) = line.find("duration_ms=") {
                return format!("{}duration_ms=<redacted>", &line[..idx]);
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve workspace root")
}

fn fixtures_root() -> PathBuf {
    workspace_root().join("selfhost").join("fixtures")
}

struct CmdOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}
