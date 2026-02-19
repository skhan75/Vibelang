#!/usr/bin/env python3
import json
import re
import subprocess
import tempfile
import time
from pathlib import Path


CODE_PATTERN = re.compile(r"^(I\d{4}):")


def run(cmd, cwd):
    completed = subprocess.run(
        cmd,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return completed.returncode, completed.stdout, completed.stderr


def extract_codes(stdout):
    codes = set()
    for line in stdout.splitlines():
        match = CODE_PATTERN.match(line.strip())
        if match:
            codes.add(match.group(1))
    return codes


def write_case(directory: Path, file_name: str, source: str):
    path = directory / file_name
    path.write_text(source.strip() + "\n")
    return path


def compute_quality(case_results):
    tp = 0
    fp = 0
    fn = 0
    for result in case_results:
        expected = result["expected"]
        predicted = result["predicted"]
        tp += len(predicted & expected)
        fp += len(predicted - expected)
        fn += len(expected - predicted)
    precision = tp / (tp + fp) if (tp + fp) else 1.0
    recall = tp / (tp + fn) if (tp + fn) else 1.0
    false_positive_rate = fp / (tp + fp + fn) if (tp + fp + fn) else 0.0
    return {
        "tp": tp,
        "fp": fp,
        "fn": fn,
        "precision": round(precision, 4),
        "recall": round(recall, 4),
        "false_positive_rate": round(false_positive_rate, 4),
    }


def main():
    repo_root = Path(__file__).resolve().parents[2]
    reports_dir = repo_root / "reports" / "phase7"
    reports_dir.mkdir(parents=True, exist_ok=True)
    trend_path = reports_dir / "intent_lint_quality_trend.json"
    markdown_path = reports_dir / "intent_lint_quality_trend.md"

    with tempfile.TemporaryDirectory(prefix="vibe_phase7_intent_quality_") as temp:
        root = Path(temp)
        cases = [
            {
                "name": "missing_intent",
                "file": "missing_intent.yb",
                "source": """
pub missingIntent(x: Int) -> Int {
  @examples {
    missingIntent(1) => 2
  }
  x + 1
}
""",
                "expected": {"I5001"},
            },
            {
                "name": "vague_intent",
                "file": "vague_intent.yb",
                "source": """
pub vagueIntent(x: Int) -> Int {
  @intent "does stuff"
  @examples {
    vagueIntent(1) => 1
  }
  x
}
""",
                "expected": {"I5002"},
            },
            {
                "name": "effect_drift",
                "file": "effect_drift.yb",
                "source": """
pub driftIntent(x: Int) -> Int {
  @intent "log input and return it"
  @examples {
    driftIntent(1) => 1
  }
  @effect alloc
  println("x")
  x
}
""",
                "expected": {"I5003"},
            },
            {
                "name": "good_intent",
                "file": "good_intent.yb",
                "source": """
pub goodIntent(x: Int) -> Int {
  @intent "increment input deterministically by one"
  @examples {
    goodIntent(1) => 2
  }
  @ensure . == x + 1
  x + 1
}
""",
                "expected": set(),
            },
        ]

        case_results = []
        for case in cases:
            case_dir = root / case["name"]
            case_dir.mkdir(parents=True, exist_ok=True)
            write_case(case_dir, case["file"], case["source"])
            exit_code, stdout, stderr = run(
                ["cargo", "run", "-q", "-p", "vibe_cli", "--", "lint", str(case_dir), "--intent"],
                repo_root,
            )
            if exit_code != 0:
                raise RuntimeError(
                    f"lint command failed for {case['name']}:\nstdout:\n{stdout}\nstderr:\n{stderr}"
                )
            predicted = extract_codes(stdout)
            case_results.append(
                {
                    "name": case["name"],
                    "expected": set(case["expected"]),
                    "predicted": predicted,
                }
            )

    quality = compute_quality(case_results)
    entry = {
        "timestamp_epoch_s": int(time.time()),
        "quality": quality,
        "cases": [
            {
                "name": item["name"],
                "expected": sorted(item["expected"]),
                "predicted": sorted(item["predicted"]),
            }
            for item in case_results
        ],
    }

    trend = {"schema": "phase7-intent-lint-quality-v1", "entries": []}
    if trend_path.exists():
        trend = json.loads(trend_path.read_text())
    trend.setdefault("entries", [])
    trend["entries"].append(entry)
    trend["entries"] = trend["entries"][-20:]
    trend_path.write_text(json.dumps(trend, indent=2) + "\n")

    markdown = [
        "# Intent Lint Quality Trend",
        "",
        f"- precision: {quality['precision']}",
        f"- recall: {quality['recall']}",
        f"- false_positive_rate: {quality['false_positive_rate']}",
        "",
        "## Latest Cases",
    ]
    for item in entry["cases"]:
        markdown.append(
            f"- {item['name']}: expected={item['expected']} predicted={item['predicted']}"
        )
    markdown_path.write_text("\n".join(markdown) + "\n")
    print(f"wrote {trend_path}")
    print(f"wrote {markdown_path}")


if __name__ == "__main__":
    main()
