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
    if not metrics_path.exists():
        fail(f"missing metrics file: {metrics_path}")
    metrics = json.loads(metrics_path.read_text())

    if metrics.get("compile_clean_ms", 0) <= 0:
        fail("compile_clean_ms must be > 0")
    if metrics.get("compile_noop_ms", 0) <= 0:
        fail("compile_noop_ms must be > 0")
    if metrics.get("dual_extension_parity_pass_rate", 0.0) < 1.0:
        fail("dual_extension_parity_pass_rate must be 1.0")
    if metrics.get("cross_target_pass_rate", 0.0) < 1.0:
        fail("cross_target_pass_rate must be 1.0")
    if not metrics.get("spec_conformance_docs_present", False):
        fail("spec_conformance_docs_present must be true")
    print("metrics validation passed")


if __name__ == "__main__":
    main()
