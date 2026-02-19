use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use vibe_diagnostics::Diagnostics;
use vibe_parser::parse_source;
use vibe_types::check_and_lower;

#[test]
fn phase7_concurrency_samples_run_expected_outputs() {
    let fixtures = [
        ("phase7/advanced/concurrency/concurrency__worker_pool.yb", "worker-pool-ok\n"),
        ("phase7/advanced/concurrency/concurrency__fan_in.yb", "fan-in-ok\n"),
        ("phase7/advanced/concurrency/concurrency__fan_out.yb", "fan-out-ok\n"),
        (
            "phase7/advanced/concurrency/concurrency__timeout_retry.yb",
            "retry-attempt-1\nretry-ok\n",
        ),
        ("phase7/stress/concurrency/concurrency__bounded_stress.yb", "stress-ok\n"),
    ];

    for (relative, expected_stdout) in fixtures {
        let source = temp_fixture_copy(relative);
        let out = run_vibe(&["run", source.to_str().expect("source path str")]);
        assert!(
            out.status.success(),
            "concurrency sample run failed for {}:\nstdout:\n{}\nstderr:\n{}",
            relative,
            out.stdout,
            out.stderr
        );
        assert_eq!(
            out.stdout, expected_stdout,
            "unexpected stdout for concurrency sample {relative}"
        );
    }
}

#[test]
fn phase7_concurrency_samples_are_deterministic_and_bounded() {
    let fixtures = [
        "phase7/advanced/concurrency/concurrency__worker_pool.yb",
        "phase7/advanced/concurrency/concurrency__fan_in.yb",
        "phase7/advanced/concurrency/concurrency__fan_out.yb",
        "phase7/advanced/concurrency/concurrency__timeout_retry.yb",
        "phase7/stress/concurrency/concurrency__bounded_stress.yb",
    ];
    for relative in fixtures {
        let source = temp_fixture_copy(relative);
        let start = Instant::now();
        let first = run_vibe(&["run", source.to_str().expect("source path str")]);
        let second = run_vibe(&["run", source.to_str().expect("source path str")]);
        let elapsed = start.elapsed();
        assert!(
            first.status.success() && second.status.success(),
            "repeat run failed for {}:\nfirst stdout:\n{}\nfirst stderr:\n{}\nsecond stdout:\n{}\nsecond stderr:\n{}",
            relative,
            first.stdout,
            first.stderr,
            second.stdout,
            second.stderr
        );
        assert_eq!(
            first.stdout, second.stdout,
            "repeat run output should be deterministic for {relative}"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "concurrency fixture should remain bounded (<5s): {} took {:?}",
            relative,
            elapsed
        );
    }
}

#[test]
fn phase7_concurrency_negative_fixtures_match_golden() {
    let fixtures = [
        "phase7/advanced/concurrency_err/concurrency_err__member_capture_in_go.yb",
        "phase7/advanced/concurrency_err/concurrency_err__shared_member_write.yb",
    ];
    for relative in fixtures {
        let file = fixture_path(relative);
        let src = fs::read_to_string(&file).expect("read concurrency err fixture");
        let all = check_output(&src);
        let expected = file.with_extension("diag");
        assert_golden(&expected, &all.to_golden());
        assert!(
            all.has_errors(),
            "concurrency_err fixture should emit errors: {relative}"
        );
    }
}

fn check_output(src: &str) -> Diagnostics {
    let parsed = parse_source(src);
    let checked = check_and_lower(&parsed.ast);
    let mut all = Diagnostics::default();
    all.extend(parsed.diagnostics.into_sorted());
    all.extend(checked.diagnostics.into_sorted());
    all
}

fn assert_golden(path: &Path, actual: &str) {
    if std::env::var("UPDATE_GOLDEN").as_deref() == Ok("1") {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create golden parent");
        }
        fs::write(path, actual).expect("write golden output");
        return;
    }
    let expected = fs::read_to_string(path).unwrap_or_else(|_| {
        panic!(
            "missing golden file: {} (run tests with UPDATE_GOLDEN=1 to generate)",
            path.display()
        )
    });
    assert_eq!(expected, actual, "golden mismatch at {}", path.display());
}

fn temp_fixture_copy(relative: &str) -> PathBuf {
    let src = fixture_path(relative);
    let contents = fs::read_to_string(&src).expect("read fixture source");
    let file_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .expect("fixture file name");
    let temp_dir = unique_temp_dir("vibe_phase7_concurrency");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let dst = temp_dir.join(file_name);
    fs::write(&dst, contents).expect("write temp fixture");
    dst
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

fn fixture_path(relative: &str) -> PathBuf {
    workspace_root()
        .join("compiler/tests/fixtures")
        .join(relative)
}

struct CmdOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}
