// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn phase7_single_thread_samples_run_expected_outputs() {
    let fixtures = [
        (
            "phase7/advanced/single_thread/single_thread__hello_world.yb",
            "phase7-hello\n",
        ),
        (
            "phase7/advanced/single_thread/single_thread__calculator.yb",
            "calc-ok\n",
        ),
        (
            "phase7/advanced/single_thread/single_thread__pipeline_transform.yb",
            "pipe-ok\n",
        ),
        (
            "phase7/advanced/single_thread/single_thread__state_machine.yb",
            "state-ok\n",
        ),
        (
            "phase7/advanced/single_thread/single_thread__language_tour.yb",
            "tour-ok\n",
        ),
    ];

    for (relative, expected_stdout) in fixtures {
        let source = temp_fixture_copy(relative);
        let first = run_vibe(&["run", source.to_str().expect("source path str")]);
        let second = run_vibe(&["run", source.to_str().expect("source path str")]);
        assert!(
            first.status.success() && second.status.success(),
            "sample run failed for {}:\nfirst stdout:\n{}\nfirst stderr:\n{}\nsecond stdout:\n{}\nsecond stderr:\n{}",
            relative,
            first.stdout,
            first.stderr,
            second.stdout,
            second.stderr
        );
        assert_eq!(
            first.stdout, expected_stdout,
            "unexpected stdout for {relative}"
        );
        assert_eq!(
            first.stdout, second.stdout,
            "repeat run output should be deterministic for {relative}"
        );
        assert_eq!(
            first.status.code(),
            second.status.code(),
            "repeat run exit code should be deterministic for {relative}"
        );
    }
}

#[test]
fn phase7_single_thread_build_artifacts_are_deterministic() {
    let fixtures = [
        "phase7/advanced/single_thread/single_thread__hello_world.yb",
        "phase7/advanced/single_thread/single_thread__calculator.yb",
        "phase7/advanced/single_thread/single_thread__pipeline_transform.yb",
        "phase7/advanced/single_thread/single_thread__state_machine.yb",
        "phase7/advanced/single_thread/single_thread__language_tour.yb",
    ];

    for relative in fixtures {
        let source = temp_fixture_copy(relative);
        let source_str = source.to_str().expect("source path str");

        let first = run_vibe(&["build", source_str]);
        assert!(
            first.status.success(),
            "first build failed for {relative}:\nstdout:\n{}\nstderr:\n{}",
            first.stdout,
            first.stderr
        );
        let first_obj = fs::read(artifact_object_path(
            &source,
            "dev",
            "x86_64-unknown-linux-gnu",
        ))
        .expect("read first object");
        let first_bin = fs::read(artifact_binary_path(
            &source,
            "dev",
            "x86_64-unknown-linux-gnu",
        ))
        .expect("read first binary");
        let first_debug = fs::read_to_string(artifact_debug_map_path(
            &source,
            "dev",
            "x86_64-unknown-linux-gnu",
        ))
        .expect("read first debug map");

        let second = run_vibe(&["build", source_str]);
        assert!(
            second.status.success(),
            "second build failed for {relative}:\nstdout:\n{}\nstderr:\n{}",
            second.stdout,
            second.stderr
        );
        let second_obj = fs::read(artifact_object_path(
            &source,
            "dev",
            "x86_64-unknown-linux-gnu",
        ))
        .expect("read second object");
        let second_bin = fs::read(artifact_binary_path(
            &source,
            "dev",
            "x86_64-unknown-linux-gnu",
        ))
        .expect("read second binary");
        let second_debug = fs::read_to_string(artifact_debug_map_path(
            &source,
            "dev",
            "x86_64-unknown-linux-gnu",
        ))
        .expect("read second debug map");

        assert_eq!(
            first_obj, second_obj,
            "object output mismatch for {relative}"
        );
        assert_eq!(
            first_bin, second_bin,
            "binary output mismatch for {relative}"
        );
        assert_eq!(
            first_debug, second_debug,
            "debug map output mismatch for {relative}"
        );
    }
}

#[test]
fn phase7_language_tour_contract_examples_pass_in_vibe_test() {
    let source = temp_fixture_copy("phase7/advanced/single_thread/single_thread__language_tour.yb");
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

fn temp_fixture_copy(relative: &str) -> PathBuf {
    let src = fixture_path(relative);
    let contents = fs::read_to_string(&src).expect("read fixture source");
    let file_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .expect("fixture file name");
    let temp_dir = unique_temp_dir("vibe_phase7_validation");
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

fn artifact_object_path(source: &Path, profile: &str, target: &str) -> PathBuf {
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
        .join(format!("{stem}.o"))
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
