#!/usr/bin/env python3
import json
from pathlib import Path


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    metrics_path = repo_root / "reports" / "phase14" / "pilot_metrics.json"
    if not metrics_path.exists():
        raise SystemExit(f"missing pilot metrics report: {metrics_path}")

    report = json.loads(metrics_path.read_text())
    if report.get("format") != "phase14-pilot-metrics-v1":
        raise SystemExit("pilot metrics format mismatch")
    if report.get("benchmark_method") != "direct_vibe_binary":
        raise SystemExit("pilot metrics benchmark_method must be `direct_vibe_binary`")

    apps = report.get("apps", {})
    if not isinstance(apps, dict) or len(apps) < 2:
        raise SystemExit("pilot metrics must include at least two apps")
    for name, result in apps.items():
        if result.get("status") != "pass":
            raise SystemExit(f"pilot app `{name}` did not pass probe checks")

    pain_points = report.get("migration_pain_points", [])
    if not isinstance(pain_points, list) or len(pain_points) == 0:
        raise SystemExit("pilot metrics must include at least one migration pain point")

    print("pilot metrics validation passed")


if __name__ == "__main__":
    main()
