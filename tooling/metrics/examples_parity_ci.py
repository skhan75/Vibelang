#!/usr/bin/env python3
import argparse
import json
import re
import subprocess
import time
from pathlib import Path
from typing import Any


def run(cmd: list[str], cwd: Path) -> dict[str, Any]:
    started = time.time()
    completed = subprocess.run(
        cmd,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return {
        "cmd": cmd,
        "exit_code": int(completed.returncode),
        "stdout": completed.stdout,
        "stderr": completed.stderr,
        "elapsed_ms": int((time.time() - started) * 1000),
    }


def has_main_entry(source: Path) -> bool:
    try:
        text = source.read_text()
    except Exception:
        return False
    return bool(
        re.search(r"\bpub\s+main\s*\(", text)
        or re.search(r"(?m)^\s*main\s*\(", text)
    )


def parse_allowlist(path: Path) -> tuple[dict[str, str], list[str]]:
    mapping: dict[str, str] = {}
    errors: list[str] = []
    if not path.exists():
        return mapping, errors

    for idx, raw in enumerate(path.read_text().splitlines(), start=1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if "|" not in line:
            errors.append(
                f"allowlist line {idx} must include checklist ID using `path | CHECKLIST-ID` format"
            )
            continue
        file_part, id_part = [part.strip() for part in line.split("|", 1)]
        if not file_part:
            errors.append(f"allowlist line {idx} has empty file path")
            continue
        if not re.fullmatch(r"[A-Z]-[0-9A-Za-z-]+", id_part):
            errors.append(
                f"allowlist line {idx} has invalid checklist ID `{id_part}` (expected pattern like `D-01`)"
            )
            continue
        mapping[file_part] = id_part
    return mapping, errors


def to_rel(repo_root: Path, file_path: Path) -> str:
    return str(file_path.relative_to(repo_root)).replace("\\", "/")


def write_report_md(path: Path, report: dict[str, Any]) -> None:
    summary = report["summary"]
    lines = [
        "# Examples CI Parity Report",
        "",
        f"- generated_at_utc: `{report['generated_at_utc']}`",
        f"- total_examples: `{summary['total_examples']}`",
        f"- check: `{summary['check_pass']}` pass / `{summary['check_fail']}` fail",
        f"- run: `{summary['run_pass']}` pass / `{summary['run_fail']}` fail / `{summary['run_expected_fail']}` expected-fail / `{summary['run_skipped_non_entry']}` skipped-non-entry",
        f"- status: `{report['status']}`",
        "",
    ]

    if report["allowlist_parse_errors"]:
        lines.extend(["## Allowlist Parse Errors", ""])
        for err in report["allowlist_parse_errors"]:
            lines.append(f"- {err}")
        lines.append("")

    if report["check_failures"]:
        lines.extend(["## Check Failures", ""])
        for row in report["check_failures"]:
            lines.append(f"- `{row['file']}`")
        lines.append("")

    if report["run_failures"]:
        lines.extend(["## Run Failures", ""])
        for row in report["run_failures"]:
            lines.append(f"- `{row['file']}`")
        lines.append("")

    if report["allowlist_unexpected_passes"]:
        lines.extend(["## Unexpected Passes (Allowlisted)", ""])
        for row in report["allowlist_unexpected_passes"]:
            lines.append(f"- `{row['file']}` (expected fail under `{row['checklist_id']}`)")
        lines.append("")

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines).rstrip() + "\n")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run VibeLang example parity checks for CI."
    )
    parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root containing Cargo.toml and examples/",
    )
    parser.add_argument(
        "--examples-dir",
        default="examples",
        help="Examples directory relative to repo root",
    )
    parser.add_argument(
        "--allowlist",
        default="examples/INTENTIONAL_FAILURES_ALLOWLIST.txt",
        help="Expected-failure allowlist file",
    )
    parser.add_argument(
        "--output-json",
        default="reports/examples/parity_ci_latest.json",
        help="Output JSON report path",
    )
    parser.add_argument(
        "--output-md",
        default="reports/examples/parity_ci_latest.md",
        help="Output markdown report path",
    )
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    examples_dir = (repo_root / args.examples_dir).resolve()
    allowlist_path = (repo_root / args.allowlist).resolve()
    output_json = (repo_root / args.output_json).resolve()
    output_md = (repo_root / args.output_md).resolve()

    if not examples_dir.exists():
        raise SystemExit(f"examples directory not found: {examples_dir}")

    build_res = run(["cargo", "build", "-q", "-p", "vibe_cli"], repo_root)
    if build_res["exit_code"] != 0:
        raise SystemExit(
            "failed to build vibe_cli for examples parity run:\n"
            + build_res["stderr"][-4000:]
        )
    vibe_bin = repo_root / "target" / "debug" / "vibe"
    if not vibe_bin.exists():
        raise SystemExit(f"expected CLI binary missing after build: {vibe_bin}")

    allowlist, allowlist_errors = parse_allowlist(allowlist_path)
    files = sorted(path for path in examples_dir.rglob("*.yb") if path.is_file())

    check_failures: list[dict[str, Any]] = []
    run_failures: list[dict[str, Any]] = []
    run_expected_failures: list[dict[str, Any]] = []
    run_skipped_non_entry: list[str] = []
    allowlist_unexpected_passes: list[dict[str, Any]] = []

    check_pass = 0
    run_pass = 0

    for source in files:
        rel = to_rel(repo_root, source)
        check_res = run([str(vibe_bin), "check", rel], repo_root)
        if check_res["exit_code"] == 0:
            check_pass += 1
        else:
            check_failures.append(
                {
                    "file": rel,
                    "exit_code": check_res["exit_code"],
                    "stderr": check_res["stderr"][-2000:],
                    "stdout": check_res["stdout"][-2000:],
                }
            )

        is_allowlisted = rel in allowlist
        if not is_allowlisted and not has_main_entry(source):
            run_skipped_non_entry.append(rel)
            continue

        run_res = run([str(vibe_bin), "run", rel], repo_root)
        if is_allowlisted:
            if run_res["exit_code"] != 0:
                run_expected_failures.append(
                    {
                        "file": rel,
                        "checklist_id": allowlist[rel],
                        "exit_code": run_res["exit_code"],
                    }
                )
            else:
                allowlist_unexpected_passes.append(
                    {"file": rel, "checklist_id": allowlist[rel]}
                )
        else:
            if run_res["exit_code"] == 0:
                run_pass += 1
            else:
                run_failures.append(
                    {
                        "file": rel,
                        "exit_code": run_res["exit_code"],
                        "stderr": run_res["stderr"][-2000:],
                        "stdout": run_res["stdout"][-2000:],
                    }
                )

    report = {
        "format": "vibe-examples-ci-parity-v1",
        "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "allowlist_path": to_rel(repo_root, allowlist_path),
        "allowlist_parse_errors": allowlist_errors,
        "summary": {
            "total_examples": len(files),
            "check_pass": check_pass,
            "check_fail": len(check_failures),
            "run_pass": run_pass,
            "run_fail": len(run_failures),
            "run_expected_fail": len(run_expected_failures),
            "run_skipped_non_entry": len(run_skipped_non_entry),
        },
        "check_failures": check_failures,
        "run_failures": run_failures,
        "run_expected_failures": run_expected_failures,
        "run_skipped_non_entry": run_skipped_non_entry,
        "allowlist_unexpected_passes": allowlist_unexpected_passes,
    }

    has_errors = bool(
        allowlist_errors
        or check_failures
        or run_failures
        or allowlist_unexpected_passes
    )
    report["status"] = "fail" if has_errors else "pass"

    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(report, indent=2) + "\n")
    write_report_md(output_md, report)

    print(f"wrote {output_json}")
    print(f"wrote {output_md}")
    if has_errors:
        raise SystemExit("examples CI parity failed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
