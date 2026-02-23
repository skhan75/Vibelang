#!/usr/bin/env python3
from pathlib import Path


def section_has_bullet(content: str, heading: str) -> bool:
    marker = f"## {heading}\n"
    idx = content.find(marker)
    if idx == -1:
        return False
    remainder = content[idx + len(marker) :]
    next_idx = remainder.find("\n## ")
    section_text = remainder if next_idx == -1 else remainder[:next_idx]
    return any(line.strip().startswith("- ") for line in section_text.splitlines())


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    preview = repo_root / "reports" / "v1" / "release_notes_preview.md"
    if not preview.exists():
        raise SystemExit(f"missing release notes preview: {preview}")
    content = preview.read_text()

    required_sections = ("Highlights", "Known Limitations", "Breaking Changes")
    for section in required_sections:
        if not section_has_bullet(content, section):
            raise SystemExit(f"release notes section `{section}` missing or empty")

    forbidden_tokens = ("TBD", "TODO", "<fill")
    for token in forbidden_tokens:
        if token in content:
            raise SystemExit(f"release notes contain forbidden placeholder token: {token}")

    print("release notes validation passed")


if __name__ == "__main__":
    main()
