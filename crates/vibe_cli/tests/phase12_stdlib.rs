use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn phase12_stdlib_module_surface_runs_end_to_end() {
    let root = unique_temp_dir("vibe_phase12_stdlib_surface");
    fs::create_dir_all(&root).expect("create temp root");
    let text_file = root.join("note.txt");
    let made_dir = root.join("made");
    let source = temp_source_file(
        "phase12_stdlib_surface",
        &format!(
            r#"
pub main() -> Int {{
  @effect io
  println(path.join("/tmp", "vibe"))
  println(path.parent("/tmp/vibe/file.txt"))
  println(path.basename("/tmp/vibe/file.txt"))
  if path.is_absolute("/tmp") {{
    println("abs")
  }}
  println(json.stringify_i64(time.duration_ms(2)))
  println(json.stringify_i64(json.parse_i64("42")))
  println(json.minify("{{ \"a\" : 1 }}"))
  if json.is_valid("{{\"a\":1}}") {{
    println("json-ok")
  }}
  println(http.status_text(200))
  println(json.stringify_i64(http.default_port("https")))
  println(http.build_request_line("GET", "/ready"))
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
            text_file = text_file.display(),
            made_dir = made_dir.display()
        ),
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
        "/tmp/vibe\n/tmp/vibe\nfile.txt\nabs\n2000\n42\n{\"a\":1}\njson-ok\nOK\n443\nGET /ready HTTP/1.1\nwrite-ok\nexists-ok\nhello-stdlib\nmkdir-ok\n"
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
  println(json.minify("{ \"k\" : 1 }"))
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
    assert_eq!(out.stdout, "0\n\ninvalid\n");
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
