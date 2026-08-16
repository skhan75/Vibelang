// Copyright 2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! VL-HTTP-01 / VL-HTTP-02: a single 0x80-0xBF byte in a header value or in ANY
//! query parameter value used to abort the whole server process (all concurrent
//! connections plus the listener), because `std.http_router` sliced peer bytes
//! with the `s[a:b]` sugar and `vibe_str_slice` rejects a non-UTF-8 boundary by
//! calling `vibe_panic`.
//!
//! The router now uses `text.slice_bytes`, which clamps instead of rejecting and
//! copies bytes verbatim. These tests assert BYTE-IDENTITY, not "looks right":
//! the value the application receives must be exactly the octets the peer sent,
//! at exactly the length the peer sent. That is the property a sanitising fix
//! (substituting U+FFFD, which is 3 bytes) destroys -- it shifts every later
//! offset and makes subsequent slices land mid-character.
//!
//! The hostile byte is placed FIRST, MIDDLE and LAST in every position, because
//! a probe set whose values all start with a safe ASCII byte never exercises the
//! failing slice at all: the abort needs the byte at `colon + 1` (header) or
//! `eq + 1` (query) to be a continuation byte.
//!
//! Invalid UTF-8 cannot be written in a `.yb` literal (the lexer reads source as
//! a Rust String and has no `\xNN` escape), so hostile bytes are minted with
//! `encoding.hex_decode` -- itself one of the runtime's unvalidated byte sources.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Header values: the hostile byte first, middle and last, plus header blocks
/// where the hostile byte is the FIRST byte of a later header line (which makes
/// the scan cursor `pos` itself land on a continuation byte).
#[test]
fn header_get_returns_hostile_bytes_verbatim_without_aborting() {
    let source = temp_source_file(
        "router_hostile_header",
        r#"
pub main() -> Int {
  @effect io
  b80 := encoding.hex_decode("80")
  bbf := encoding.hex_decode("bf")
  ba9 := encoding.hex_decode("a9")

  // hostile continuation byte FIRST in the value (VL-HTTP-01)
  println(encoding.hex_encode(http_router.header_get("X-T:" + b80 + "tail", "X-T")))
  println(encoding.hex_encode(http_router.header_get("X-T:" + bbf + "tail", "X-T")))
  println(encoding.hex_encode(http_router.header_get("X-T:" + ba9, "X-T")))
  // the exact saved crash input VL-HTTP-01: "Host:" then a bare 0x80
  println(encoding.hex_encode(http_router.header_get("Host:" + b80, "Host")))
  // hostile byte MIDDLE and LAST
  println(encoding.hex_encode(http_router.header_get("X-T:a" + b80 + "b", "X-T")))
  println(encoding.hex_encode(http_router.header_get("X-T:ab" + bbf, "X-T")))
  // hostile byte starts a LATER header line, wanted key comes after it
  println(encoding.hex_encode(http_router.header_get("A: 1\r\n" + b80 + "B: 2\r\nC: 3", "C")))
  println(encoding.hex_encode(http_router.header_get("A: 1\r\n" + bbf + "B: 2\r\nC: 3", "C")))
  println(encoding.hex_encode(http_router.header_get("A: 1\r\n" + ba9 + "B: 2\r\nC: 3", "C")))
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "hostile header bytes must not abort the process:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(
        out.stdout,
        concat!(
            "807461696c\n", // 0x80 "tail"  -- hostile byte first
            "bf7461696c\n", // 0xBF "tail"
            "a9\n",         // lone 0xA9 (a stranded UTF-8 continuation byte)
            "80\n",         // saved crash input VL-HTTP-01
            "618062\n",     // "a" 0x80 "b" -- hostile byte middle
            "6162bf\n",     // "ab" 0xBF    -- hostile byte last
            "33\n",         // "3" from "C: 3" behind a hostile line start
            "33\n",
            "33\n",
        ),
        "header values must be byte-identical to what the peer sent"
    );
}

/// Query values, including parameters the application never reads: the value
/// slice used to be computed for every parameter before the key was compared,
/// so an unread parameter could abort the process (VL-HTTP-02).
#[test]
fn query_get_returns_hostile_bytes_verbatim_without_aborting() {
    let source = temp_source_file(
        "router_hostile_query",
        r#"
pub main() -> Int {
  @effect io
  b80 := encoding.hex_decode("80")
  bbf := encoding.hex_decode("bf")
  ba9 := encoding.hex_decode("a9")

  // hostile byte FIRST in the value of the parameter the app DOES read
  println(encoding.hex_encode(http_router.query_get("b=" + b80 + "xyz", "b")))
  println(encoding.hex_encode(http_router.query_get("b=" + bbf + "xyz", "b")))
  println(encoding.hex_encode(http_router.query_get("b=" + ba9, "b")))
  // hostile byte MIDDLE and LAST
  println(encoding.hex_encode(http_router.query_get("b=x" + b80 + "y", "b")))
  println(encoding.hex_encode(http_router.query_get("b=xy" + bbf, "b")))
  // hostile byte lives in a parameter the app NEVER reads (VL-HTTP-02)
  println(encoding.hex_encode(http_router.query_get("a=1&q=" + b80 + "zz&b=9", "b")))
  println(encoding.hex_encode(http_router.query_get("zzz=" + b80, "b")))
  // hostile byte at the START of an unread parameter, so the scan cursor lands on it
  println(encoding.hex_encode(http_router.query_get("a=1&" + bbf + "q=2&b=9", "b")))
  println(encoding.hex_encode(http_router.query_get("a=1&" + b80 + "q=2&b=9", "b")))
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "hostile query bytes must not abort the process:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(
        out.stdout,
        concat!(
            "8078797a\n", // 0x80 "xyz" -- hostile byte first
            "bf78797a\n", // 0xBF "xyz"
            "a9\n",       // lone 0xA9
            "788079\n",   // "x" 0x80 "y" -- middle
            "7879bf\n",   // "xy" 0xBF    -- last
            "39\n",       // "9" -- hostile byte was in unread parameter "q"
            "\n",         // saved crash input VL-HTTP-02 shape: "b" is absent
            "39\n",       // hostile byte started unread parameter "q"
            "39\n",
        ),
        "query values must be byte-identical to what the peer sent"
    );
}

/// Well-formed multi-byte UTF-8 must survive the router unchanged. `slice_bytes`
/// drops the boundary check, so this is the guard against it silently mangling
/// legitimate input.
#[test]
fn router_preserves_well_formed_multibyte_utf8() {
    let source = temp_source_file(
        "router_legit_multibyte",
        r#"
pub main() -> Int {
  @effect io
  two := encoding.hex_decode("c3a9")
  three := encoding.hex_decode("e282ac")
  four := encoding.hex_decode("f09f92a9")

  println(encoding.hex_encode(http_router.header_get("X-T:" + two, "X-T")))
  println(encoding.hex_encode(http_router.header_get("X-T:" + three, "X-T")))
  println(encoding.hex_encode(http_router.header_get("X-T:" + four, "X-T")))
  println(encoding.hex_encode(http_router.header_get("X-T:a" + two + "b", "X-T")))
  println(encoding.hex_encode(http_router.query_get("b=" + two, "b")))
  println(encoding.hex_encode(http_router.query_get("b=" + four, "b")))
  println(encoding.hex_encode(http_router.query_get("a=" + three + "&b=" + four, "b")))
  println(http_router.query_get("q=hi&b=ok", "b"))
  println(http_router.header_get("Host: example.test\r\nX-Trace: abc", "X-Trace"))
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
        concat!(
            "c3a9\n",
            "e282ac\n",
            "f09f92a9\n",
            "61c3a962\n",
            "c3a9\n",
            "f09f92a9\n",
            "f09f92a9\n",
            "ok\n",
            "abc\n",
        ),
        "well-formed UTF-8 and ASCII must round-trip through the router unchanged"
    );
}

/// `text.slice_bytes` is TOTAL: out-of-range bounds clamp instead of aborting,
/// and the result is always a verbatim subrange. This is what makes the router
/// safe no matter what offsets `text.index_of` computes on peer bytes.
#[test]
fn slice_bytes_clamps_instead_of_panicking() {
    let source = temp_source_file(
        "slice_bytes_total",
        r#"
pub main() -> Int {
  @effect io
  s := "abcde"
  println(text.slice_bytes(s, 0, 99))
  println(text.slice_bytes(s, -7, 2))
  println("[" + text.slice_bytes(s, 4, 1) + "]")
  println("[" + text.slice_bytes(s, 99, 120) + "]")
  println("[" + text.slice_bytes("", -1, 5) + "]")
  // a mid-character cut is returned verbatim, never rewritten
  println(encoding.hex_encode(text.slice_bytes(encoding.hex_decode("c3a9"), 1, 2)))
  println(convert.to_str(text.byte_at("abc", 0)))
  println(convert.to_str(text.byte_at("abc", 99)))
  println(convert.to_str(text.byte_at(encoding.hex_decode("c3a9"), 1)))
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "slice_bytes/byte_at must never abort:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(
        out.stdout,
        concat!(
            "abcde\n", // end clamped to len
            "ab\n",    // start clamped to 0
            "[]\n",    // end < start collapses to empty
            "[]\n",    // start beyond len clamps to len
            "[]\n",    // empty input
            "a9\n",    // mid-character cut returned verbatim, NOT replaced
            "97\n",    // 'a'
            "-1\n",    // out of range
            "169\n",   // 0xA9 continuation byte read without a boundary check
        )
    );
}

/// The panic contract for TRUSTED program data is unchanged. A program that
/// slices its own string across a character boundary is a real bug and must
/// still fail loudly -- the leniency is scoped to code paths that handle bytes
/// from strangers, not to values that look invalid.
#[test]
fn trusted_slice_sugar_still_panics_on_bad_boundary() {
    for (label, program, expected) in [
        (
            "mid_character",
            "s := \"caf\u{e9}\"\n  t := s[0:4]\n  println(t)",
            "panic: string slice boundary is not UTF-8 aligned",
        ),
        (
            "out_of_range",
            "s := \"abc\"\n  t := s[0:99]\n  println(t)",
            "panic: invalid string slice range",
        ),
        (
            "index_mid_character",
            "s := \"caf\u{e9}\"\n  b := s[4]\n  println(convert.to_str(b))",
            "panic: string index is not a UTF-8 boundary",
        ),
    ] {
        let source = temp_source_file(
            &format!("trusted_panic_{label}"),
            &format!(
                r#"
pub main() -> Int {{
  @effect io
  {program}
  0
}}
"#
            ),
        );
        let out = run_vibe(&["run", source.to_str().expect("source path str")]);
        assert!(
            !out.status.success(),
            "trusted slice `{label}` must still abort:\nstdout:\n{}\nstderr:\n{}",
            out.stdout,
            out.stderr
        );
        assert!(
            out.stderr.contains(expected),
            "trusted slice `{label}` must keep its panic message `{expected}`:\nstderr:\n{}",
            out.stderr
        );
    }
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
