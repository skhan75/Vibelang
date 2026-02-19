use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn lint_intent_reports_missing_public_intent() {
    let project = temp_project_with_source(
        "intent_sample.vibe",
        r#"
pub addOne(x: Int) -> Int {
  x + 1
}
"#,
    );
    let out = run_vibe(&[
        "lint",
        project.to_str().expect("project path str"),
        "--intent",
    ]);
    assert!(
        out.status.success(),
        "vibe lint failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("I5001"),
        "missing-intent finding should be reported:\n{}",
        out.stdout
    );
}

#[test]
fn lint_intent_changed_mode_uses_index_delta_when_no_git() {
    let project = temp_project_with_source(
        "changed_sample.yb",
        r#"
pub increment(x: Int) -> Int {
  x + 1
}
"#,
    );

    let first = run_vibe(&[
        "lint",
        project.to_str().expect("project path str"),
        "--intent",
    ]);
    assert!(
        first.status.success(),
        "initial lint failed:\nstdout:\n{}\nstderr:\n{}",
        first.stdout,
        first.stderr
    );

    let second = run_vibe(&[
        "lint",
        project.to_str().expect("project path str"),
        "--intent",
        "--changed",
    ]);
    assert!(
        second.status.success(),
        "changed lint failed:\nstdout:\n{}\nstderr:\n{}",
        second.stdout,
        second.stderr
    );
    assert!(
        second.stdout.contains("no changed source files"),
        "expected no changed files after immediate rerun:\n{}",
        second.stdout
    );

    let source = project.join("changed_sample.yb");
    let mut updated = fs::read_to_string(&source).expect("read source");
    updated.push('\n');
    fs::write(&source, updated).expect("write source");

    let third = run_vibe(&[
        "lint",
        project.to_str().expect("project path str"),
        "--intent",
        "--changed",
    ]);
    assert!(
        third.status.success(),
        "changed lint after edit failed:\nstdout:\n{}\nstderr:\n{}",
        third.stdout,
        third.stderr
    );
    assert!(
        !third.stdout.contains("no changed source files"),
        "expected changed file detection after edit:\n{}",
        third.stdout
    );
}

#[test]
fn lint_intent_suggestions_are_verifier_gated() {
    let project = temp_project_with_source(
        "suggest_sample.yb",
        r#"
pub sum(a: Int, b: Int) -> Int {
  a + b
}
"#,
    );
    let out = run_vibe(&[
        "lint",
        project.to_str().expect("project path str"),
        "--intent",
        "--suggest",
    ]);
    assert!(
        out.status.success(),
        "vibe lint --suggest failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("suggestions:"),
        "suggestions header missing:\n{}",
        out.stdout
    );
    assert!(
        out.stdout.contains("verified=true"),
        "suggestion should be compiler-verified before surfacing:\n{}",
        out.stdout
    );
}

#[test]
fn lint_intent_supports_budget_mode_and_opt_in_telemetry() {
    let project = temp_project_with_source(
        "telemetry_sample.yb",
        r#"
pub multiply(a: Int, b: Int) -> Int {
  a * b
}
"#,
    );
    let telemetry_file = project.join("reports/phase5/telemetry.json");
    let out = run_vibe(&[
        "lint",
        project.to_str().expect("project path str"),
        "--intent",
        "--mode",
        "hybrid",
        "--max-requests-per-day",
        "0",
        "--telemetry-out",
        telemetry_file.to_str().expect("telemetry path str"),
    ]);
    assert!(
        out.status.success(),
        "vibe lint failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout
            .contains("intent lint returned partial results (latency/budget guard)."),
        "expected latency budget partial-result message:\n{}",
        out.stdout
    );
    assert!(
        telemetry_file.exists(),
        "telemetry file should exist at {}",
        telemetry_file.display()
    );
    let telemetry_json = fs::read_to_string(&telemetry_file).expect("read telemetry json");
    assert!(
        telemetry_json.contains("\"requests\""),
        "telemetry payload should contain request counters:\n{}",
        telemetry_json
    );
}

fn temp_project_with_source(file_name: &str, source: &str) -> PathBuf {
    let dir = unique_temp_dir("vibe_phase5_intent_lint");
    fs::create_dir_all(&dir).expect("create temp project dir");
    fs::write(dir.join(file_name), source.trim_start()).expect("write test source");
    dir
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos))
}

fn run_vibe(args: &[&str]) -> CmdOutput {
    let output = Command::new(vibe_bin())
        .args(args)
        .current_dir(workspace_root())
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

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve workspace root")
}

struct CmdOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}
