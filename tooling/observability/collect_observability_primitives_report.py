#!/usr/bin/env python3
import json
import subprocess
import tempfile
from pathlib import Path


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )


def parse_index_stats(raw_stdout: str) -> dict[str, str]:
    for line in raw_stdout.splitlines():
        if not line.startswith("index stats:"):
            continue
        values: dict[str, str] = {}
        for token in line.replace("index stats:", "").strip().split():
            if "=" not in token:
                continue
            key, value = token.split("=", 1)
            values[key.strip()] = value.strip()
        return values
    return {}


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    report_dir = repo_root / "reports" / "phase13"
    report_dir.mkdir(parents=True, exist_ok=True)

    contracts_doc = repo_root / "docs" / "observability" / "contracts.md"
    if not contracts_doc.exists():
        raise SystemExit(f"missing observability contracts doc: {contracts_doc}")
    contracts_text = contracts_doc.read_text(errors="ignore")
    for marker in (
        "Structured Log Envelope",
        "Metrics Envelope",
        "Trace Span Envelope",
    ):
        if marker not in contracts_text:
            raise SystemExit(f"observability contracts doc missing section: {marker}")

    with tempfile.TemporaryDirectory(prefix="vibe_observability_") as temp_dir:
        fixture = Path(temp_dir) / "obs_probe.yb"
        fixture.write_text(
            """pub main() -> Int {
  @effect alloc
  payload := []
  payload.append(1)
  payload.len()
}
"""
        )
        index_run = run(
            ["cargo", "run", "-q", "-p", "vibe_cli", "--", "index", str(fixture.parent), "--stats"],
            repo_root,
        )
        if index_run.returncode != 0:
            raise SystemExit(
                "observability probe index run failed\n"
                f"stdout:\n{index_run.stdout}\n"
                f"stderr:\n{index_run.stderr}"
            )
        stats = parse_index_stats(index_run.stdout)
        required_metric_fields = ("cache_hits", "cache_misses", "memory_bytes", "memory_ratio")
        missing = [field for field in required_metric_fields if field not in stats]
        if missing:
            raise SystemExit(f"index stats missing required metric fields: {missing}")

    report = {
        "format": "phase13-observability-primitives-v1",
        "contracts_doc": "docs/observability/contracts.md",
        "index_metrics_fields": {
            "cache_hits": stats.get("cache_hits", "0"),
            "cache_misses": stats.get("cache_misses", "0"),
            "memory_bytes": stats.get("memory_bytes", "0"),
            "memory_ratio": stats.get("memory_ratio", "0"),
        },
    }
    (report_dir / "observability_primitives.json").write_text(
        json.dumps(report, indent=2) + "\n"
    )
    (report_dir / "observability_primitives.md").write_text(
        "# Phase 13 Observability Primitives Report\n\n"
        "- status: pass\n"
        "- contracts_doc: `docs/observability/contracts.md`\n"
        "- index_metrics_fields:\n"
        f"  - cache_hits: {report['index_metrics_fields']['cache_hits']}\n"
        f"  - cache_misses: {report['index_metrics_fields']['cache_misses']}\n"
        f"  - memory_bytes: {report['index_metrics_fields']['memory_bytes']}\n"
        f"  - memory_ratio: {report['index_metrics_fields']['memory_ratio']}\n"
    )
    print(f"wrote {report_dir / 'observability_primitives.json'}")


if __name__ == "__main__":
    main()
