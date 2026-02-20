#!/usr/bin/env python3
import sys
from pathlib import Path


def fail(msg: str) -> None:
    print(f"phase7 coverage matrix validation failed: {msg}")
    sys.exit(1)


def require_contains(text: str, needle: str, label: str) -> None:
    if needle not in text:
        fail(f"missing {label}: `{needle}`")


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    matrix_path = repo_root / "reports" / "phase7" / "language_validation_matrix.md"
    catalog_path = repo_root / "reports" / "phase7" / "sample_programs_catalog.md"

    if not matrix_path.exists():
        fail(f"missing matrix report: {matrix_path}")
    if not catalog_path.exists():
        fail(f"missing sample catalog report: {catalog_path}")

    matrix = matrix_path.read_text()
    catalog = catalog_path.read_text()

    required_rows = [
        "Single-thread Sample Programs",
        "Concurrency Patterns",
        "Intent Lint Match/Drift/Changed Mode",
        "Algorithmic Recursion Programs",
        "Memory and Ownership Safety Smokes",
    ]
    for row in required_rows:
        require_contains(matrix, row, "matrix row")

    required_commands = [
        "cargo test -p vibe_cli --test phase7_validation",
        "cargo test -p vibe_cli --test phase7_concurrency",
        "cargo test -p vibe_cli --test phase7_intent_validation",
        "cargo test -p vibe_cli --test phase7_v1_tightening",
    ]
    for command in required_commands:
        require_contains(matrix, command, "validation command")

    required_catalog_sections = [
        "Algorithmic + Recursion Stress Samples",
        "Memory / Ownership Stress Samples",
        "Unknown Sendability Negative",
    ]
    for section in required_catalog_sections:
        require_contains(catalog, section, "catalog section")

    print("phase7 coverage matrix validation passed")


if __name__ == "__main__":
    main()
