#!/usr/bin/env python3
import json
from pathlib import Path


def require(path: Path) -> None:
    if not path.exists():
        raise SystemExit(f"missing required governance file: {path}")


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    report_root = repo_root / "reports" / "v1"
    report_root.mkdir(parents=True, exist_ok=True)

    lts = repo_root / "docs" / "support" / "lts_support_windows.md"
    compat = repo_root / "docs" / "policy" / "compatibility_guarantees.md"
    versioning = repo_root / "docs" / "policy" / "versioning_compatibility.md"
    support_matrix = repo_root / "docs" / "targets" / "support_matrix.md"
    for path in (lts, compat, versioning, support_matrix):
        require(path)

    report = {
        "format": "v1-lts-support-exercise-v1",
        "checked_files": [
            "docs/support/lts_support_windows.md",
            "docs/policy/compatibility_guarantees.md",
            "docs/policy/versioning_compatibility.md",
            "docs/targets/support_matrix.md",
        ],
        "status": "pass",
    }
    (report_root / "lts_support_exercise.json").write_text(json.dumps(report, indent=2) + "\n")
    (report_root / "lts_support_exercise.md").write_text(
        "# LTS Support Exercise\n\n"
        "- status: pass\n"
        "- checked_files:\n"
        "  - `docs/support/lts_support_windows.md`\n"
        "  - `docs/policy/compatibility_guarantees.md`\n"
        "  - `docs/policy/versioning_compatibility.md`\n"
        "  - `docs/targets/support_matrix.md`\n"
    )
    print(f"wrote {report_root / 'lts_support_exercise.json'}")


if __name__ == "__main__":
    main()
