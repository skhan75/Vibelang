#!/usr/bin/env python3
import json
import sys
from pathlib import Path


MIN_PRECISION = 0.75
MIN_RECALL = 0.75
MAX_FALSE_POSITIVE_RATE = 0.25


def fail(msg):
    print(f"intent lint quality validation failed: {msg}")
    sys.exit(1)


def main():
    repo_root = Path(__file__).resolve().parents[2]
    trend_path = repo_root / "reports" / "phase7" / "intent_lint_quality_trend.json"
    if not trend_path.exists():
        fail(f"missing trend artifact: {trend_path}")

    data = json.loads(trend_path.read_text())
    entries = data.get("entries", [])
    if not entries:
        fail("trend artifact has no entries")
    latest = entries[-1]
    quality = latest.get("quality", {})
    precision = float(quality.get("precision", 0.0))
    recall = float(quality.get("recall", 0.0))
    false_positive_rate = float(quality.get("false_positive_rate", 1.0))

    if precision < MIN_PRECISION:
        fail(f"precision {precision:.4f} is below minimum {MIN_PRECISION:.2f}")
    if recall < MIN_RECALL:
        fail(f"recall {recall:.4f} is below minimum {MIN_RECALL:.2f}")
    if false_positive_rate > MAX_FALSE_POSITIVE_RATE:
        fail(
            f"false_positive_rate {false_positive_rate:.4f} exceeds maximum {MAX_FALSE_POSITIVE_RATE:.2f}"
        )
    print("intent lint quality validation passed")


if __name__ == "__main__":
    main()
