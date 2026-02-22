#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path


def fail(message: str) -> None:
    print(f"phase13 grammar validation failed: {message}")
    sys.exit(1)


def read_json(path: Path) -> dict:
    if not path.exists():
        fail(f"missing json file: {path}")
    try:
        return json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        fail(f"invalid json at {path}: {exc}")


def ensure_required_repository_blocks(grammar: dict) -> None:
    repo = grammar.get("repository")
    if not isinstance(repo, dict):
        fail("grammar.repository must be an object")
    required = {
        "comments",
        "contracts",
        "keywords",
        "storageModifiers",
        "types",
        "booleansAndNone",
        "numbers",
        "strings",
        "chars",
        "moduleAndImportPaths",
        "functionDecl",
        "functionCalls",
        "operators",
    }
    missing = sorted(required - set(repo.keys()))
    if missing:
        fail(f"grammar.repository missing keys: {', '.join(missing)}")


def ensure_patterns_reference_repository(grammar: dict) -> None:
    patterns = grammar.get("patterns")
    if not isinstance(patterns, list) or not patterns:
        fail("grammar.patterns must be a non-empty list")
    includes = {
        item.get("include")
        for item in patterns
        if isinstance(item, dict) and isinstance(item.get("include"), str)
    }
    for required in (
        "#comments",
        "#contracts",
        "#keywords",
        "#numbers",
        "#strings",
        "#operators",
    ):
        if required not in includes:
            fail(f"grammar.patterns missing include reference: {required}")


def load_fixture_text(fixtures_root: Path) -> str:
    if not fixtures_root.exists():
        fail(f"fixtures root missing: {fixtures_root}")
    sources = []
    for path in fixtures_root.rglob("*.yb"):
        if (not path.is_file()) or "/.yb/" in str(path):
            continue
        sources.append(path.read_text(errors="ignore"))
    for path in fixtures_root.rglob("*.vibe"):
        if (not path.is_file()) or "/.yb/" in str(path):
            continue
        sources.append(path.read_text(errors="ignore"))
    if not sources:
        fail(f"no fixture source files found under {fixtures_root}")
    return "\n".join(sources)


def ensure_fixture_coverage(text: str) -> None:
    checks = {
        "contract annotation usage": r"@(intent|require|ensure|effect)\b",
        "line comments": r"//",
        "string literals": r"\"(?:\\.|[^\"\\])*\"",
        "integer literals": r"\b\d+\b",
        "boolean literals": r"\b(true|false)\b",
        "control flow keywords": r"\b(if|for|while|match|select)\b",
        "binding operator": r":=",
    }
    missing = []
    for label, pattern in checks.items():
        if re.search(pattern, text) is None:
            missing.append(label)
    if missing:
        fail("fixture corpus missing expected token categories: " + ", ".join(missing))


def ensure_language_configuration(language_cfg: dict) -> None:
    comments = language_cfg.get("comments", {})
    if comments.get("lineComment") != "//":
        fail("language-configuration lineComment must be //")
    brackets = language_cfg.get("brackets")
    if not isinstance(brackets, list) or len(brackets) < 3:
        fail("language-configuration brackets must include {}, [], ()")
    if "wordPattern" not in language_cfg:
        fail("language-configuration must define wordPattern")


def ensure_snippets(snippets: dict) -> None:
    if not isinstance(snippets, dict) or not snippets:
        fail("snippets file must be a non-empty object")
    required_prefixes = {"main", "fnintent", "require", "select", "containers"}
    found = {
        body.get("prefix")
        for body in snippets.values()
        if isinstance(body, dict) and isinstance(body.get("prefix"), str)
    }
    missing = sorted(required_prefixes - found)
    if missing:
        fail(f"snippets missing required prefixes: {', '.join(missing)}")


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    vscode_root = repo_root / "editor-support" / "vscode"
    grammar_path = vscode_root / "syntaxes" / "vibelang.tmLanguage.json"
    language_cfg_path = vscode_root / "language-configuration.json"
    snippets_path = vscode_root / "snippets" / "vibelang.code-snippets"
    fixtures_root = repo_root / "compiler" / "tests" / "fixtures" / "phase7"

    grammar = read_json(grammar_path)
    language_cfg = read_json(language_cfg_path)
    snippets = read_json(snippets_path)
    fixture_text = load_fixture_text(fixtures_root)

    ensure_required_repository_blocks(grammar)
    ensure_patterns_reference_repository(grammar)
    ensure_language_configuration(language_cfg)
    ensure_snippets(snippets)
    ensure_fixture_coverage(fixture_text)

    print("phase13 grammar validation passed")


if __name__ == "__main__":
    main()

