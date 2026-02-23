#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path


BANNED_TOKENS = {
    "teh",
    "recieve",
    "occured",
    "seperate",
    "langauge",
}


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    reports_root = repo_root / "reports" / "docs"
    reports_root.mkdir(parents=True, exist_ok=True)
    files = sorted((repo_root / "docs").rglob("*.md")) + sorted(
        (repo_root / "book" / "src").glob("*.md")
    )

    findings: list[dict[str, str]] = []
    word_re = re.compile(r"[A-Za-z']+")
    for file_path in files:
        text = file_path.read_text(errors="ignore")
        for token in word_re.findall(text):
            lowered = token.lower()
            if lowered in BANNED_TOKENS:
                findings.append(
                    {
                        "file": str(file_path.relative_to(repo_root)),
                        "token": token,
                    }
                )

    report = {"format": "docs-spell-check-v1", "findings": findings}
    (reports_root / "spell_check.json").write_text(json.dumps(report, indent=2) + "\n")
    if findings:
        print(f"spell check findings: {len(findings)}")
        sys.exit(1)
    print("docs spell check passed")


if __name__ == "__main__":
    main()
