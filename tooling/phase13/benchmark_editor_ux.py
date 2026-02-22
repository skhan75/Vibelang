#!/usr/bin/env python3
import argparse
import json
import re
import subprocess
import tempfile
import time
from pathlib import Path

from jsonrpc_client import JsonRpcClient


def fail(message: str) -> None:
    print(f"phase13 benchmark failed: {message}")
    raise SystemExit(1)


def run(cmd: list[str], cwd: Path) -> tuple[int, str, str]:
    completed = subprocess.run(
        cmd,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return completed.returncode, completed.stdout, completed.stderr


def parse_index_stats(stdout: str) -> dict[str, int]:
    line = ""
    for candidate in stdout.splitlines():
        if candidate.startswith("index stats:"):
            line = candidate
            break
    if not line:
        fail("index --stats output missing `index stats:` line")
    stats: dict[str, int] = {}
    for token in line.replace("index stats:", "").split():
        if "=" not in token:
            continue
        key, value = token.split("=", 1)
        value = value.strip()
        if re.fullmatch(r"[0-9]+", value):
            stats[key] = int(value)
    return stats


def file_uri(path: Path) -> str:
    return f"file://{path.as_posix()}"


def collect_lsp_metrics(repo_root: Path, fixture: Path) -> dict[str, int]:
    source = fixture.read_text()
    with tempfile.TemporaryDirectory(prefix="vibe_phase13_bench_") as temp_dir:
        index_root = Path(temp_dir) / "index"
        command = [
            "cargo",
            "run",
            "-q",
            "-p",
            "vibe_cli",
            "--",
            "lsp",
            "--transport",
            "jsonrpc",
            "--index-root",
            str(index_root),
        ]
        client = JsonRpcClient(command, cwd=str(repo_root))

        _, initialize_ms = client.request("initialize", {})
        uri = file_uri(fixture)

        start_open = time.perf_counter()
        client.notify(
            "textDocument/didOpen",
            {
                "textDocument": {
                    "uri": uri,
                    "version": 1,
                    "languageId": "vibelang",
                    "text": source,
                }
            },
        )
        client.wait_for_notification("textDocument/publishDiagnostics")
        did_open_ms = int((time.perf_counter() - start_open) * 1000)

        _, completion_ms = client.request(
            "textDocument/completion",
            {"textDocument": {"uri": uri}, "position": {"line": 0, "character": 3}},
        )
        _, formatting_ms = client.request(
            "textDocument/formatting",
            {"textDocument": {"uri": uri}, "options": {"tabSize": 2, "insertSpaces": True}},
        )
        _, shutdown_ms = client.request("shutdown", {})
        client.notify("exit", {})
        return_code, _, stderr = client.close()
        if return_code not in (0, None):
            fail(f"jsonrpc lsp exited with code {return_code}: {stderr}")

    return {
        "lsp_initialize_ms": initialize_ms,
        "lsp_did_open_ms": did_open_ms,
        "lsp_completion_ms": completion_ms,
        "lsp_formatting_ms": formatting_ms,
        "lsp_shutdown_ms": shutdown_ms,
    }


def enforce_budgets(repo_root: Path, metrics: dict[str, int], enforce: bool) -> None:
    budgets_path = repo_root / "reports" / "v1" / "quality_budgets.json"
    if not budgets_path.exists():
        return
    budgets = json.loads(budgets_path.read_text())
    editor = budgets.get("editor_ux_benchmarks")
    if not isinstance(editor, dict):
        return

    checks = [
        ("max_lsp_initialize_ms", "lsp_initialize_ms"),
        ("max_lsp_did_open_ms", "lsp_did_open_ms"),
        ("max_lsp_completion_ms", "lsp_completion_ms"),
        ("max_lsp_formatting_ms", "lsp_formatting_ms"),
        ("max_lsp_shutdown_ms", "lsp_shutdown_ms"),
        ("max_index_cold_ms", "index_cold_ms"),
        ("max_index_incremental_ms", "index_incremental_ms"),
        ("max_index_memory_bytes", "index_memory_bytes"),
    ]
    violations = []
    for budget_key, metric_key in checks:
        budget = editor.get(budget_key)
        metric = metrics.get(metric_key)
        if not isinstance(budget, int) or budget <= 0:
            continue
        if not isinstance(metric, int):
            continue
        if metric > budget:
            violations.append((metric_key, metric, budget_key, budget))
    if violations and enforce:
        details = "; ".join(
            f"{metric_key}={metric} exceeds {budget_key}={budget}"
            for metric_key, metric, budget_key, budget in violations
        )
        fail(details)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--enforce", action="store_true", help="enforce budget thresholds")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[2]
    fixture = repo_root / "compiler" / "tests" / "fixtures" / "phase7" / "basic" / "syntax" / "syntax__literals_and_comments.yb"
    if not fixture.exists():
        fail(f"benchmark fixture missing: {fixture}")

    status, stdout, stderr = run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "vibe_cli",
            "--",
            "index",
            "compiler/tests/fixtures",
            "--rebuild",
            "--stats",
        ],
        repo_root,
    )
    if status != 0:
        fail(f"index benchmark command failed:\nstdout:\n{stdout}\nstderr:\n{stderr}")

    index_stats = parse_index_stats(stdout)
    lsp_metrics = collect_lsp_metrics(repo_root, fixture)
    metrics = {
        "generated_at_epoch_s": int(time.time()),
        "index_cold_ms": int(index_stats.get("cold_ms", 0)),
        "index_incremental_ms": int(index_stats.get("incremental_ms", 0)),
        "index_memory_bytes": int(index_stats.get("memory_bytes", 0)),
        **lsp_metrics,
    }

    reports_root = repo_root / "reports" / "phase13"
    reports_root.mkdir(parents=True, exist_ok=True)
    metrics_path = reports_root / "editor_ux_metrics.json"
    metrics_path.write_text(json.dumps(metrics, indent=2) + "\n")

    enforce_budgets(repo_root, metrics, enforce=args.enforce)
    print(f"wrote {metrics_path}")
    print(json.dumps(metrics, indent=2))


if __name__ == "__main__":
    main()

