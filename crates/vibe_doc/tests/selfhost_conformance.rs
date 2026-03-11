// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use vibe_doc::{extract_docs, render_markdown};

#[test]
fn host_docs_formatter_matches_selfhost_fixture_outputs() {
    for fixture in fixture_names() {
        let input = fs::read_to_string(fixtures_root().join(format!("{fixture}.input")))
            .expect("read docs formatter input fixture");
        let selfhost = fs::read_to_string(fixtures_root().join(format!("{fixture}.selfhost.out")))
            .expect("read docs formatter selfhost output fixture");
        let host = render_markdown(fixture, &extract_docs(&input));
        assert_eq!(
            host, selfhost,
            "fixture `{fixture}` diverged between host docs formatter and selfhost fixture output"
        );
    }
}

#[test]
fn host_docs_formatter_repeat_runs_are_deterministic() {
    for fixture in fixture_names() {
        let input = fs::read_to_string(fixtures_root().join(format!("{fixture}.input")))
            .expect("read docs formatter input fixture");
        let first = render_markdown(fixture, &extract_docs(&input));
        let second = render_markdown(fixture, &extract_docs(&input));
        assert_eq!(first, second, "repeat runs differ for fixture `{fixture}`");
    }
}

#[test]
fn selfhost_docs_formatter_examples_execute_via_vibe_test() {
    let out = run_vibe_test_for_selfhost();
    assert!(
        out.status.success(),
        "selfhost docs formatter execution failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("failed=0"),
        "expected passing selfhost examples in output, got:\n{}",
        out.stdout
    );
}

#[test]
fn selfhost_docs_formatter_repeat_runs_are_deterministic() {
    let first = run_vibe_test_for_selfhost();
    let second = run_vibe_test_for_selfhost();
    assert!(
        first.status.success() && second.status.success(),
        "repeat selfhost runs failed:\nfirst stdout:\n{}\nfirst stderr:\n{}\nsecond stdout:\n{}\nsecond stderr:\n{}",
        first.stdout,
        first.stderr,
        second.stdout,
        second.stderr
    );
    assert_eq!(
        normalize_summary(&first.stdout),
        normalize_summary(&second.stdout),
        "selfhost docs formatter summary should be deterministic except timing fields"
    );
}

fn fixture_names() -> Vec<&'static str> {
    vec!["docs_basic", "docs_multi"]
}

fn run_vibe_test_for_selfhost() -> CmdOutput {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "vibe_cli",
            "--",
            "test",
            "selfhost/docs_formatter_core.yb",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("run vibe test for selfhost docs formatter");
    CmdOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn normalize_summary(stdout: &str) -> String {
    stdout
        .lines()
        .map(|line| {
            if let Some(idx) = line.find("duration_ms=") {
                return format!("{}duration_ms=<redacted>", &line[..idx]);
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve workspace root")
}

fn fixtures_root() -> PathBuf {
    workspace_root().join("selfhost").join("fixtures")
}

struct CmdOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}
