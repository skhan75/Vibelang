// Copyright 2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Handle-based stdlib APIs (str_builder, chan, net/ws) hand out opaque Int
// handles. The native runtime keeps a live-handle registry per family, so a
// forged, stale, or arithmetically-derived handle must abort with a clean
// named panic (SIGABRT, exit 134) instead of dereferencing arbitrary memory
// (SIGSEGV) or touching an unrelated file descriptor. Legitimate usage must
// be unchanged.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
#[test]
fn forged_str_builder_handle_panics_cleanly() {
    let out = run_program(
        "handle_forged_sb",
        r#"
pub main() -> Int {
  @effect alloc
  @effect mut_state
  str_builder.append(12345, "x")
  0
}
"#,
    );
    assert_clean_handle_panic(&out, "invalid str_builder handle");
}

#[cfg(unix)]
#[test]
fn stale_str_builder_handle_after_finish_panics_cleanly() {
    let out = run_program(
        "handle_stale_sb",
        r#"
pub main() -> Int {
  @effect io
  @effect alloc
  @effect mut_state
  sb := str_builder.new(4)
  str_builder.append(sb, "x")
  println(str_builder.finish(sb))
  str_builder.append(sb, "use-after-finish")
  0
}
"#,
    );
    assert_clean_handle_panic(&out, "invalid str_builder handle");
}

#[test]
fn legit_str_builder_chain_is_unchanged() {
    let out = run_program(
        "handle_legit_sb",
        r#"
pub main() -> Int {
  @effect io
  @effect alloc
  @effect mut_state
  sb := str_builder.new(64)
  str_builder.append(sb, "hello")
  str_builder.append(sb, " world")
  println(str_builder.finish(sb))
  0
}
"#,
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "expected clean exit:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("hello world"),
        "builder chain output missing:\n{}",
        out.stdout
    );
}

#[cfg(unix)]
#[test]
fn forged_channel_handle_panics_cleanly() {
    // Channels are nominally typed at the language level, so forging one
    // takes a direct @native binding with an Int handle.
    let out = run_program(
        "handle_forged_chan",
        r#"
fake_send(h: Int, v: Int) -> Int {
  @native("vibe_chan_send_i64")
}

pub main() -> Int {
  fake_send(12345, 7)
  0
}
"#,
    );
    assert_clean_handle_panic(&out, "invalid channel handle");
}

#[test]
fn legit_channel_round_trip_is_unchanged() {
    let out = run_program(
        "handle_legit_chan",
        r#"
pub main() -> Int {
  @effect io
  @effect alloc
  @effect concurrency
  @effect mut_state
  ch := chan(1)
  ch.send(42)
  println(convert.to_str(ch.recv()))
  0
}
"#,
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "expected clean exit:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("42"),
        "channel round-trip output missing:\n{}",
        out.stdout
    );
}

#[cfg(unix)]
#[test]
fn forged_net_write_handle_panics_cleanly() {
    let out = run_program(
        "handle_forged_net_write",
        r#"
pub main() -> Int {
  @effect net
  net.write(12345, "x")
  0
}
"#,
    );
    assert_clean_handle_panic(&out, "invalid net handle");
}

#[cfg(unix)]
#[test]
fn forged_net_close_handle_panics_cleanly() {
    // net.close on a forged fd could close an unrelated descriptor owned
    // by the process; it must panic instead.
    let out = run_program(
        "handle_forged_net_close",
        r#"
pub main() -> Int {
  @effect net
  net.close(12345)
  0
}
"#,
    );
    assert_clean_handle_panic(&out, "invalid net handle");
}

#[cfg(unix)]
#[test]
fn net_write_to_non_net_fd_panics_cleanly() {
    // fd 1 is stdout: open in the process, but never handed out by
    // net.listen/accept/connect, so it is not a net handle.
    let out = run_program(
        "handle_net_stdout_fd",
        r#"
pub main() -> Int {
  @effect net
  net.write(1, "injected")
  0
}
"#,
    );
    assert_clean_handle_panic(&out, "invalid net handle");
}

#[cfg(unix)]
#[test]
fn forged_ws_frame_handle_panics_cleanly() {
    let out = run_program(
        "handle_forged_ws",
        r#"
pub main() -> Int {
  @effect net
  ws.write_frame(12345, "x")
  0
}
"#,
    );
    assert_clean_handle_panic(&out, "invalid net handle");
}

#[test]
fn legit_net_listen_port_close_is_unchanged() {
    let out = run_program(
        "handle_legit_net",
        r#"
pub main() -> Int {
  @effect io
  @effect net
  listener := net.listen("127.0.0.1", 0)
  if net.listener_port(listener) > 0 {
    println("port-ok")
  }
  if net.close(listener) {
    println("close-ok")
  }
  0
}
"#,
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "expected clean exit:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("port-ok") && out.stdout.contains("close-ok"),
        "net listener lifecycle output missing:\n{}",
        out.stdout
    );
}

fn assert_clean_handle_panic(out: &CmdOutput, message: &str) {
    assert_eq!(
        out.status.code(),
        Some(134),
        "expected 128+6 (SIGABRT) from vibe_panic:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr.contains(&format!("panic: {message}")),
        "missing `panic: {message}` on stderr:\n{}",
        out.stderr
    );
    assert!(
        !out.stderr.contains("SIGSEGV"),
        "forged handle must not segfault:\n{}",
        out.stderr
    );
}

fn run_program(prefix: &str, source: &str) -> CmdOutput {
    let dir = unique_temp_dir(prefix);
    fs::create_dir_all(&dir).expect("create temp dir");
    let file = dir.join("main.yb");
    fs::write(&file, source.trim_start()).expect("write temp source");
    let output = Command::new(env!("CARGO_BIN_EXE_vibe"))
        .args(["run", file.to_str().expect("source path str")])
        .current_dir(&dir)
        .output()
        .expect("run vibe command");
    CmdOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos))
}

struct CmdOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}
