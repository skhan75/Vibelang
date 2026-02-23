#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path


LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


def is_external(target: str) -> bool:
    return target.startswith("http://") or target.startswith("https://") or target.startswith(
        "mailto:"
    )


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    reports_root = repo_root / "reports" / "docs"
    reports_root.mkdir(parents=True, exist_ok=True)
    files = sorted((repo_root / "docs").rglob("*.md")) + sorted(
        (repo_root / "book" / "src").glob("*.md")
    )

    missing: list[dict[str, str]] = []
    for file_path in files:
        text = file_path.read_text(errors="ignore")
        for target in LINK_RE.findall(text):
            clean = target.strip()
            if not clean or clean.startswith("#") or is_external(clean):
                continue
            clean = clean.split("#", 1)[0]
            if clean.startswith("<") and clean.endswith(">"):
                clean = clean[1:-1]
            if clean.startswith("/"):
                resolved = repo_root / clean[1:]
            else:
                resolved = (file_path.parent / clean).resolve()
            if not resolved.exists():
                missing.append(
                    {
                        "source": str(file_path.relative_to(repo_root)),
                        "target": target,
                    }
                )

    report = {"format": "docs-link-check-v1", "missing_links": missing}
    (reports_root / "link_check.json").write_text(json.dumps(report, indent=2) + "\n")
    if missing:
        print(f"missing links: {len(missing)}")
        sys.exit(1)
    print("docs link check passed")


if __name__ == "__main__":
    main()
