use std::process::{Command, ExitStatus};

#[test]
fn version_text_includes_required_fields() {
    let out = run_vibe(&["--version"]);
    assert!(
        out.status.success(),
        "--version should succeed\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    let text = out.stdout.trim();
    assert!(text.starts_with("vibe "));
    assert!(text.contains("commit="));
    assert!(text.contains("target="));
    assert!(text.contains("profile="));
}

#[test]
fn version_json_is_stable_and_contains_keys() {
    let first = run_vibe(&["--version", "--json"]);
    let second = run_vibe(&["--version", "--json"]);
    assert!(
        first.status.success() && second.status.success(),
        "--version --json should succeed\nfirst stdout:\n{}\nfirst stderr:\n{}\nsecond stdout:\n{}\nsecond stderr:\n{}",
        first.stdout,
        first.stderr,
        second.stdout,
        second.stderr
    );
    assert_eq!(
        first.stdout, second.stdout,
        "version json output should be deterministic"
    );
    let json = first.stdout.trim();
    assert!(json.starts_with('{'));
    assert!(json.ends_with('}'));
    assert!(json.contains("\"name\":\"vibe\""));
    assert!(json.contains("\"version\":\""));
    assert!(json.contains("\"commit\":\""));
    assert!(json.contains("\"target\":\""));
    assert!(json.contains("\"profile\":\""));
}

#[test]
fn version_rejects_unknown_extra_args() {
    let out = run_vibe(&["--version", "--bad"]);
    assert!(
        !out.status.success(),
        "invalid version args should fail\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(out.stderr.contains("usage: vibe --version [--json]"));
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
