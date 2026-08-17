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

/// Fix round 1 finding I2: `base64_matches_the_rfc4648_vectors` above only
/// ever feeds NUL-free ASCII through `base64_encode_bytes`, so it cannot
/// distinguish the correct struct-`len` implementation from a `strlen`-based
/// one -- for those inputs `strlen == len` by construction. This test uses
/// `bytes.from_hex("0001020300")`, which both starts AND ends with 0x00, so
/// a `strlen`-based length reads as 0 and a correct one reads as 5. Expected
/// output computed independently with Python's `base64.b64encode`, not by
/// reading this implementation's own output back.
#[test]
fn base64_encode_bytes_survives_embedded_nul_bytes() {
    let source = temp_source_file(
        "bytes_base64_embedded_nul",
        r#"
pub main() -> Int {
  @effect io
  b := bytes.from_hex("0001020300")
  println(encoding.base64_encode_bytes(b))
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("utf-8 path")]);
    assert!(out.status.success(), "vibe run failed:\n{}", out.stderr);
    assert_eq!(
        out.stdout.trim(),
        "AAECAwA=",
        "base64 of bytes 00 01 02 03 00, independently computed"
    );
}

/// Fix round 1 finding I1: `base64_decode_bytes` had zero test coverage --
/// the only reference to it anywhere in the repo was its own declaration.
/// This round-trips four inputs chosen to exercise every padding arm of the
/// shared decode loop (`vibe_base64_decode_core`): a zero-length input, a
/// 4-byte input (length % 3 == 1, the double-`==`-padding arm), a 5-byte
/// input (length % 3 == 2, the single-`=`-padding arm), and a 12-byte input
/// (length % 3 == 0, no padding). Every one of the four also carries at
/// least one embedded 0x00 byte (three of them start or end with one), so
/// this exercises `base64_decode_bytes`'s new scratch/size-arithmetic/error
/// path on real binary data, not just on the encode side. Byte-exactness is
/// checked through `bytes.to_hex`, never `bytes.to_str` (which would itself
/// truncate at the first 0x00 and hide exactly the bug this task fixes).
#[test]
fn base64_round_trips_binary_data_through_encode_and_decode() {
    let source = temp_source_file(
        "bytes_base64_round_trip",
        r#"
pub main() -> Int {
  @effect io

  a := bytes.new(0)
  enc_a := encoding.base64_encode_bytes(a)
  dec_a := encoding.base64_decode_bytes(enc_a)
  println(enc_a)
  println(bytes.to_hex(dec_a))

  b := bytes.from_hex("ff0000ff")
  enc_b := encoding.base64_encode_bytes(b)
  dec_b := encoding.base64_decode_bytes(enc_b)
  println(enc_b)
  println(bytes.to_hex(dec_b))

  c := bytes.from_hex("0001020300")
  enc_c := encoding.base64_encode_bytes(c)
  dec_c := encoding.base64_decode_bytes(enc_c)
  println(enc_c)
  println(bytes.to_hex(dec_c))

  d := bytes.from_hex("89504e470d0a1a0a00000000")
  enc_d := encoding.base64_encode_bytes(d)
  dec_d := encoding.base64_decode_bytes(enc_d)
  println(enc_d)
  println(bytes.to_hex(dec_d))

  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("utf-8 path")]);
    assert!(out.status.success(), "vibe run failed:\n{}", out.stderr);
    // trim_end, not trim: the first two printed lines are legitimately empty
    // (the zero-length case), and a plain trim() would eat them.
    let lines: Vec<&str> = out.stdout.trim_end().lines().collect();

    assert_eq!(lines[0], "", "empty input encodes to an empty string");
    assert_eq!(lines[1], "", "empty input round-trips to an empty Bytes");

    assert_eq!(
        lines[2], "/wAA/w==",
        "4 bytes (len%3==1): the double-'==' padding arm, independently computed"
    );
    assert_eq!(
        lines[3], "ff0000ff",
        "4-byte round trip must reproduce the original bytes exactly"
    );

    assert_eq!(
        lines[4], "AAECAwA=",
        "5 bytes (len%3==2): the single-'=' padding arm, independently computed"
    );
    assert_eq!(
        lines[5], "0001020300",
        "5-byte round trip must reproduce the original bytes, including both embedded zeros"
    );

    assert_eq!(
        lines[6], "iVBORw0KGgoAAAAA",
        "12 bytes (len%3==0): the no-padding arm, independently computed"
    );
    assert_eq!(
        lines[7], "89504e470d0a1a0a00000000",
        "12-byte round trip must reproduce the original bytes, including all four trailing zeros"
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
