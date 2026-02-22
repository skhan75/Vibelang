#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def fail(msg: str) -> None:
    print(f"v1 quality budget validation failed: {msg}")
    sys.exit(1)


def read_json(path: Path) -> dict:
    if not path.exists():
        fail(f"missing file: {path}")
    try:
        return json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        fail(f"invalid json in {path}: {exc}")


def require_positive_int(obj: dict, key: str, context: str) -> int:
    val = obj.get(key)
    if not isinstance(val, int) or val <= 0:
        fail(f"{context}.{key} must be a positive integer")
    return val


def require_non_negative_number(obj: dict, key: str, context: str) -> float:
    val = obj.get(key)
    if not isinstance(val, (int, float)) or val < 0:
        fail(f"{context}.{key} must be a non-negative number")
    return float(val)


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    budgets_path = repo_root / "reports" / "v1" / "quality_budgets.json"
    metrics_path = repo_root / "reports" / "phase6" / "metrics" / "phase6_metrics.json"

    budgets = read_json(budgets_path)
    metrics = read_json(metrics_path)

    compile_benchmarks = budgets.get("compile_benchmarks")
    if not isinstance(compile_benchmarks, dict):
        fail("compile_benchmarks section is required")
    max_clean_ms = require_positive_int(compile_benchmarks, "max_clean_ms", "compile_benchmarks")
    max_noop_ms = require_positive_int(compile_benchmarks, "max_noop_ms", "compile_benchmarks")
    max_incremental_ms = require_positive_int(
        compile_benchmarks, "max_incremental_ms", "compile_benchmarks"
    )
    max_runtime_smoke_ms = require_positive_int(
        compile_benchmarks, "max_runtime_smoke_ms", "compile_benchmarks"
    )
    min_index_cache_hit_rate = require_non_negative_number(
        compile_benchmarks, "min_index_cache_hit_rate", "compile_benchmarks"
    )
    if min_index_cache_hit_rate > 1.0:
        fail("compile_benchmarks.min_index_cache_hit_rate must be <= 1.0")

    compile_clean_ms = metrics.get("compile_clean_ms")
    compile_noop_ms = metrics.get("compile_noop_ms")
    compile_incremental_ms = metrics.get("compile_incremental_ms")
    runtime_smoke_ms = metrics.get("runtime_smoke_ms")
    index_cache_hit_rate = metrics.get("index_cache_hit_rate")
    index_memory_bytes = metrics.get("index_memory_bytes")
    index_memory_ratio = metrics.get("index_memory_ratio")
    if not isinstance(compile_clean_ms, int) or compile_clean_ms <= 0:
        fail("phase6 metrics compile_clean_ms must be a positive integer")
    if not isinstance(compile_noop_ms, int) or compile_noop_ms <= 0:
        fail("phase6 metrics compile_noop_ms must be a positive integer")
    if not isinstance(compile_incremental_ms, int) or compile_incremental_ms < 0:
        fail("phase6 metrics compile_incremental_ms must be a non-negative integer")
    if not isinstance(runtime_smoke_ms, int) or runtime_smoke_ms <= 0:
        fail("phase6 metrics runtime_smoke_ms must be a positive integer")
    if not isinstance(index_cache_hit_rate, (int, float)) or not 0.0 <= index_cache_hit_rate <= 1.0:
        fail("phase6 metrics index_cache_hit_rate must be within [0, 1]")
    if not isinstance(index_memory_bytes, int) or index_memory_bytes < 0:
        fail("phase6 metrics index_memory_bytes must be a non-negative integer")
    if not isinstance(index_memory_ratio, (int, float)) or index_memory_ratio < 0:
        fail("phase6 metrics index_memory_ratio must be a non-negative number")

    if compile_clean_ms > max_clean_ms:
        fail(
            f"compile_clean_ms={compile_clean_ms} exceeds budget max_clean_ms={max_clean_ms}"
        )
    if compile_noop_ms > max_noop_ms:
        fail(f"compile_noop_ms={compile_noop_ms} exceeds budget max_noop_ms={max_noop_ms}")
    if compile_incremental_ms > max_incremental_ms:
        fail(
            f"compile_incremental_ms={compile_incremental_ms} exceeds budget max_incremental_ms={max_incremental_ms}"
        )
    if float(index_cache_hit_rate) < min_index_cache_hit_rate:
        fail(
            "index_cache_hit_rate="
            f"{float(index_cache_hit_rate):.4f} is below min_index_cache_hit_rate={min_index_cache_hit_rate:.4f}"
        )
    if runtime_smoke_ms > max_runtime_smoke_ms:
        fail(
            f"runtime_smoke_ms={runtime_smoke_ms} exceeds budget max_runtime_smoke_ms={max_runtime_smoke_ms}"
        )

    stress_budgets = budgets.get("stress_budgets")
    if not isinstance(stress_budgets, dict):
        fail("stress_budgets section is required")
    require_positive_int(stress_budgets, "max_concurrency_smoke_seconds", "stress_budgets")
    require_positive_int(stress_budgets, "max_memory_smoke_seconds", "stress_budgets")
    max_index_memory_bytes = require_positive_int(
        stress_budgets, "max_index_memory_bytes", "stress_budgets"
    )
    max_index_memory_ratio = require_non_negative_number(
        stress_budgets, "max_index_memory_ratio", "stress_budgets"
    )
    if max_index_memory_ratio == 0:
        fail("stress_budgets.max_index_memory_ratio must be > 0")
    if index_memory_bytes > max_index_memory_bytes:
        fail(
            f"index_memory_bytes={index_memory_bytes} exceeds budget max_index_memory_bytes={max_index_memory_bytes}"
        )
    if float(index_memory_ratio) > max_index_memory_ratio:
        fail(
            f"index_memory_ratio={float(index_memory_ratio):.4f} exceeds budget max_index_memory_ratio={max_index_memory_ratio:.4f}"
        )

    memory_lanes = budgets.get("memory_lanes")
    if not isinstance(memory_lanes, dict):
        fail("memory_lanes section is required")
    require_default_gc_lane = memory_lanes.get("require_default_gc_lane")
    if not isinstance(require_default_gc_lane, bool):
        fail("memory_lanes.require_default_gc_lane must be boolean")
    require_default_valgrind_lane = memory_lanes.get("require_default_valgrind_lane")
    if not isinstance(require_default_valgrind_lane, bool):
        fail("memory_lanes.require_default_valgrind_lane must be boolean")

    gc_lane_pass = metrics.get("memory_gc_lane_pass")
    if not isinstance(gc_lane_pass, bool):
        fail("phase6 metrics memory_gc_lane_pass must be boolean")
    valgrind_lane_pass = metrics.get("memory_valgrind_lane_pass")
    if not isinstance(valgrind_lane_pass, bool):
        fail("phase6 metrics memory_valgrind_lane_pass must be boolean")
    valgrind_lane_skipped = metrics.get("memory_valgrind_lane_skipped")
    if not isinstance(valgrind_lane_skipped, bool):
        fail("phase6 metrics memory_valgrind_lane_skipped must be boolean")

    if require_default_gc_lane and not gc_lane_pass:
        fail("memory GC default lane is required but reported as failed")
    if require_default_valgrind_lane and (not valgrind_lane_pass or valgrind_lane_skipped):
        fail("memory valgrind default lane is required but did not run cleanly")

    coverage_requirements = budgets.get("coverage_requirements")
    if not isinstance(coverage_requirements, dict):
        fail("coverage_requirements section is required")
    required_tests = {
        "require_frontend_fixtures": "frontend_fixtures.rs",
        "require_phase7_validation": "phase7_validation.rs",
        "require_phase7_concurrency": "phase7_concurrency.rs",
        "require_phase7_intent_validation": "phase7_intent_validation.rs",
        "require_phase7_v1_tightening": "phase7_v1_tightening.rs",
    }
    tests_dir = repo_root / "crates" / "vibe_cli" / "tests"
    for key, test_file in required_tests.items():
        enabled = coverage_requirements.get(key)
        if not isinstance(enabled, bool):
            fail(f"coverage_requirements.{key} must be boolean")
        if enabled and not (tests_dir / test_file).exists():
            fail(f"required test file missing: {tests_dir / test_file}")

    print("v1 quality budget validation passed")


if __name__ == "__main__":
    main()
