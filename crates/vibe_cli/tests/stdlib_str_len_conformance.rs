// Copyright 2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Conformance: `.len()` (and slicing) must work on strings returned by every
//! Str-returning stdlib function.
//!
//! Regression context (2026-08 audit, P0-15): codegen classified string
//! receivers with a hardcoded stdlib allowlist. Calls missing from that list
//! (e.g. `json.stringify_i64`, `crypto.uuid_v4`, `http_router.*`,
//! `metrics.snapshot_json`, `str_builder.finish`) had `.len()` dispatched to
//! `vibe_container_len`, which read the string bytes as a container tag and
//! aborted with "panic: container len called on unsupported container".
//! The fix consults the stdlib return-type registry (`__ns_call__<ns>.<fn>`)
//! as ground truth, so every `-> Str` stdlib function is covered uniformly.
//!
//! Str-returning stdlib functions enumerated from stdlib/std/*.yb:
//!   cli.arg
//!   convert.to_str, convert.to_str_f64, convert.format_f64
//!   crypto.sha256, crypto.hmac_sha256, crypto.random_bytes, crypto.uuid_v4
//!   encoding.hex_encode, encoding.hex_decode, encoding.base64_encode,
//!   encoding.base64_decode, encoding.url_encode, encoding.url_decode
//!   env.get, env.get_required
//!   fs.read_text
//!   http.status_text, http.build_request_line, http.build_response,
//!   http.response, http.request (*)
//!   http_router.header_get, http_router.query_get, http_router.json_response,
//!   http_router.text_response, http_router.route
//!   http_server.format_response, http_server.cors_headers
//!   json.stringify, json.stringify_pretty, json.stringify_i64, json.minify
//!   json.encode / json.builder.finish (compiler-special-cased codecs/builder)
//!   metrics.snapshot_json
//!   net.read (*), net.resolve
//!   path.join, path.parent, path.basename
//!   regex.replace_all
//!   str_builder.finish
//!   text.trim, text.replace, text.to_lower, text.to_upper, text.split_part
//!   ws.read_frame (*)
//!
//! (*) http.request, net.read, and ws.read_frame need a live remote/socket, so
//! this test does not invoke them. Their classification comes from the same
//! registry (and they were already in the old allowlist), and their socket
//! behavior is covered by phase12_stdlib.rs.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const CONFORMANCE_SOURCE: &str = r#"
type P015User {
  id: Int,
  name: Str
}

pub main() -> Int {
  @effect io
  @effect alloc
  @effect net
  @effect nondet
  println(convert.to_str(convert.to_str(42).len()))
  println(convert.to_str(convert.to_str_f64(1.5).len()))
  println(convert.to_str(convert.format_f64(1.5, 2).len()))
  println(convert.to_str(crypto.sha256("abc").len()))
  println(convert.to_str(crypto.hmac_sha256("key", "data").len()))
  println(convert.to_str(crypto.random_bytes(8).len()))
  println(convert.to_str(crypto.uuid_v4().len()))
  println(convert.to_str(encoding.hex_encode("hi").len()))
  println(convert.to_str(encoding.hex_decode("6869").len()))
  println(convert.to_str(encoding.base64_encode("hi").len()))
  println(convert.to_str(encoding.base64_decode("aGk=").len()))
  println(convert.to_str(encoding.url_encode("a b").len()))
  println(convert.to_str(encoding.url_decode("a%20b").len()))
  println(convert.to_str(env.get("VIBE_STRLEN_CONF").len()))
  println(convert.to_str(env.get_required("VIBE_STRLEN_CONF").len()))
  println(convert.to_str(fs.read_text("VIBE_STRLEN_FILE").len()))
  println(convert.to_str(http.status_text(200).len()))
  println(convert.to_str(http.build_request_line("GET", "/x").len()))
  println(convert.to_str(http.build_response(200, "hi").len()))
  println(convert.to_str(http.response(HttpResponse { status: 200, headers: "", body: "hi" }).len()))
  println(convert.to_str(http_router.header_get("X-K: v\r\n", "X-K").len()))
  println(convert.to_str(http_router.query_get("a=1&bb=22", "bb").len()))
  println(convert.to_str(http_router.json_response(200, "\{}").len()))
  println(convert.to_str(http_router.text_response(200, "hi").len()))
  req := http_server.parse_request("GET /x?a=1 HTTP/1.1\r\nHost: h\r\n\r\n")
  println(convert.to_str(http_router.route(req, "GET", "/x", fn(r: HttpServerRequest) -> Str { "handled:" + r.path }, "fb").len()))
  println(convert.to_str(http_server.format_response(200, "", "hi").len()))
  println(convert.to_str(http_server.cors_headers().len()))
  println(convert.to_str(json.stringify(json.parse("\{\"k\":1}")).len()))
  println(convert.to_str(json.stringify_pretty(json.parse("\{\"k\":1}")).len()))
  println(convert.to_str(json.stringify_i64(12345).len()))
  println(convert.to_str(json.minify(" \{ \"k\" : 1 } ").len()))
  println(convert.to_str(metrics.snapshot_json().len()))
  println(convert.to_str(path.join("/a", "b").len()))
  println(convert.to_str(path.parent("/a/b").len()))
  println(convert.to_str(path.basename("/a/b").len()))
  println(convert.to_str(regex.replace_all("aaa", "a", "b").len()))
  h := str_builder.new(8)
  str_builder.append(h, "ab")
  println(convert.to_str(str_builder.finish(h).len()))
  println(convert.to_str(text.trim(" x ").len()))
  println(convert.to_str(text.replace("aaa", "a", "b").len()))
  println(convert.to_str(text.to_lower("AB").len()))
  println(convert.to_str(text.to_upper("ab").len()))
  println(convert.to_str(text.split_part("a,b,c", ",", 1).len()))
  if cli.arg(99).len() == 0 {
    println("cli_arg_ok")
  }
  if net.resolve("localhost").len() >= 3 {
    println("resolve_ok")
  }
  println(convert.to_str(json.encode(P015User { id: 1, name: "x" }).len()))
  jb := json.builder.new(64)
  jb = json.builder.begin_object(jb)
  jb = json.builder.key(jb, "k")
  jb = json.builder.value_i64(jb, 5)
  jb = json.builder.end_object(jb)
  println(convert.to_str(json.builder.finish(jb).len()))
  u := json.stringify_i64(98765)
  println(u[1:4])
  0
}
"#;

