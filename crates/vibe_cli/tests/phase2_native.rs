use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn hello_world_build_and_run() {
    let source = temp_fixture_copy("build/hello_world.vibe");
    let build = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        build.status.success(),
        "build failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );

    let binary = artifact_binary_path(&source, "dev", "x86_64-unknown-linux-gnu");
    assert!(
        binary.exists(),
        "binary should be emitted at {}",
        binary.display()
    );

    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "hello from vibelang\n");
}

#[test]
fn function_call_fixture_runs() {
    let source = temp_fixture_copy("build/function_calls.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "from helper\n");
}

#[test]
fn if_control_flow_fixture_runs() {
    let source = temp_fixture_copy("build/control_flow_if.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "if-branch\n");
}

#[test]
fn deterministic_build_binary_and_metadata() {
    let source = temp_fixture_copy("build/hello_world.vibe");
    let source_str = source.to_str().expect("source path str");

    let first = run_vibe(&["build", source_str]);
    assert!(
        first.status.success(),
        "first build failed:\nstdout:\n{}\nstderr:\n{}",
        first.stdout,
        first.stderr
    );
    let first_bin = fs::read(artifact_binary_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read first binary");

    let second = run_vibe(&["build", source_str]);
    assert!(
        second.status.success(),
        "second build failed:\nstdout:\n{}\nstderr:\n{}",
        second.stdout,
        second.stderr
    );
    let second_bin = fs::read(artifact_binary_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read second binary");

    assert_eq!(first_bin, second_bin, "binary output must be deterministic");
    assert_eq!(
        first.stdout, second.stdout,
        "build metadata output should be stable"
    );
}

#[test]
fn unsupported_while_loop_returns_codegen_error() {
    let source = temp_fixture_copy("build_err/while_loop.vibe");
    let out = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "build unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    let expected = fs::read_to_string(fixture_path("build_err/while_loop.diag"))
        .expect("read build_err golden");
    assert_eq!(
        expected.trim(),
        out.stderr.trim(),
        "build error output mismatch"
    );
}

fn temp_fixture_copy(relative: &str) -> PathBuf {
    let src = fixture_path(relative);
    let contents = fs::read_to_string(&src).expect("read fixture source");
    let file_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .expect("fixture file name");
    let temp_dir = unique_temp_dir("vibe_phase2_native");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let dst = temp_dir.join(file_name);
    fs::write(&dst, contents).expect("write temp fixture");
    dst
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

fn fixture_path(relative: &str) -> PathBuf {
    workspace_root()
        .join("compiler/tests/fixtures")
        .join(relative)
}

fn artifact_binary_path(source: &Path, profile: &str, target: &str) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("source stem");
    source
        .parent()
        .expect("source parent")
        .join(".vibe")
        .join("artifacts")
        .join(profile)
        .join(target)
        .join(stem)
}

struct CmdOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}
