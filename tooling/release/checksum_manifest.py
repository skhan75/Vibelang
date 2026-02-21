#!/usr/bin/env python3
"""
Build and compare packaged checksum manifests.

This helper keeps the GitHub workflow YAML small and deterministic while
providing explicit policy checks for packaged-release reproducibility.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
from pathlib import Path
import sys
from typing import Dict, Tuple


def parse_checksums_line(raw: str) -> Tuple[str, str] | None:
    line = raw.strip()
    if not line:
        return None
    parts = line.split()
    if len(parts) < 2:
        raise ValueError(f"invalid checksum line: `{raw.rstrip()}`")
    digest = parts[0].strip().lower()
    name = parts[-1].strip()
    if len(digest) != 64 or any(ch not in "0123456789abcdef" for ch in digest):
        raise ValueError(f"invalid sha256 digest in line: `{raw.rstrip()}`")
    return digest, name


def build_manifest(
    checksums_dir: Path, out_path: Path, ref_name: str, commit_sha: str, version: str
) -> None:
    files = sorted(checksums_dir.glob("checksums-*.txt"))
    if not files:
        raise FileNotFoundError(
            f"no checksum manifests found under `{checksums_dir}` (expected checksums-*.txt)"
        )

    targets: Dict[str, Dict[str, str]] = {}
    for f in files:
        target = f.stem.removeprefix("checksums-")
        entries: Dict[str, str] = {}
        for raw in f.read_text(encoding="utf-8").splitlines():
            parsed = parse_checksums_line(raw)
            if parsed is None:
                continue
            digest, name = parsed
            entries[name] = digest
        targets[target] = dict(sorted(entries.items()))

    manifest = {
        "schema": "v1",
        "generated_at_utc": dt.datetime.now(tz=dt.timezone.utc)
        .replace(microsecond=0)
        .isoformat(),
        "source": {
            "ref_name": ref_name,
            "commit_sha": commit_sha,
            "version": version,
        },
        "targets": dict(sorted(targets.items())),
    }
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def compare_manifests(
    current_path: Path, baseline_path: Path, report_path: Path
) -> Tuple[bool, str]:
    current = json.loads(current_path.read_text(encoding="utf-8"))
    if not baseline_path.exists():
        report = (
            "# Packaged Reproducibility Report\n\n"
            "Baseline manifest not found.\n"
            "Comparison skipped for this run.\n"
        )
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(report, encoding="utf-8")
        return True, "baseline-missing"

    baseline = json.loads(baseline_path.read_text(encoding="utf-8"))

    baseline_targets: Dict[str, Dict[str, str]] = baseline.get("targets", {})
    current_targets: Dict[str, Dict[str, str]] = current.get("targets", {})

    changed = []
    added = []
    removed = []

    for target in sorted(set(baseline_targets) | set(current_targets)):
        b = baseline_targets.get(target, {})
        c = current_targets.get(target, {})
        for name in sorted(set(b) | set(c)):
            b_digest = b.get(name)
            c_digest = c.get(name)
            if b_digest is None:
                added.append((target, name, c_digest))
            elif c_digest is None:
                removed.append((target, name, b_digest))
            elif b_digest != c_digest:
                changed.append((target, name, b_digest, c_digest))

    same_identity = (
        baseline.get("source", {}).get("ref_name") == current.get("source", {}).get("ref_name")
        and baseline.get("source", {}).get("version")
        == current.get("source", {}).get("version")
    )

    status = "pass"
    fail = False
    if same_identity and (changed or added or removed):
        status = "fail"
        fail = True
    elif changed or added or removed:
        status = "drift-allowed"

    lines = [
        "# Packaged Reproducibility Report",
        "",
        f"- Status: `{status}`",
        f"- Baseline: `{baseline_path}`",
        f"- Current: `{current_path}`",
        f"- Same release identity (ref+version): `{str(same_identity).lower()}`",
        "",
    ]

    if changed:
        lines.extend(["## Changed Checksums", ""])
        for target, name, old, new in changed:
            lines.append(f"- `{target}` `{name}`: `{old}` -> `{new}`")
        lines.append("")
    if added:
        lines.extend(["## Added Entries", ""])
        for target, name, digest in added:
            lines.append(f"- `{target}` `{name}`: `{digest}`")
        lines.append("")
    if removed:
        lines.extend(["## Removed Entries", ""])
        for target, name, digest in removed:
            lines.append(f"- `{target}` `{name}`: `{digest}`")
        lines.append("")
    if not changed and not added and not removed:
        lines.extend(["No checksum differences detected.", ""])

    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return (not fail), status


def main() -> int:
    parser = argparse.ArgumentParser(description="Build/compare packaged checksum manifests")
    sub = parser.add_subparsers(dest="command", required=True)

    build = sub.add_parser("build", help="build consolidated checksum manifest")
    build.add_argument("--checksums-dir", required=True, type=Path)
    build.add_argument("--out", required=True, type=Path)
    build.add_argument("--ref-name", required=True)
    build.add_argument("--commit-sha", required=True)
    build.add_argument("--version", required=True)

    compare = sub.add_parser("compare", help="compare current manifest against baseline")
    compare.add_argument("--current", required=True, type=Path)
    compare.add_argument("--baseline", required=True, type=Path)
    compare.add_argument("--report", required=True, type=Path)

    args = parser.parse_args()

    if args.command == "build":
        build_manifest(
            checksums_dir=args.checksums_dir,
            out_path=args.out,
            ref_name=args.ref_name,
            commit_sha=args.commit_sha,
            version=args.version,
        )
        return 0

    if args.command == "compare":
        ok, status = compare_manifests(
            current_path=args.current,
            baseline_path=args.baseline,
            report_path=args.report,
        )
        if not ok:
            print(
                f"checksum manifest comparison failed: drift detected for same release identity (status={status})",
                file=sys.stderr,
            )
            return 1
        print(f"checksum manifest comparison status: {status}")
        return 0

    parser.print_help()
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
