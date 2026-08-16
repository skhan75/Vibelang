// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Native-execution coverage for `old(...)` snapshot semantics in `@ensure`
//! (audit P0-06): `old(expr)` must observe the value at function entry, not
//! re-evaluate post-body state. Before the fix, correct mutating code aborted
//! and violating code passed silently, in dev and release alike.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const MUTATED_FIELD_CORRECT: &str = r#"
type Box {
  v: Int
}

bump(mut b: Box) -> Int {
  @ensure b.v == old(b.v) + 1
  b.v = b.v + 1
  b.v
}

pub main() -> Int {
  @effect alloc
  @effect io
  b := Box { v: 1 }
  bump(b)
  println("bump-ok")
  0
}
"#;

const MUTATED_FIELD_VIOLATION: &str = r#"
type Account {
  balance: Int
}

sneaky_add(mut acct: Account) -> Int {
  @ensure acct.balance == old(acct.balance)
  acct.balance = acct.balance + 999999
  acct.balance
}

pub main() -> Int {
  @effect alloc
  @effect io
  a := Account { balance: 100 }
  sneaky_add(a)
  println("should-not-print")
  0
}
"#;

const REASSIGNED_SCALAR_CORRECT: &str = r#"
bump(mut x: Int) -> Int {
  @ensure x == old(x) + 1
  x = x + 1
  x
}

pub main() -> Int {
  @effect io
  bump(5)
  println("scalar-ok")
  0
}
"#;

const REASSIGNED_SCALAR_VIOLATION: &str = r#"
bump(mut x: Int) -> Int {
  @ensure x == old(x)
  x = x + 1
  x
}

pub main() -> Int {
  @effect io
  bump(5)
  println("should-not-print")
  0
}
"#;

/// README-shaped transfer: both `@ensure ... old(...)` clauses over mutated
/// record fields, plus `@require` guards evaluated before the snapshots.
const TRANSFER_README_SHAPE: &str = r#"
type Account {
  id: Int,
  balance: Int
}

transfer(mut from: Account, mut to: Account, amount: Int) -> Int {
  @require amount > 0
  @require from.balance >= amount
  @ensure to.balance == old(to.balance) + amount
  @ensure from.balance == old(from.balance) - amount
  from.balance = from.balance - amount
  to.balance = to.balance + amount
  amount
}

pub main() -> Int {
  @effect alloc
  @effect io
  a := Account { id: 1, balance: 100 }
  b := Account { id: 2, balance: 50 }
  transfer(a, b, 30)
  if a.balance == 70 {
    if b.balance == 80 {
      println("transfer-ok")
    } else {
      println("transfer-bad")
    }
  } else {
    println("transfer-bad")
  }
  0
}
"#;

#[test]
fn dev_mutated_field_correct_code_passes() {
    let source = temp_source_file("old_snapshot_field_pass", MUTATED_FIELD_CORRECT);
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "correct mutated-field program aborted:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "bump-ok\n");
}

#[test]
fn dev_mutated_field_violation_aborts() {
    let source = temp_source_file("old_snapshot_field_fail", MUTATED_FIELD_VIOLATION);
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "silent balance mutation must violate `old(...)` ensure:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr
            .contains("contract @ensure failed in native execution"),
        "missing deterministic ensure failure marker:\n{}",
        out.stderr
    );
    assert!(
        !out.stdout.contains("should-not-print"),
        "main continued past violated contract:\n{}",
        out.stdout
    );
}

#[test]
fn dev_reassigned_scalar_correct_code_passes() {
    let source = temp_source_file("old_snapshot_scalar_pass", REASSIGNED_SCALAR_CORRECT);
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "correct reassigned-scalar program aborted:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "scalar-ok\n");
}

#[test]
fn dev_reassigned_scalar_violation_aborts() {
    let source = temp_source_file("old_snapshot_scalar_fail", REASSIGNED_SCALAR_VIOLATION);
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "reassignment must violate `x == old(x)`:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr
            .contains("contract @ensure failed in native execution"),
        "missing deterministic ensure failure marker:\n{}",
        out.stderr
    );
}

#[test]
fn dev_readme_transfer_shape_runs_correctly() {
    let source = temp_source_file("old_snapshot_transfer", TRANSFER_README_SHAPE);
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "README transfer shape aborted:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "transfer-ok\n");
}

#[test]
fn release_binary_passes_correct_old_snapshot_code() {
    let source = temp_source_file("old_snapshot_release_pass", TRANSFER_README_SHAPE);
    let output = build_release_and_run(&source);
    assert!(
        output.status.success(),
        "release binary aborted on correct code:\nstdout:\n{}\nstderr:\n{}",
        output.stdout,
        output.stderr
    );
    assert_eq!(output.stdout, "transfer-ok\n");
}

#[test]
fn release_binary_aborts_on_old_snapshot_violation() {
    let source = temp_source_file("old_snapshot_release_fail", MUTATED_FIELD_VIOLATION);
    let output = build_release_and_run(&source);
    assert!(
        !output.status.success(),
        "release binary passed a violated `old(...)` ensure:\nstdout:\n{}\nstderr:\n{}",
        output.stdout,
        output.stderr
    );
    assert!(
        output
            .stderr
            .contains("contract @ensure failed in native execution"),
        "missing deterministic ensure failure marker:\n{}",
        output.stderr
    );
}

fn build_release_and_run(source: &Path) -> CmdOutput {
    let build = run_vibe(&[
        "build",
        source.to_str().expect("source path str"),
        "--profile",
        "release",
    ]);
    assert!(
        build.status.success(),
        "release build failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_path(source, "release", host_target_triple());
    let output = Command::new(&binary)
        .current_dir(workspace_root())
        .output()
        .expect("run built binary");
    CmdOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// Mirror of `default_build_target()` in the CLI: artifact paths embed the
/// host triple, so tests must derive it instead of hardcoding one platform.
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
