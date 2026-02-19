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
fn hello_world_build_and_run_with_yb_extension() {
    let source = temp_fixture_copy_with_extension("build/hello_world.vibe", "yb");
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
fn concurrency_go_select_fixture_runs() {
    let source = temp_fixture_copy("build/concurrency_go_select.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "go-worker\nselect-recv\n");
}

#[test]
fn select_default_fixture_runs() {
    let source = temp_fixture_copy("build/select_default.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "default\n");
}

#[test]
fn select_closed_fixture_runs() {
    let source = temp_fixture_copy("build/select_closed.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "closed\n");
}

#[test]
fn select_multi_receive_fixture_runs() {
    let source = temp_fixture_copy("build/select_multi_receive.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "from-ch2\n");
}

#[test]
fn select_after_timeout_fixture_runs() {
    let source = temp_fixture_copy("build/select_after_timeout.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "after\n");
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
    let first_debug_map = fs::read_to_string(artifact_debug_map_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read first debug map");

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
    let second_debug_map = fs::read_to_string(artifact_debug_map_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read second debug map");

    assert_eq!(first_bin, second_bin, "binary output must be deterministic");
    assert_eq!(
        first_debug_map, second_debug_map,
        "debug map output must be deterministic"
    );
    assert_eq!(
        first.stdout, second.stdout,
        "build metadata output should be stable"
    );
}

#[test]
fn vibe_test_runs_contract_examples() {
    let source = temp_fixture_copy("contract_ok/topk_contracts.vibe");
    let out = run_vibe(&["test", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "vibe test failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("examples=2 passed=2 failed=0"),
        "unexpected test summary:\n{}",
        out.stdout
    );
}

#[test]
fn vibe_test_enforces_contract_runtime_checks_by_default() {
    let source = temp_fixture_copy("build/contract_runtime_require.vibe");
    let out = run_vibe(&["test", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "vibe test unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("failed=1"),
        "expected one failing example in summary:\n{}",
        out.stdout
    );
    assert!(
        out.stderr.contains("contract @require failed"),
        "expected require failure details:\n{}",
        out.stderr
    );
}

#[test]
fn vibe_test_can_disable_contract_runtime_checks_with_env_override() {
    let source = temp_fixture_copy("build/contract_runtime_require.vibe");
    let out = run_vibe_with_env(
        &["test", source.to_str().expect("source path str")],
        &[("VIBE_CONTRACT_CHECKS", "off")],
    );
    assert!(
        out.status.success(),
        "vibe test should pass when runtime contract checks are disabled:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("failed=0"),
        "expected no failing examples when checks are disabled:\n{}",
        out.stdout
    );
}

#[test]
fn build_accepts_debuginfo_flag_and_writes_metadata() {
    let source = temp_fixture_copy("build/hello_world.vibe");
    let source_str = source.to_str().expect("source path str");
    let out = run_vibe(&["build", source_str, "--debuginfo", "full"]);
    assert!(
        out.status.success(),
        "build failed with debuginfo=full:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    let debug_map = fs::read_to_string(artifact_debug_map_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read debug map");
    assert!(
        debug_map.contains("debuginfo=full"),
        "debug map should record debuginfo level:\n{debug_map}"
    );
}

#[test]
fn while_loop_fixture_runs() {
    let source = temp_fixture_copy("build/while_loop.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "");
}

#[test]
fn repeat_loop_fixture_runs() {
    let source = temp_fixture_copy("build/repeat_loop.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "");
}

#[test]
fn unsupported_member_access_has_stable_codegen_diagnostic() {
    let source = temp_fixture_copy("build_err/member_access_unsupported.vibe");
    let out = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "build unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    let expected = fs::read_to_string(fixture_path("build_err/member_access_unsupported.diag"))
        .expect("read build_err golden");
    assert_eq!(
        expected.trim(),
        out.stderr.trim(),
        "build error output mismatch"
    );
}

#[test]
fn vibe_test_directory_accepts_mixed_extensions_when_stems_differ() {
    let temp_dir = unique_temp_dir("vibe_phase2_mixed_ext_ok");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    fs::write(temp_dir.join("alpha.vibe"), "alpha() -> Int { 0 }\n").expect("write alpha");
    fs::write(temp_dir.join("beta.yb"), "beta() -> Int { 0 }\n").expect("write beta");

    let out = run_vibe(&["test", temp_dir.to_str().expect("temp dir str")]);
    assert!(
        out.status.success(),
        "vibe test should accept mixed source extensions:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
}

#[test]
fn vibe_test_rejects_same_stem_across_extensions() {
    let temp_dir = unique_temp_dir("vibe_phase2_mixed_ext_conflict");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    fs::write(temp_dir.join("same.vibe"), "same() -> Int { 0 }\n").expect("write same vibe");
    fs::write(temp_dir.join("same.yb"), "same() -> Int { 0 }\n").expect("write same yb");

    let out = run_vibe(&["test", temp_dir.to_str().expect("temp dir str")]);
    assert!(
        !out.status.success(),
        "vibe test unexpectedly succeeded with conflicting stems:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr
            .contains("conflicting source files share a stem across extensions"),
        "expected collision guard diagnostic:\n{}",
        out.stderr
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

fn temp_fixture_copy_with_extension(relative: &str, ext: &str) -> PathBuf {
    let src = temp_fixture_copy(relative);
    let dst = src.with_extension(ext);
    fs::rename(&src, &dst).expect("rename temp fixture extension");
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
    run_vibe_with_env(args, &[])
}

fn run_vibe_with_env(args: &[&str], envs: &[(&str, &str)]) -> CmdOutput {
    let mut command = Command::new(vibe_bin());
    command.args(args).current_dir(workspace_root());
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output().expect("run vibe command");
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
        .join(".yb")
        .join("artifacts")
        .join(profile)
        .join(target)
        .join(stem)
}

fn artifact_debug_map_path(source: &Path, profile: &str, target: &str) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("source stem");
    source
        .parent()
        .expect("source parent")
        .join(".yb")
        .join("artifacts")
        .join(profile)
        .join(target)
        .join(format!("{stem}.debug.map"))
}

struct CmdOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}
