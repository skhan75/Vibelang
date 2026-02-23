#!/usr/bin/env python3
import json
import resource
import subprocess
import time
from pathlib import Path


def run(cmd: list[str], cwd: Path) -> dict[str, object]:
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    start = time.time()
    completed = subprocess.run(
        cmd,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    elapsed_ms = int((time.time() - start) * 1000)
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    cpu_s = (after.ru_utime + after.ru_stime) - (before.ru_utime + before.ru_stime)
    return {
        "cmd": cmd,
        "exit": completed.returncode,
        "elapsed_ms": elapsed_ms,
        "cpu_ms": int(cpu_s * 1000),
        "stdout": completed.stdout,
        "stderr": completed.stderr,
    }


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
    reports_root = repo_root / "reports" / "v1"
    reports_root.mkdir(parents=True, exist_ok=True)

    build = run(["cargo", "build", "--release", "--locked", "-p", "vibe_cli"], repo_root)
    if build["exit"] != 0:
        raise SystemExit(
            "release benchmark collection failed during build\n"
            f"stdout:\n{build['stdout']}\n"
            f"stderr:\n{build['stderr']}"
        )

    vibe_bin = repo_root / "target" / "release" / "vibe"
    if not vibe_bin.exists():
        raise SystemExit(f"missing vibe binary: {vibe_bin}")

    hello = repo_root / "compiler" / "tests" / "fixtures" / "build" / "hello_world.vibe"
    check_out = run([str(vibe_bin), "check", str(hello)], repo_root)
    run_out = run([str(vibe_bin), "run", str(hello)], repo_root)
    index_out = run(
        [
            str(vibe_bin),
            "index",
            str(repo_root / "compiler" / "tests" / "fixtures" / "build"),
            "--stats",
        ],
        repo_root,
    )

    for command_out in (check_out, run_out, index_out):
        if command_out["exit"] != 0:
            raise SystemExit(
                "release benchmark collection command failed\n"
                f"cmd={command_out['cmd']}\n"
                f"stdout:\n{command_out['stdout']}\n"
                f"stderr:\n{command_out['stderr']}"
            )

    index_stats = parse_index_stats(str(index_out["stdout"]))
    memory_bytes = int(index_stats.get("memory_bytes", "0") or "0")
    latency_ms = int(run_out["elapsed_ms"])
    compile_ms = int(check_out["elapsed_ms"])
    cpu_ms = int(check_out["cpu_ms"]) + int(run_out["cpu_ms"]) + int(index_out["cpu_ms"])

    metrics = {
        "generated_at_epoch_s": int(time.time()),
        "profile": "release",
        "benchmark_method": "vibe_release_binary_smoke",
        "vibe_binary": "target/release/vibe",
        "compile_latency_ms": compile_ms,
        "runtime_latency_ms": latency_ms,
        "cpu_ms": cpu_ms,
        "memory_bytes": memory_bytes,
    }

    (reports_root / "release_benchmarks.json").write_text(json.dumps(metrics, indent=2) + "\n")
    summary = (
        "# V1 Release Benchmark Snapshot\n\n"
        f"- benchmark_method: {metrics['benchmark_method']}\n"
        f"- profile: {metrics['profile']}\n"
        f"- compile_latency_ms: {metrics['compile_latency_ms']}\n"
        f"- runtime_latency_ms: {metrics['runtime_latency_ms']}\n"
        f"- cpu_ms: {metrics['cpu_ms']}\n"
        f"- memory_bytes: {metrics['memory_bytes']}\n"
    )
    (reports_root / "release_benchmarks.md").write_text(summary)
    print(f"wrote {reports_root / 'release_benchmarks.json'}")


if __name__ == "__main__":
    main()
