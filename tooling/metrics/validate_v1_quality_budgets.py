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
    max_runtime_smoke_ms = require_positive_int(
        compile_benchmarks, "max_runtime_smoke_ms", "compile_benchmarks"
    )

    compile_clean_ms = metrics.get("compile_clean_ms")
    compile_noop_ms = metrics.get("compile_noop_ms")
    runtime_smoke_ms = metrics.get("runtime_smoke_ms")
    if not isinstance(compile_clean_ms, int) or compile_clean_ms <= 0:
        fail("phase6 metrics compile_clean_ms must be a positive integer")
    if not isinstance(compile_noop_ms, int) or compile_noop_ms <= 0:
        fail("phase6 metrics compile_noop_ms must be a positive integer")
    if not isinstance(runtime_smoke_ms, int) or runtime_smoke_ms <= 0:
        fail("phase6 metrics runtime_smoke_ms must be a positive integer")

    if compile_clean_ms > max_clean_ms:
        fail(
            f"compile_clean_ms={compile_clean_ms} exceeds budget max_clean_ms={max_clean_ms}"
        )
    if compile_noop_ms > max_noop_ms:
        fail(f"compile_noop_ms={compile_noop_ms} exceeds budget max_noop_ms={max_noop_ms}")
    if runtime_smoke_ms > max_runtime_smoke_ms:
        fail(
            f"runtime_smoke_ms={runtime_smoke_ms} exceeds budget max_runtime_smoke_ms={max_runtime_smoke_ms}"
        )

    stress_budgets = budgets.get("stress_budgets")
    if not isinstance(stress_budgets, dict):
        fail("stress_budgets section is required")
    require_positive_int(stress_budgets, "max_concurrency_smoke_seconds", "stress_budgets")
    require_positive_int(stress_budgets, "max_memory_smoke_seconds", "stress_budgets")

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
