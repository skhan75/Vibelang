// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Regression tests for string interpolation of `Str` values.
//!
//! The parser desugars every `{x}` interpolation segment into a
//! `convert.to_str(x)` call, and HIR lowering rewrites `convert.to_str(s)`
//! with a `Str` argument into a passthrough of `s` itself. The stdlib
//! argument-type check briefly rejected that call shape (E2265, "expected
//! `Int`, got `Str`"), which broke
//! `examples/01_basics/75_string_interpolation.yb` — the example is not in
//! the examples manifest, so the suites stayed green. These tests pin the
//! example (dev and release) and the accepted/rejected interpolation shapes
//! end to end.
//!
//! Ground truth (pre-Theme-A binary, empirically verified): Str and Int
//! interpolation worked; Bool and Float interpolation died as spanless
//! Cranelift verifier errors. Bool now fails at check time with a spanned
//! E2265 (an improvement); Float remains a codegen issue (P1-12) and is
//! deliberately not covered here.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_75_EXPECTED_STDOUT: &str = "Hello World!\n\
You have 3 items\n\
Hi, VibeLang!\n\
Use {braces} for interpolation\n\
No interpolation here\n";

const STR_AND_INT_INTERPOLATION: &str = r#"
pub main() -> Int {
  @effect io
  s := "world"
  n := 42
  println("hello {s}")
  println("num {n}")
  0
}
"#;

const DIRECT_CONVERT_TO_STR: &str = r#"
pub main() -> Int {
  @effect io
  s := "abc"
  println(convert.to_str(s))
  println(convert.to_str("lit"))
  println(convert.to_str(42))
  0
}
"#;

const BOOL_INTERPOLATION: &str = r#"
pub main() -> Int {
  @effect io
  b := true
  println("flag {b}")
  0
}
"#;

#[test]
fn example_75_string_interpolation_builds_and_runs_dev() {
    assert_example_75_builds_and_runs("dev");
}

#[test]
fn example_75_string_interpolation_builds_and_runs_release() {
    assert_example_75_builds_and_runs("release");
}

fn assert_example_75_builds_and_runs(profile: &str) {
    let example = workspace_root().join("examples/01_basics/75_string_interpolation.yb");
    let example_str = example.to_str().expect("example path str");
    let build = run_vibe(&["build", example_str, "--profile", profile]);
    assert!(
        build.status.success(),
        "{profile} build of 75_string_interpolation.yb failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_path(&example, profile, host_target_triple());
    let output = Command::new(&binary)
        .current_dir(workspace_root())
        .output()
        .expect("run built example binary");
    assert!(
        output.status.success(),
        "{profile} 75_string_interpolation binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        EXAMPLE_75_EXPECTED_STDOUT,
        "{profile} 75_string_interpolation produced wrong output"
    );
}

#[test]
fn str_and_int_interpolation_build_and_run() {
    let source = temp_source_file("interp_str_int", STR_AND_INT_INTERPOLATION);
    assert_builds_and_prints(&source, "hello world\nnum 42\n");
}

#[test]
fn direct_convert_to_str_calls_build_and_run() {
    // `convert.to_str(<Str>)` must stay a real passthrough (print the
    // string, not a pointer rendered as an integer) and
    // `convert.to_str(<Int>)` must still format the integer.
    let source = temp_source_file("direct_to_str", DIRECT_CONVERT_TO_STR);
    assert_builds_and_prints(&source, "abc\nlit\n42\n");
}

#[test]
fn bool_interpolation_fails_at_check_time_with_e2265() {
    // Pre-Theme-A this was a spanless Cranelift verifier ICE; the spanned
    // check-time error is the accepted behavior.
    let source = temp_source_file("interp_bool", BOOL_INTERPOLATION);
    let source_str = source.to_str().expect("source path str");
    let build = run_vibe(&["build", source_str]);
    assert!(
        !build.status.success(),
        "Bool interpolation unexpectedly built:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    assert!(
        build.stderr.contains(
            "E2265: error: argument 1 type mismatch in call to `convert.to_str`: \
             expected `Int`, got `Bool`"
        ),
        "missing spanned E2265 for Bool interpolation:\n{}",
        build.stderr
    );
    assert!(
        !build.stderr.contains("Verifier errors"),
        "Bool interpolation must fail in the checker, not as a Cranelift ICE:\n{}",
        build.stderr
    );
}

fn assert_builds_and_prints(source: &Path, expected_stdout: &str) {
    let source_str = source.to_str().expect("source path str");
    let build = run_vibe(&["build", source_str]);
    assert!(
        build.status.success(),
        "dev build failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_path(source, "dev", host_target_triple());
    let output = Command::new(&binary)
        .current_dir(workspace_root())
        .output()
        .expect("run built binary");
    assert!(
        output.status.success(),
        "built binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected_stdout,
        "built binary produced wrong output"
    );
}

/// Mirrors `default_build_target()` in `crates/vibe_cli/src/main.rs` so the
/// artifact path matches whatever host this test runs on.
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
