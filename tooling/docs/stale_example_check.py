#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    reports_root = repo_root / "reports" / "docs"
    reports_root.mkdir(parents=True, exist_ok=True)
    files = sorted((repo_root / "book" / "src").glob("*.md"))

    findings: list[dict[str, str]] = []
    legacy_extension_allowlist = {"ch09_migration_compatibility.md"}
    for file_path in files:
        text = file_path.read_text(errors="ignore")
        if ".vibe" in text and file_path.name not in legacy_extension_allowlist:
            findings.append(
                {
                    "file": str(file_path.relative_to(repo_root)),
                    "issue": "legacy .vibe extension used in book content",
                }
            )
        if "TODO" in text:
            findings.append(
                {
                    "file": str(file_path.relative_to(repo_root)),
                    "issue": "TODO marker present in published chapter",
                }
            )

    report = {"format": "docs-stale-example-check-v1", "findings": findings}
    (reports_root / "stale_example_check.json").write_text(json.dumps(report, indent=2) + "\n")
    if findings:
        print(f"stale example findings: {len(findings)}")
        sys.exit(1)
    print("docs stale-example check passed")


if __name__ == "__main__":
    main()
