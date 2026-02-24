#!/usr/bin/env python3
import argparse
import json
import math
import os
import platform
import re
import resource
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path


TIME_MARKER = "__MAXRSS_KB__="


def fail(message: str) -> None:
    raise SystemExit(f"cross-language benchmark collection failed: {message}")


def run_with_metrics(cmd: list[str], cwd: Path) -> dict[str, object]:
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    start = time.perf_counter()
    completed = subprocess.run(
        ["/usr/bin/time", "-f", f"{TIME_MARKER}%M", *cmd],
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    elapsed_ms = int((time.perf_counter() - start) * 1000)
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    cpu_s = (after.ru_utime + after.ru_stime) - (before.ru_utime + before.ru_stime)

    max_rss_kb = 0
    stderr_lines: list[str] = []
    for line in completed.stderr.splitlines():
        if line.startswith(TIME_MARKER):
            raw = line.replace(TIME_MARKER, "", 1).strip()
            try:
                max_rss_kb = int(raw)
            except ValueError:
                max_rss_kb = 0
            continue
        stderr_lines.append(line)

    return {
        "cmd": cmd,
        "exit_code": completed.returncode,
        "elapsed_ms": elapsed_ms,
        "cpu_ms": int(cpu_s * 1000),
        "max_rss_kb": max_rss_kb,
        "stdout": completed.stdout,
        "stderr": "\n".join(stderr_lines),
    }


def run_quick(cmd: list[str], cwd: Path) -> str:
    try:
        completed = subprocess.run(
            cmd,
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
    except FileNotFoundError:
        return "unavailable"
    if completed.returncode != 0:
        return "unavailable"
    lines = [line.strip() for line in completed.stdout.splitlines() if line.strip()]
    return lines[0] if lines else "unavailable"


def parse_result(stdout: str) -> tuple[int, int]:
    lines = [line.strip() for line in stdout.splitlines() if line.strip()]
    if len(lines) < 3:
        fail(
            "benchmark output is missing expected trailing RESULT/checksum/ops lines. "
            f"stdout=\n{stdout}"
        )
    tag, checksum_raw, ops_raw = lines[-3], lines[-2], lines[-1]
    if tag != "RESULT":
        fail(
            "benchmark output contract mismatch: expected trailing tag `RESULT`, "
            f"got `{tag}`. stdout=\n{stdout}"
        )
    try:
        checksum = int(checksum_raw)
        ops = int(ops_raw)
    except ValueError as exc:
        fail(f"failed to parse checksum/ops from benchmark output: {exc}")
    return checksum, ops


def percentile(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    ordered = sorted(values)
    idx = max(0, min(len(ordered) - 1, math.ceil(p * len(ordered)) - 1))
    return float(ordered[idx])


def median_abs_deviation(values: list[float]) -> float:
    if not values:
        return 0.0
    med = statistics.median(values)
    deviations = [abs(value - med) for value in values]
    return float(statistics.median(deviations))


def summarize_runs(runs: list[dict[str, object]]) -> dict[str, float]:
    wall = [float(run["elapsed_ms"]) for run in runs]
    cpu = [float(run["cpu_ms"]) for run in runs]
    rss = [float(run["max_rss_kb"]) for run in runs]
    mean_wall = float(statistics.fmean(wall))
    stddev_wall = float(statistics.pstdev(wall)) if len(wall) > 1 else 0.0
    rsd = (stddev_wall / mean_wall * 100.0) if mean_wall > 0.0 else 0.0
    return {
        "mean_wall_ms": mean_wall,
        "median_wall_ms": float(statistics.median(wall)),
        "p95_wall_ms": percentile(wall, 0.95),
        "p99_wall_ms": percentile(wall, 0.99),
        "stddev_wall_ms": stddev_wall,
        "mad_wall_ms": median_abs_deviation(wall),
        "rsd_wall_pct": rsd,
        "mean_cpu_ms": float(statistics.fmean(cpu)),
        "mean_max_rss_kb": float(statistics.fmean(rss)),
    }


def parse_vibe_build_output(stdout: str) -> Path:
    for line in stdout.splitlines():
        stripped = line.strip()
        if not stripped.startswith("built "):
            continue
        match = re.match(r"^built\s+([^\s]+)\s+\(object:", stripped)
        if match:
            return Path(match.group(1))
    fail(f"unable to parse VibeLang build output binary path.\nstdout:\n{stdout}")


def ensure_vibe_binary(repo_root: Path) -> Path:
    candidate_paths: list[Path] = []
    target_dir_env = os.environ.get("CARGO_TARGET_DIR")
    if target_dir_env:
        candidate_paths.append(Path(target_dir_env) / "release" / "vibe")
    candidate_paths.append(repo_root / "target" / "release" / "vibe")
    for candidate in candidate_paths:
        if candidate.exists():
            return candidate
    build = run_with_metrics(
        ["cargo", "build", "--release", "--locked", "-p", "vibe_cli", "--bin", "vibe"],
        repo_root,
    )
    if int(build["exit_code"]) != 0:
        fail(
            "failed to build release vibe binary\n"
            f"stdout:\n{build['stdout']}\n"
            f"stderr:\n{build['stderr']}"
        )
    for candidate in candidate_paths:
        if candidate.exists():
            return candidate
    fail(f"expected vibe binary missing in any candidate path: {candidate_paths}")


def detect_tsc_command() -> list[str]:
    if shutil.which("tsc"):
        return ["tsc"]
    if shutil.which("npx"):
        return ["npx", "--yes", "-p", "typescript", "tsc"]
    fail("TypeScript compiler not available; install `tsc` or provide `npx`")


def compile_case_binary(
    repo_root: Path,
    source: Path,
    language: str,
    build_dir: Path,
    vibe_bin: Path,
) -> tuple[dict[str, object], list[str]]:
    build_dir.mkdir(parents=True, exist_ok=True)
    run_cmd: list[str]
    artifact_path: Path
    if language == "vibelang":
        compile_cmd = [str(vibe_bin), "build", str(source), "--profile", "release"]
        compile_out = run_with_metrics(compile_cmd, repo_root)
        if int(compile_out["exit_code"]) != 0:
            fail(
                f"vibelang compile failed for `{source}`\n"
                f"stdout:\n{compile_out['stdout']}\n"
                f"stderr:\n{compile_out['stderr']}"
            )
        artifact_path = parse_vibe_build_output(str(compile_out["stdout"]))
        if not artifact_path.is_absolute():
            artifact_path = repo_root / artifact_path
        run_cmd = [str(artifact_path)]
    elif language == "c":
        artifact_path = build_dir / "bench_c"
        compile_cmd = [
            "gcc",
            "-O3",
            "-DNDEBUG",
            "-pthread",
            str(source),
            "-o",
            str(artifact_path),
        ]
        compile_out = run_with_metrics(compile_cmd, repo_root)
        if int(compile_out["exit_code"]) != 0:
            fail(
                f"c compile failed for `{source}`\n"
                f"stdout:\n{compile_out['stdout']}\n"
                f"stderr:\n{compile_out['stderr']}"
            )
        run_cmd = [str(artifact_path)]
    elif language == "rust":
        artifact_path = build_dir / "bench_rust"
        compile_cmd = [
            "rustc",
            str(source),
            "-C",
            "opt-level=3",
            "-C",
            "codegen-units=1",
            "-C",
            "debuginfo=0",
            "-o",
            str(artifact_path),
        ]
        compile_out = run_with_metrics(compile_cmd, repo_root)
        if int(compile_out["exit_code"]) != 0:
            fail(
                f"rust compile failed for `{source}`\n"
                f"stdout:\n{compile_out['stdout']}\n"
                f"stderr:\n{compile_out['stderr']}"
            )
        run_cmd = [str(artifact_path)]
    elif language == "go":
        artifact_path = build_dir / "bench_go"
        compile_cmd = [
            "go",
            "build",
            "-trimpath",
            "-ldflags",
            "-s -w",
            "-o",
            str(artifact_path),
            str(source),
        ]
        compile_out = run_with_metrics(compile_cmd, repo_root)
        if int(compile_out["exit_code"]) != 0:
            fail(
                f"go compile failed for `{source}`\n"
                f"stdout:\n{compile_out['stdout']}\n"
                f"stderr:\n{compile_out['stderr']}"
            )
        run_cmd = [str(artifact_path)]
    elif language == "python":
        artifact_path = source
        compile_cmd = ["python3", "-m", "py_compile", str(source)]
        compile_out = run_with_metrics(compile_cmd, repo_root)
        if int(compile_out["exit_code"]) != 0:
            fail(
                f"python compile check failed for `{source}`\n"
                f"stdout:\n{compile_out['stdout']}\n"
                f"stderr:\n{compile_out['stderr']}"
            )
        run_cmd = ["python3", str(source)]
    elif language == "typescript":
        tsc_cmd = detect_tsc_command()
        artifact_path = build_dir / "main.js"
        compile_cmd = [
            *tsc_cmd,
            "--target",
            "ES2020",
            "--module",
            "commonjs",
            "--skipLibCheck",
            "--outDir",
            str(build_dir),
            str(source),
        ]
        compile_out = run_with_metrics(compile_cmd, repo_root)
        if int(compile_out["exit_code"]) != 0:
            fail(
                f"typescript compile failed for `{source}`\n"
                f"stdout:\n{compile_out['stdout']}\n"
                f"stderr:\n{compile_out['stderr']}"
            )
        run_cmd = ["node", str(artifact_path)]
    else:
        fail(f"unsupported language `{language}`")

    if not artifact_path.exists():
        fail(f"compiled artifact does not exist: {artifact_path}")

    compile_record = {
        "command": compile_out["cmd"],
        "exit_code": int(compile_out["exit_code"]),
        "elapsed_ms": int(compile_out["elapsed_ms"]),
        "cpu_ms": int(compile_out["cpu_ms"]),
        "max_rss_kb": int(compile_out["max_rss_kb"]),
        "binary_path": str(artifact_path),
        "binary_size_bytes": int(artifact_path.stat().st_size),
    }
    return compile_record, run_cmd


def execute_case(
    repo_root: Path,
    run_cmd: list[str],
    run_label: str,
    warmup_runs: int,
    measured_runs: int,
) -> tuple[dict[str, object], int, int]:
    warmups: list[dict[str, object]] = []
    runs: list[dict[str, object]] = []
    checksum_ref: int | None = None
    ops_ref: int | None = None

    for _ in range(warmup_runs):
        out = run_with_metrics(run_cmd, repo_root)
        if int(out["exit_code"]) != 0:
            fail(
                f"warmup run failed for `{run_label}`\n"
                f"cmd={run_cmd}\n"
                f"stdout:\n{out['stdout']}\n"
                f"stderr:\n{out['stderr']}"
            )
        checksum, ops = parse_result(str(out["stdout"]))
        warmups.append(
            {
                "elapsed_ms": int(out["elapsed_ms"]),
                "cpu_ms": int(out["cpu_ms"]),
                "max_rss_kb": int(out["max_rss_kb"]),
                "checksum": checksum,
                "ops": ops,
            }
        )
        if checksum_ref is None:
            checksum_ref = checksum
            ops_ref = ops
        elif checksum_ref != checksum or ops_ref != ops:
            fail(f"warmup run produced inconsistent output for `{run_label}`")

    for _ in range(measured_runs):
        out = run_with_metrics(run_cmd, repo_root)
        if int(out["exit_code"]) != 0:
            fail(
                f"measured run failed for `{run_label}`\n"
                f"cmd={run_cmd}\n"
                f"stdout:\n{out['stdout']}\n"
                f"stderr:\n{out['stderr']}"
            )
        checksum, ops = parse_result(str(out["stdout"]))
        if checksum_ref is None:
            checksum_ref = checksum
            ops_ref = ops
        elif checksum_ref != checksum or ops_ref != ops:
            fail(f"measured run produced inconsistent output for `{run_label}`")
        runs.append(
            {
                "elapsed_ms": int(out["elapsed_ms"]),
                "cpu_ms": int(out["cpu_ms"]),
                "max_rss_kb": int(out["max_rss_kb"]),
                "checksum": checksum,
                "ops": ops,
            }
        )

    if checksum_ref is None or ops_ref is None:
        fail(f"no run output captured for `{run_label}`")

    runtime_record = {
        "warmups": warmups,
        "runs": runs,
        "summary": summarize_runs(runs),
    }
    return runtime_record, checksum_ref, ops_ref


def geomean(values: list[float]) -> float:
    if not values:
        return 0.0
    if any(value <= 0 for value in values):
        return 0.0
    return math.exp(sum(math.log(value) for value in values) / len(values))


def summarize_cross_language(cases: list[dict[str, object]], languages: list[str]) -> dict[str, object]:
    rows: list[dict[str, object]] = []
    baselines = [language for language in languages if language != "vibelang"]
    vibe_vs_baseline: dict[str, list[float]] = {language: [] for language in baselines}

    for case in cases:
        case_id = str(case["id"])
        language_data = case["languages"]
        mean_by_language: dict[str, float] = {}
        for language in languages:
            summary = language_data[language]["runtime"]["summary"]
            mean_by_language[language] = float(summary["mean_wall_ms"])

        row = {
            "case_id": case_id,
            "mean_wall_ms": mean_by_language,
            "vibelang_ratio_vs": {},
        }
        vibe_mean = mean_by_language.get("vibelang")
        if vibe_mean and vibe_mean > 0:
            for baseline in baselines:
                baseline_mean = mean_by_language.get(baseline)
                if baseline_mean and baseline_mean > 0:
                    ratio = vibe_mean / baseline_mean
                    vibe_vs_baseline[baseline].append(ratio)
                    row["vibelang_ratio_vs"][baseline] = ratio
                    row[f"vibelang_ratio_vs_{baseline}"] = ratio
                else:
                    row[f"vibelang_ratio_vs_{baseline}"] = None
        rows.append(row)

    geomean_map = {language: geomean(values) for language, values in vibe_vs_baseline.items()}
    out = {
        "per_case": rows,
        "geomean_vibelang_ratio_vs": geomean_map,
    }
    for language, ratio in geomean_map.items():
        out[f"geomean_vibelang_ratio_vs_{language}"] = ratio
    return out


def build_profile_drift(current_report: dict, comparison_report: dict) -> dict[str, object]:
    current_profile = str(current_report.get("profile", "unknown"))
    comparison_profile = str(comparison_report.get("profile", "unknown"))
    languages = list(current_report.get("languages", []))
    comparison_languages = list(comparison_report.get("languages", []))
    if languages != comparison_languages:
        fail(
            f"profile drift comparison language mismatch: "
            f"{current_profile}={languages} {comparison_profile}={comparison_languages}"
        )
    baselines = [language for language in languages if language != "vibelang"]

    current_cases = {str(case["id"]): case for case in current_report.get("cases", [])}
    comparison_cases = {str(case["id"]): case for case in comparison_report.get("cases", [])}
    current_case_ids = list(current_cases.keys())
    comparison_case_ids = list(comparison_cases.keys())
    shared_case_ids = [case_id for case_id in current_case_ids if case_id in comparison_cases]
    missing_in_comparison = [
        case_id for case_id in current_case_ids if case_id not in comparison_cases
    ]
    missing_in_current = [
        case_id for case_id in comparison_case_ids if case_id not in current_cases
    ]
    if not shared_case_ids:
        fail(
            f"profile drift comparison has no shared cases: "
            f"{current_profile} vs {comparison_profile}"
        )

    geomean_drift: dict[str, dict[str, float]] = {}
    current_summary = current_report.get("summary", {})
    comparison_summary = comparison_report.get("summary", {})
    geomean_map_current = current_summary.get("geomean_vibelang_ratio_vs", {})
    geomean_map_comparison = comparison_summary.get("geomean_vibelang_ratio_vs", {})
    for baseline in baselines:
        before = float(geomean_map_comparison.get(baseline, 0.0))
        after = float(geomean_map_current.get(baseline, 0.0))
        delta_abs = after - before
        delta_pct = (delta_abs / before * 100.0) if before != 0.0 else 0.0
        geomean_drift[baseline] = {
            "before": before,
            "after": after,
            "delta_abs": delta_abs,
            "delta_pct": delta_pct,
        }

    per_case_drift: list[dict[str, object]] = []
    for case_id in shared_case_ids:
        current_case = current_cases[case_id]
        comparison_case = comparison_cases[case_id]
        current_vibe = float(
            current_case["languages"]["vibelang"]["runtime"]["summary"]["mean_wall_ms"]
        )
        comparison_vibe = float(
            comparison_case["languages"]["vibelang"]["runtime"]["summary"]["mean_wall_ms"]
        )
        delta_abs = current_vibe - comparison_vibe
        delta_pct = (delta_abs / comparison_vibe * 100.0) if comparison_vibe != 0.0 else 0.0
        per_case_drift.append(
            {
                "case_id": case_id,
                "before_vibelang_mean_wall_ms": comparison_vibe,
                "after_vibelang_mean_wall_ms": current_vibe,
                "delta_abs_ms": delta_abs,
                "delta_pct": delta_pct,
            }
        )

    return {
        "format": "vibe-cross-language-profile-drift-v1",
        "current_profile": current_profile,
        "comparison_profile": comparison_profile,
        "generated_at_utc": current_report.get("generated_at_utc", "unknown"),
        "shared_case_count": len(shared_case_ids),
        "missing_in_comparison": missing_in_comparison,
        "missing_in_current": missing_in_current,
        "geomean_drift": geomean_drift,
        "per_case_vibelang_drift": per_case_drift,
    }


def build_profile_drift_markdown(drift: dict[str, object]) -> str:
    current_profile = str(drift["current_profile"])
    comparison_profile = str(drift["comparison_profile"])
    lines: list[str] = []
    lines.append("# Cross-Language Profile Drift")
    lines.append("")
    lines.append(f"- current_profile: `{current_profile}`")
    lines.append(f"- comparison_profile: `{comparison_profile}`")
    lines.append(f"- generated_at_utc: `{drift.get('generated_at_utc', 'unknown')}`")
    lines.append(f"- shared_case_count: `{drift.get('shared_case_count', 0)}`")
    missing_in_comparison = drift.get("missing_in_comparison", [])
    missing_in_current = drift.get("missing_in_current", [])
    if isinstance(missing_in_comparison, list) and missing_in_comparison:
        lines.append(f"- missing_in_comparison: `{missing_in_comparison}`")
    if isinstance(missing_in_current, list) and missing_in_current:
        lines.append(f"- missing_in_current: `{missing_in_current}`")
    lines.append("")
    lines.append("## Geomean Drift")
    lines.append("")
    lines.append("| baseline | before | after | delta_abs | delta_pct |")
    lines.append("| --- | ---: | ---: | ---: | ---: |")
    geomean_drift = drift.get("geomean_drift", {})
    if isinstance(geomean_drift, dict):
        for baseline, row in geomean_drift.items():
            if not isinstance(row, dict):
                continue
            lines.append(
                f"| {baseline} | {float(row.get('before', 0.0)):.3f} | "
                f"{float(row.get('after', 0.0)):.3f} | "
                f"{float(row.get('delta_abs', 0.0)):.3f} | "
                f"{float(row.get('delta_pct', 0.0)):.2f}% |"
            )
    lines.append("")
    lines.append("## Per-Case Vibe Drift")
    lines.append("")
    lines.append("| case | before_ms | after_ms | delta_abs_ms | delta_pct |")
    lines.append("| --- | ---: | ---: | ---: | ---: |")
    per_case = drift.get("per_case_vibelang_drift", [])
    if isinstance(per_case, list):
        for row in per_case:
            if not isinstance(row, dict):
                continue
            lines.append(
                f"| {row.get('case_id', 'unknown')} | "
                f"{float(row.get('before_vibelang_mean_wall_ms', 0.0)):.3f} | "
                f"{float(row.get('after_vibelang_mean_wall_ms', 0.0)):.3f} | "
                f"{float(row.get('delta_abs_ms', 0.0)):.3f} | "
                f"{float(row.get('delta_pct', 0.0)):.2f}% |"
            )
    lines.append("")
    lines.append(
        "Interpretation: for runtime deltas in this table, negative delta is improvement (faster)."
    )
    lines.append("")
    return "\n".join(lines)


def detect_cpu_model() -> str:
    cpuinfo = Path("/proc/cpuinfo")
    if not cpuinfo.exists():
        return "unavailable"
    for line in cpuinfo.read_text(errors="ignore").splitlines():
        if "model name" in line:
            _, _, value = line.partition(":")
            return value.strip()
    return "unavailable"


def detect_cpu_governor() -> str:
    governor_path = Path("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
    if not governor_path.exists():
        return "unavailable"
    try:
        return governor_path.read_text().strip() or "unavailable"
    except OSError:
        return "unavailable"


def detect_physical_cores() -> int:
    cpuinfo = Path("/proc/cpuinfo")
    if not cpuinfo.exists():
        return 0
    physical_core_ids: set[tuple[str, str]] = set()
    physical_id = "0"
    core_id = "0"
    for line in cpuinfo.read_text(errors="ignore").splitlines():
        if not line.strip():
            if physical_id or core_id:
                physical_core_ids.add((physical_id, core_id))
            physical_id = "0"
            core_id = "0"
            continue
        key, _, value = line.partition(":")
        key = key.strip()
        value = value.strip()
        if key == "physical id":
            physical_id = value
        elif key == "core id":
            core_id = value
    if physical_core_ids:
        return len(physical_core_ids)
    logical = os.cpu_count()
    return logical if isinstance(logical, int) else 0


def detect_meminfo_kb(field: str) -> int:
    meminfo = Path("/proc/meminfo")
    if not meminfo.exists():
        return 0
    pattern = re.compile(rf"^{re.escape(field)}:\s+(\d+)\s+kB$")
    for line in meminfo.read_text(errors="ignore").splitlines():
        match = pattern.match(line.strip())
        if match:
            return int(match.group(1))
    return 0


def detect_numa_nodes() -> list[str]:
    node_root = Path("/sys/devices/system/node")
    if not node_root.exists():
        return []
    nodes = [path.name for path in node_root.iterdir() if path.name.startswith("node")]
    return sorted(nodes)


def detect_git_revision(repo_root: Path) -> str:
    try:
        completed = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=repo_root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
    except FileNotFoundError:
        return "unavailable"
    if completed.returncode != 0:
        return "unavailable"
    return completed.stdout.strip() or "unavailable"


def detect_is_wsl() -> bool:
    release_path = Path("/proc/sys/kernel/osrelease")
    if not release_path.exists():
        return False
    text = release_path.read_text(errors="ignore").lower()
    return "microsoft" in text


def collect_environment(
    repo_root: Path,
    vibe_bin: Path,
    launch_argv: list[str],
    profile: str,
) -> dict[str, object]:
    memory_total_kb = detect_meminfo_kb("MemTotal")
    swap_total_kb = detect_meminfo_kb("SwapTotal")
    git_revision = detect_git_revision(repo_root)
    logical_cpus = os.cpu_count()
    return {
        "hostname": platform.node(),
        "platform": platform.platform(),
        "kernel_release": platform.release(),
        "architecture": platform.machine(),
        "cpu_model": detect_cpu_model(),
        "cpu_governor": detect_cpu_governor(),
        "logical_cpus": logical_cpus if isinstance(logical_cpus, int) else 0,
        "physical_cores": detect_physical_cores(),
        "memory_total_kb": memory_total_kb,
        "swap_total_kb": swap_total_kb,
        "numa_nodes": detect_numa_nodes(),
        "is_wsl": detect_is_wsl(),
        "benchmark_launch": {
            "argv": launch_argv,
            "profile": profile,
        },
        "source_revisions": {
            "repo_git_revision": git_revision,
            "toolchain_revision": git_revision,
        },
        "tool_versions": {
            "gcc": run_quick(["gcc", "--version"], repo_root),
            "rustc": run_quick(["rustc", "--version"], repo_root),
            "go": run_quick(["go", "version"], repo_root),
            "python3": run_quick(["python3", "--version"], repo_root),
            "node": run_quick(["node", "--version"], repo_root),
            "tsc": run_quick(["tsc", "--version"], repo_root),
            "vibe": run_quick([str(vibe_bin), "--version"], repo_root),
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--manifest",
        default="runtime/benchmarks/cross_lang/manifest.json",
        help="Manifest path relative to repo root.",
    )
    parser.add_argument(
        "--profile",
        choices=["quick", "full"],
        default="quick",
        help="Benchmark run profile.",
    )
    parser.add_argument(
        "--output-root",
        default="reports/benchmarks/cross_lang",
        help="Output root relative to repo root.",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[2]
    manifest_path = repo_root / args.manifest
    if not manifest_path.exists():
        fail(f"manifest not found: {manifest_path}")
    manifest = json.loads(manifest_path.read_text())

    profiles = manifest.get("profiles", {})
    if args.profile not in profiles:
        fail(f"profile `{args.profile}` not found in manifest")
    profile_cfg = profiles[args.profile]
    warmup_runs = int(profile_cfg.get("warmup_runs", 1))
    measured_runs = int(profile_cfg.get("measured_runs", 5))
    if warmup_runs < 0 or measured_runs <= 0:
        fail("invalid profile run counts in manifest")

    languages = manifest.get("languages", [])
    if not isinstance(languages, list) or not languages:
        fail("manifest languages must be a non-empty list")
    if any(not isinstance(language, str) for language in languages):
        fail("manifest languages must be a list of strings")
    if "vibelang" not in languages:
        fail("manifest languages must include `vibelang`")
    supported_languages = {"vibelang", "c", "rust", "go", "python", "typescript"}
    unknown_languages = [language for language in languages if language not in supported_languages]
    if unknown_languages:
        fail(f"manifest contains unsupported languages: {unknown_languages}")

    output_root = repo_root / args.output_root
    profile_dir = output_root / args.profile
    latest_dir = output_root / "latest"
    profile_dir.mkdir(parents=True, exist_ok=True)
    latest_dir.mkdir(parents=True, exist_ok=True)
    build_root = profile_dir / "build"
    build_root.mkdir(parents=True, exist_ok=True)

    vibe_bin = ensure_vibe_binary(repo_root)
    environment = collect_environment(
        repo_root,
        vibe_bin,
        launch_argv=sys.argv,
        profile=args.profile,
    )
    generated_epoch_s = int(time.time())
    generated_iso = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(generated_epoch_s))

    cases_out: list[dict[str, object]] = []
    for case in manifest.get("cases", []):
        case_id = case["id"]
        case_sources = case["sources"]
        expected_ops = int(case["expected_ops"])

        case_result: dict[str, object] = {
            "id": case_id,
            "category": case["category"],
            "description": case["description"],
            "expected_ops": expected_ops,
            "languages": {},
        }
        checksum_by_language: dict[str, int] = {}
        ops_by_language: dict[str, int] = {}

        for language in languages:
            source_rel = case_sources[language]
            source_path = repo_root / source_rel
            if not source_path.exists():
                fail(f"source for `{case_id}` `{language}` not found: {source_path}")
            language_build_dir = build_root / case_id / language
            compile_record, run_cmd = compile_case_binary(
                repo_root,
                source_path,
                language,
                language_build_dir,
                vibe_bin,
            )
            runtime_record, checksum, ops = execute_case(
                repo_root,
                run_cmd=run_cmd,
                run_label=f"{case_id}:{language}",
                warmup_runs=warmup_runs,
                measured_runs=measured_runs,
            )
            if ops != expected_ops:
                fail(
                    f"ops mismatch for case `{case_id}` language `{language}`: "
                    f"expected {expected_ops}, got {ops}"
                )
            checksum_by_language[language] = checksum
            ops_by_language[language] = ops
            case_result["languages"][language] = {
                "source": source_rel,
                "compile": compile_record,
                "runtime": runtime_record,
                "result": {"checksum": checksum, "ops": ops},
            }

        checksums = list(checksum_by_language.values())
        ops_values = list(ops_by_language.values())
        case_result["parity"] = {
            "checksum_match": all(value == checksums[0] for value in checksums),
            "ops_match": all(value == ops_values[0] for value in ops_values),
            "checksum_by_language": checksum_by_language,
            "ops_by_language": ops_by_language,
        }
        if not case_result["parity"]["checksum_match"]:
            fail(f"cross-language checksum mismatch for case `{case_id}`")
        if not case_result["parity"]["ops_match"]:
            fail(f"cross-language ops mismatch for case `{case_id}`")
        cases_out.append(case_result)

    summary = summarize_cross_language(cases_out, languages)
    report = {
        "format": "vibe-cross-language-benchmarks-v1",
        "suite_name": manifest.get("suite_name", "cross_lang_starter8"),
        "profile": args.profile,
        "languages": languages,
        "generated_at_epoch_s": generated_epoch_s,
        "generated_at_utc": generated_iso,
        "run_config": {"warmup_runs": warmup_runs, "measured_runs": measured_runs},
        "manifest_path": str(manifest_path.relative_to(repo_root)),
        "environment": environment,
        "cases": cases_out,
        "summary": summary,
    }

    profile_results = profile_dir / "results.json"
    latest_results = latest_dir / "results.json"
    profile_results.write_text(json.dumps(report, indent=2) + "\n")
    latest_results.write_text(json.dumps(report, indent=2) + "\n")
    print(f"wrote {profile_results}")
    print(f"wrote {latest_results}")

    other_profile = "full" if args.profile == "quick" else "quick"
    other_results = output_root / other_profile / "results.json"
    if other_results.exists():
        other_report = json.loads(other_results.read_text())
        drift = build_profile_drift(report, other_report)
        drift_json_name = f"drift_vs_{other_profile}.json"
        drift_md_name = f"drift_vs_{other_profile}.md"
        profile_drift_json = profile_dir / drift_json_name
        profile_drift_md = profile_dir / drift_md_name
        latest_drift_json = latest_dir / "trend.json"
        latest_drift_md = latest_dir / "trend.md"
        profile_drift_json.write_text(json.dumps(drift, indent=2) + "\n")
        profile_drift_md.write_text(build_profile_drift_markdown(drift) + "\n")
        latest_drift_json.write_text(json.dumps(drift, indent=2) + "\n")
        latest_drift_md.write_text(build_profile_drift_markdown(drift) + "\n")
        print(f"wrote {profile_drift_json}")
        print(f"wrote {profile_drift_md}")
        print(f"wrote {latest_drift_json}")
        print(f"wrote {latest_drift_md}")


if __name__ == "__main__":
    main()

