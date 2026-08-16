// Copyright 2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! A `Bytes` value must keep every byte, including 0x00, which is exactly what
//! `Str` cannot do. The `00` bytes in the middle of the hex input are the whole
//! test: measured on 2026-08-16, a 16-byte PNG header read through `net.read`
//! reached the program as 8 bytes, because `Str` length is `strlen`.

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
fn bytes_keeps_nul_bytes_that_str_would_truncate() {
    let source = temp_source_file(
        "bytes_roundtrip",
        r#"
pub main() -> Int {
  @effect io
  b := bytes.from_hex("89504e470d0a1a0a00000000")
  println(convert.to_str(bytes.len(b)))
  println(bytes.to_hex(b))
  s := bytes.to_str(b)
  println(convert.to_str(s.len()))
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("utf-8 path")]);
    assert!(out.status.success(), "vibe run failed:\n{}", out.stderr);
    let lines: Vec<&str> = out.stdout.trim().lines().collect();
    assert_eq!(lines[0], "12", "Bytes must report all 12 bytes");
    assert_eq!(
        lines[1], "89504e470d0a1a0a00000000",
        "byte-identical round trip"
    );
    assert_eq!(
        lines[2], "8",
        "Str truncates at the first NUL, which is why Bytes exists"
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
