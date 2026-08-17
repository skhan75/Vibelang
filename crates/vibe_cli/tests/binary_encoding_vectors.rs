// Copyright 2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Checked against published vectors rather than against our own output, so a
//! wrong implementation cannot certify itself. Base64 vectors are RFC 4648
//! section 10; the SHA-256 value for empty input is the published one.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::time::{SystemTime, UNIX_EPOCH};

struct CmdOutput {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

#[test]
fn base64_matches_the_rfc4648_vectors() {
    let source = temp_source_file(
        "bytes_base64_vectors",
        r#"
pub main() -> Int {
  @effect io
  println(encoding.base64_encode_bytes(bytes.from_str("")))
  println(encoding.base64_encode_bytes(bytes.from_str("f")))
  println(encoding.base64_encode_bytes(bytes.from_str("fo")))
  println(encoding.base64_encode_bytes(bytes.from_str("foo")))
  println(encoding.base64_encode_bytes(bytes.from_str("foob")))
  println(encoding.base64_encode_bytes(bytes.from_str("fooba")))
  println(encoding.base64_encode_bytes(bytes.from_str("foobar")))
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("utf-8 path")]);
    assert!(out.status.success(), "vibe run failed:\n{}", out.stderr);
    let lines: Vec<&str> = out.stdout.trim_end().lines().collect();
    assert_eq!(
        lines,
        vec!["", "Zg==", "Zm8=", "Zm9v", "Zm9vYg==", "Zm9vYmE=", "Zm9vYmFy"]
    );
}

#[test]
fn sha256_of_empty_input_matches_the_published_value() {
    let source = temp_source_file(
        "bytes_sha256_empty",
        r#"
pub main() -> Int {
  @effect io
  println(crypto.sha256_bytes(bytes.new(0)))
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("utf-8 path")]);
    assert!(out.status.success(), "vibe run failed:\n{}", out.stderr);
    assert_eq!(
        out.stdout.trim(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn hashing_binary_input_uses_every_byte() {
    // The strlen-based `crypto.sha256` sees both of these as one zero-length
    // input and returns the same digest. The byte-taking form must not.
    let source = temp_source_file(
        "bytes_sha256_binary",
        r#"
pub main() -> Int {
  @effect io
  short := bytes.from_hex("00")
  long := bytes.from_hex("0001020300")
  println(crypto.sha256_bytes(short))
  println(crypto.sha256_bytes(long))
  println(convert.to_str(bytes.len(long)))
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("utf-8 path")]);
    assert!(out.status.success(), "vibe run failed:\n{}", out.stderr);
    let lines: Vec<&str> = out.stdout.trim().lines().collect();
    assert_ne!(
        lines[0], lines[1],
        "two different byte sequences must not share a digest"
    );
    assert_eq!(lines[2], "5", "all five bytes must be hashed");
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
