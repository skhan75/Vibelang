#!/usr/bin/env python3
import json
from pathlib import Path


def load_json(path: Path) -> dict:
    if not path.exists():
        raise SystemExit(f"missing required docs quality input: {path}")
    return json.loads(path.read_text())


def status_from_findings(items: list) -> str:
    return "pass" if not items else "fail"


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    reports_root = repo_root / "reports" / "docs"
    reports_root.mkdir(parents=True, exist_ok=True)

    snippet = load_json(reports_root / "snippet_validation.json")
    links = load_json(reports_root / "link_check.json")
    spell = load_json(reports_root / "spell_check.json")
    stale = load_json(reports_root / "stale_example_check.json")
    coverage = load_json(reports_root / "docs_coverage.json")

    usability_checklist = repo_root / "docs" / "usability" / "docs_walkthrough_checklist.md"
    usability_present = usability_checklist.exists()

    summary = {
        "format": "docs-quality-report-v1",
        "snippet_validation_status": "pass"
        if not snippet.get("failures")
        else "fail",
        "link_check_status": status_from_findings(links.get("missing_links", [])),
        "spell_check_status": status_from_findings(spell.get("findings", [])),
        "stale_example_status": status_from_findings(stale.get("findings", [])),
        "docs_coverage_ratio": coverage.get("spec_documented_ratio", 0.0),
        "book_chapter_count": coverage.get("book_chapter_count", 0),
        "usability_checklist_present": usability_present,
    }

    (reports_root / "documentation_quality.json").write_text(
        json.dumps(summary, indent=2) + "\n"
    )
    report_md = (
        "# Documentation Quality Report\n\n"
        f"- snippet_validation_status: {summary['snippet_validation_status']}\n"
        f"- link_check_status: {summary['link_check_status']}\n"
        f"- spell_check_status: {summary['spell_check_status']}\n"
        f"- stale_example_status: {summary['stale_example_status']}\n"
        f"- docs_coverage_ratio: {summary['docs_coverage_ratio']}\n"
        f"- book_chapter_count: {summary['book_chapter_count']}\n"
        f"- usability_checklist_present: {summary['usability_checklist_present']}\n"
    )
    (reports_root / "documentation_quality.md").write_text(report_md)
    print(f"wrote {reports_root / 'documentation_quality.md'}")


if __name__ == "__main__":
    main()
