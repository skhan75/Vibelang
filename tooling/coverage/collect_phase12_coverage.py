#!/usr/bin/env python3
import argparse
import json
import subprocess
import time
from pathlib import Path


SUITES = [
    (
        "stdlib_surface",
        ["cargo", "test", "-p", "vibe_cli", "--test", "phase12_stdlib"],
    ),
    (
        "package_lifecycle",
        ["cargo", "test", "-p", "vibe_cli", "--test", "phase12_package_ecosystem"],
    ),
    (
        "qa_ergonomics",
        ["cargo", "test", "-p", "vibe_cli", "--test", "phase12_test_ergonomics"],
    ),
    (
        "pkg_solver_core",
        ["cargo", "test", "-p", "vibe_pkg"],
    ),
]


def run_suite(repo_root: Path, name: str, command: list[str]) -> dict:
    started = time.time()
    proc = subprocess.run(
        command,
        cwd=repo_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    duration_ms = int((time.time() - started) * 1000)
    return {
        "suite": name,
        "command": " ".join(command),
        "passed": proc.returncode == 0,
        "exit_code": proc.returncode,
        "duration_ms": duration_ms,
        "stdout": proc.stdout[-8000:],
        "stderr": proc.stderr[-8000:],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Collect Phase 12 coverage surface report")
    parser.add_argument(
        "--output",
        default="reports/phase12/coverage_summary.json",
        help="output JSON report path relative to repo root",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[2]
    output_path = repo_root / args.output
    output_path.parent.mkdir(parents=True, exist_ok=True)

    suites = [run_suite(repo_root, name, command) for (name, command) in SUITES]
    passed = sum(1 for suite in suites if suite["passed"])
    total = len(suites)
    coverage_percent = round((passed / total) * 100.0, 2) if total else 0.0
    summary = {
        "phase": "12",
        "generated_at_unix_s": int(time.time()),
        "surface_passed": passed,
        "surface_total": total,
        "surface_coverage_percent": coverage_percent,
        "suites": suites,
    }
    output_path.write_text(json.dumps(summary, indent=2) + "\n")
    print(f"wrote phase12 coverage summary to {output_path}")
    print(
        f"coverage surfaces: passed={passed}/{total} percent={coverage_percent:.2f}"
    )


if __name__ == "__main__":
    main()
