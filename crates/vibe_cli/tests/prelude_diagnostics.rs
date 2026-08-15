// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! The stdlib prelude is injected into every module-free compilation unit. It
//! is invisible source: the user has no file to open and nothing to fix, so its
//! diagnostics must never reach a user-facing surface. These tests pin both
//! halves of that contract — the prelude stays silent, and the user's own
//! diagnostics are untouched.

mod common;

use common::host_target_triple;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const HELLO_WORLD: &str =
    "pub main() -> Int {\n  @effect io\n  println(\"hello-vibelang\")\n  0\n}\n";

#[test]
fn hello_world_build_emits_no_diagnostics() {
    let source = temp_source_file("vibe_prelude_diag_build", HELLO_WORLD);
    let build = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        build.status.success(),
        "build failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    assert_eq!(
        build.stderr, "",
        "a clean hello world must build with empty stderr; got:\n{}",
        build.stderr
    );

    let binary = artifact_binary_path(&source, "dev", host_target_triple());
    assert!(
        binary.exists(),
        "binary should be emitted at {}",
        binary.display()
    );
}

#[test]
fn hello_world_run_emits_no_diagnostics() {
    let source = temp_source_file("vibe_prelude_diag_run", HELLO_WORLD);
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "hello-vibelang\n");
    assert_eq!(
        run.stderr, "",
        "a clean hello world must run with empty stderr; got:\n{}",
        run.stderr
    );
}

#[test]
fn hello_world_check_reports_ok_only() {
    let source = temp_source_file("vibe_prelude_diag_check", HELLO_WORLD);
    let check = run_vibe(&["check", source.to_str().expect("source path str")]);
    assert!(
        check.status.success(),
        "check failed:\nstdout:\n{}\nstderr:\n{}",
        check.stdout,
        check.stderr
    );
    assert_eq!(check.stdout, "OK\n");
    assert_eq!(check.stderr, "");
}

/// Suppression is scoped to the prelude, not to the diagnostic codes the
/// prelude happens to trigger: the same E2004/E3003 pair must still fire on the
/// user's own declarations, with the user file's spans.
#[test]
fn user_declarations_still_report_e2004_and_e3003() {
    let source = temp_source_file(
        "vibe_prelude_diag_user",
        "pub no_return_type() {\n  @effect net\n  1\n}\n\npub main() -> Int {\n  @effect io\n  println(\"x\")\n  0\n}\n",
    );
    let check = run_vibe(&["check", source.to_str().expect("source path str")]);
    assert!(
        check.status.success(),
        "check should succeed with warnings only:\nstdout:\n{}\nstderr:\n{}",
        check.stdout,
        check.stderr
    );
    assert_eq!(
        check.stdout,
        "E2004: warning: public function `no_return_type` should have explicit return type @ 1:5-4:1\n\
         E3003: info: declared effect `net` was not observed @ 1:5-4:1\n",
        "user-owned E2004/E3003 must survive prelude suppression"
    );
}

/// A genuine error that merely *names* a stdlib symbol belongs to the user's
/// code and must stay visible.
#[test]
fn user_error_naming_stdlib_symbol_is_not_suppressed() {
    let source = temp_source_file(
        "vibe_prelude_diag_stdlib_ref",
        "pub main() -> Int {\n  @effect io\n  n := log_info(\"x\")\n  n\n}\n",
    );
    let check = run_vibe(&["check", source.to_str().expect("source path str")]);
    assert!(
        !check.status.success(),
        "check should fail on an unknown identifier:\nstdout:\n{}\nstderr:\n{}",
        check.stdout,
        check.stderr
    );
    assert!(
        check
            .stdout
            .contains("E2001: error: unknown identifier `log_info` @ 3:8-3:15"),
        "expected the user's E2001 with its own span:\n{}",
        check.stdout
    );
}

/// A user declaration that collides with a prelude name is *not* prelude-owned,
/// so the resulting duplicate-definition errors stay visible.
#[test]
fn prelude_name_collision_still_fails_loudly() {
    let source = temp_source_file(
        "vibe_prelude_diag_collision",
        "pub spawn() -> Int {\n  7\n}\n\npub main() -> Int {\n  @effect io\n  println(\"x\")\n  0\n}\n",
    );
    let build = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        !build.status.success(),
        "redefining a prelude function must not build:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    assert!(
        build
            .stderr
            .contains("E2002: error: duplicate function `spawn`"),
        "expected the duplicate-definition error:\n{}",
        build.stderr
    );
}

/// Regression: suppression is scoped per declaration, and its one name-based
/// carve-out compares functions against functions. A user *type* named like a
/// prelude *function* (`spawn`, `ok`, `join`, `route`, … are injected unmangled)
/// must not resurrect that function's warnings — those point past the end of the
/// user's file, and unlike a function/function collision no error explains them.
#[test]
fn user_type_sharing_a_prelude_function_name_reports_nothing() {
    let source = temp_source_file(
        "vibe_prelude_diag_type_collision",
        "type spawn {\n  x: Int\n}\n\npub main() -> Int {\n  @effect io\n  println(\"x\")\n  0\n}\n",
    );
    let check = run_vibe(&["check", source.to_str().expect("source path str")]);
    assert!(
        check.status.success(),
        "check failed:\nstdout:\n{}\nstderr:\n{}",
        check.stdout,
        check.stderr
    );
    assert_eq!(
        check.stdout, "OK\n",
        "a user type named after a prelude function must not surface prelude diagnostics"
    );
    assert_eq!(check.stderr, "");
}

