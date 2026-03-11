// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn phase12_stdlib_module_surface_runs_end_to_end() {
    let root = unique_temp_dir("vibe_phase12_stdlib_surface");
    fs::create_dir_all(&root).expect("create temp root");
    let text_file = root.join("note.txt");
    let made_dir = root.join("made");
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local tcp test server");
    let server_port = listener.local_addr().expect("listener addr").port();
    let server_thread = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept client");
        let mut buf = [0u8; 16];
        let _ = stream.read(&mut buf).expect("read client payload");
        stream.write_all(b"pong").expect("write server response");
    });
    let http_listener = TcpListener::bind("127.0.0.1:0").expect("bind local http test server");
    let http_port = http_listener
        .local_addr()
        .expect("http listener addr")
        .port();
    let http_thread = std::thread::spawn(move || {
        for _ in 0..3 {
            let (mut stream, _) = http_listener.accept().expect("accept http client");
            let mut req = [0u8; 4096];
            let n = stream.read(&mut req).expect("read http request");
            let raw = String::from_utf8_lossy(&req[..n]);
            let body = if raw.starts_with("POST ") {
                "posted"
            } else {
                "ready"
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write http response");
        }
    });
    let source = temp_source_file(
        "phase12_stdlib_surface",
        &format!(
            r#"
type User {{
  id: Int,
  name: Str,
  active: Bool
}}

pub main() -> Int {{
  @effect io
  @effect alloc
  @effect net
  println(path.join("/tmp", "vibe"))
  println(path.parent("/tmp/vibe/file.txt"))
  println(path.basename("/tmp/vibe/file.txt"))
  if path.is_absolute("/tmp") {{
    println("abs")
  }}
  println(json.stringify_i64(time.duration_ms(2)))
  println(convert.to_str(convert.to_int("123")))
  println(convert.to_str_f64(convert.to_float("3.5")))
  println(convert.to_str(convert.parse_i64("7")))
  println(convert.to_str_f64(convert.parse_f64("2.25")))
  println(text.trim("  hi  "))
  if text.contains("abc", "b") {{
    println("contains")
  }}
  if text.starts_with("abc", "a") {{
    println("starts")
  }}
  if text.ends_with("abc", "c") {{
    println("ends")
  }}
  println(text.replace("a-b-c", "-", "+"))
  println(text.to_lower("HeLLo"))
  println(text.to_upper("HeLLo"))
  println(json.stringify_i64(text.byte_len("abc")))
  println(text.split_part("a,b,c", ",", 1))
  println(encoding.hex_encode("A"))
  println(encoding.hex_decode("4142"))
  println(encoding.base64_encode("hi"))
  println(encoding.base64_decode("aGk="))
  println(encoding.url_encode("a b"))
  println(encoding.url_decode("a%20b"))
  if time.monotonic_now_ms() > 0 {{
    println("mono-ok")
  }}
  println(json.stringify_i64(json.parse_i64("42")))
  println(json.minify("{{ \"a\" : 1 }}"))
  println(json.parse("{{ \"a\" : 1 }}"))
  println(json.stringify("{{\"b\":2}}"))
  println(json.stringify("hello-stdlib"))
  fallback := User {{ id: 1, name: "fallback", active: false }}
  decoded := json.decode_User("{{\"id\":7,\"name\":\"sam\",\"active\":true}}", fallback)
  println(json.encode_User(decoded))
  println(json.stringify_i64(decoded.id))
  println(decoded.name)
  if decoded.active {{
    println("active")
  }}
  decoded2 := json.decode_User("{{\"id\":2}}", fallback)
  println(json.encode_User(decoded2))
  println(json.stringify_i64(decoded2.id))
  println(decoded2.name)
  if decoded2.active {{
    println("active-2")
  }} else {{
    println("inactive-2")
  }}
  if json.is_valid("{{\"a\":1}}") {{
    println("json-ok")
  }}
  println(http.status_text(200))
  println(json.stringify_i64(http.default_port("https")))
  println(http.build_request_line("GET", "/ready"))
  println(env.get("VIBE_PHASE12_ENV"))
  if env.has("VIBE_PHASE12_ENV") {{
    println("env-has")
  }}
  println(env.get_required("VIBE_PHASE12_ENV"))
  println(json.stringify_i64(cli.args_len()))
  println(cli.arg(0))
  log.error("phase12-log")
  println(http.get("http://127.0.0.1:{http_port}/ready", 2000))
  println(json.stringify_i64(http.request_status("GET", "http://127.0.0.1:{http_port}/ready", "", 2000)))
  println(http.post("http://127.0.0.1:{http_port}/submit", "payload", 2000))
  listener := net.listen("127.0.0.1", 0)
  if net.listener_port(listener) > 0 {{
    println("listen-ok")
  }}
  if net.close(listener) {{
    println("listen-close-ok")
  }}
  conn := net.connect("127.0.0.1", {server_port})
  println(json.stringify_i64(net.write(conn, "ping")))
  println(net.read(conn, 16))
  if net.close(conn) {{
    println("net-close-ok")
  }}
  println(net.resolve("localhost"))
  if fs.write_text("{text_file}", "hello-stdlib") {{
    println("write-ok")
  }}
  if fs.exists("{text_file}") {{
    println("exists-ok")
  }}
  println(fs.read_text("{text_file}"))
  if fs.create_dir("{made_dir}") {{
    println("mkdir-ok")
  }}
  0
}}
"#,
            server_port = server_port,
            http_port = http_port,
            text_file = text_file.display(),
            made_dir = made_dir.display()
        ),
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    server_thread.join().expect("join local tcp test server");
    http_thread.join().expect("join local http test server");
    assert!(
        out.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(
        out.stdout,
        "/tmp/vibe\n/tmp/vibe\nfile.txt\nabs\n2000\n123\n3.5\n7\n2.25\nhi\ncontains\nstarts\nends\na+b+c\nhello\nHELLO\n3\nb\n41\nAB\naGk=\nhi\na%20b\na b\nmono-ok\n42\n{\"a\":1}\n{\"a\":1}\n{\"b\":2}\n\"hello-stdlib\"\n{\"id\":7,\"name\":\"sam\",\"active\":true}\n7\nsam\nactive\n{\"id\":2,\"name\":\"fallback\",\"active\":false}\n2\nfallback\ninactive-2\njson-ok\nOK\n443\nGET /ready HTTP/1.1\nphase12\nenv-has\nphase12\n0\n\nready\n200\nposted\nlisten-ok\nlisten-close-ok\n4\npong\nnet-close-ok\n127.0.0.1\nwrite-ok\nexists-ok\nhello-stdlib\nmkdir-ok\n"
    );
}

#[cfg(feature = "bench-runtime")]
#[test]
fn phase12_bench_stdlib_surface_runs_end_to_end() {
    let source = temp_source_file(
        "phase12_bench_stdlib_surface",
        r#"
pub main() -> Int {
  @effect io
  @effect net
  println(bench.md5_hex("abc"))
  println(bench.json_canonical("{ \"a\" : 1 }"))
  println(bench.json_repeat_array(bench.json_canonical("{\"a\":1}"), 3))
  println(json.stringify_i64(bench.http_server_bench(10)))
  println(bench.secp256k1(1))
  print(bench.edigits(27))
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
        "900150983cd24fb0d6963f7d28e17f72\n{\"a\":1}\n[{\"a\":1},{\"a\":1},{\"a\":1}]\n55\nbac4db182bd8e59da66ec3b0e1759a102ff7308a916cccb51c68253d4bf32c16,a6bf6c1c8c9261f65983cfb987e0e8b4b8a7cc34376454b27f5adf7e36dc15d0\n2718281828\t:10\n4590452353\t:20\n6028747   \t:27\n"
    );
}

#[test]
fn phase12_stdlib_outputs_are_deterministic_across_runs() {
    let source = temp_source_file(
        "phase12_stdlib_deterministic",
        r#"
pub main() -> Int {
  @effect io
  println(path.join("/a", "b"))
  println(convert.to_str(convert.to_int("88")))
  println(convert.to_str_f64(convert.parse_f64("1.75")))
  println(json.minify("{ \"k\" : 1 }"))
  println(json.parse("{\"k\":1}"))
  println(json.stringify("k=v"))
  println(http.build_request_line("POST", "/submit"))
  println(json.stringify_i64(time.duration_ms(3)))
  0
}
"#,
    );
    let first = run_vibe(&["run", source.to_str().expect("source path str")]);
    let second = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        first.status.success() && second.status.success(),
        "determinism run failed:\nfirst stdout:\n{}\nfirst stderr:\n{}\nsecond stdout:\n{}\nsecond stderr:\n{}",
        first.stdout,
        first.stderr,
        second.stdout,
        second.stderr
    );
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn phase12_stdlib_error_model_is_stable() {
    let missing = unique_temp_dir("vibe_phase12_stdlib_missing")
        .join("missing.txt")
        .display()
        .to_string();
    let source = temp_source_file(
        "phase12_stdlib_error_model",
        &format!(
            r#"
pub main() -> Int {{
  @effect io
  println(json.stringify_i64(json.parse_i64("not-a-number")))
  println(convert.to_str(convert.to_int("not-an-int")))
  println(json.parse("not-json"))
  println(fs.read_text("{missing}"))
  if json.is_valid("nope") {{
    println("bad")
  }} else {{
    println("invalid")
  }}
  0
}}
"#
        ),
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "0\n0\n\n\ninvalid\n");
}

#[test]
fn phase12_stdlib_type_errors_are_reported_for_invalid_calls() {
    let source = temp_source_file(
        "phase12_stdlib_type_error",
        r#"
pub main() -> Int {
  @effect io
  fs.write_text("x.txt", 1)
}
"#,
    );
    let out = run_vibe(&["check", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "check unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout
            .contains("`fs.write_text` argument 2 expects `Str`, got `Int`"),
        "expected stdlib argument type mismatch diagnostic:\n{}",
        out.stdout
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
    let output = Command::new(vibe_bin())
        .args(args)
        .env("VIBE_PHASE12_ENV", "phase12")
        .current_dir(workspace_root())
        .output()
        .expect("run vibe command");
    CmdOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn vibe_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vibe")
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
