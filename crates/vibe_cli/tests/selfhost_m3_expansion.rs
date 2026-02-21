use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use vibe_diagnostics::Diagnostics;
use vibe_mir::{lower_hir_to_mir, mir_debug_dump};
use vibe_parser::parse_source;
use vibe_types::check_and_lower;

#[test]
fn host_shadow_outputs_match_m3_slice_fixtures() {
    let artifact_root = artifact_root();
    fs::create_dir_all(&artifact_root).expect("create M3 artifact root");
    for fixture in slice_fixtures() {
        let src_path = workspace_root().join(fixture.input_fixture);
        let src = fs::read_to_string(&src_path)
            .unwrap_or_else(|e| panic!("read source fixture `{}`: {e}", src_path.display()));
        let host = host_output_for_slice(fixture.kind, &src);

        let expected_path = workspace_root().join(fixture.selfhost_fixture);
        let expected = fs::read_to_string(&expected_path).unwrap_or_else(|e| {
            panic!(
                "read selfhost fixture output `{}`: {e}",
                expected_path.display()
            )
        });
        let diff = line_diff(&expected, &host);
        write_slice_artifacts(&artifact_root, fixture.id, &host, &expected, &diff);

        assert_eq!(
            host,
            expected,
            "M3 shadow parity mismatch for `{}`; inspect artifacts under `{}`",
            fixture.id,
            artifact_root.display()
        );
    }
}

#[test]
fn m3_slice_repeat_runs_are_deterministic() {
    for fixture in slice_fixtures() {
        let src_path = workspace_root().join(fixture.input_fixture);
        let src = fs::read_to_string(&src_path)
            .unwrap_or_else(|e| panic!("read source fixture `{}`: {e}", src_path.display()));
        let first = host_output_for_slice(fixture.kind, &src);
        let second = host_output_for_slice(fixture.kind, &src);
        assert_eq!(
            first, second,
            "M3 host output is not deterministic for `{}`",
            fixture.id
        );
    }
}

#[test]
fn m3_shadow_performance_budgets_are_within_thresholds() {
    let loops = parse_env_usize("VIBE_M3_PERF_LOOPS", 20);
    let max_latency_overhead_pct = parse_env_f64("VIBE_M3_MAX_LATENCY_OVERHEAD_PCT", 400.0);
    let max_memory_overhead_bytes = parse_env_usize("VIBE_M3_MAX_MEMORY_OVERHEAD_BYTES", 32768);
    let loaded = load_slices();
    let artifact_root = artifact_root();
    fs::create_dir_all(&artifact_root).expect("create M3 artifact root for perf metrics");

    let baseline_start = Instant::now();
    let mut baseline_peak_bytes = 0usize;
    for _ in 0..loops {
        for slice in &loaded {
            let host = host_output_for_slice(slice.kind, &slice.src);
            baseline_peak_bytes = baseline_peak_bytes.max(host.len());
        }
    }
    let baseline_ms = baseline_start.elapsed().as_secs_f64() * 1000.0;

    let shadow_start = Instant::now();
    let mut shadow_peak_bytes = 0usize;
    for _ in 0..loops {
        for slice in &loaded {
            let host = host_output_for_slice(slice.kind, &slice.src);
            let diff = line_diff(&slice.expected, &host);
            shadow_peak_bytes =
                shadow_peak_bytes.max(host.len() + slice.expected.len() + diff.len());
            assert_eq!(
                host, slice.expected,
                "shadow performance pass observed parity drift in `{}`",
                slice.id
            );
        }
    }
    let shadow_ms = shadow_start.elapsed().as_secs_f64() * 1000.0;

    let normalized_baseline_ms = baseline_ms.max(1.0);
    let latency_overhead_pct = ((shadow_ms - baseline_ms) / normalized_baseline_ms) * 100.0;
    let memory_overhead_bytes = shadow_peak_bytes.saturating_sub(baseline_peak_bytes);

    write_performance_metrics(
        &artifact_root,
        loops,
        baseline_ms,
        shadow_ms,
        latency_overhead_pct,
        baseline_peak_bytes,
        shadow_peak_bytes,
        memory_overhead_bytes,
    );

    assert!(
        latency_overhead_pct <= max_latency_overhead_pct,
        "M3 shadow latency overhead {:.2}% exceeded budget {:.2}%",
        latency_overhead_pct,
        max_latency_overhead_pct
    );
    assert!(
        memory_overhead_bytes <= max_memory_overhead_bytes,
        "M3 shadow memory overhead {} bytes exceeded budget {} bytes",
        memory_overhead_bytes,
        max_memory_overhead_bytes
    );
}

