#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def fail(message: str) -> None:
    print(f"release benchmark validation failed: {message}")
    sys.exit(1)


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    report_path = repo_root / "reports" / "v1" / "release_benchmarks.json"
    if not report_path.exists():
        fail(f"missing report: {report_path}")

    metrics = json.loads(report_path.read_text())
    if metrics.get("benchmark_method") != "vibe_release_binary_smoke":
        fail("benchmark_method must be `vibe_release_binary_smoke`")
    if metrics.get("profile") != "release":
        fail("profile must be `release`")

    compile_latency_ms = metrics.get("compile_latency_ms", 0)
    runtime_latency_ms = metrics.get("runtime_latency_ms", 0)
    cpu_ms = metrics.get("cpu_ms", 0)
    memory_bytes = metrics.get("memory_bytes", 0)
    if not isinstance(compile_latency_ms, int) or compile_latency_ms <= 0:
        fail("compile_latency_ms must be a positive integer")
    if not isinstance(runtime_latency_ms, int) or runtime_latency_ms <= 0:
        fail("runtime_latency_ms must be a positive integer")
    if not isinstance(cpu_ms, int) or cpu_ms <= 0:
        fail("cpu_ms must be a positive integer")
    if not isinstance(memory_bytes, int) or memory_bytes <= 0:
        fail("memory_bytes must be a positive integer")
    print("release benchmark validation passed")


if __name__ == "__main__":
    main()
