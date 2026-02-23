#!/usr/bin/env python3
import json
from pathlib import Path


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    reports_v1 = repo_root / "reports" / "v1"

    required = [
        reports_v1 / "hosted_rc_cycles.json",
        reports_v1 / "phase10_13_exit_audit.json",
        reports_v1 / "ga_freeze_bundle_manifest.json",
        reports_v1 / "ga_readiness_announcement.md",
    ]
    for path in required:
        if not path.exists():
            raise SystemExit(f"missing GA evidence artifact: {path}")

    hosted = json.loads((reports_v1 / "hosted_rc_cycles.json").read_text())
    if hosted.get("format") != "v1-hosted-rc-cycles-v1":
        raise SystemExit("hosted_rc_cycles format mismatch")
    if hosted.get("cycle_count", 0) < 2:
        raise SystemExit("at least two hosted RC cycles are required")

    phase = json.loads((reports_v1 / "phase10_13_exit_audit.json").read_text())
    if phase.get("format") != "v1-phase10-13-exit-audit-v1":
        raise SystemExit("phase10_13_exit_audit format mismatch")
    if not phase.get("artifacts"):
        raise SystemExit("phase10_13_exit_audit must include artifacts")

    freeze = json.loads((reports_v1 / "ga_freeze_bundle_manifest.json").read_text())
    if freeze.get("format") != "v1-ga-freeze-manifest-v1":
        raise SystemExit("ga_freeze_bundle_manifest format mismatch")
    checksums = freeze.get("checksums_sha256", {})
    if not isinstance(checksums, dict) or len(checksums) == 0:
        raise SystemExit("ga_freeze_bundle_manifest must contain checksums")

    print("GA promotion evidence validation passed")


if __name__ == "__main__":
    main()
