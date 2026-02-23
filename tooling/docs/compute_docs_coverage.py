#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    reports_root = repo_root / "reports" / "docs"
    reports_root.mkdir(parents=True, exist_ok=True)
    matrix_path = repo_root / "docs" / "spec" / "spec_coverage_matrix.md"
    if not matrix_path.exists():
        raise SystemExit(f"missing spec coverage matrix: {matrix_path}")

    rows = []
    for line in matrix_path.read_text(errors="ignore").splitlines():
        if not line.startswith("| SPEC-"):
            continue
        parts = [part.strip() for part in line.strip().strip("|").split("|")]
        if len(parts) < 6:
            continue
        rows.append(parts)

    total = len(rows)
    documented = 0
    implemented = 0
    implemented_statuses = {"implemented", "done", "local-pass", "validated"}
    for row in rows:
        status = row[3].lower()
        evidence = row[5].strip()
        if evidence and evidence != "-":
            documented += 1
        if status in implemented_statuses:
            implemented += 1

    chapter_count = len(list((repo_root / "book" / "src").glob("ch*.md")))
    coverage_ratio = (documented / total) if total else 0.0
    report = {
        "format": "docs-coverage-v1",
        "spec_rows_total": total,
        "spec_rows_documented": documented,
        "spec_rows_implemented": implemented,
        "spec_documented_ratio": round(coverage_ratio, 4),
        "book_chapter_count": chapter_count,
    }
    (reports_root / "docs_coverage.json").write_text(json.dumps(report, indent=2) + "\n")
    if chapter_count < 10:
        raise SystemExit(f"docs coverage check failed: expected >=10 chapters, got {chapter_count}")
    if coverage_ratio < 0.5:
        raise SystemExit(
            "docs coverage check failed: spec documented ratio below 0.5 "
            f"({coverage_ratio:.4f})"
        )
    print(
        "docs coverage computed: "
        f"rows={total}, documented={documented}, implemented={implemented}, chapters={chapter_count}"
    )


if __name__ == "__main__":
    main()
