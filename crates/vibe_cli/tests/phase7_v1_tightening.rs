use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use vibe_diagnostics::Diagnostics;
use vibe_parser::parse_source;
use vibe_types::check_and_lower;

#[test]
fn phase7_algorithmic_recursion_samples_run_expected_outputs() {
    let fixtures = [
        (
            "phase7/stress/algorithmic/algorithmic__fibonacci_recursive.yb",
            "fib-ok\n",
        ),
        (
            "phase7/stress/algorithmic/algorithmic__factorial_recursive.yb",
            "factorial-ok\n",
        ),
        (
            "phase7/stress/algorithmic/algorithmic__generate_parentheses_count.yb",
            "paren-ok\n",
        ),
    ];
    for (relative, expected_stdout) in fixtures {
        let source = temp_fixture_copy(relative);
        let first = run_vibe(&["run", source.to_str().expect("source path str")]);
        let second = run_vibe(&["run", source.to_str().expect("source path str")]);
        assert!(
            first.status.success() && second.status.success(),
            "algorithmic sample failed for {}:\nfirst stdout:\n{}\nfirst stderr:\n{}\nsecond stdout:\n{}\nsecond stderr:\n{}",
            relative,
            first.stdout,
            first.stderr,
            second.stdout,
            second.stderr
        );
        assert_eq!(
            first.stdout, expected_stdout,
            "unexpected stdout for {relative}"
        );
        assert_eq!(
            first.stdout, second.stdout,
            "repeat run output should be deterministic for {relative}"
        );
    }
}

#[test]
fn phase7_memory_heap_pressure_smoke_is_bounded() {
    let fixtures = [
        (
            "phase7/stress/memory/memory__heap_pressure_loop.yb",
            "heap-ok\n",
        ),
        (
            "phase7/stress/memory/memory__container_pressure_loop.yb",
            "container-mem-ok\n",
        ),
    ];
    for (relative, expected_stdout) in fixtures {
        let source = temp_fixture_copy(relative);
        let start = Instant::now();
        let first = run_vibe(&["run", source.to_str().expect("source path str")]);
        let second = run_vibe(&["run", source.to_str().expect("source path str")]);
        let elapsed = start.elapsed();
        assert!(
            first.status.success() && second.status.success(),
            "memory smoke failed for {}:\nfirst stdout:\n{}\nfirst stderr:\n{}\nsecond stdout:\n{}\nsecond stderr:\n{}",
            relative,
            first.stdout,
            first.stderr,
            second.stdout,
            second.stderr
        );
        assert_eq!(
            first.stdout, expected_stdout,
            "unexpected memory smoke output for {relative}"
        );
        assert_eq!(
            first.stdout, second.stdout,
            "repeat memory smoke output should be deterministic for {relative}"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "memory smoke should remain bounded (<5s) for {}, got {:?}",
            relative,
            elapsed
        );
    }
}

#[test]
fn phase7_ownership_sendability_smokes_cover_positive_and_negative_paths() {
    let positive = temp_fixture_copy("phase7/stress/ownership/ownership__chan_sendable.yb");
    let out = run_vibe(&["run", positive.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "ownership positive smoke failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "ownership-ok\n");

    let container_sendable =
        fixture_path("phase7/stress/ownership/ownership__list_map_sendable.yb");
    let container_src =
        fs::read_to_string(&container_sendable).expect("read ownership sendable fixture");
    let container_diags = check_output(&container_src);
    assert!(
        !container_diags.has_errors(),
        "container sendable ownership fixture should pass:\n{}",
        container_diags.to_golden()
    );

    let negative = fixture_path("phase7/stress/ownership/ownership_err__unknown_sendability_go.yb");
    let src = fs::read_to_string(&negative).expect("read ownership negative fixture");
    let all = check_output(&src);
    let expected = negative.with_extension("diag");
    assert_golden(&expected, &all.to_golden());
    assert!(
        all.has_errors(),
        "ownership negative fixture should emit errors"
    );

    let negative_map = fixture_path("ownership_err/map_non_sendable_value_in_go.yb");
    let map_src = fs::read_to_string(&negative_map).expect("read map ownership negative fixture");
    let map_diags = check_output(&map_src);
    let map_expected = negative_map.with_extension("diag");
    assert_golden(&map_expected, &map_diags.to_golden());
    assert!(
        map_diags.has_errors(),
        "map ownership negative fixture should emit errors"
    );
}

#[test]
fn phase7_gc_observable_smoke_is_default_lane() {
    let source = temp_fixture_copy("phase7/stress/memory/memory__gc_observable_placeholder.yb");
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "gc-observable smoke failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "gc-observe-placeholder\n");
}

#[test]
fn phase7_memory_valgrind_leak_check_default_lane() {
    if !valgrind_available() {
        eprintln!("skipping valgrind leak smoke (valgrind not installed in this environment)");
        return;
    }

    let source = temp_fixture_copy("phase7/stress/memory/memory__heap_pressure_loop.yb");
    let build = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        build.status.success(),
        "build failed for valgrind smoke:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_path(&source, "dev", "x86_64-unknown-linux-gnu");
    let output = Command::new("valgrind")
        .args([
            "--leak-check=full",
            "--errors-for-leak-kinds=definite",
            "--error-exitcode=99",
            binary.to_str().expect("binary path str"),
        ])
        .current_dir(workspace_root())
        .output()
        .expect("run valgrind");
    assert!(
        output.status.success(),
        "valgrind leak smoke failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
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

fn valgrind_available() -> bool {
    Command::new("valgrind")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn temp_fixture_copy(relative: &str) -> PathBuf {
    let src = fixture_path(relative);
    let contents = fs::read_to_string(&src).expect("read fixture source");
    let file_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .expect("fixture file name");
    let temp_dir = unique_temp_dir("vibe_phase7_v1_tightening");
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

fn artifact_binary_path(source: &Path, profile: &str, target: &str) -> PathBuf {
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

struct CmdOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}
