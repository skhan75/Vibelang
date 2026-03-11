// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use vibe_diagnostics::{Diagnostic, Diagnostics, Severity, Span};

#[test]
fn host_diagnostics_formatter_matches_selfhost_fixture_outputs() {
    for fixture in fixture_names() {
        let fixture_key = fs::read_to_string(fixtures_root().join(format!("{fixture}.input")))
            .expect("read diagnostics formatter input fixture");
        let selfhost = fs::read_to_string(fixtures_root().join(format!("{fixture}.selfhost.out")))
            .expect("read diagnostics formatter selfhost output fixture");
        let host = host_golden_for_fixture(fixture_key.trim());
        assert_eq!(
            host, selfhost,
            "fixture `{fixture}` diverged between host diagnostics formatter and selfhost fixture output"
        );
    }
}

#[test]
fn host_diagnostics_formatter_repeat_runs_are_deterministic() {
    for fixture in fixture_names() {
        let fixture_key = fs::read_to_string(fixtures_root().join(format!("{fixture}.input")))
            .expect("read diagnostics formatter input fixture");
        let first = host_golden_for_fixture(fixture_key.trim());
        let second = host_golden_for_fixture(fixture_key.trim());
        assert_eq!(first, second, "repeat runs differ for fixture `{fixture}`");
    }
}

#[test]
fn selfhost_diagnostics_formatter_examples_execute_via_vibe_test() {
    let out = run_vibe_test_for_selfhost();
    assert!(
        out.status.success(),
        "selfhost diagnostics formatter execution failed:\nstdout:\n{}\nstderr:\n{}",
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
fn selfhost_diagnostics_formatter_repeat_runs_are_deterministic() {
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
        "selfhost diagnostics formatter summary should be deterministic except timing fields"
    );
}

fn fixture_names() -> Vec<&'static str> {
    vec!["diagnostics_basic", "diagnostics_severity"]
}

fn host_golden_for_fixture(name: &str) -> String {
    match name {
        "diagnostics_basic" => {
            let mut diagnostics = Diagnostics::default();
            diagnostics.push(Diagnostic::new(
                "E2002",
                Severity::Warning,
                "later span",
                Span::new(3, 1, 3, 5),
            ));
            diagnostics.push(Diagnostic::new(
                "E1001",
                Severity::Error,
                "earliest span",
                Span::new(1, 1, 1, 5),
            ));
            diagnostics.to_golden()
        }
        "diagnostics_severity" => {
            let mut diagnostics = Diagnostics::default();
            diagnostics.push(Diagnostic::new(
                "E3001",
                Severity::Info,
                "info same span",
                Span::new(5, 2, 5, 6),
            ));
            diagnostics.push(
                Diagnostic::new(
                    "E3001",
                    Severity::Warning,
                    "warning same span",
                    Span::new(5, 2, 5, 6),
                )
                .with_related("context note", Span::new(9, 1, 9, 2)),
            );
            diagnostics.push(Diagnostic::new(
                "E3001",
                Severity::Error,
                "error same span",
                Span::new(5, 2, 5, 6),
            ));
            diagnostics.to_golden()
        }
        other => panic!("unknown diagnostics fixture key: {other}"),
    }
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
            "selfhost/diagnostics_formatter_core.yb",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("run vibe test for selfhost diagnostics formatter");
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
