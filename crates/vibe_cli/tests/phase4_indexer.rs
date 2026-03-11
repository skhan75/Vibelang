// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn vibe_index_rebuild_writes_snapshot() {
    let project_dir = temp_fixture_project("snapshots/pipeline_sample.vibe");
    let out = run_vibe(&[
        "index",
        project_dir.to_str().expect("project dir str"),
        "--rebuild",
        "--stats",
    ]);
    assert!(
        out.status.success(),
        "vibe index failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("index stats:"),
        "expected stats output:\n{}",
        out.stdout
    );
    assert!(
        out.stdout.contains("cold_ms=")
            && out.stdout.contains("incremental_ms=")
            && out.stdout.contains("memory_bytes="),
        "stats output should include performance metrics:\n{}",
        out.stdout
    );
    let snapshot = project_dir.join(".yb/index/index_v1.json");
    assert!(
        snapshot.exists(),
        "index snapshot should exist at {}",
        snapshot.display()
    );
}

#[test]
fn vibe_check_best_effort_refreshes_index() {
    let file = temp_fixture_file("parse_ok/basic_function.vibe");
    let out = run_vibe(&["check", file.to_str().expect("file str")]);
    assert!(
        out.status.success(),
        "vibe check failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    let snapshot = file
        .parent()
        .expect("file parent")
        .join(".yb/index/index_v1.json");
    assert!(
        snapshot.exists(),
        "best-effort check index refresh should persist snapshot at {}",
        snapshot.display()
    );
}

#[test]
fn index_snapshot_is_deterministic_for_same_inputs() {
    let project_dir = temp_fixture_project("snapshots/pipeline_sample.vibe");
    let args = [
        "index",
        project_dir.to_str().expect("project dir str"),
        "--rebuild",
    ];
    let first = run_vibe(&args);
    assert!(
        first.status.success(),
        "first index run failed:\nstdout:\n{}\nstderr:\n{}",
        first.stdout,
        first.stderr
    );
    let snapshot = project_dir.join(".yb/index/index_v1.json");
    let first_snapshot = fs::read_to_string(&snapshot).expect("read first snapshot");

    let second = run_vibe(&args);
    assert!(
        second.status.success(),
        "second index run failed:\nstdout:\n{}\nstderr:\n{}",
        second.stdout,
        second.stderr
    );
    let second_snapshot = fs::read_to_string(&snapshot).expect("read second snapshot");
    assert_eq!(
        first_snapshot, second_snapshot,
        "index snapshot should be deterministic for same source+toolchain"
    );
}

#[test]
fn lsp_jsonrpc_transport_supports_lifecycle_requests() {
    let project_dir = temp_fixture_project("snapshots/pipeline_sample.vibe");
    let index_root = project_dir.join(".yb/index");
    let mut child = Command::new(vibe_bin())
        .args([
            "lsp",
            "--transport",
            "jsonrpc",
            "--index-root",
            index_root.to_str().expect("index root str"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(workspace_root())
        .spawn()
        .expect("spawn vibe lsp");

    {
        let stdin = child.stdin.as_mut().expect("lsp stdin");
        let initialize = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let shutdown = r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":{}}"#;
        let exit = r#"{"jsonrpc":"2.0","method":"exit","params":{}}"#;
        let framed = format!(
            "{}{}{}",
            framed_jsonrpc(initialize),
            framed_jsonrpc(shutdown),
            framed_jsonrpc(exit)
        );
        write!(stdin, "{framed}").expect("write jsonrpc messages");
    }
    let output = child.wait_with_output().expect("collect lsp output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "vibe lsp failed:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("\"id\":1") && stdout.contains("\"id\":2"),
        "initialize/shutdown responses missing from stdout:\n{}",
        stdout
    );
}

#[test]
fn lsp_legacy_transport_supports_shutdown_command() {
    let project_dir = temp_fixture_project("snapshots/pipeline_sample.vibe");
    let index_root = project_dir.join(".yb/index");
    let mut child = Command::new(vibe_bin())
        .args([
            "lsp",
            "--transport",
            "legacy",
            "--index-root",
            index_root.to_str().expect("index root str"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(workspace_root())
        .spawn()
        .expect("spawn legacy vibe lsp");

    {
        let stdin = child.stdin.as_mut().expect("legacy lsp stdin");
        writeln!(stdin, "{{\"method\":\"shutdown\"}}").expect("write legacy shutdown command");
    }
    let output = child.wait_with_output().expect("collect legacy lsp output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "legacy vibe lsp failed:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("shutdown"),
        "legacy shutdown response missing from stdout:\n{}",
        stdout
    );
}

fn temp_fixture_project(relative: &str) -> PathBuf {
    let source = fixture_path(relative);
    let contents = fs::read_to_string(&source).expect("read fixture source");
    let temp_dir = unique_temp_dir("vibe_phase4_indexer_project");
    fs::create_dir_all(&temp_dir).expect("create temp project dir");
    let destination = temp_dir.join(
        source
            .file_name()
            .and_then(|n| n.to_str())
            .expect("fixture file name"),
    );
    fs::write(&destination, contents).expect("write fixture source");
    temp_dir
}

#[test]
fn vibe_index_accepts_yb_source_files() {
    let project_dir = unique_temp_dir("vibe_phase4_indexer_yb");
    fs::create_dir_all(&project_dir).expect("create temp project dir");
    fs::write(project_dir.join("main.yb"), "main() -> Int { 0 }\n").expect("write yb source");

    let out = run_vibe(&[
        "index",
        project_dir.to_str().expect("project dir str"),
        "--rebuild",
    ]);
    assert!(
        out.status.success(),
        "vibe index failed on .yb source:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    let snapshot = project_dir.join(".yb/index/index_v1.json");
    assert!(
        snapshot.exists(),
        "index snapshot should exist at {}",
        snapshot.display()
    );
}

fn temp_fixture_file(relative: &str) -> PathBuf {
    let source = fixture_path(relative);
    let contents = fs::read_to_string(&source).expect("read fixture source");
    let temp_dir = unique_temp_dir("vibe_phase4_indexer_file");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let destination = temp_dir.join(
        source
            .file_name()
            .and_then(|n| n.to_str())
            .expect("fixture file name"),
    );
    fs::write(&destination, contents).expect("write fixture source");
    destination
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

fn framed_jsonrpc(payload: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", payload.len(), payload)
}

struct CmdOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}
