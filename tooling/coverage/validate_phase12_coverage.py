#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def fail(message: str) -> None:
    print(f"phase12 coverage validation failed: {message}")
    sys.exit(1)


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    summary_path = repo_root / "reports" / "phase12" / "coverage_summary.json"
    if not summary_path.exists():
        fail(f"missing summary file: {summary_path}")
    try:
        summary = json.loads(summary_path.read_text())
    except json.JSONDecodeError as exc:
        fail(f"invalid json in summary file: {exc}")

    suites = summary.get("suites")
    if not isinstance(suites, list) or not suites:
        fail("`suites` must be a non-empty array")
    failed = [suite for suite in suites if not suite.get("passed")]
    if failed:
        names = ", ".join(str(suite.get("suite", "unknown")) for suite in failed)
        fail(f"one or more coverage suites failed: {names}")

    coverage_percent = summary.get("surface_coverage_percent")
    if not isinstance(coverage_percent, (int, float)):
        fail("`surface_coverage_percent` must be numeric")
    if float(coverage_percent) < 100.0:
        fail(f"coverage percent is below required threshold: {coverage_percent}")

    print("phase12 coverage validation passed")


if __name__ == "__main__":
    main()
