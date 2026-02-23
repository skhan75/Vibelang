#!/usr/bin/env python3
import json
import os
import re
import subprocess
import tempfile
import time
from pathlib import Path


def run(cmd, cwd):
    start = time.time()
    completed = subprocess.run(
        cmd,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    elapsed_ms = int((time.time() - start) * 1000)
    return {
        "cmd": cmd,
        "exit": completed.returncode,
        "elapsed_ms": elapsed_ms,
        "stdout": completed.stdout,
        "stderr": completed.stderr,
    }


def parse_index_stats(raw_stdout: str) -> dict:
    line = ""
    for candidate in raw_stdout.splitlines():
        if candidate.startswith("index stats:"):
            line = candidate
            break
    if not line:
        return {}
    out = {}
    for token in line.replace("index stats:", "").strip().split():
        if "=" not in token:
            continue
        key, value = token.split("=", 1)
        out[key.strip()] = value.strip()
    return out


def build_vibe_binary(repo_root: Path) -> Path:
    build = run(["cargo", "build", "--release", "--locked", "-p", "vibe_cli"], repo_root)
    if build["exit"] != 0:
        raise RuntimeError(
            "failed to build release vibe binary:\n"
            f"stdout:\n{build['stdout']}\n"
            f"stderr:\n{build['stderr']}"
        )
    vibe_bin = repo_root / "target" / "release" / "vibe"
    if not vibe_bin.exists():
        raise RuntimeError(f"missing built vibe binary: {vibe_bin}")
    return vibe_bin


def main():
    repo_root = Path(__file__).resolve().parents[2]
    reports_dir = repo_root / "reports" / "phase6" / "metrics"
    reports_dir.mkdir(parents=True, exist_ok=True)
    vibe_bin = build_vibe_binary(repo_root)

    hello_fixture = repo_root / "compiler" / "tests" / "fixtures" / "build" / "hello_world.vibe"
    contract_fixture = (
        repo_root / "compiler" / "tests" / "fixtures" / "contract_ok" / "topk_contracts.vibe"
    )

    with tempfile.TemporaryDirectory(prefix="vibe_phase6_metrics_") as temp_dir:
        temp_root = Path(temp_dir)
        benchmark_root = temp_root / "benchmark"
        benchmark_root.mkdir(parents=True, exist_ok=True)
        hello_local = benchmark_root / "hello_world.vibe"
        contract_local = benchmark_root / "topk_contracts.vibe"
        hello_local.write_text(hello_fixture.read_text())
        contract_local.write_text(contract_fixture.read_text())

        compile_clean = run([str(vibe_bin), "check", str(hello_local)], repo_root)
        compile_noop = run([str(vibe_bin), "check", str(hello_local)], repo_root)
        compile_incremental = run(
            [str(vibe_bin), "index", str(benchmark_root), "--stats"], repo_root
        )
        index_stats = parse_index_stats(compile_incremental["stdout"])
        runtime_smoke = run([str(vibe_bin), "run", str(hello_local)], repo_root)
        contract_run = run([str(vibe_bin), "test", str(contract_local)], repo_root)

        parity_results = []
        for ext in ("vibe", "yb"):
            fixture_dir = temp_root / f"parity_{ext}"
            fixture_dir.mkdir(parents=True, exist_ok=True)
            fixture = fixture_dir / f"hello.{ext}"
            fixture.write_text(hello_fixture.read_text())
            check_out = run([str(vibe_bin), "check", str(fixture)], repo_root)
            build_out = run([str(vibe_bin), "build", str(fixture)], repo_root)
            run_out = run([str(vibe_bin), "run", str(fixture)], repo_root)
            test_out = run([str(vibe_bin), "test", str(fixture)], repo_root)
            lint_out = run([str(vibe_bin), "lint", str(fixture), "--intent"], repo_root)
            index_out = run([str(vibe_bin), "index", str(fixture_dir)], repo_root)
            parity_results.append(
                check_out["exit"] == 0
                and build_out["exit"] == 0
                and run_out["exit"] == 0
                and test_out["exit"] == 0
                and lint_out["exit"] == 0
                and index_out["exit"] == 0
            )

    intent_lint = run(
        [str(vibe_bin), "lint", str(repo_root / "compiler" / "tests" / "fixtures"), "--intent"],
        repo_root,
    )
    cross_target_gate = run(
        [
            "cargo",
            "test",
            "-q",
            "-p",
            "vibe_runtime",
            "ensure_supported_target_accepts_phase6_targets",
        ],
        repo_root,
    )
    memory_gc_lane = run(
        [
            "cargo",
            "test",
            "-q",
            "-p",
            "vibe_cli",
            "--test",
            "phase7_v1_tightening",
            "phase7_gc_observable_smoke_is_default_lane",
        ],
        repo_root,
    )
    memory_valgrind_lane = run(
        [
            "cargo",
            "test",
            "-q",
            "-p",
            "vibe_cli",
            "--test",
            "phase7_v1_tightening",
            "phase7_memory_valgrind_leak_check_default_lane",
        ],
        repo_root,
    )

    yb_count = 0
    vibe_count = 0
    for root, dirs, files in os.walk(repo_root):
        dirs[:] = [d for d in dirs if d not in {".git", "target", ".yb", ".vibe"}]
        for file in files:
            if file.endswith(".yb"):
                yb_count += 1
            elif file.endswith(".vibe"):
                vibe_count += 1
    total_sources = yb_count + vibe_count
    yb_ratio = (yb_count / total_sources) if total_sources else 1.0

    unsupported_diag_hits = 0
    for path in repo_root.rglob("*.md"):
        text = path.read_text(errors="ignore")
        unsupported_diag_hits += len(re.findall(r"E340\d", text))

    metrics = {
        "generated_at_epoch_s": int(time.time()),
        "compile_clean_ms": compile_clean["elapsed_ms"],
        "compile_noop_ms": compile_noop["elapsed_ms"],
        "compile_incremental_ms": int(index_stats.get("incremental_ms", 0) or 0),
        "index_cache_hits": int(index_stats.get("cache_hits", 0) or 0),
        "index_cache_misses": int(index_stats.get("cache_misses", 0) or 0),
        "index_cache_hit_rate": float(index_stats.get("cache_hit_rate", 0.0) or 0.0),
        "index_memory_bytes": int(index_stats.get("memory_bytes", 0) or 0),
        "index_memory_ratio": float(index_stats.get("memory_ratio", 0.0) or 0.0),
        "runtime_smoke_pass": runtime_smoke["exit"] == 0,
        "runtime_smoke_ms": runtime_smoke["elapsed_ms"],
        "memory_gc_lane_pass": memory_gc_lane["exit"] == 0,
        "memory_valgrind_lane_pass": memory_valgrind_lane["exit"] == 0,
        "memory_valgrind_lane_skipped": "skipping valgrind leak smoke"
        in (memory_valgrind_lane["stdout"] + memory_valgrind_lane["stderr"]),
        "contract_example_pass": contract_run["exit"] == 0,
        "contract_summary": contract_run["stdout"].strip(),
        "intent_lint_pass": intent_lint["exit"] == 0,
        "intent_lint_findings_lines": len(
            [line for line in intent_lint["stdout"].splitlines() if line.startswith("[")]
        ),
        "developer_time_to_first_binary_ms": runtime_smoke["elapsed_ms"],
        "spec_conformance_docs_present": all(
            [
                (repo_root / "docs" / "spec" / "syntax.md").exists(),
                (repo_root / "docs" / "spec" / "semantics.md").exists(),
                (repo_root / "docs" / "spec" / "contracts.md").exists(),
            ]
        ),
        "unsupported_feature_signal_count": unsupported_diag_hits,
        "cross_target_pass_rate": 1.0 if cross_target_gate["exit"] == 0 else 0.0,
        "source_extension_yb_count": yb_count,
        "source_extension_vibe_count": vibe_count,
        "source_extension_yb_ratio": yb_ratio,
        "dual_extension_parity_pass_rate": sum(1 for ok in parity_results if ok)
        / len(parity_results),
        "benchmark_method": "direct_vibe_binary",
        "vibe_binary": "target/release/vibe",
    }

    (reports_dir / "phase6_metrics.json").write_text(json.dumps(metrics, indent=2) + "\n")
    summary = (
        "# Phase 6 Metrics Snapshot\n\n"
        f"- benchmark_method: {metrics['benchmark_method']}\n"
        f"- compile_clean_ms: {metrics['compile_clean_ms']}\n"
        f"- compile_noop_ms: {metrics['compile_noop_ms']}\n"
        f"- compile_incremental_ms: {metrics['compile_incremental_ms']}\n"
        f"- index_cache_hit_rate: {metrics['index_cache_hit_rate']:.4f}\n"
        f"- runtime_smoke_pass: {metrics['runtime_smoke_pass']}\n"
        f"- memory_gc_lane_pass: {metrics['memory_gc_lane_pass']}\n"
        f"- memory_valgrind_lane_pass: {metrics['memory_valgrind_lane_pass']}\n"
        f"- dual_extension_parity_pass_rate: {metrics['dual_extension_parity_pass_rate']:.2f}\n"
        f"- source_extension_yb_ratio: {metrics['source_extension_yb_ratio']:.2f}\n"
    )
    (reports_dir / "phase6_metrics.md").write_text(summary)
    print(f"wrote {reports_dir / 'phase6_metrics.json'}")


if __name__ == "__main__":
    main()
