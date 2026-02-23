#!/usr/bin/env python3
import json
from pathlib import Path


def ensure_non_empty_list(value: object, field: str) -> list[str]:
    if not isinstance(value, list) or not value or not all(isinstance(x, str) for x in value):
        raise SystemExit(f"release notes input field `{field}` must be a non-empty string list")
    return value


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    reports_dir = repo_root / "reports" / "v1"
    reports_dir.mkdir(parents=True, exist_ok=True)

    inputs_path = reports_dir / "release_notes_inputs.json"
    if not inputs_path.exists():
        raise SystemExit(f"missing release notes input: {inputs_path}")
    raw = json.loads(inputs_path.read_text())

    candidate = raw.get("candidate")
    date = raw.get("date")
    if not isinstance(candidate, str) or not candidate:
        raise SystemExit("release notes input field `candidate` must be a non-empty string")
    if not isinstance(date, str) or not date:
        raise SystemExit("release notes input field `date` must be a non-empty string")
    highlights = ensure_non_empty_list(raw.get("highlights"), "highlights")
    known_limitations = ensure_non_empty_list(raw.get("known_limitations"), "known_limitations")
    breaking_changes = ensure_non_empty_list(raw.get("breaking_changes"), "breaking_changes")

    preview_md = reports_dir / "release_notes_preview.md"
    preview_json = reports_dir / "release_notes_preview.json"
    markdown = (
        "# V1 Release Notes Preview\n\n"
        f"- candidate: `{candidate}`\n"
        f"- date: `{date}`\n\n"
        "## Highlights\n\n"
        + "\n".join(f"- {entry}" for entry in highlights)
        + "\n\n## Known Limitations\n\n"
        + "\n".join(f"- {entry}" for entry in known_limitations)
        + "\n\n## Breaking Changes\n\n"
        + "\n".join(f"- {entry}" for entry in breaking_changes)
        + "\n\n## Evidence Links\n\n"
        "- `reports/v1/readiness_dashboard.md`\n"
        "- `reports/v1/release_candidate_checklist.md`\n"
        "- `docs/release/known_limitations_gate.md`\n"
    )
    preview_md.write_text(markdown)

    summary = {
        "format": "v1-release-notes-preview-v1",
        "candidate": candidate,
        "date": date,
        "counts": {
            "highlights": len(highlights),
            "known_limitations": len(known_limitations),
            "breaking_changes": len(breaking_changes),
        },
    }
    preview_json.write_text(json.dumps(summary, indent=2) + "\n")
    print(f"wrote {preview_md}")


if __name__ == "__main__":
    main()
