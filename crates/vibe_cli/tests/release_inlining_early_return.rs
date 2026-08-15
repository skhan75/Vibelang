// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Regression tests for the release-profile MIR inliner miscompiling early
//! returns. `rewrite_returns_to_assign` turned every callee `return` into an
//! unguarded `let`, so a guard clause fell through into the statements after
//! it and the last assignment won: `min2(1, 100)` returned `100` in release
//! and `1` in dev, with no diagnostic. Callees whose `return`s are not all in
//! tail position (and callees using `?`, an early return in expression form)
//! are no longer inlined, so release must now agree with dev.
//!
//! Every program here binds the call to a local *before* using it
//! (`m1 := min2(1, 100)`), because the inliner only fires on
//! `MirStmt::Let { expr: MirExpr::Call }`. A call nested in argument position
//! (`println(json.stringify_i64(min2(1, 100)))`) is never inlined and would
//! make these tests vacuous.

mod common;

use common::host_target_triple;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Guard clause followed by a fall-through tail expression.
const EARLY_RETURN_GUARD: &str = r#"
min2(a: Int, b: Int) -> Int {
  if a < b {
    return a
  }
  b
}

pub main() -> Int {
  @effect io
  m1 := min2(1, 100)
  println(json.stringify_i64(m1))
  m2 := min2(100, 1)
  println(json.stringify_i64(m2))
  0
}
"#;

/// Three stacked guards: pre-fix release collapsed every input to the last arm.
const MULTI_GUARD: &str = r#"
classify(n: Int) -> Int {
  if n < 0 {
    return 0
  }
  if n == 0 {
    return 1
  }
  if n < 10 {
    return 2
  }
  3
}

pub main() -> Int {
  @effect io
  c1 := classify(-5)
  println(json.stringify_i64(c1))
  c2 := classify(0)
  println(json.stringify_i64(c2))
  c3 := classify(5)
  println(json.stringify_i64(c3))
  c4 := classify(50)
  println(json.stringify_i64(c4))
  0
}
"#;

/// Every path returns; pre-fix this failed the release build outright because
/// dead-code elimination dropped both per-branch assignments.
const EVERY_PATH_RETURNS: &str = r#"
pick(flag: Bool) -> Int {
  if flag {
    return 7
  } else {
    return 9
  }
}

pub main() -> Int {
  @effect io
  p1 := pick(true)
  println(json.stringify_i64(p1))
  p2 := pick(false)
  println(json.stringify_i64(p2))
  0
}
"#;

/// Anti-over-conservatism companion: a callee whose only `return` is its tail
/// expression must keep inlining and keep producing the right value.
const TAIL_RETURN_ONLY: &str = r#"
add3(a: Int, b: Int) -> Int {
  s := a + b
  s + 3
}

pub main() -> Int {
  @effect io
  t1 := add3(4, 5)
  println(json.stringify_i64(t1))
  0
}
"#;

#[test]
fn release_guard_clause_callee_returns_the_guarded_value() {
    let stdout = build_and_run_release("inline_guard_clause", EARLY_RETURN_GUARD);
    assert_eq!(
        stdout, "1\n1\n",
        "release inlining dropped the early return of min2"
    );
}

#[test]
fn release_multi_guard_callee_keeps_every_arm() {
    let stdout = build_and_run_release("inline_multi_guard", MULTI_GUARD);
    assert_eq!(
        stdout, "0\n1\n2\n3\n",
        "release inlining collapsed classify to its final arm"
    );
}

#[test]
fn release_every_path_returns_callee_builds_and_runs() {
    let stdout = build_and_run_release("inline_every_path", EVERY_PATH_RETURNS);
    assert_eq!(
        stdout, "7\n9\n",
        "release inlining mis-selected the branch of pick"
    );
}

#[test]
fn release_tail_return_callee_still_correct() {
    let stdout = build_and_run_release("inline_tail_return", TAIL_RETURN_ONLY);
    assert_eq!(stdout, "12\n", "tail-return-only callee miscompiled");
}

#[test]
fn release_builds_and_runs_question_operator_example() {
    // `?` lowers to a bare Cranelift `return_` out of the enclosing function,
    // so inlining a `?`-bearing callee produced spanless "Verifier errors" on
    // this shipped example in release.
    let example = workspace_root().join("examples/01_basics/71_result_ok_err_question.yb");
    let example_str = example.to_str().expect("example path str");
    let build = run_vibe(&["build", example_str, "--profile", "release"]);
    assert!(
        build.status.success(),
        "release build of the `?` example failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_path(&example, "release", host_target_triple());
    let output = Command::new(&binary)
        .current_dir(workspace_root())
        .output()
        .expect("run built release binary");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "done\n",
        "release `?` example produced wrong output"
    );
}

/// Builds `source` with `--profile release`, runs the artifact, and returns
/// stdout. Panics with the compiler or runtime output on any failure.
fn build_and_run_release(prefix: &str, source: &str) -> String {
    let file = temp_source_file(prefix, source);
    let file_str = file.to_str().expect("source path str");
    let build = run_vibe(&["build", file_str, "--profile", "release"]);
    assert!(
        build.status.success(),
        "release build of {prefix} failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_path(&file, "release", host_target_triple());
    let output = Command::new(&binary)
        .current_dir(workspace_root())
        .output()
        .expect("run built release binary");
    assert!(
        output.status.success(),
        "release binary for {prefix} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn temp_source_file(prefix: &str, source: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos));
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

fn run_vibe(args: &[&str]) -> CmdOutput {
    let output = Command::new(env!("CARGO_BIN_EXE_vibe"))
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