const CONFORMANCE_EXPECTED: &str = "2\n3\n4\n64\n64\n16\n36\n4\n2\n4\n2\n5\n3\n7\n7\n9\n2\n15\n215\n215\n1\n2\n262\n256\n10\n59\n154\n7\n12\n5\n7\n27\n4\n2\n1\n3\n2\n1\n3\n2\n2\n1\ncli_arg_ok\nresolve_ok\n19\n7\n876\n";

#[test]
fn stdlib_str_returns_support_len_and_slice() {
    let dir = unique_temp_dir("vibe_stdlib_str_len_conformance");
    fs::create_dir_all(&dir).expect("create temp dir");
    let input = dir.join("input.txt");
    fs::write(&input, "hello-fs!").expect("write fs.read_text fixture");
    let source_path = dir.join("main.yb");
    let source = CONFORMANCE_SOURCE
        .trim_start()
        .replace("VIBE_STRLEN_FILE", input.to_str().expect("input path str"));
    fs::write(&source_path, source).expect("write temp source");
    let out = run_vibe(
        &["run", source_path.to_str().expect("source path str")],
        &[("VIBE_STRLEN_CONF", "seven77")],
    );
    assert!(
        out.status.success(),
        "conformance run failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, CONFORMANCE_EXPECTED);
}

#[test]
fn stringify_i64_len_audit_repro_runs_clean() {
    // Audit P0-15 repro: this aborted with
    // "panic: container len called on unsupported container" (exit 134).
    let dir = unique_temp_dir("vibe_stringify_i64_len_repro");
    fs::create_dir_all(&dir).expect("create temp dir");
    let source_path = dir.join("main.yb");
    fs::write(
        &source_path,
        r#"pub main() -> Int {
  @effect io
  t := json.stringify_i64(12345)
  n := t.len()
  println(convert.to_str(n))
  0
}
"#,
    )
    .expect("write temp source");
    let out = run_vibe(
        &["run", source_path.to_str().expect("source path str")],
        &[],
    );
    assert!(
        out.status.success(),
        "repro run failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "5\n");
}

#[test]
fn stringify_i64_len_audit_repro_built_binary_runs_clean() {
    let dir = unique_temp_dir("vibe_stringify_i64_len_repro_bin");
    fs::create_dir_all(&dir).expect("create temp dir");
    let source_path = dir.join("main.yb");
    fs::write(
        &source_path,
        r#"pub main() -> Int {
  @effect io
  t := json.stringify_i64(12345)
  println(convert.to_str(t.len()))
  0
}
"#,
    )
    .expect("write temp source");
    let build = run_vibe(
        &["build", source_path.to_str().expect("source path str")],
        &[],
    );
    assert!(
        build.status.success(),
        "build failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_for_host(&dir, "dev", "main");
    let output = Command::new(&binary)
        .current_dir(&dir)
        .output()
        .expect("run built binary");
    assert!(
        output.status.success(),
        "built binary failed (status {:?}):\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "5\n");
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

fn run_vibe(args: &[&str], envs: &[(&str, &str)]) -> CmdOutput {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_vibe"));
    cmd.args(args).current_dir(workspace_root());
    for (key, value) in envs {
        cmd.env(key, value);
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
