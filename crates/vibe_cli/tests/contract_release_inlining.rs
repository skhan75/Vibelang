// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Regression tests for the release-profile inliner ICE on contract-bearing
//! callees ("unknown local `<param>` in function `main`"): the MIR inliner
//! used to copy `@require`/`@ensure` checks into callers without remapping
//! their operands. Contract-bearing callees are now excluded from inlining,
//! so `--profile release` must build these programs and enforce their
//! contracts exactly like dev.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const CONTRACT_CALLEE_VALID: &str = r#"
divide(a: Int, b: Int) -> Int {
  @require b != 0
  a / b
}

pub main() -> Int {
  @effect io
  value := divide(10, 2)
  println(json.stringify_i64(value))
  0
}
"#;

const CONTRACT_CALLEE_VIOLATION: &str = r#"
divide(a: Int, b: Int) -> Int {
  @require b != 0
  a / b
}

pub main() -> Int {
  @effect io
  value := divide(10, 0)
  println(json.stringify_i64(value))
  0
}
"#;

#[test]
fn release_profile_builds_and_runs_contract_bearing_callee() {
    let source = temp_source_file("contract_release_valid", CONTRACT_CALLEE_VALID);
    let source_str = source.to_str().expect("source path str");
    let build = run_vibe(&["build", source_str, "--profile", "release"]);
    assert!(
        build.status.success(),
        "release build of contract-bearing callee failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_path(&source, "release", host_target_triple());
    let output = Command::new(&binary)
        .current_dir(workspace_root())
        .output()
        .expect("run built release binary");
    assert!(
        output.status.success(),
        "release binary with valid contract call failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "5\n",
        "release binary returned a wrong value for divide(10, 2)"
    );
}

#[test]
fn release_profile_contract_violation_still_aborts() {
    let source = temp_source_file("contract_release_violation", CONTRACT_CALLEE_VIOLATION);
    let source_str = source.to_str().expect("source path str");
    let build = run_vibe(&["build", source_str, "--profile", "release"]);
    assert!(
        build.status.success(),
        "release build of violating program failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_path(&source, "release", host_target_triple());
    let output = Command::new(&binary)
        .current_dir(workspace_root())
        .output()
        .expect("run built release binary");
    assert!(
        !output.status.success(),
        "release binary unexpectedly succeeded for @require violation:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("contract @require failed in native execution"),
        "missing deterministic require failure marker in release:\n{}",
        stderr
    );
}

#[test]
fn release_profile_builds_contract_example() {
    // One of the six examples the release inliner used to ICE on.
    let example =
        workspace_root().join("examples/10_contracts_intent/66_list_transform_contracts.yb");
    let example_str = example.to_str().expect("example path str");
    let build = run_vibe(&["build", example_str, "--profile", "release"]);
    assert!(
        build.status.success(),
        "release build of contract example failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_path(&example, "release", host_target_triple());
    let output = Command::new(&binary)
        .current_dir(workspace_root())
        .output()
        .expect("run built release example binary");
    assert!(
        output.status.success(),
        "release contract example binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "list-transform-contract-ok\n",
        "release contract example produced wrong output"
    );
}

/// Mirrors `default_build_target()` in `crates/vibe_cli/src/main.rs` so the
/// artifact path matches whatever host this test runs on (the older contract
/// tests hardcode x86_64-unknown-linux-gnu and fail on other hosts).
fn host_target_triple() -> &'static str {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        _ => "x86_64-unknown-linux-gnu",
    }
}

fn temp_source_file(prefix: &str, source: &str) -> PathBuf {
    let dir = unique_temp_dir(prefix);
    fs::create_dir_all(&dir).expect("create temp dir");
    let file = dir.join("main.yb");
    fs::write(&file, source.trim_start()).expect("write temp source");
    file
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
