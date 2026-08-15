// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0
//
// `vibe run` must not mask fatal signals: a child killed by signal N is
// reported with a one-line stderr message and exit code 128 + N, while
// normal exits (0 and nonzero) pass through unchanged.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
#[test]
fn run_reports_segfaulting_program_signal() {
    // A garbage (nonzero, unmapped) list handle dereferences an invalid
    // pointer in the native runtime: deterministic SIGSEGV. (str_builder,
    // chan, and net handles are registry-validated and panic cleanly now,
    // so this binds an unvalidated list native directly.)
    let source = temp_source_file(
        "run_signal_segv",
        r#"
force_segv(h: Int, v: Int) -> Int {
  @native("vibe_list_append_i64")
}

pub main() -> Int {
  force_segv(1, 1)
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert_eq!(
        out.status.code(),
        Some(139),
        "expected 128+11 for SIGSEGV:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr
            .contains("error: program terminated by signal 11 (SIGSEGV)"),
        "missing signal report line:\n{}",
        out.stderr
    );
}

#[cfg(unix)]
#[test]
fn run_reports_contract_abort_signal() {
    // Contract violations abort() in the native runtime: SIGABRT.
    let source = temp_source_file(
        "run_signal_abrt",
        r#"
needs_positive(x: Int) -> Int {
  @require x > 0
  x
}

pub main() -> Int {
  needs_positive(0)
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert_eq!(
        out.status.code(),
        Some(134),
        "expected 128+6 for SIGABRT:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr
            .contains("contract @require failed in native execution"),
        "missing contract failure marker:\n{}",
        out.stderr
    );
    assert!(
        out.stderr
            .contains("error: program terminated by signal 6 (SIGABRT)"),
        "missing signal report line:\n{}",
        out.stderr
    );
}

#[test]
fn run_preserves_successful_exit() {
    let source = temp_source_file(
        "run_signal_ok",
        r#"
pub main() -> Int {
  @effect io
  println("signal-ok")
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "expected clean exit:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "signal-ok\n");
    assert!(
        !out.stderr.contains("terminated by signal"),
        "unexpected signal report on clean exit:\n{}",
        out.stderr
    );
}

#[test]
fn run_preserves_nonzero_exit_code() {
    let source = temp_source_file(
        "run_signal_exit2",
        r#"
pub main() -> Int {
  2
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "expected pass-through of child exit code 2:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        !out.stderr.contains("terminated by signal"),
        "unexpected signal report on normal nonzero exit:\n{}",
        out.stderr
    );
}

fn temp_source_file(prefix: &str, source: &str) -> PathBuf {
    let dir = unique_temp_dir(prefix);
    fs::create_dir_all(&dir).expect("create temp dir");
    let file = dir.join("main.yb");
    fs::write(&file, source.trim_start()).expect("write temp source");
    file
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos))
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
