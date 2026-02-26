#!/usr/bin/env python3
import argparse
import json
import re
from pathlib import Path
from typing import Any


def fail(message: str) -> None:
    raise SystemExit(f"adapter parity validation failed: {message}")


def expect_dict(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        fail(f"{name} must be an object")
    return value


def expect_list(value: Any, name: str) -> list[Any]:
    if not isinstance(value, list):
        fail(f"{name} must be a list")
    return value


def load_data_file(path: Path) -> dict[str, Any]:
    if not path.exists():
        fail(f"required file missing: {path}")
    raw = path.read_text()
    try:
        parsed = json.loads(raw)
    except json.JSONDecodeError:
        try:
            import yaml  # type: ignore
        except Exception as exc:
            fail(
                f"failed to parse {path} as JSON and PyYAML is unavailable: {exc}"
            )
        try:
            parsed = yaml.safe_load(raw)
        except Exception as exc:
            fail(f"failed to parse {path} as YAML: {exc}")
    if not isinstance(parsed, dict):
        fail(f"{path} root must be an object")
    return parsed


def analyze_adapter_file(path: Path) -> dict[str, Any]:
    text = path.read_text()
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    literal_prints = sum(
        1
        for line in lines
        if re.match(r'^println\("[^"]*"\)$', line) is not None
    )
    proxy_signatures = [
        "Primes up to   160000    14683",
        "A 30.279",
        "agggtaaa|tttaccct 2",
        ">ONE Homo sapiens alu",
        "9f8a9edb47ee2f885325cdc8a18591f4",
        "80bf2dee6461725c8200bfced3c695b7",
        "bac4db182bd8e59d",
    ]
    matched_signatures = [sig for sig in proxy_signatures if sig in text]
    suspicious = bool(matched_signatures) or literal_prints >= 15
    return {
        "literal_print_count": literal_prints,
        "matched_proxy_signatures": matched_signatures,
        "suspicious": suspicious,
    }


def validate_manifest(
    manifest: dict[str, Any],
    matrix: dict[str, Any],
    adapter_root: Path,
    publication_mode: bool,
) -> dict[str, Any]:
    if manifest.get("format") != "vibe-plbci-parity-manifest-v1":
        fail("parity manifest format mismatch")
    if matrix.get("format") != "vibe-third-party-language-matrix-v1":
        fail("language matrix format mismatch")

    required_problems = [
        str(item) for item in expect_list(matrix.get("problems"), "matrix.problems")
    ]
    manifest_rows = expect_list(manifest.get("problems"), "manifest.problems")
    allowed_statuses = {"canonical", "proxy", "blocked"}

    by_name: dict[str, dict[str, Any]] = {}
    duplicates: list[str] = []
    for item in manifest_rows:
        row = expect_dict(item, "manifest.problems[]")
        name = str(row.get("name", "")).strip()
        if not name:
            fail("manifest problem entry missing `name`")
        if name in by_name:
            duplicates.append(name)
            continue
        status = str(row.get("status", "")).strip()
        if status not in allowed_statuses:
            fail(
                f"manifest problem `{name}` has invalid status `{status}` "
                f"(allowed: {sorted(allowed_statuses)})"
            )
        if not str(row.get("owner", "")).strip():
            fail(f"manifest problem `{name}` missing `owner`")
        if not str(row.get("evidence", "")).strip():
            fail(f"manifest problem `{name}` missing `evidence`")
        _ = expect_list(row.get("exit_criteria", []), f"manifest `{name}` exit_criteria")
        by_name[name] = row
    if duplicates:
        fail("duplicate problem entries in manifest: " + ", ".join(sorted(duplicates)))

    missing = [problem for problem in required_problems if problem not in by_name]
    extra = [problem for problem in by_name if problem not in set(required_problems)]

    warnings: list[str] = []
    violations: list[str] = []
    suspicious_canonical: list[str] = []
    noncanonical: list[str] = []

    analysis: dict[str, Any] = {}
    for problem in required_problems:
        row = by_name[problem]
        status = str(row.get("status", "blocked")).strip()
        if status != "canonical":
            noncanonical.append(problem)
        adapter_file = adapter_root / "algorithm" / problem / "1.yb"
        if not adapter_file.exists():
            violations.append(f"adapter source missing for `{problem}`: {adapter_file}")
            continue
        adapter_analysis = analyze_adapter_file(adapter_file)
        analysis[problem] = adapter_analysis
        if adapter_analysis["suspicious"]:
            warnings.append(
                f"heuristic proxy signal in `{problem}` "
                f"(literal_print_count={adapter_analysis['literal_print_count']}, "
                f"matched_signatures={adapter_analysis['matched_proxy_signatures']})"
            )
            if status == "canonical":
                suspicious_canonical.append(problem)

    if missing:
        violations.append("manifest missing problems: " + ", ".join(sorted(missing)))
    if extra:
        warnings.append("manifest has extra unmapped problems: " + ", ".join(sorted(extra)))

    if publication_mode:
        if noncanonical:
            violations.append(
                "publication mode requires all problems canonical; noncanonical entries: "
                + ", ".join(sorted(noncanonical))
            )
        if suspicious_canonical:
            violations.append(
                "heuristic/manfiest disagreement for canonical entries: "
                + ", ".join(sorted(suspicious_canonical))
            )

    status = "pass" if not violations else "fail"
    return {
        "format": "vibe-parity-validation-v1",
        "status": status,
        "publication_mode": publication_mode,
        "required_problem_count": len(required_problems),
        "manifest_problem_count": len(by_name),
        "noncanonical_count": len(noncanonical),
        "warnings": warnings,
        "violations": violations,
        "analysis": analysis,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--manifest",
        default="benchmarks/third_party/plbci/adapters/vibelang/PARITY_MANIFEST.yaml",
        help="Parity manifest path relative to repo root.",
    )
    parser.add_argument(
        "--matrix-file",
        default="benchmarks/third_party/plbci/config/language_matrix.json",
        help="Language matrix path relative to repo root.",
    )
    parser.add_argument(
        "--adapter-root",
        default="benchmarks/third_party/plbci/adapters/vibelang",
        help="VibeLang adapter root path relative to repo root.",
    )
    parser.add_argument(
        "--publication-mode",
        action="store_true",
        help="Require all parity checks needed for strict publication runs.",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[2]
    manifest_path = repo_root / args.manifest
    matrix_path = repo_root / args.matrix_file
    adapter_root = repo_root / args.adapter_root

    manifest = load_data_file(manifest_path)
    matrix = load_data_file(matrix_path)
    result = validate_manifest(
        manifest=manifest,
        matrix=matrix,
        adapter_root=adapter_root,
        publication_mode=args.publication_mode,
    )
    print(json.dumps(result, indent=2))
    if result["violations"]:
        fail("; ".join(str(item) for item in result["violations"]))


if __name__ == "__main__":
    main()