#[test]
fn selfhost_m3_shadow_contract_examples_execute_via_vibe_test() {
    let out = run_vibe_test_for_selfhost();
    assert!(
        out.status.success(),
        "selfhost M3 shadow contract execution failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("examples=3 passed=3 failed=0"),
        "unexpected selfhost M3 contract summary:\n{}",
        out.stdout
    );
}

#[derive(Clone, Copy)]
enum SliceKind {
    ParserDiagnosticsNormalization,
    TypeDiagnosticsOrdering,
    MirFormatting,
}

struct SliceFixture {
    id: &'static str,
    kind: SliceKind,
    input_fixture: &'static str,
    selfhost_fixture: &'static str,
}

struct LoadedSlice {
    id: &'static str,
    kind: SliceKind,
    src: String,
    expected: String,
}

fn slice_fixtures() -> Vec<SliceFixture> {
    vec![
        SliceFixture {
            id: "m3_parser_diag_normalization",
            kind: SliceKind::ParserDiagnosticsNormalization,
            input_fixture: "compiler/tests/fixtures/parse_err/multiple_errors.vibe",
            selfhost_fixture: "selfhost/fixtures/m3_parser_diag_normalization.selfhost.out",
        },
        SliceFixture {
            id: "m3_type_diag_ordering",
            kind: SliceKind::TypeDiagnosticsOrdering,
            input_fixture: "compiler/tests/fixtures/type_err/map_set_value_mismatch.yb",
            selfhost_fixture: "selfhost/fixtures/m3_type_diag_ordering.selfhost.out",
        },
        SliceFixture {
            id: "m3_mir_formatting",
            kind: SliceKind::MirFormatting,
            input_fixture: "compiler/tests/fixtures/snapshots/pipeline_sample.vibe",
            selfhost_fixture: "selfhost/fixtures/m3_mir_formatting.selfhost.out",
        },
    ]
}

fn load_slices() -> Vec<LoadedSlice> {
    slice_fixtures()
        .into_iter()
        .map(|fixture| {
            let src_path = workspace_root().join(fixture.input_fixture);
            let expected_path = workspace_root().join(fixture.selfhost_fixture);
            let src = fs::read_to_string(&src_path)
                .unwrap_or_else(|e| panic!("read source fixture `{}`: {e}", src_path.display()));
            let expected = fs::read_to_string(&expected_path).unwrap_or_else(|e| {
                panic!(
                    "read selfhost fixture output `{}`: {e}",
                    expected_path.display()
                )
            });
            LoadedSlice {
                id: fixture.id,
                kind: fixture.kind,
                src,
                expected,
            }
        })
        .collect()
}

fn host_output_for_slice(kind: SliceKind, src: &str) -> String {
    match kind {
        SliceKind::ParserDiagnosticsNormalization => parse_source(src).diagnostics.to_golden(),
        SliceKind::TypeDiagnosticsOrdering => combined_diagnostics(src).to_golden(),
        SliceKind::MirFormatting => {
            let parsed = parse_source(src);
            let checked = check_and_lower(&parsed.ast);
            match lower_hir_to_mir(&checked.hir) {
                Ok(mir) => mir_debug_dump(&mir),
                Err(err) => format!("MIR lowering failed: {err}\n"),
            }
        }
    }
}

