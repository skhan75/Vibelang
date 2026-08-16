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

/// `bytes.new`, `bytes.get`, `bytes.from_str`: the three cheap functions.
/// `new` allocates zeroed bytes and clamps a negative length to zero;
/// `get` is total, returning the byte value in range and `-1` at or past
/// either end; `from_str` stops at the first `0x00`, exactly like a `Str`
/// itself does, which this proves by round-tripping through a `to_str`
/// result that already lost its tail (see the module README).
#[test]
fn bytes_new_get_from_str_basics() {
    let source = temp_source_file(
        "bytes_new_get_from_str",
        r#"
pub main() -> Int {
  @effect io
  b := bytes.new(5)
  println(convert.to_str(bytes.len(b)))
  println(bytes.to_hex(b))
  neg := bytes.new(-3)
  println(convert.to_str(bytes.len(neg)))

  v := bytes.from_hex("00ff7f")
  println(convert.to_str(bytes.get(v, 0)))
  println(convert.to_str(bytes.get(v, 1)))
  println(convert.to_str(bytes.get(v, 2)))
  println(convert.to_str(bytes.get(v, -1)))
  println(convert.to_str(bytes.get(v, 3)))
  println(convert.to_str(bytes.get(v, 999)))

  plain := bytes.from_str("hello")
  println(convert.to_str(bytes.len(plain)))
  println(bytes.to_hex(plain))

  nb := bytes.from_hex("41004243")
  s := bytes.to_str(nb)
  back := bytes.from_str(s)
  println(convert.to_str(bytes.len(back)))
  println(bytes.to_hex(back))
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("utf-8 path")]);
    assert!(out.status.success(), "vibe run failed:\n{}", out.stderr);
    let lines: Vec<&str> = out.stdout.trim().lines().collect();
    assert_eq!(lines[0], "5", "new(5) has length 5");
    assert_eq!(lines[1], "0000000000", "new(5) is all zero bytes");
    assert_eq!(lines[2], "0", "new(-3) clamps to length 0");
    assert_eq!(lines[3], "0", "get(v, 0) == 0x00");
    assert_eq!(lines[4], "255", "get(v, 1) == 0xff");
    assert_eq!(lines[5], "127", "get(v, 2) == 0x7f");
    assert_eq!(lines[6], "-1", "get with a negative index");
    assert_eq!(lines[7], "-1", "get at index == len");
    assert_eq!(lines[8], "-1", "get past the end");
    assert_eq!(lines[9], "5", "from_str(\"hello\") has length 5");
    assert_eq!(lines[10], "68656c6c6f", "from_str(\"hello\") round-trips");
    assert_eq!(
        lines[11], "1",
        "from_str stops at the first NUL, same as Str"
    );
    assert_eq!(lines[12], "41", "only the byte before the NUL survives");
}

/// `bytes.slice` is the function with the only non-trivial logic of the
/// nine: clamped-bounds copying. This is the case a silent off-by-one
/// would hide in, so every documented clamp rule gets its own assertion:
/// `start<0 -> 0`, `start>len -> len`, `end>len -> len`, `end<start ->
/// empty`, plus the full range, an empty result, and a range that spans an
/// embedded NUL (checked with `to_hex`, never `to_str`, since `to_str`
/// cannot represent the NUL this test exists to catch).
#[test]
fn bytes_slice_clamps_to_documented_rules() {
    let source = temp_source_file(
        "bytes_slice_clamps",
        r#"
pub main() -> Int {
  @effect io
  buf := bytes.from_hex("0102030405")

  s1 := bytes.slice(buf, -3, 3)
  println(bytes.to_hex(s1))
  println(convert.to_str(bytes.len(s1)))

  s2 := bytes.slice(buf, 100, 100)
  println(bytes.to_hex(s2))
  println(convert.to_str(bytes.len(s2)))

  s3 := bytes.slice(buf, 0, 999)
  println(bytes.to_hex(s3))
  println(convert.to_str(bytes.len(s3)))

  s4 := bytes.slice(buf, 4, 1)
  println(bytes.to_hex(s4))
  println(convert.to_str(bytes.len(s4)))

  s5 := bytes.slice(buf, 0, 5)
  println(bytes.to_hex(s5))
  println(convert.to_str(bytes.len(s5)))

  s6 := bytes.slice(buf, 2, 2)
  println(bytes.to_hex(s6))
  println(convert.to_str(bytes.len(s6)))

  buf2 := bytes.from_hex("aa00bb00cc")
  s7 := bytes.slice(buf2, 1, 4)
  println(bytes.to_hex(s7))
  println(convert.to_str(bytes.len(s7)))
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("utf-8 path")]);
    assert!(out.status.success(), "vibe run failed:\n{}", out.stderr);
    let lines: Vec<&str> = out.stdout.trim().lines().collect();
    assert_eq!(lines[0], "010203", "start<0 clamps to 0");
    assert_eq!(lines[1], "3", "");
    assert_eq!(
        lines[2], "",
        "start>len and end>len both clamp to len -> empty"
    );
    assert_eq!(lines[3], "0", "");
    assert_eq!(lines[4], "0102030405", "end>len clamps to len -> full copy");
    assert_eq!(lines[5], "5", "");
    assert_eq!(lines[6], "", "end<start collapses to empty");
    assert_eq!(lines[7], "0", "");
    assert_eq!(lines[8], "0102030405", "the full range, unclamped");
    assert_eq!(lines[9], "5", "");
    assert_eq!(lines[10], "", "start==end is an empty result");
    assert_eq!(lines[11], "0", "");
    assert_eq!(
        lines[12], "00bb00",
        "a range spanning an embedded NUL is byte-exact, checked via to_hex"
    );
    assert_eq!(lines[13], "3", "");
}

/// `bytes.concat` is the other function with real logic. Joining two
/// values where at least one contains an embedded NUL must produce a
/// length equal to the sum and a hex encoding equal to the two operands'
/// hex encodings concatenated; joining with an empty value on either side
/// must be a no-op.
#[test]
fn bytes_concat_joins_and_preserves_nul_bytes() {
    let source = temp_source_file(
        "bytes_concat_joins",
        r#"
pub main() -> Int {
  @effect io
  a := bytes.from_hex("aa00bb")
  b := bytes.from_hex("00cc")
  c := bytes.concat(a, b)
  println(convert.to_str(bytes.len(c)))
  println(bytes.to_hex(c))

  empty := bytes.new(0)
  left_empty := bytes.concat(empty, a)
  println(convert.to_str(bytes.len(left_empty)))
  println(bytes.to_hex(left_empty))

  right_empty := bytes.concat(a, empty)
  println(convert.to_str(bytes.len(right_empty)))
  println(bytes.to_hex(right_empty))
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("utf-8 path")]);
    assert!(out.status.success(), "vibe run failed:\n{}", out.stderr);
    let lines: Vec<&str> = out.stdout.trim().lines().collect();
    assert_eq!(lines[0], "5", "joined length is the sum of the two lengths");
    assert_eq!(
        lines[1], "aa00bb00cc",
        "joined hex is the concatenation of the two hex strings, NUL preserved"
    );
    assert_eq!(
        lines[2], "3",
        "concat with an empty left operand is a no-op"
    );
    assert_eq!(lines[3], "aa00bb", "");
    assert_eq!(
        lines[4], "3",
        "concat with an empty right operand is a no-op"
    );
    assert_eq!(lines[5], "aa00bb", "");
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
