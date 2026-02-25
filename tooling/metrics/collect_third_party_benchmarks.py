#!/usr/bin/env python3
import argparse
import json
import math
import os
import platform
import re
import shlex
import shutil
import statistics
import subprocess
import time
from pathlib import Path
from typing import Any


def fail(message: str) -> None:
    raise SystemExit(f"third-party benchmark collection failed: {message}")


def run(
    cmd: list[str],
    cwd: Path,
    env: dict[str, str] | None = None,
) -> dict[str, Any]:
    start = time.perf_counter()
    completed = subprocess.run(
        cmd,
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    elapsed_ms = int((time.perf_counter() - start) * 1000)
    return {
        "cmd": cmd,
        "exit_code": completed.returncode,
        "elapsed_ms": elapsed_ms,
        "stdout": completed.stdout,
        "stderr": completed.stderr,
    }


def run_quick(cmd: list[str], cwd: Path) -> str:
    try:
        completed = subprocess.run(
            cmd,
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
    except FileNotFoundError:
        return "unavailable"
    if completed.returncode != 0:
        out = completed.stdout.strip()
        err = completed.stderr.strip()
        return out or err or "unavailable"
    lines = [line.strip() for line in completed.stdout.splitlines() if line.strip()]
    if lines:
        return lines[0]
    err_lines = [line.strip() for line in completed.stderr.splitlines() if line.strip()]
    return err_lines[0] if err_lines else "unavailable"


def geomean(values: list[float]) -> float:
    if not values:
        return 0.0
    if any(value <= 0.0 for value in values):
        return 0.0
    return math.exp(sum(math.log(value) for value in values) / len(values))


def summarize(values: list[float]) -> dict[str, float]:
    if not values:
        return {
            "mean": 0.0,
            "median": 0.0,
            "min": 0.0,
            "max": 0.0,
            "stddev": 0.0,
        }
    return {
        "mean": float(statistics.fmean(values)),
        "median": float(statistics.median(values)),
        "min": float(min(values)),
        "max": float(max(values)),
        "stddev": float(statistics.pstdev(values)) if len(values) > 1 else 0.0,
    }


def detect_cpu_model() -> str:
    cpuinfo = Path("/proc/cpuinfo")
    if not cpuinfo.exists():
        return "unavailable"
    for line in cpuinfo.read_text(errors="ignore").splitlines():
        if "model name" in line:
            _, _, value = line.partition(":")
            return value.strip()
    return "unavailable"


def detect_mem_total_kb() -> int:
    meminfo = Path("/proc/meminfo")
    if not meminfo.exists():
        return 0
    for line in meminfo.read_text(errors="ignore").splitlines():
        if not line.startswith("MemTotal:"):
            continue
        parts = line.split()
        if len(parts) >= 2 and parts[1].isdigit():
            return int(parts[1])
    return 0


def detect_git_revision(repo_root: Path) -> str:
    result = run(["git", "rev-parse", "HEAD"], repo_root)
    if int(result["exit_code"]) != 0:
        return "unavailable"
    return str(result["stdout"]).strip() or "unavailable"


def collect_environment(repo_root: Path) -> dict[str, Any]:
    logical_cpus = os.cpu_count()
    return {
        "hostname": platform.node(),
        "platform": platform.platform(),
        "kernel_release": platform.release(),
        "architecture": platform.machine(),
        "cpu_model": detect_cpu_model(),
        "logical_cpus": logical_cpus if isinstance(logical_cpus, int) else 0,
        "memory_total_kb": detect_mem_total_kb(),
        "source_revisions": {
            "repo_git_revision": detect_git_revision(repo_root),
        },
        "tool_versions": {
            "dotnet": run_quick(["dotnet", "--version"], repo_root),
            "hyperfine": run_quick(["hyperfine", "--version"], repo_root),
            "docker": run_quick(["docker", "--version"], repo_root),
            "vibe": run_quick(["vibe", "--version"], repo_root),
        },
    }


def apply_path_hints() -> None:
    hints = [
        Path.home() / ".dotnet",
        Path.home() / ".local" / "bin",
        Path.home() / ".cargo" / "bin",
    ]
    existing = [segment for segment in os.environ.get("PATH", "").split(":") if segment]
    for hint in hints:
        hint_str = str(hint)
        if hint.exists() and hint_str not in existing:
            existing.insert(0, hint_str)
    os.environ["PATH"] = ":".join(existing)


def _check_binary(binary: str) -> dict[str, Any]:
    path = shutil.which(binary)
    return {
        "binary": binary,
        "found": path is not None,
        "path": path or "",
    }


def preflight_checks(
    repo_root: Path,
    languages: list[str],
    no_docker: bool,
) -> dict[str, Any]:
    core_bins = ["git", "dotnet", "hyperfine", "vibe"]
    core_checks = [_check_binary(binary) for binary in core_bins]

    language_binaries: dict[str, list[str]] = {
        "vibelang": ["vibe"],
        "c": ["gcc", "g++", "clang", "clang++"],
        "cpp": ["g++", "clang++"],
        "rust": ["rustc", "cargo"],
        "go": ["go"],
        "zig": ["zig"],
        "swift": ["swift", "swiftc"],
        "kotlin": ["java", "kotlinc"],
        "elixir": ["elixir", "mix"],
        "python": ["python3", "pypy3", "pyston3"],
        "typescript": ["deno"],
    }

    language_checks: dict[str, list[dict[str, Any]]] = {}
    for language in languages:
        binaries = language_binaries.get(language, [])
        language_checks[language] = [_check_binary(binary) for binary in binaries]

    docker_check: dict[str, Any] = {"required": not no_docker}
    if not no_docker:
        docker_check.update(_check_binary("docker"))
        if bool(docker_check.get("found")):
            try:
                info = subprocess.run(
                    ["docker", "info", "--format", "{{.ServerVersion}}"],
                    cwd=repo_root,
                    capture_output=True,
                    text=True,
                    timeout=10,
                )
                docker_check["daemon_exit_code"] = int(info.returncode)
                docker_check["daemon_stdout"] = str(info.stdout).strip()
                docker_check["daemon_stderr"] = str(info.stderr).strip()
                docker_check["daemon_ok"] = int(info.returncode) == 0
            except subprocess.TimeoutExpired as exc:
                docker_check["daemon_exit_code"] = 124
                docker_check["daemon_stdout"] = str(exc.stdout or "").strip()
                docker_check["daemon_stderr"] = "docker info timed out after 10s"
                docker_check["daemon_ok"] = False
        else:
            docker_check["daemon_exit_code"] = 1
            docker_check["daemon_stdout"] = ""
            docker_check["daemon_stderr"] = "docker binary not found"
            docker_check["daemon_ok"] = False
    else:
        docker_check.update(
            {
                "binary": "docker",
                "found": bool(shutil.which("docker")),
                "path": shutil.which("docker") or "",
                "daemon_ok": False,
                "daemon_stdout": "",
                "daemon_stderr": "docker check skipped by --no-docker",
            }
        )

    errors: list[str] = []
    for row in core_checks:
        if not bool(row["found"]):
            errors.append(f"missing required binary: {row['binary']}")

    if not no_docker and not bool(docker_check.get("daemon_ok")):
        errors.append(
            "docker daemon unavailable; enable Docker and verify `docker info` succeeds"
        )

    if no_docker:
        for language, checks in language_checks.items():
            for row in checks:
                if not bool(row["found"]):
                    errors.append(
                        f"missing local toolchain binary `{row['binary']}` for language `{language}`"
                    )

    status = "ok" if not errors else "failed"
    return {
        "status": status,
        "mode": "no-docker" if no_docker else "docker",
        "core_checks": core_checks,
        "docker_check": docker_check,
        "language_checks": language_checks,
        "errors": errors,
    }


def ensure_plbci_checkout(
    repo_root: Path,
    checkout_dir: Path,
    repo_url: str,
    ref: str,
) -> Path:
    if not checkout_dir.exists():
        checkout_dir.parent.mkdir(parents=True, exist_ok=True)
        clone = run(["git", "clone", repo_url, str(checkout_dir)], repo_root)
        if int(clone["exit_code"]) != 0:
            fail(
                "failed to clone PLB-CI repository\n"
                f"stdout:\n{clone['stdout']}\n"
                f"stderr:\n{clone['stderr']}"
            )
    fetch = run(["git", "fetch", "--all", "--tags", "--prune"], checkout_dir)
    if int(fetch["exit_code"]) != 0:
        fail(
            "failed to fetch PLB-CI repository updates\n"
            f"stdout:\n{fetch['stdout']}\n"
            f"stderr:\n{fetch['stderr']}"
        )
    checkout = run(["git", "checkout", ref], checkout_dir)
    if int(checkout["exit_code"]) != 0:
        fail(
            f"failed to checkout PLB-CI ref `{ref}`\n"
            f"stdout:\n{checkout['stdout']}\n"
            f"stderr:\n{checkout['stderr']}"
        )
    return checkout_dir


def stage_config(
    repo_root: Path,
    matrix: dict[str, Any],
    profile: str,
    plbci_dir: Path,
    adapter_root: Path,
    bench_template: Path,
    stage_dir: Path,
) -> tuple[Path, Path]:
    if profile not in matrix.get("profiles", {}):
        fail(f"profile `{profile}` missing from matrix config")
    profile_cfg = matrix["profiles"][profile]

    if stage_dir.exists():
        shutil.rmtree(stage_dir)
    stage_dir.mkdir(parents=True, exist_ok=True)

    template = bench_template.read_text()
    bench_yaml = template
    problems = matrix.get("problems", [])
    if not isinstance(problems, list) or not problems:
        fail("matrix problems list is empty; cannot map repeat placeholders")
    for raw_problem in problems:
        problem = str(raw_problem).strip()
        normalized = problem.replace("-", "_")
        profile_key = f"{normalized}_repeat"
        if profile_key not in profile_cfg:
            fail(
                f"profile `{profile}` missing repeat key `{profile_key}` "
                f"for problem `{problem}`"
            )
        placeholder = f"__{normalized.upper()}_REPEAT__"
        bench_yaml = bench_yaml.replace(placeholder, str(int(profile_cfg[profile_key])))
    unresolved = sorted(set(re.findall(r"__[A-Z0-9_]+_REPEAT__", bench_yaml)))
    if unresolved:
        fail(
            "unresolved repeat placeholders after matrix expansion: "
            + ", ".join(unresolved)
        )
    (stage_dir / "bench.yaml").write_text(bench_yaml)

    for lang in matrix.get("languages", []):
        bench_file = str(lang["bench_file"])
        source = str(lang.get("source", "plbci"))
        if source == "adapter":
            src = adapter_root / bench_file
        else:
            src = plbci_dir / "bench" / bench_file
        if not src.exists():
            fail(f"bench file missing for language `{lang['id']}`: {src}")
        shutil.copy2(src, stage_dir / bench_file)

    adapter_algo_root = adapter_root / "algorithm"
    target_algo_root = plbci_dir / "bench" / "algorithm"
    if not adapter_algo_root.exists():
        fail(f"adapter algorithm root missing: {adapter_algo_root}")
    for path in adapter_algo_root.rglob("*"):
        if not path.is_file():
            continue
        rel = path.relative_to(adapter_algo_root)
        dst = target_algo_root / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, dst)

    return stage_dir / "bench.yaml", target_algo_root


def plbci_cmd(
    bench_tool_cwd: Path,
    bench_yaml: Path,
    algorithm_root: Path,
    build_output: Path,
    task: str,
    languages: list[str],
    problems: list[str],
    no_docker: bool,
    force_rebuild: bool = False,
    ignore_missing: bool = False,
) -> list[str]:
    cmd = [
        "dotnet",
        "run",
        "-c",
        "Release",
        "--project",
        "tool",
        "--",
        "--config",
        str(bench_yaml),
        "--algorithm",
        str(algorithm_root),
        "--build-output",
        str(build_output),
        "--task",
        task,
        "--langs",
        *languages,
        "--problems",
        *problems,
    ]
    if no_docker:
        cmd.append("--no-docker")
    if force_rebuild and task == "build":
        cmd.append("--force-rebuild")
    if ignore_missing and task in {"test", "bench"}:
        cmd.append("--ignore-missing")
    return cmd


def run_plbci_suite(
    bench_tool_cwd: Path,
    bench_yaml: Path,
    algorithm_root: Path,
    build_output: Path,
    languages: list[str],
    problems: list[str],
    no_docker: bool,
) -> dict[str, Any]:
    build_output.mkdir(parents=True, exist_ok=True)
    logs: dict[str, Any] = {}

    build_cmd = plbci_cmd(
        bench_tool_cwd=bench_tool_cwd,
        bench_yaml=bench_yaml,
        algorithm_root=algorithm_root,
        build_output=build_output,
        task="build",
        languages=languages,
        problems=problems,
        no_docker=no_docker,
        force_rebuild=True,
    )
    logs["build"] = run(build_cmd, bench_tool_cwd)

    test_cmd = plbci_cmd(
        bench_tool_cwd=bench_tool_cwd,
        bench_yaml=bench_yaml,
        algorithm_root=algorithm_root,
        build_output=build_output,
        task="test",
        languages=languages,
        problems=problems,
        no_docker=no_docker,
        ignore_missing=True,
    )
    logs["test"] = run(test_cmd, bench_tool_cwd)

    bench_cmd = plbci_cmd(
        bench_tool_cwd=bench_tool_cwd,
        bench_yaml=bench_yaml,
        algorithm_root=algorithm_root,
        build_output=build_output,
        task="bench",
        languages=languages,
        problems=problems,
        no_docker=no_docker,
        ignore_missing=True,
    )
    logs["bench"] = run(bench_cmd, bench_tool_cwd)
    return logs


def parse_plbci_runtime(build_output: Path, languages: list[str]) -> dict[str, Any]:
    result_root = build_output / "_results"
    runtime: dict[str, Any] = {
        "tool": "plbci",
        "result_root": str(result_root),
        "languages": {},
        "comparisons": {},
    }

    per_problem_by_lang: dict[str, dict[str, dict[str, float]]] = {}
    for lang in languages:
        lang_dir = result_root / lang
        if not lang_dir.exists():
            runtime["languages"][lang] = {
                "status": "unavailable",
                "reason": "no result directory produced",
                "records": [],
                "problem_metrics": {},
            }
            continue
        records: list[dict[str, Any]] = []
        for json_path in sorted(lang_dir.glob("*.json")):
            payload = json.loads(json_path.read_text())
            if str(payload.get("lang", "")).lower() != str(lang).lower():
                continue
            problem = str(payload.get("test", "unknown"))
            record = {
                "problem": problem,
                "input": str(payload.get("input", "")),
                "time_ms": float(payload.get("timeMS", 0.0)),
                "time_stddev_ms": float(payload.get("timeStdDevMS", 0.0)),
                "mem_bytes": float(payload.get("memBytes", 0.0)),
                "cpu_ms": float(payload.get("cpuTimeMS", 0.0)),
                "compiler": str(payload.get("compiler", "unknown")),
                "compiler_version": str(payload.get("compilerVersion", "unknown")),
                "source": str(payload.get("code", "")),
                "raw_json": str(json_path),
            }
            records.append(record)
        problem_metrics: dict[str, dict[str, float]] = {}
        grouped: dict[str, list[dict[str, Any]]] = {}
        for row in records:
            grouped.setdefault(str(row["problem"]), []).append(row)
        for problem, rows in grouped.items():
            times = [float(row["time_ms"]) for row in rows if float(row["time_ms"]) > 0.0]
            mems = [float(row["mem_bytes"]) for row in rows if float(row["mem_bytes"]) > 0.0]
            cpus = [float(row["cpu_ms"]) for row in rows if float(row["cpu_ms"]) > 0.0]
            metric = {
                "samples": float(len(times)),
                "mean_time_ms": summarize(times)["mean"],
                "median_time_ms": summarize(times)["median"],
                "min_time_ms": summarize(times)["min"],
                "max_time_ms": summarize(times)["max"],
                "time_stddev_ms": summarize(times)["stddev"],
                "mean_mem_bytes": summarize(mems)["mean"],
                "mean_cpu_ms": summarize(cpus)["mean"],
            }
            problem_metrics[problem] = metric
            per_problem_by_lang.setdefault(problem, {})[lang] = metric
        status = "ok" if records else "unavailable"
        runtime["languages"][lang] = {
            "status": status,
            "records": records,
            "problem_metrics": problem_metrics,
        }

    vibe_metrics = runtime["languages"].get("vibelang", {}).get("problem_metrics", {})
    for lang in languages:
        if lang == "vibelang":
            continue
        baseline_metrics = runtime["languages"].get(lang, {}).get("problem_metrics", {})
        shared = sorted(set(vibe_metrics.keys()).intersection(set(baseline_metrics.keys())))
        ratios: list[float] = []
        per_problem_ratios: dict[str, float] = {}
        for problem in shared:
            vibe_time = float(vibe_metrics[problem]["mean_time_ms"])
            baseline_time = float(baseline_metrics[problem]["mean_time_ms"])
            if vibe_time <= 0.0 or baseline_time <= 0.0:
                continue
            ratio = vibe_time / baseline_time
            ratios.append(ratio)
            per_problem_ratios[problem] = ratio
        runtime["comparisons"][lang] = {
            "shared_problems": shared,
            "geomean_vibelang_ratio": geomean(ratios),
            "per_problem_ratio": per_problem_ratios,
        }
    runtime["per_problem_table"] = per_problem_by_lang
    return runtime


def choose_binary_size(build_dir: Path) -> int:
    if not build_dir.exists():
        return 0
    candidate_sizes: list[int] = []
    for path in build_dir.rglob("*"):
        if not path.is_file():
            continue
        if path.name.startswith("__") and path.suffix == ".json":
            continue
        try:
            candidate_sizes.append(path.stat().st_size)
        except OSError:
            continue
    return max(candidate_sizes) if candidate_sizes else 0


def parse_hyperfine_result(json_path: Path) -> dict[str, float]:
    payload = json.loads(json_path.read_text())
    results = payload.get("results", [])
    if not isinstance(results, list) or not results:
        return {
            "mean_ms": 0.0,
            "stddev_ms": 0.0,
            "min_ms": 0.0,
            "max_ms": 0.0,
        }
    first = results[0]
    mean_s = float(first.get("mean", 0.0))
    std_s = float(first.get("stddev", 0.0))
    min_s = float(first.get("min", 0.0))
    max_s = float(first.get("max", 0.0))
    return {
        "mean_ms": mean_s * 1000.0,
        "stddev_ms": std_s * 1000.0,
        "min_ms": min_s * 1000.0,
        "max_ms": max_s * 1000.0,
    }


def run_hyperfine(
    cmd_string: str,
    export_json: Path,
    runs: int,
    warmup: int,
    cwd: Path,
) -> dict[str, Any]:
    export_json.parent.mkdir(parents=True, exist_ok=True)
    cmd = [
        "hyperfine",
        "--runs",
        str(runs),
        "--warmup",
        str(warmup),
        "--export-json",
        str(export_json),
        cmd_string,
    ]
    return run(cmd, cwd)


def collect_compile_lanes(
    bench_tool_cwd: Path,
    bench_yaml: Path,
    algorithm_root: Path,
    compile_build_root: Path,
    profile_cfg: dict[str, Any],
    languages: list[str],
    problems: list[str],
    no_docker: bool,
) -> dict[str, Any]:
    compile_section: dict[str, Any] = {
        "tool": "hyperfine",
        "languages": {},
        "comparisons": {},
    }
    if shutil.which("hyperfine") is None:
        for lang in languages:
            compile_section["languages"][lang] = {
                "status": "unavailable",
                "reason": "hyperfine not installed",
            }
        return compile_section

    runs = int(profile_cfg.get("hyperfine_runs", 4))
    warmup = int(profile_cfg.get("hyperfine_warmup", 1))
    probe_problem = "helloworld" if "helloworld" in problems else problems[0]

    for lang in languages:
        lang_build_output = compile_build_root / lang
        lang_build_output.mkdir(parents=True, exist_ok=True)

        force_cmd = plbci_cmd(
            bench_tool_cwd=bench_tool_cwd,
            bench_yaml=bench_yaml,
            algorithm_root=algorithm_root,
            build_output=lang_build_output,
            task="build",
            languages=[lang],
            problems=[probe_problem],
            no_docker=no_docker,
            force_rebuild=True,
        )
        incremental_cmd = plbci_cmd(
            bench_tool_cwd=bench_tool_cwd,
            bench_yaml=bench_yaml,
            algorithm_root=algorithm_root,
            build_output=lang_build_output,
            task="build",
            languages=[lang],
            problems=[probe_problem],
            no_docker=no_docker,
            force_rebuild=False,
        )
        warm = run(force_cmd, bench_tool_cwd)
        if int(warm["exit_code"]) != 0:
            compile_section["languages"][lang] = {
                "status": "unavailable",
                "reason": "cold build command failed",
                "cold_command": force_cmd,
                "stderr": str(warm["stderr"])[-4000:],
            }
            continue

        smoke = run(incremental_cmd, bench_tool_cwd)
        if int(smoke["exit_code"]) != 0:
            compile_section["languages"][lang] = {
                "status": "unavailable",
                "reason": "incremental build command failed",
                "incremental_command": incremental_cmd,
                "stderr": str(smoke["stderr"])[-4000:],
            }
            continue

        cold_json = lang_build_output / "cold_hyperfine.json"
        inc_json = lang_build_output / "incremental_hyperfine.json"
        cold_cmd_string = shlex.join(force_cmd)
        inc_cmd_string = shlex.join(incremental_cmd)
        cold_run = run_hyperfine(
            cmd_string=cold_cmd_string,
            export_json=cold_json,
            runs=runs,
            warmup=0,
            cwd=bench_tool_cwd,
        )
        inc_run = run_hyperfine(
            cmd_string=inc_cmd_string,
            export_json=inc_json,
            runs=runs,
            warmup=warmup,
            cwd=bench_tool_cwd,
        )
        if int(cold_run["exit_code"]) != 0 or int(inc_run["exit_code"]) != 0:
            compile_section["languages"][lang] = {
                "status": "failed",
                "reason": "hyperfine run failed",
                "cold_exit_code": int(cold_run["exit_code"]),
                "incremental_exit_code": int(inc_run["exit_code"]),
                "cold_stderr": str(cold_run["stderr"])[-4000:],
                "incremental_stderr": str(inc_run["stderr"])[-4000:],
            }
            continue

        binary_size = choose_binary_size(lang_build_output)
        compile_section["languages"][lang] = {
            "status": "ok",
            "probe_problem": probe_problem,
            "cold": parse_hyperfine_result(cold_json),
            "incremental": parse_hyperfine_result(inc_json),
            "binary_size_bytes": binary_size,
            "cold_command": force_cmd,
            "incremental_command": incremental_cmd,
            "cold_hyperfine_json": str(cold_json),
            "incremental_hyperfine_json": str(inc_json),
        }

    vibe = compile_section["languages"].get("vibelang", {})
    vibe_cold = float(vibe.get("cold", {}).get("mean_ms", 0.0)) if isinstance(vibe, dict) else 0.0
    for lang in languages:
        if lang == "vibelang":
            continue
        baseline = compile_section["languages"].get(lang, {})
        base_cold = (
            float(baseline.get("cold", {}).get("mean_ms", 0.0))
            if isinstance(baseline, dict)
            else 0.0
        )
        ratio = (vibe_cold / base_cold) if vibe_cold > 0.0 and base_cold > 0.0 else 0.0
        compile_section["comparisons"][lang] = {
            "vibelang_cold_ratio": ratio,
        }
    return compile_section


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n")


def build_category_rollup(
    runtime_section: dict[str, Any],
    compile_section: dict[str, Any],
    languages: list[str],
) -> dict[str, Any]:
    runtime_langs = runtime_section.get("languages", {})
    compile_langs = compile_section.get("languages", {})
    runtime_cmp = runtime_section.get("comparisons", {})
    compile_cmp = compile_section.get("comparisons", {})

    memory_by_language: dict[str, float] = {}
    concurrency_by_language: dict[str, float] = {}
    productivity_by_language: dict[str, float] = {}

    if isinstance(runtime_langs, dict):
        for lang in languages:
            row = runtime_langs.get(lang, {})
            if not isinstance(row, dict):
                continue
            records = row.get("records", [])
            mem_values: list[float] = []
            if isinstance(records, list):
                for record in records:
                    if not isinstance(record, dict):
                        continue
                    mem = float(record.get("mem_bytes", 0.0))
                    if mem > 0.0:
                        mem_values.append(mem)
            memory_by_language[lang] = summarize(mem_values)["mean"] if mem_values else 0.0

            problem_metrics = row.get("problem_metrics", {})
            if isinstance(problem_metrics, dict):
                coro = problem_metrics.get("coro-prime-sieve", {})
                if isinstance(coro, dict):
                    concurrency_by_language[lang] = float(coro.get("mean_time_ms", 0.0))

    if isinstance(compile_langs, dict):
        for lang in languages:
            row = compile_langs.get(lang, {})
            if not isinstance(row, dict):
                continue
            inc = row.get("incremental", {})
            if isinstance(inc, dict):
                productivity_by_language[lang] = float(inc.get("mean_ms", 0.0))

    vibelang_runtime_stability = 0.0
    vibe_row = runtime_langs.get("vibelang", {}) if isinstance(runtime_langs, dict) else {}
    if isinstance(vibe_row, dict):
        records = vibe_row.get("records", [])
        rsd_values: list[float] = []
        if isinstance(records, list):
            for record in records:
                if not isinstance(record, dict):
                    continue
                time_ms = float(record.get("time_ms", 0.0))
                std_ms = float(record.get("time_stddev_ms", 0.0))
                if time_ms > 0.0 and std_ms >= 0.0:
                    rsd_values.append(std_ms / time_ms)
        vibelang_runtime_stability = summarize(rsd_values)["mean"] if rsd_values else 0.0

    return {
        "runtime_performance": runtime_cmp if isinstance(runtime_cmp, dict) else {},
        "memory_footprint": {
            "mean_mem_bytes_by_language": memory_by_language,
        },
        "concurrency_performance": {
            "problem": "coro-prime-sieve",
            "mean_time_ms_by_language": concurrency_by_language,
        },
        "compile_performance": compile_cmp if isinstance(compile_cmp, dict) else {},
        "developer_productivity_proxy": {
            "incremental_compile_mean_ms_by_language": productivity_by_language,
        },
        "ai_native_proxy": {
            "vibelang_runtime_relative_stddev": vibelang_runtime_stability,
            "vibelang_incremental_compile_mean_ms": productivity_by_language.get("vibelang", 0.0),
            "notes": (
                "AI-native productivity is proxied by incremental compile feedback and "
                "runtime stability; replace with direct agent-task benchmarks when available."
            ),
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--profile",
        choices=["quick", "full"],
        default="quick",
        help="Benchmark profile to collect.",
    )
    parser.add_argument(
        "--output-root",
        default="reports/benchmarks/third_party",
        help="Output root relative to repo root.",
    )
    parser.add_argument(
        "--matrix-file",
        default="benchmarks/third_party/plbci/config/language_matrix.json",
        help="Language matrix file relative to repo root.",
    )
    parser.add_argument(
        "--bench-template",
        default="benchmarks/third_party/plbci/config/bench.template.yaml",
        help="PLB-CI benchmark template file relative to repo root.",
    )
    parser.add_argument(
        "--adapter-root",
        default="benchmarks/third_party/plbci/adapters/vibelang",
        help="Adapter root for VibeLang PLB-CI integration.",
    )
    parser.add_argument(
        "--checkout-dir",
        default=".cache/third_party/plbci",
        help="PLB-CI checkout directory relative to repo root.",
    )
    parser.add_argument(
        "--no-docker",
        action="store_true",
        help="Disable docker runtime for PLB-CI tasks.",
    )
    parser.add_argument(
        "--preflight-only",
        action="store_true",
        help="Run dependency/toolchain preflight checks and exit.",
    )
    parser.add_argument(
        "--allow-preflight-degraded",
        action="store_true",
        help="Continue collection even when preflight reports missing lanes.",
    )
    parser.add_argument(
        "--skip-runtime",
        action="store_true",
        help="Skip PLB-CI runtime/memory/concurrency lanes.",
    )
    parser.add_argument(
        "--skip-compile",
        action="store_true",
        help="Skip hyperfine compile-lane collection.",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[2]
    apply_path_hints()
    matrix_path = repo_root / args.matrix_file
    bench_template = repo_root / args.bench_template
    adapter_root = repo_root / args.adapter_root
    output_root = repo_root / args.output_root
    checkout_dir = repo_root / args.checkout_dir

    if not matrix_path.exists():
        fail(f"matrix file missing: {matrix_path}")
    if not bench_template.exists():
        fail(f"benchmark template missing: {bench_template}")
    if not adapter_root.exists():
        fail(f"adapter root missing: {adapter_root}")

    matrix = json.loads(matrix_path.read_text())
    profile_cfg = matrix.get("profiles", {}).get(args.profile)
    if not isinstance(profile_cfg, dict):
        fail(f"profile `{args.profile}` missing in language matrix")
    plbci_repo = str(matrix.get("plbci_repo", "")).strip()
    plbci_ref = str(matrix.get("plbci_ref", "main")).strip()
    if not plbci_repo:
        fail("matrix must define `plbci_repo`")

    languages = [str(item["id"]) for item in matrix.get("languages", [])]
    if not languages:
        fail("matrix languages list is empty")
    problems = [str(problem) for problem in matrix.get("problems", [])]
    if not problems:
        fail("matrix problems list is empty")

    preflight = preflight_checks(
        repo_root=repo_root,
        languages=languages,
        no_docker=args.no_docker,
    )
    if args.preflight_only:
        print(json.dumps(preflight, indent=2))
        if str(preflight.get("status")) != "ok":
            raise SystemExit(1)
        return
    if str(preflight.get("status")) != "ok":
        if args.allow_preflight_degraded:
            preflight["status"] = "degraded"
            preflight["degraded_mode"] = True
            preflight["degraded_reason"] = "explicit override via --allow-preflight-degraded"
        else:
            errors = preflight.get("errors", [])
            error_lines = [str(item) for item in errors if str(item).strip()]
            fail(
                "preflight checks failed:\n- " + "\n- ".join(error_lines)
                if error_lines
                else "preflight checks failed"
            )

    plbci_dir = ensure_plbci_checkout(
        repo_root=repo_root,
        checkout_dir=checkout_dir,
        repo_url=plbci_repo,
        ref=plbci_ref,
    )
    bench_tool_cwd = plbci_dir / "bench"
    if not bench_tool_cwd.exists():
        fail(f"PLB-CI bench directory missing: {bench_tool_cwd}")

    cache_root = repo_root / ".cache" / "third_party_bench"
    stage_dir = cache_root / "configs" / f"{args.profile}_plbci_config"
    bench_yaml, algorithm_root = stage_config(
        repo_root=repo_root,
        matrix=matrix,
        profile=args.profile,
        plbci_dir=plbci_dir,
        adapter_root=adapter_root,
        bench_template=bench_template,
        stage_dir=stage_dir,
    )

    runtime_section: dict[str, Any] = {
        "status": "skipped",
        "reason": "runtime lane skipped by flag",
        "tool": "plbci",
        "languages": {},
        "comparisons": {},
        "per_problem_table": {},
    }
    plbci_logs: dict[str, Any] = {}
    build_output = cache_root / args.profile / "plbci_build"
    if not args.skip_runtime:
        plbci_logs = run_plbci_suite(
            bench_tool_cwd=bench_tool_cwd,
            bench_yaml=bench_yaml,
            algorithm_root=algorithm_root,
            build_output=build_output,
            languages=languages,
            problems=problems,
            no_docker=args.no_docker,
        )
        runtime_section = parse_plbci_runtime(build_output=build_output, languages=languages)
        runtime_section["task_logs"] = plbci_logs

    compile_section: dict[str, Any] = {
        "status": "skipped",
        "reason": "compile lane skipped by flag",
        "tool": "hyperfine",
        "languages": {},
        "comparisons": {},
    }
    compile_build_root = cache_root / args.profile / "compile_build"
    if not args.skip_compile:
        compile_section = collect_compile_lanes(
            bench_tool_cwd=bench_tool_cwd,
            bench_yaml=bench_yaml,
            algorithm_root=algorithm_root,
            compile_build_root=compile_build_root,
            profile_cfg=profile_cfg,
            languages=languages,
            problems=problems,
            no_docker=args.no_docker,
        )

    now_epoch = int(time.time())
    now_utc = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(now_epoch))
    stamp = time.strftime("%Y%m%d_%H%M%SZ", time.gmtime(now_epoch))
    report = {
        "format": "vibe-third-party-benchmarks-v1",
        "suite_name": "plbci_hyperfine_matrix",
        "profile": args.profile,
        "generated_at_epoch_s": now_epoch,
        "generated_at_utc": now_utc,
        "timestamp_id": stamp,
        "matrix_file": str(matrix_path.relative_to(repo_root)),
        "bench_template": str(bench_template.relative_to(repo_root)),
        "languages": languages,
        "problems": problems,
        "tooling": {
            "plbci_repo": plbci_repo,
            "plbci_ref": plbci_ref,
            "plbci_checkout": str(plbci_dir),
            "bench_yaml_used": str(bench_yaml),
            "docker_enabled": not args.no_docker,
        },
        "environment": collect_environment(repo_root),
        "preflight": preflight,
        "runtime": runtime_section,
        "compile": compile_section,
        "categories": build_category_rollup(
            runtime_section=runtime_section,
            compile_section=compile_section,
            languages=languages,
        ),
    }

    profile_dir = output_root / args.profile
    latest_dir = output_root / "latest"
    history_dir = output_root / "history"
    profile_results = profile_dir / "results.json"
    latest_results = latest_dir / "results.json"
    history_results = history_dir / f"{stamp}_{args.profile}_results.json"

    write_json(profile_results, report)
    write_json(latest_results, report)
    write_json(history_results, report)

    raw_dir = profile_dir / "raw"
    raw_dir.mkdir(parents=True, exist_ok=True)
    if plbci_logs:
        write_json(raw_dir / "plbci_task_logs.json", plbci_logs)
    write_json(raw_dir / "matrix_snapshot.json", matrix)

    print(f"wrote {profile_results}")
    print(f"wrote {latest_results}")
    print(f"wrote {history_results}")


if __name__ == "__main__":
    main()
