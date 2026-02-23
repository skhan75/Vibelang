#!/usr/bin/env python3
import json
from pathlib import Path


def require(path: Path) -> None:
    if not path.exists():
        raise SystemExit(f"missing required security governance file: {path}")


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    report_root = repo_root / "reports" / "v1"
    report_root.mkdir(parents=True, exist_ok=True)

    cve = repo_root / "docs" / "security" / "cve_response_workflow.md"
    disclosure = repo_root / "docs" / "security" / "disclosure_policy.md"
    package_policy = repo_root / "docs" / "package" / "security_policy.md"
    release_blocker = repo_root / "docs" / "release" / "release_blocker_policy.md"
    for path in (cve, disclosure, package_policy, release_blocker):
        require(path)

    report = {
        "format": "v1-security-response-exercise-v1",
        "checked_files": [
            "docs/security/cve_response_workflow.md",
            "docs/security/disclosure_policy.md",
            "docs/package/security_policy.md",
            "docs/release/release_blocker_policy.md",
        ],
        "status": "pass",
    }
    (report_root / "security_response_exercise.json").write_text(
        json.dumps(report, indent=2) + "\n"
    )
    (report_root / "security_response_exercise.md").write_text(
        "# Security Response Exercise\n\n"
        "- status: pass\n"
        "- checked_files:\n"
        "  - `docs/security/cve_response_workflow.md`\n"
        "  - `docs/security/disclosure_policy.md`\n"
        "  - `docs/package/security_policy.md`\n"
        "  - `docs/release/release_blocker_policy.md`\n"
    )
    print(f"wrote {report_root / 'security_response_exercise.json'}")


if __name__ == "__main__":
    main()