/// Suppression must not be able to turn a broken stdlib into a silent
/// miscompile. Points `VIBE_STDLIB_PATH` at a copy of the real stdlib carrying a
/// return-type mismatch and asserts the build still fails with that error.
#[test]
fn broken_stdlib_still_fails_the_build() {
    let stdlib_copy = unique_temp_dir("vibe_prelude_diag_broken_stdlib").join("std");
    copy_dir(&workspace_root().join("stdlib/std"), &stdlib_copy);
    let log_yb = stdlib_copy.join("log.yb");
    let mut log_src = fs::read_to_string(&log_yb).expect("read stdlib log.yb copy");
    log_src.push_str("\npub broken_ret() -> Int {\n  \"not an int\"\n}\n");
    fs::write(&log_yb, log_src).expect("write broken stdlib log.yb");

    let source = temp_source_file("vibe_prelude_diag_broken_stdlib_src", HELLO_WORLD);
    let build = run_vibe_with_stdlib(
        &["build", source.to_str().expect("source path str")],
        &stdlib_copy,
    );
    assert!(
        !build.status.success(),
        "a stdlib that does not type-check must fail the build, not compile silently:\n\
         stdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    assert!(
        build.stderr.contains(
            "E2201: error: return type mismatch in `broken_ret`: declared `Int`, inferred `Str`"
        ),
        "expected the prelude-origin error to survive suppression:\n{}",
        build.stderr
    );
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create stdlib copy dir");
    for entry in fs::read_dir(from).expect("read stdlib dir") {
        let entry = entry.expect("stdlib dir entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("entry file type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copy stdlib file");
        }
    }
}

/// Rust-side twin of `tooling/phase13/check_diagnostics_parity.py`: the CLI and
/// the editor must report the same diagnostic codes for the same file. The CLI
/// injects the stdlib prelude and the LSP does not, so any prelude leak shows up
/// here as extra CLI-only codes.
#[test]
fn cli_and_lsp_report_the_same_diagnostic_codes() {
    let fixture = workspace_root()
        .join("compiler/tests/fixtures/phase7/basic/typecheck")
        .join("typecheck__unknown_symbol_and_mismatch.yb");
    let source = fs::read_to_string(&fixture).expect("read parity fixture");
    let copy = temp_source_file("vibe_prelude_diag_parity", &source);

    let check = run_vibe(&["check", copy.to_str().expect("source path str")]);
    let cli_codes = diagnostic_codes(&format!("{}\n{}", check.stdout, check.stderr));
    assert!(
        !cli_codes.is_empty(),
        "no diagnostic codes captured from `vibe check`:\nstdout:\n{}\nstderr:\n{}",
        check.stdout,
        check.stderr
    );

    let index_root = unique_temp_dir("vibe_prelude_diag_parity_index");
    let mut session = vibe_lsp::LspSession::open_or_create(&index_root).expect("open lsp session");
    let lsp_codes: BTreeSet<String> = session
        .open_document(&copy, &source, None)
        .expect("lsp open document")
        .into_iter()
        .map(|d| d.code)
        .collect();

    assert_eq!(
        cli_codes, lsp_codes,
        "diagnostic code parity mismatch between `vibe check` and the LSP"
    );
}

/// Collects every `E<four digits>` token, matching how the CI parity script
/// scrapes codes out of CLI output.
fn diagnostic_codes(text: &str) -> BTreeSet<String> {
    let bytes: Vec<char> = text.chars().collect();
    let mut out = BTreeSet::new();
    for (idx, ch) in bytes.iter().enumerate() {
        if *ch != 'E' || idx + 4 >= bytes.len() {
            continue;
        }
        let digits = &bytes[idx + 1..idx + 5];
        if digits.iter().all(char::is_ascii_digit) {
            out.insert(std::iter::once('E').chain(digits.iter().copied()).collect());
        }
    }
    out
}

fn temp_source_file(prefix: &str, source: &str) -> PathBuf {
    let temp_dir = unique_temp_dir(prefix);
    fs::create_dir_all(&temp_dir).expect("create temp source dir");
    let file = temp_dir.join("main.yb");
    fs::write(&file, source).expect("write temp source");
    file
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos))
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
    run_vibe_inner(args, None)
}

fn run_vibe_with_stdlib(args: &[&str], stdlib_root: &Path) -> CmdOutput {
    run_vibe_inner(args, Some(stdlib_root))
}

fn run_vibe_inner(args: &[&str], stdlib_root: Option<&Path>) -> CmdOutput {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_vibe"));
    cmd.args(args).current_dir(workspace_root());
    if let Some(root) = stdlib_root {
        cmd.env("VIBE_STDLIB_PATH", root);
    }
    let output = cmd.output().expect("run vibe command");
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