fn combined_diagnostics(src: &str) -> Diagnostics {
    let parsed = parse_source(src);
    let checked = check_and_lower(&parsed.ast);
    let mut all = Diagnostics::default();
    all.extend(parsed.diagnostics.into_sorted());
    all.extend(checked.diagnostics.into_sorted());
    all
}

fn write_slice_artifacts(root: &Path, id: &str, host: &str, expected: &str, diff: &str) {
    let dir = root.join(id);
    fs::create_dir_all(&dir).expect("create slice artifact directory");
    fs::write(dir.join("host.out"), host).expect("write host artifact");
    fs::write(dir.join("selfhost.out"), expected).expect("write selfhost artifact");
    fs::write(dir.join("diff.txt"), diff).expect("write diff artifact");
    let status = if diff == "no differences\n" {
        "pass\n"
    } else {
        "drift-detected\n"
    };
    fs::write(dir.join("status.txt"), status).expect("write status artifact");
}

fn line_diff(expected: &str, actual: &str) -> String {
    let expected_lines = expected.lines().collect::<Vec<_>>();
    let actual_lines = actual.lines().collect::<Vec<_>>();
    let max_len = expected_lines.len().max(actual_lines.len());

    let mut out = String::new();
    for idx in 0..max_len {
        let e = expected_lines.get(idx).copied();
        let a = actual_lines.get(idx).copied();
        if e != a {
            out.push_str(&format!(
                "L{} expected={:?} actual={:?}\n",
                idx + 1,
                e.unwrap_or("<missing>"),
                a.unwrap_or("<missing>")
            ));
        }
    }
    if out.is_empty() {
        "no differences\n".to_string()
    } else {
        out
    }
}

fn write_performance_metrics(
    root: &Path,
    loops: usize,
    baseline_ms: f64,
    shadow_ms: f64,
    latency_overhead_pct: f64,
    baseline_peak_bytes: usize,
    shadow_peak_bytes: usize,
    memory_overhead_bytes: usize,
) {
    let json = format!(
        concat!(
            "{{\n",
            "  \"loops\": {},\n",
            "  \"baseline_ms\": {:.3},\n",
            "  \"shadow_ms\": {:.3},\n",
            "  \"latency_overhead_pct\": {:.3},\n",
            "  \"baseline_peak_bytes\": {},\n",
            "  \"shadow_peak_bytes\": {},\n",
            "  \"memory_overhead_bytes\": {}\n",
            "}}\n"
        ),
        loops,
        baseline_ms,
        shadow_ms,
        latency_overhead_pct,
        baseline_peak_bytes,
        shadow_peak_bytes,
        memory_overhead_bytes
    );
    let markdown = format!(
        concat!(
            "# M3 Shadow Performance Metrics\n\n",
            "- loops: {}\n",
            "- baseline_ms: {:.3}\n",
            "- shadow_ms: {:.3}\n",
            "- latency_overhead_pct: {:.3}\n",
            "- baseline_peak_bytes: {}\n",
            "- shadow_peak_bytes: {}\n",
            "- memory_overhead_bytes: {}\n"
        ),
        loops,
        baseline_ms,
        shadow_ms,
        latency_overhead_pct,
        baseline_peak_bytes,
        shadow_peak_bytes,
        memory_overhead_bytes
    );

    fs::write(root.join("performance_metrics.json"), json).expect("write perf metrics json");
    fs::write(root.join("performance_metrics.md"), markdown).expect("write perf metrics markdown");
}

fn run_vibe_test_for_selfhost() -> CmdOutput {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "vibe_cli",
            "--",
            "test",
            "selfhost/frontend_shadow_slices.yb",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("run vibe test for M3 shadow slices");
    CmdOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn artifact_root() -> PathBuf {
    if let Ok(path) = env::var("VIBE_SELFHOST_M3_ARTIFACT_DIR") {
        return PathBuf::from(path);
    }
    env::temp_dir().join("vibelang-selfhost-m3")
}

fn parse_env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(default)
}

fn parse_env_f64(name: &str, default: f64) -> f64 {
    env::var(name)
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(default)
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
