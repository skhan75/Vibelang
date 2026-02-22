use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn native_contracts_pass_for_valid_inputs() {
    let source = temp_source_file(
        "contract_runtime_pass",
        r#"
inc_checked(x: Int) -> Int {
  @require x >= 0
  @ensure . > x
  x + 1
}

pub main() -> Int {
  @effect io
  inc_checked(4)
  println("contract-ok")
  0
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "native contract pass path failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "contract-ok\n");
}

#[test]
fn native_contract_require_failure_is_blocking() {
    let source = temp_source_file(
        "contract_runtime_require_fail",
        r#"
needs_positive(x: Int) -> Int {
  @require x > 0
  @ensure . > 0
  x
}

pub main() -> Int {
  needs_positive(0)
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "native require failure unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr
            .contains("contract @require failed in native execution"),
        "missing deterministic require failure marker:\n{}",
        out.stderr
    );
}

#[test]
fn built_binary_preserves_native_contract_enforcement() {
    let source = temp_source_file(
        "contract_runtime_binary_ensure_fail",
        r#"
ensure_gt(x: Int) -> Int {
  @ensure . > x
  x
}

pub main() -> Int {
  ensure_gt(7)
}
"#,
    );
    let build = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        build.status.success(),
        "build failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_path(&source, "dev", "x86_64-unknown-linux-gnu");
    let output = Command::new(&binary)
        .current_dir(workspace_root())
        .output()
        .expect("run built binary");
    assert!(
        !output.status.success(),
        "built binary unexpectedly succeeded for ensure violation:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("contract @ensure failed in native execution"),
        "missing deterministic ensure failure marker:\n{}",
        stderr
    );
}

fn temp_source_file(prefix: &str, source: &str) -> PathBuf {
    let dir = unique_temp_dir(prefix);
    fs::create_dir_all(&dir).expect("create temp dir");
    let file = dir.join("main.yb");
    fs::write(&file, source.trim_start()).expect("write temp source");
    file
}

fn artifact_binary_path(source: &PathBuf, profile: &str, target: &str) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("source stem");
    source
        .parent()
        .expect("source parent")
        .join(".yb")
        .join("artifacts")
        .join(profile)
        .join(target)
        .join(stem)
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
