#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def fail(msg):
    print(f"metrics validation failed: {msg}")
    sys.exit(1)


def main():
    repo_root = Path(__file__).resolve().parents[2]
    metrics_path = repo_root / "reports" / "phase6" / "metrics" / "phase6_metrics.json"
    budgets_path = repo_root / "reports" / "v1" / "quality_budgets.json"
    if not metrics_path.exists():
        fail(f"missing metrics file: {metrics_path}")
    if not budgets_path.exists():
        fail(f"missing quality budgets file: {budgets_path}")
    metrics = json.loads(metrics_path.read_text())
    budgets = json.loads(budgets_path.read_text())
    compile_budgets = budgets.get("compile_benchmarks", {})
    if not isinstance(compile_budgets, dict):
        fail("compile_benchmarks section missing from quality budgets")

    compile_clean_ms = metrics.get("compile_clean_ms", 0)
    compile_noop_ms = metrics.get("compile_noop_ms", 0)
    compile_incremental_ms = metrics.get("compile_incremental_ms", 0)
    index_cache_hit_rate = metrics.get("index_cache_hit_rate", 0.0)

    if compile_clean_ms <= 0:
        fail("compile_clean_ms must be > 0")
    if compile_noop_ms <= 0:
        fail("compile_noop_ms must be > 0")
    if compile_incremental_ms < 0:
        fail("compile_incremental_ms must be >= 0")

    max_clean_ms = compile_budgets.get("max_clean_ms")
    max_noop_ms = compile_budgets.get("max_noop_ms")
    max_incremental_ms = compile_budgets.get("max_incremental_ms")
    min_index_cache_hit_rate = compile_budgets.get("min_index_cache_hit_rate")
    if not isinstance(max_clean_ms, int) or max_clean_ms <= 0:
        fail("compile_benchmarks.max_clean_ms must be a positive integer")
    if not isinstance(max_noop_ms, int) or max_noop_ms <= 0:
        fail("compile_benchmarks.max_noop_ms must be a positive integer")
    if not isinstance(max_incremental_ms, int) or max_incremental_ms <= 0:
        fail("compile_benchmarks.max_incremental_ms must be a positive integer")
    if not isinstance(min_index_cache_hit_rate, (int, float)) or min_index_cache_hit_rate < 0:
        fail("compile_benchmarks.min_index_cache_hit_rate must be a non-negative number")

    if compile_clean_ms > max_clean_ms:
        fail(f"compile_clean_ms={compile_clean_ms} exceeds max_clean_ms={max_clean_ms}")
    if compile_noop_ms > max_noop_ms:
        fail(f"compile_noop_ms={compile_noop_ms} exceeds max_noop_ms={max_noop_ms}")
    if compile_incremental_ms > max_incremental_ms:
        fail(
            f"compile_incremental_ms={compile_incremental_ms} exceeds max_incremental_ms={max_incremental_ms}"
        )
    if index_cache_hit_rate < float(min_index_cache_hit_rate):
        fail(
            "index_cache_hit_rate="
            f"{index_cache_hit_rate:.4f} below min_index_cache_hit_rate={float(min_index_cache_hit_rate):.4f}"
        )

    if metrics.get("benchmark_method") != "direct_vibe_binary":
        fail("benchmark_method must be `direct_vibe_binary`")
    if metrics.get("dual_extension_parity_pass_rate", 0.0) < 1.0:
        fail("dual_extension_parity_pass_rate must be 1.0")
    if metrics.get("cross_target_pass_rate", 0.0) < 1.0:
        fail("cross_target_pass_rate must be 1.0")
    if not metrics.get("spec_conformance_docs_present", False):
        fail("spec_conformance_docs_present must be true")
    print("metrics validation passed")


if __name__ == "__main__":
    main()
