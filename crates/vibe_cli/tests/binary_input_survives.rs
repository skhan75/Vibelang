// Copyright 2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Measured on 2026-08-16 against a compiled server using `net.read`: 16 bytes
//! of PNG header on the wire, 8 bytes seen by the program, because `Str` length
//! is `strlen` and the header's ninth byte is 0x00. `net.read_bytes` and
//! `fs.read_bytes` keep all 16.

use std::fs;
use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// 89 50 4e 47 0d 0a 1a 0a 00 00 00 0d 49 48 44 52 — a real PNG header. The
/// three zero bytes at offsets 8, 9 and 10 are why this is the chosen payload.
const PNG_HEADER: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
];

#[test]
fn a_png_header_survives_fs_read_bytes() {
    let dir = unique_temp_dir("bytes_fs");
    fs::create_dir_all(&dir).expect("create temp dir");
    let blob = dir.join("header.png");
    fs::write(&blob, PNG_HEADER).expect("write binary fixture");

    let source = temp_source_file(
        "bytes_fs_read",
        &format!(
            r#"
pub main() -> Int {{
  @effect io
  b := fs.read_bytes("{}")
  println(convert.to_str(bytes.len(b)))
  println(bytes.to_hex(b))
  0
}}
"#,
            blob.display()
        ),
    );
    let out = run_vibe(&["run", source.to_str().expect("utf-8 path")]);
    assert!(out.status.success(), "vibe run failed:\n{}", out.stderr);
    let lines: Vec<&str> = out.stdout.trim().lines().collect();
    assert_eq!(lines[0], "16", "every byte of the file must be readable");
    assert_eq!(lines[1], "89504e470d0a1a0a0000000d49484452");
}

#[test]
fn a_png_header_survives_net_read_bytes() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("local addr").port();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        stream.write_all(PNG_HEADER).expect("write payload");
    });

    let source = temp_source_file(
        "bytes_net_read",
        r#"
pub main() -> Int {
  @effect io
  @effect net
  port := convert.to_int(cli.arg(0))
  conn := net.connect("127.0.0.1", port)
  b := net.read_bytes(conn, 16384)
  println(convert.to_str(bytes.len(b)))
  println(bytes.to_hex(b))
  net.close(conn)
  0
}
"#,
    );
    let out = run_vibe(&[
        "run",
        source.to_str().expect("utf-8 path"),
        "--",
        &port.to_string(),
    ]);
    server.join().expect("server thread");
    assert!(out.status.success(), "vibe run failed:\n{}", out.stderr);
    let lines: Vec<&str> = out.stdout.trim().lines().collect();
    assert_eq!(lines[0], "16", "every byte off the socket must survive");
    assert_eq!(lines[1], "89504e470d0a1a0a0000000d49484452");
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
    status: ExitStatus,
    stdout: String,
    stderr: String,
}
