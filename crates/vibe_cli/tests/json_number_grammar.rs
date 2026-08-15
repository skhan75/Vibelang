// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! RFC 8259 number grammar in the native JSON runtime: negative exponents
//! must parse, '+' is only legal as an exponent sign, and malformed numbers
//! must be rejected consistently by `json.parse` (panic) and `json.is_valid`
//! (false).

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn json_parse_accepts_rfc8259_exponents_and_round_trips() {
    let source = temp_source_file(
        "json_number_exponents",
        r#"
pub main() -> Int {
  @effect io
  println(json.stringify(json.parse("1e-5")))
  println(json.stringify(json.parse("-2.5e-10")))
  println(json.stringify(json.parse("1e+5")))
  println(json.stringify(json.parse("1E-3")))
  println(json.stringify(json.parse("1e300")))
  println(json.stringify(json.parse("[0.1, 1e300, -2.5e-10]")))
  println(json.stringify(json.parse(json.stringify(json.parse("1e-5")))))
  println(json.stringify(json.parse(json.stringify(json.parse("[0.1, 1e300, -2.5e-10]")))))
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(
        out.stdout,
        "1e-05\n-2.5e-10\n1e+05\n0.001\n1e+300\n[0.1,1e+300,-2.5e-10]\n1e-05\n[0.1,1e+300,-2.5e-10]\n"
    );
}

#[test]
fn json_parse_rejects_malformed_numbers_with_clean_panic() {
    // Each malformed input aborts the process, so every case runs alone.
    for (label, input) in [
        ("dangling_neg_exp", "1e-"),
        ("dangling_pos_exp", "1e+"),
        ("leading_plus", "+1"),
        ("sign_after_point", "1.-5"),
    ] {
        let source = temp_source_file(
            &format!("json_number_reject_{label}"),
            &format!(
                r#"
pub main() -> Int {{
  @effect io
  println(json.stringify(json.parse("{input}")))
  0
}}
"#
            ),
        );
        let out = run_vibe(&["run", source.to_str().expect("source path str")]);
        assert!(
            !out.status.success(),
            "json.parse(\"{input}\") unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
            out.stdout,
            out.stderr
        );
        assert!(
            out.stderr.contains("panic: json.parse invalid"),
            "json.parse(\"{input}\") should panic with the documented error model:\nstderr:\n{}",
            out.stderr
        );
    }
}

#[test]
fn json_is_valid_agrees_with_parse_on_number_grammar() {
    let source = temp_source_file(
        "json_number_is_valid",
        r#"
pub main() -> Int {
  @effect io
  print_validity(json.is_valid("1e-5"))
  print_validity(json.is_valid("-2.5e-10"))
  print_validity(json.is_valid("1e+5"))
  print_validity(json.is_valid("1E-3"))
  print_validity(json.is_valid("1e300"))
  print_validity(json.is_valid("[0.1, 1e300, -2.5e-10]"))
  print_validity(json.is_valid("1e-"))
  print_validity(json.is_valid("1e+"))
  print_validity(json.is_valid("+1"))
  print_validity(json.is_valid("1.-5"))
  0
}

print_validity(flag: Bool) -> Int {
  @effect io
  if flag {
    println("valid")
  } else {
    println("invalid")
  }
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(
        out.stdout,
        "valid\nvalid\nvalid\nvalid\nvalid\nvalid\ninvalid\ninvalid\ninvalid\ninvalid\n"
    );
}

fn temp_source_file(prefix: &str, source: &str) -> PathBuf {
    let dir = unique_temp_dir(prefix);
    fs::create_dir_all(&dir).expect("create temp source dir");
    let file = dir.join("main.yb");
    fs::write(&file, source.trim_start()).expect("write temp source file");
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
