#!/usr/bin/env python3
import re
import sys
from pathlib import Path


def fail(msg: str) -> None:
    print(f"spec coverage validation failed: {msg}")
    sys.exit(1)


def parse_rule_rows(matrix_text: str) -> list[tuple[str, str, str, str]]:
    rows: list[tuple[str, str, str, str]] = []
    for raw_line in matrix_text.splitlines():
        line = raw_line.strip()
        if not line.startswith("| SPEC-"):
            continue
        parts = [p.strip() for p in line.split("|")]
        # Expected shape with leading/trailing separators:
        # ["", rule, area, requirement, status, evidence, notes, ""]
        if len(parts) < 8:
            fail(f"malformed matrix row: {line}")
        rule_id = parts[1]
        status = parts[4]
        evidence = parts[5]
        notes = parts[6]
        rows.append((rule_id, status, evidence, notes))
    return rows


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    matrix_path = repo_root / "docs" / "spec" / "spec_coverage_matrix.md"
    if not matrix_path.exists():
        fail(f"missing matrix file: {matrix_path}")

    matrix = matrix_path.read_text()
    rows = parse_rule_rows(matrix)
    if not rows:
        fail("no SPEC-* rows found in coverage matrix")

    required_rules = [
        "SPEC-SYN-001",
        "SPEC-SYN-002",
        "SPEC-OPT-001",
        "SPEC-CON-001",
        "SPEC-EFF-001",
        "SPEC-TYP-001",
        "SPEC-NUM-001",
        "SPEC-NUM-002",
        "SPEC-MUT-001",
        "SPEC-STR-001",
        "SPEC-CNT-001",
        "SPEC-CFG-001",
        "SPEC-CONC-001",
        "SPEC-ASY-001",
        "SPEC-THR-001",
        "SPEC-OWN-001",
        "SPEC-MEM-001",
        "SPEC-GC-001",
        "SPEC-ERR-001",
        "SPEC-MOD-001",
        "SPEC-ABI-001",
    ]
    seen = {rule_id for rule_id, _, _, _ in rows}
    for rule in required_rules:
        if rule not in seen:
            fail(f"missing required rule row: {rule}")

    deferred_count = 0
    for rule_id, status, evidence, notes in rows:
        if status not in {"implemented", "deferred"}:
            fail(f"{rule_id} has invalid status `{status}`")

        evidence_paths = re.findall(r"`([^`]+)`", evidence)
        if not evidence_paths:
            fail(f"{rule_id} has no evidence paths")
        for rel_path in evidence_paths:
            # Skip plain labels that are not repository paths.
            if rel_path.startswith("workflow "):
                continue
            if "://" in rel_path:
                continue
            abs_path = repo_root / rel_path
            if not abs_path.exists():
                fail(f"{rule_id} references missing evidence path: {rel_path}")

        if status == "deferred":
            deferred_count += 1
            if "deferred" not in notes.lower():
                fail(
                    f"{rule_id} is deferred but notes do not explain deferred status"
                )

    if deferred_count == 0:
        fail("matrix must contain at least one deferred rule for honest tracking")

    print("spec coverage validation passed")


if __name__ == "__main__":
    main()
