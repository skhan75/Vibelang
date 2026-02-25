#!/usr/bin/env python3
import argparse
import json
from pathlib import Path
from typing import Any


def fail(message: str) -> None:
    raise SystemExit(f"third-party benchmark validation failed: {message}")


def expect_dict(value: Any, name: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        fail(f"{name} must be an object")
    return value


def expect_list(value: Any, name: str) -> list[Any]:
    if not isinstance(value, list):
        fail(f"{name} must be a list")
    return value


def evaluate_budget(
    report: dict[str, Any],
    budgets: dict[str, Any],
    mode: str,
) -> dict[str, Any]:
    violations: list[str] = []
    warnings: list[str] = []

    runtime = expect_dict(report.get("runtime"), "runtime")
    compile_section = expect_dict(report.get("compile"), "compile")
    runtime_cmp_raw = runtime.get("comparisons", {})
    compile_cmp_raw = compile_section.get("comparisons", {})
    runtime_cmp = runtime_cmp_raw if isinstance(runtime_cmp_raw, dict) else {}
    compile_cmp = compile_cmp_raw if isinstance(compile_cmp_raw, dict) else {}

    runtime_limits = budgets.get("runtime_geomean_ratio_max", {})
    if isinstance(runtime_limits, dict):
        for baseline, limit in runtime_limits.items():
            if not isinstance(limit, (int, float)):
                continue
            row = runtime_cmp.get(str(baseline), {})
            if not isinstance(row, dict):
                warnings.append(f"runtime ratio missing for baseline `{baseline}`")
                continue
            ratio = float(row.get("geomean_vibelang_ratio", 0.0))
            if ratio <= 0.0:
                warnings.append(f"runtime ratio missing/zero for baseline `{baseline}`")
                continue
            if ratio > float(limit):
                violations.append(
                    f"runtime geomean ratio exceeded for {baseline}: "
                    f"current={ratio:.3f} limit={float(limit):.3f}"
                )

    compile_limits = budgets.get("compile_cold_ratio_max", {})
    if isinstance(compile_limits, dict):
        for baseline, limit in compile_limits.items():
            if not isinstance(limit, (int, float)):
                continue
            row = compile_cmp.get(str(baseline), {})
            if not isinstance(row, dict):
                warnings.append(f"compile ratio missing for baseline `{baseline}`")
                continue
            ratio = float(row.get("vibelang_cold_ratio", 0.0))
            if ratio <= 0.0:
                warnings.append(f"compile ratio missing/zero for baseline `{baseline}`")
                continue
            if ratio > float(limit):
                violations.append(
                    f"compile cold ratio exceeded for {baseline}: "
                    f"current={ratio:.3f} limit={float(limit):.3f}"
                )

    required_runtime = budgets.get("required_runtime_languages", [])
    if isinstance(required_runtime, list):
        runtime_langs_raw = runtime.get("languages", {})
        runtime_langs = runtime_langs_raw if isinstance(runtime_langs_raw, dict) else {}
        allow_missing = set(
            str(item) for item in budgets.get("allow_unavailable_runtime_languages", [])
        )
        for lang in required_runtime:
            lang_id = str(lang)
            row = runtime_langs.get(lang_id, {})
            status = str(row.get("status", "missing")) if isinstance(row, dict) else "missing"
            if status != "ok" and lang_id not in allow_missing:
                violations.append(
                    f"required runtime language `{lang_id}` not available (status={status})"
                )
            elif status != "ok":
                warnings.append(
                    f"runtime language `{lang_id}` unavailable but allowlisted (status={status})"
                )

    required_compile = budgets.get("required_compile_languages", [])
    if isinstance(required_compile, list):
        compile_langs_raw = compile_section.get("languages", {})
        compile_langs = compile_langs_raw if isinstance(compile_langs_raw, dict) else {}
        allow_missing = set(
            str(item) for item in budgets.get("allow_unavailable_compile_languages", [])
        )
        for lang in required_compile:
            lang_id = str(lang)
            row = compile_langs.get(lang_id, {})
            status = str(row.get("status", "missing")) if isinstance(row, dict) else "missing"
            if status != "ok" and lang_id not in allow_missing:
                violations.append(
                    f"required compile language `{lang_id}` not available (status={status})"
                )
            elif status != "ok":
                warnings.append(
                    f"compile language `{lang_id}` unavailable but allowlisted (status={status})"
                )

    required_problems = budgets.get("required_problems", [])
    if isinstance(required_problems, list):
        per_problem_raw = runtime.get("per_problem_table", {})
        per_problem = per_problem_raw if isinstance(per_problem_raw, dict) else {}
        for problem in required_problems:
            if str(problem) not in per_problem:
                violations.append(f"required benchmark problem `{problem}` missing")

    status = "pass" if not violations else "fail"
    if mode == "warn":
        status = "warn" if violations else "pass"
        warnings.extend([f"[warn-mode] {item}" for item in violations])
        violations = []

    return {
        "status": status,
        "mode": mode,
        "violations": violations,
        "warnings": warnings,
    }


def to_md_ratio(value: float) -> str:
    if value <= 0.0:
        return "n/a"
    return f"{value:.3f}"


def build_summary(report: dict[str, Any], budget: dict[str, Any]) -> str:
    profile = str(report.get("profile", "unknown"))
    generated = str(report.get("generated_at_utc", "unknown"))
    runtime = expect_dict(report.get("runtime"), "runtime")
    compile_section = expect_dict(report.get("compile"), "compile")
    runtime_cmp_raw = runtime.get("comparisons", {})
    compile_cmp_raw = compile_section.get("comparisons", {})
    runtime_cmp = runtime_cmp_raw if isinstance(runtime_cmp_raw, dict) else {}
    compile_cmp = compile_cmp_raw if isinstance(compile_cmp_raw, dict) else {}
    languages = [str(lang) for lang in expect_list(report.get("languages"), "languages")]

    lines: list[str] = []
    lines.append("# Third-Party Benchmark Summary")
    lines.append("")
    lines.append(f"- profile: `{profile}`")
    lines.append(f"- generated_at_utc: `{generated}`")
    lines.append(f"- budget_status: `{budget['status']}`")
    lines.append("")

    lines.append("## Runtime Geomean Ratios (VibeLang vs Baselines)")
    lines.append("")
    lines.append("| baseline | vibelang_ratio |")
    lines.append("| --- | ---: |")
    for baseline, row in sorted(runtime_cmp.items()):
        if not isinstance(row, dict):
            continue
        ratio = float(row.get("geomean_vibelang_ratio", 0.0))
        lines.append(f"| {baseline} | {to_md_ratio(ratio)} |")
    lines.append("")
    lines.append(
        "Interpretation: ratio > 1.0 means VibeLang is slower on average; ratio < 1.0 means faster."
    )
    lines.append("")

    lines.append("## Compile Cold Ratios (VibeLang vs Baselines)")
    lines.append("")
    lines.append("| baseline | vibelang_cold_ratio |")
    lines.append("| --- | ---: |")
    for baseline, row in sorted(compile_cmp.items()):
        if not isinstance(row, dict):
            continue
        ratio = float(row.get("vibelang_cold_ratio", 0.0))
        lines.append(f"| {baseline} | {to_md_ratio(ratio)} |")
    lines.append("")

    categories_raw = report.get("categories", {})
    categories = categories_raw if isinstance(categories_raw, dict) else {}
    memory_by_language: dict[str, float] = {}
    productivity_by_language: dict[str, float] = {}
    concurrency_by_language: dict[str, float] = {}
    if categories:
        memory = categories.get("memory_footprint", {})
        if isinstance(memory, dict):
            mem_map = memory.get("mean_mem_bytes_by_language", {})
            if isinstance(mem_map, dict):
                memory_by_language = {
                    str(lang): float(value) for lang, value in mem_map.items() if isinstance(value, (int, float))
                }
        productivity = categories.get("developer_productivity_proxy", {})
        if isinstance(productivity, dict):
            prod_map = productivity.get("incremental_compile_mean_ms_by_language", {})
            if isinstance(prod_map, dict):
                productivity_by_language = {
                    str(lang): float(value) for lang, value in prod_map.items() if isinstance(value, (int, float))
                }
        concurrency = categories.get("concurrency_performance", {})
        if isinstance(concurrency, dict):
            conc_map = concurrency.get("mean_time_ms_by_language", {})
            if isinstance(conc_map, dict):
                concurrency_by_language = {
                    str(lang): float(value) for lang, value in conc_map.items() if isinstance(value, (int, float))
                }

    lines.append("## Category Snapshot")
    lines.append("")
    lines.append("| language | memory_mean_bytes | incremental_compile_ms | coro_prime_sieve_ms |")
    lines.append("| --- | ---: | ---: | ---: |")
    for lang in languages:
        mem = memory_by_language.get(lang, 0.0)
        inc = productivity_by_language.get(lang, 0.0)
        conc = concurrency_by_language.get(lang, 0.0)
        lines.append(
            f"| {lang} | "
            f"{(f'{mem:.0f}' if mem > 0.0 else 'n/a')} | "
            f"{(f'{inc:.3f}' if inc > 0.0 else 'n/a')} | "
            f"{(f'{conc:.3f}' if conc > 0.0 else 'n/a')} |"
        )
    lines.append("")

    ai_native = categories.get("ai_native_proxy", {})
    if isinstance(ai_native, dict):
        lines.append("## AI-Native Proxy Signals")
        lines.append("")
        rsd = float(ai_native.get("vibelang_runtime_relative_stddev", 0.0))
        inc = float(ai_native.get("vibelang_incremental_compile_mean_ms", 0.0))
        lines.append(
            f"- vibelang_runtime_relative_stddev: `{rsd:.6f}`"
            if rsd > 0.0
            else "- vibelang_runtime_relative_stddev: `n/a`"
        )
        lines.append(
            f"- vibelang_incremental_compile_mean_ms: `{inc:.3f}`"
            if inc > 0.0
            else "- vibelang_incremental_compile_mean_ms: `n/a`"
        )
        note = str(ai_native.get("notes", "")).strip()
        if note:
            lines.append(f"- note: {note}")
        lines.append("")

    per_problem_raw = runtime.get("per_problem_table", {})
    per_problem = per_problem_raw if isinstance(per_problem_raw, dict) else {}
    lines.append("## Runtime Mean Time by Problem (ms)")
    lines.append("")
    table_headers = ["problem"] + languages
    lines.append("| " + " | ".join(table_headers) + " |")
    lines.append("| " + " | ".join(["---"] + ["---:" for _ in languages]) + " |")
    for problem, lang_rows in sorted(per_problem.items()):
        row = [problem]
        for lang in languages:
            metric = {}
            if isinstance(lang_rows, dict):
                metric = lang_rows.get(lang, {})
            mean_time = 0.0
            if isinstance(metric, dict):
                mean_time = float(metric.get("mean_time_ms", 0.0))
            row.append(f"{mean_time:.3f}" if mean_time > 0.0 else "n/a")
        lines.append("| " + " | ".join(row) + " |")
    lines.append("")

    wins: list[str] = []
    gaps: list[tuple[str, float]] = []
    for baseline, row in runtime_cmp.items():
        if not isinstance(row, dict):
            continue
        ratio = float(row.get("geomean_vibelang_ratio", 0.0))
        if ratio <= 0.0:
            continue
        if ratio < 1.0:
            wins.append(f"Runtime: faster than {baseline} (ratio={ratio:.3f})")
        else:
            gaps.append((f"Runtime: slower than {baseline}", ratio))
    for baseline, row in compile_cmp.items():
        if not isinstance(row, dict):
            continue
        ratio = float(row.get("vibelang_cold_ratio", 0.0))
        if ratio <= 0.0:
            continue
        if ratio < 1.0:
            wins.append(f"Compile: faster than {baseline} (ratio={ratio:.3f})")
        else:
            gaps.append((f"Compile: slower than {baseline}", ratio))
    gaps.sort(key=lambda item: item[1], reverse=True)

    lines.append("## Wins")
    lines.append("")
    if wins:
        for item in wins:
            lines.append(f"- {item}")
    else:
        lines.append("- No clear wins in this run.")
    lines.append("")

    lines.append("## Gaps and Improvement Opportunities")
    lines.append("")
    if gaps:
        for label, ratio in gaps[:6]:
            lines.append(f"- {label} (ratio={ratio:.3f})")
    else:
        lines.append("- No major gaps detected.")
    lines.append("")

    lines.append("## Simple-language analysis")
    lines.append("")
    if gaps:
        lines.append(
            "- VibeLang still has performance gaps versus some baselines. Focus next on the worst ratios first."
        )
    else:
        lines.append("- Current run shows stable competitiveness against configured baselines.")
    if wins:
        lines.append("- There are measurable strengths that can be highlighted in public benchmark notes.")
    lines.append(
        "- Keep fairness caveats explicit: toolchain versions, host environment, and benchmark semantics affect results."
    )
    lines.append("")

    lines.append("## Budget Gate Output")
    lines.append("")
    lines.append(f"- mode: `{budget['mode']}`")
    lines.append(f"- status: `{budget['status']}`")
    violations = budget.get("violations", [])
    warnings = budget.get("warnings", [])
    if isinstance(violations, list) and violations:
        lines.append("- violations:")
        for item in violations:
            lines.append(f"  - {item}")
    if isinstance(warnings, list) and warnings:
        lines.append("- warnings:")
        for item in warnings:
            lines.append(f"  - {item}")
    if (not violations) and (not warnings):
        lines.append("- no budget issues detected")
    lines.append("")
    return "\n".join(lines)


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--results",
        default="reports/benchmarks/third_party/latest/results.json",
        help="Results JSON path relative to repo root.",
    )
    parser.add_argument(
        "--budget-file",
        default="reports/benchmarks/third_party/analysis/performance_budgets.json",
        help="Budget file path relative to repo root.",
    )
    parser.add_argument(
        "--enforcement-mode",
        choices=["warn", "strict"],
        default="strict",
        help="warn: report warnings only. strict: fail on violations.",
    )
    parser.add_argument(
        "--detailed-report-dir",
        default="reports/benchmarks/third_party/analysis",
        help="Directory for timestamped detailed summaries.",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[2]
    results_path = repo_root / args.results
    budget_path = repo_root / args.budget_file
    detailed_dir = repo_root / args.detailed_report_dir

    if not results_path.exists():
        fail(f"results file missing: {results_path}")
    if not budget_path.exists():
        fail(f"budget file missing: {budget_path}")

    report = json.loads(results_path.read_text())
    if report.get("format") != "vibe-third-party-benchmarks-v1":
        fail("results format mismatch")
    budgets = json.loads(budget_path.read_text())
    if budgets.get("format") != "vibe-third-party-performance-budget-v1":
        fail("budget file format mismatch")

    budget_result = evaluate_budget(report, budgets, args.enforcement_mode)
    summary = build_summary(report, budget_result)

    summary_path = results_path.with_name("summary.md")
    write(summary_path, summary + "\n")

    profile = str(report.get("profile", "unknown"))
    cross_root = results_path.parents[1]
    profile_summary = cross_root / profile / "summary.md"
    latest_summary = cross_root / "latest" / "summary.md"
    if results_path == cross_root / "latest" / "results.json":
        write(profile_summary, summary + "\n")
    else:
        write(latest_summary, summary + "\n")

    stamp = str(report.get("timestamp_id", "unknown"))
    detailed_path = detailed_dir / f"{stamp}_detailed_summary.md"
    write(detailed_path, summary + "\n")

    print("third-party benchmark validation completed")
    print(f"wrote {summary_path}")
    print(f"wrote {detailed_path}")

    if args.enforcement_mode == "strict" and budget_result["violations"]:
        fail("; ".join(str(item) for item in budget_result["violations"]))


if __name__ == "__main__":
    main()
