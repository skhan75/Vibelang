// Copyright 2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Execution conformance for nested cross-namespace stdlib calls
//! (audit P0-12): `println(convert.to_str(text.byte_len("abc")))` and three
//! sibling combos built with exit 0 and then SIGSEGVed (exit 139) in the
//! built binary — strlen received the raw Int result of the inner call as a
//! char*. Root cause was NOT codegen argument materialization: the HIR
//! `convert.to_str` passthrough (vibe_types::lower_expr) guessed the inner
//! member call's type from its first argument (Str) and elided the `to_str`
//! call entirely. The fix consults the stdlib return-type registry for
//! namespace-call arguments.
//!
//! Matrix per the audit: the 4 crashing combos, Int-returning inner into
//! `convert.to_str`, Str-into-Str cross-namespace nestings, 3-deep nesting,
//! and the temp-variable form — each built AND executed, dev and release.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const MATRIX_SOURCE: &str = r#"pub main() -> Int {
  @effect io
  println(convert.to_str(text.byte_len("abc")))
  println(convert.to_str(text.index_of("abc", "b")))
  println(convert.to_str(text.byte_len(crypto.sha256("x"))))
  println(convert.to_str(text.byte_len(text.to_upper("abc"))))
  println(text.to_upper(encoding.base64_encode("hi")))
  println(text.to_upper(text.trim("  vibe  ")))
  println(convert.to_str(crypto.sha256("x")))
  println(text.split_part("a,b,c", ",", text.index_of("ab", "b")))
  s := convert.to_str(text.byte_len("abcdef"))
  println(s)
  0
}
"#;

/// Line-by-line ground truth:
/// - byte_len("abc") = 3; index_of("abc","b") = 1 (audit crash combos 1/3)
/// - byte_len(sha256("x")) = 64: sha256 hex digest length (audit combo 4,
///   3-deep, Str-returning innermost)
/// - byte_len(to_upper("abc")) = 3: same-arity 3-deep, Str-into-Str inner
/// - to_upper(base64_encode("hi")) = "AGK=": cross-namespace Str-into-Str
/// - to_upper(trim("  vibe  ")) = "VIBE": same-namespace Str-into-Str
/// - to_str(sha256("x")): Str-returning inner stays a correct passthrough
/// - split_part("a,b,c", ",", index_of("ab","b")) = "b": Int-returning inner
///   as a non-first argument
/// - temp-variable form prints 6
const MATRIX_EXPECTED: &str = "3\n1\n64\n3\nAGK=\nVIBE\n\
2d711642b726b04401627ca9fbac32f5c8530fb1903cc4db02258717921a4881\nb\n6\n";

#[test]
fn nested_cross_namespace_matrix_dev_binary() {
    build_and_expect(MATRIX_SOURCE, "dev", MATRIX_EXPECTED, "p012_matrix_dev");
}

#[test]
fn nested_cross_namespace_matrix_release_binary() {
    build_and_expect(
        MATRIX_SOURCE,
        "release",
        MATRIX_EXPECTED,
        "p012_matrix_release",
    );
}

#[test]
fn audit_repro_temp_binding_runs_clean() {
    // Audit P0-12 repro 2, verbatim: the binding form crashed identically
    // (the passthrough elided `to_str` at the `:=` site, so `s` held Int 3).
    build_and_expect(
        r#"pub main() -> Int {
  @effect io
  s := convert.to_str(text.byte_len("abc"))
  println(s)
  0
}
"#,
        "dev",
        "3\n",
        "p012_temp_binding",
    );
}

/// Build `source` at `profile`, then execute the produced native binary and
/// assert exit 0 with exactly `expected` on stdout.
fn build_and_expect(source: &str, profile: &str, expected: &str, tag: &str) {
    let dir = unique_temp_dir(tag);
    fs::create_dir_all(&dir).expect("create temp dir");
    let source_path = dir.join("main.yb");
    fs::write(&source_path, source).expect("write temp source");
    let source_str = source_path.to_str().expect("source path str");
    let args: Vec<&str> = match profile {
        "dev" => vec!["build", source_str],
        _ => vec!["build", source_str, "--profile", profile],
    };
    let build = run_vibe(&args);
    assert!(
        build.status.success(),
        "build ({profile}) failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_for_host(&dir, profile, "main");
    let output = Command::new(&binary)
        .current_dir(&dir)
        .output()
        .expect("run built binary");
    assert!(
        output.status.success(),
        "built binary ({profile}) failed (status {:?}):\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

/// Resolve `<source dir>/.yb/artifacts/<profile>/<host triple>/<stem>` without
/// hardcoding a target triple: scan the single per-host triple directory the
/// build produced.
fn artifact_binary_for_host(source_dir: &std::path::Path, profile: &str, stem: &str) -> PathBuf {
    let profile_dir = source_dir.join(".yb").join("artifacts").join(profile);
    let mut candidates: Vec<PathBuf> = fs::read_dir(&profile_dir)
        .unwrap_or_else(|e| panic!("read artifacts dir {}: {e}", profile_dir.display()))
        .filter_map(|entry| {
            let triple_dir = entry.expect("read artifacts entry").path();
            let candidate = triple_dir.join(stem);
            candidate.is_file().then_some(candidate)
        })
        .collect();
    candidates.sort();
    assert_eq!(
        candidates.len(),
        1,
        "expected exactly one built artifact for `{stem}` under {}, found {candidates:?}",
        profile_dir.display()
    );
    candidates.remove(0)
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
