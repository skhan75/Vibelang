use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn phase12_vibe_test_supports_filter_and_json_report() {
    let suite = temp_suite_dir("vibe_phase12_test_filter");
    let out = run_vibe(&[
        "test",
        suite.to_str().expect("suite str"),
        "--filter",
        "alpha",
        "--json",
    ]);
    assert!(
        out.status.success(),
        "vibe test failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(out.stdout.contains("\"files_discovered\": 3"));
    assert!(out.stdout.contains("\"files_selected\": 1"));
    assert!(out.stdout.contains("\"filter\": \"alpha\""));
}

#[test]
fn phase12_vibe_test_supports_sharding() {
    let suite = temp_suite_dir("vibe_phase12_test_shard");
    let shard1 = run_vibe(&[
        "test",
        suite.to_str().expect("suite str"),
        "--shard",
        "1/2",
        "--json",
    ]);
    let shard2 = run_vibe(&[
        "test",
        suite.to_str().expect("suite str"),
        "--shard",
        "2/2",
        "--json",
    ]);
    assert!(
        shard1.status.success() && shard2.status.success(),
        "sharded test run failed:\nshard1 stdout:\n{}\nshard1 stderr:\n{}\nshard2 stdout:\n{}\nshard2 stderr:\n{}",
        shard1.stdout,
        shard1.stderr,
        shard2.stdout,
        shard2.stderr
    );
    assert!(shard1.stdout.contains("\"files_selected\": 2"));
    assert!(shard1.stdout.contains("\"shard\": \"1/2\""));
    assert!(shard2.stdout.contains("\"files_selected\": 1"));
    assert!(shard2.stdout.contains("\"shard\": \"2/2\""));
}

#[test]
fn phase12_vibe_test_rejects_invalid_shard_specs() {
    let suite = temp_suite_dir("vibe_phase12_test_bad_shard");
    let out = run_vibe(&[
        "test",
        suite.to_str().expect("suite str"),
        "--shard",
        "0/2",
    ]);
    assert!(
        !out.status.success(),
        "invalid shard spec unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(out.stderr.contains("shard index must be in range"));
}

fn temp_suite_dir(prefix: &str) -> PathBuf {
    let dir = unique_temp_dir(prefix);
    fs::create_dir_all(&dir).expect("create suite dir");
    fs::write(dir.join("alpha.yb"), "pub main() -> Int { 0 }\n").expect("write alpha");
    fs::write(dir.join("beta.yb"), "pub main() -> Int { 0 }\n").expect("write beta");
    fs::write(dir.join("gamma.yb"), "pub main() -> Int { 0 }\n").expect("write gamma");
    dir
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
