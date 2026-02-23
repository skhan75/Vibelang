#!/usr/bin/env python3
import json
import re
import subprocess
import tempfile
from pathlib import Path


FENCE_RE = re.compile(r"```(vibe|yb)\n(.*?)\n```", re.DOTALL)


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )


def gather_snippets(path: Path) -> list[tuple[int, str]]:
    text = path.read_text(errors="ignore")
    snippets: list[tuple[int, str]] = []
    for match in FENCE_RE.finditer(text):
        start_line = text[: match.start()].count("\n") + 1
        snippets.append((start_line, match.group(2)))
    return snippets


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    reports_root = repo_root / "reports" / "docs"
    reports_root.mkdir(parents=True, exist_ok=True)

    markdown_files = sorted((repo_root / "book" / "src").glob("*.md"))
    failures: list[dict[str, object]] = []
    checked = 0
    with tempfile.TemporaryDirectory(prefix="vibe_docs_snippets_") as temp_dir:
        temp_root = Path(temp_dir)
        for file_path in markdown_files:
            for line, snippet in gather_snippets(file_path):
                checked += 1
                snippet_file = temp_root / f"{file_path.stem}_{checked}.yb"
                snippet_file.write_text(snippet + "\n")
                out = run(
                    ["cargo", "run", "-q", "-p", "vibe_cli", "--", "check", str(snippet_file)],
                    repo_root,
                )
                if out.returncode != 0:
                    failures.append(
                        {
                            "file": str(file_path.relative_to(repo_root)),
                            "line": line,
                            "stdout": out.stdout.strip(),
                            "stderr": out.stderr.strip(),
                        }
                    )

    report = {
        "format": "docs-snippet-validation-v1",
        "checked_snippets": checked,
        "failures": failures,
    }
    (reports_root / "snippet_validation.json").write_text(json.dumps(report, indent=2) + "\n")
    if checked == 0:
        raise SystemExit("docs snippet validation found no `vibe` snippets in book/src")
    if failures:
        raise SystemExit(
            f"docs snippet validation failed: {len(failures)} snippet(s) did not compile"
        )
    print(f"validated {checked} snippet(s)")


if __name__ == "__main__":
    main()
