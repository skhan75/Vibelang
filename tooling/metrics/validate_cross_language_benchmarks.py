#!/usr/bin/env python3
import argparse
import json
from pathlib import Path


def fail(message: str) -> None:
    raise SystemExit(f"cross-language benchmark validation failed: {message}")


def expect_int(obj: dict, key: str, ctx: str, minimum: int | None = None) -> int:
    value = obj.get(key)
    if not isinstance(value, int):
        fail(f"{ctx}.{key} must be an integer")
    if minimum is not None and value < minimum:
        fail(f"{ctx}.{key} must be >= {minimum}")
    return value


def expect_number(obj: dict, key: str, ctx: str, minimum: float | None = None) -> float:
    value = obj.get(key)
    if not isinstance(value, (int, float)):
        fail(f"{ctx}.{key} must be a number")
    out = float(value)
    if minimum is not None and out < minimum:
        fail(f"{ctx}.{key} must be >= {minimum}")
    return out


def report_languages(report: dict) -> list[str]:
    languages = report.get("languages")
    if isinstance(languages, list) and languages:
        return [str(language) for language in languages]
    first_case = report["cases"][0] if report.get("cases") else {}
    case_languages = first_case.get("languages", {})
    if isinstance(case_languages, dict):
        return list(case_languages.keys())
    return []


def geomean_ratio_for(report: dict, baseline: str) -> float:
    summary = report.get("summary", {})
    ratio_map = summary.get("geomean_vibelang_ratio_vs", {})
    if isinstance(ratio_map, dict) and baseline in ratio_map:
        value = ratio_map[baseline]
        if isinstance(value, (int, float)):
            return float(value)
    value = summary.get(f"geomean_vibelang_ratio_vs_{baseline}")
    if isinstance(value, (int, float)):
        return float(value)
    fail(f"missing geomean ratio for baseline `{baseline}`")


def case_index(report: dict) -> dict[str, dict]:
    out: dict[str, dict] = {}
    for case in report.get("cases", []):
        case_id = str(case.get("id", ""))
        if case_id:
            out[case_id] = case
    return out


def summarize_case_ratios(report: dict) -> dict[str, dict[str, float]]:
    out: dict[str, dict[str, float]] = {}
    summary = report.get("summary", {})
    per_case = summary.get("per_case", [])
    if not isinstance(per_case, list):
        return out
    for row in per_case:
        if not isinstance(row, dict):
            continue
        case_id = str(row.get("case_id", ""))
        ratio_map = row.get("vibelang_ratio_vs", {})
        if not case_id or not isinstance(ratio_map, dict):
            continue
        clean_ratios: dict[str, float] = {}
        for baseline, ratio in ratio_map.items():
            if isinstance(ratio, (int, float)):
                clean_ratios[str(baseline)] = float(ratio)
        out[case_id] = clean_ratios
    return out


def evaluate_budget_gate(
    report: dict,
    budgets: dict,
    baseline_report: dict | None,
) -> tuple[list[str], list[str]]:
    violations: list[str] = []
    warnings: list[str] = []

    geomean_budget = budgets.get("geomean_ratio_max", {})
    if isinstance(geomean_budget, dict):
        for baseline, limit in geomean_budget.items():
            if not isinstance(limit, (int, float)):
                continue
            current = geomean_ratio_for(report, str(baseline))
            if current > float(limit):
                violations.append(
                    f"geomean ratio exceeded for {baseline}: current={current:.3f} limit={float(limit):.3f}"
                )

    case_ratio_budget = budgets.get("case_ratio_max", {})
    case_ratios = summarize_case_ratios(report)
    if isinstance(case_ratio_budget, dict):
        for case_id, baseline_limits in case_ratio_budget.items():
            if not isinstance(baseline_limits, dict):
                continue
            current_case = case_ratios.get(str(case_id), {})
            for baseline, limit in baseline_limits.items():
                if not isinstance(limit, (int, float)):
                    continue
                current_ratio = current_case.get(str(baseline))
                if current_ratio is None:
                    continue
                if current_ratio > float(limit):
                    violations.append(
                        f"case ratio exceeded for {case_id} vs {baseline}: current={current_ratio:.3f} limit={float(limit):.3f}"
                    )

    rerun_policy = budgets.get("rerun_policy", {})
    if isinstance(rerun_policy, dict) and baseline_report is not None:
        hotspots = rerun_policy.get("hotspot_cases", [])
        rsd_threshold = float(rerun_policy.get("rsd_wall_pct_threshold", 20.0))
        regression_threshold = float(rerun_policy.get("regression_pct_threshold", 10.0))
        if isinstance(hotspots, list):
            current_cases = case_index(report)
            baseline_cases = case_index(baseline_report)
            for hotspot in hotspots:
                case_id = str(hotspot)
                if case_id not in current_cases or case_id not in baseline_cases:
                    continue
                current_vibe = float(
                    current_cases[case_id]["languages"]["vibelang"]["runtime"]["summary"]["mean_wall_ms"]
                )
                baseline_vibe = float(
                    baseline_cases[case_id]["languages"]["vibelang"]["runtime"]["summary"]["mean_wall_ms"]
                )
                current_rsd = float(
                    current_cases[case_id]["languages"]["vibelang"]["runtime"]["summary"].get(
                        "rsd_wall_pct", 0.0
                    )
                )
                if baseline_vibe <= 0.0:
                    continue
                regression_pct = ((current_vibe - baseline_vibe) / baseline_vibe) * 100.0
                if regression_pct > regression_threshold and current_rsd >= rsd_threshold:
                    warnings.append(
                        f"rerun recommended for noisy hotspot {case_id}: regression={regression_pct:.2f}% rsd={current_rsd:.2f}%"
                    )

    return violations, warnings


def build_summary_markdown(
    report: dict,
    baseline_report: dict | None = None,
    baseline_label: str | None = None,
    budget_result: dict | None = None,
) -> str:
    profile = report["profile"]
    generated = report["generated_at_utc"]
    env = report.get("environment", {})
    cpu = env.get("cpu_model", "unknown")
    kernel = env.get("kernel_release", "unknown")
    cpu_governor = env.get("cpu_governor", "unknown")
    is_wsl = env.get("is_wsl", False)
    run_cfg = report["run_config"]
    languages = report_languages(report)
    baselines = [language for language in languages if language != "vibelang"]
    lines: list[str] = []
    lines.append("# Cross-Language Benchmark Summary")
    lines.append("")
    lines.append(f"- profile: `{profile}`")
    lines.append(f"- generated_at_utc: `{generated}`")
    lines.append(
        f"- runs: warmup={run_cfg['warmup_runs']} measured={run_cfg['measured_runs']}"
    )
    lines.append(f"- cpu_model: `{cpu}`")
    lines.append(f"- kernel_release: `{kernel}`")
    lines.append(f"- cpu_governor: `{cpu_governor}`")
    lines.append(f"- is_wsl: `{is_wsl}`")
    lines.append("")
    table_columns = (
        ["case"]
        + [f"{language} mean ms" for language in languages]
        + [f"vibe/{baseline}" for baseline in baselines]
    )
    lines.append("| " + " | ".join(table_columns) + " |")
    lines.append("| " + " | ".join(["---"] + ["---:" for _ in table_columns[1:]]) + " |")

    for case in report["cases"]:
        langs = case["languages"]
        vibe_ms = float(langs["vibelang"]["runtime"]["summary"]["mean_wall_ms"])
        row: list[str] = [str(case["id"])]
        for language in languages:
            mean_ms = float(langs[language]["runtime"]["summary"]["mean_wall_ms"])
            row.append(f"{mean_ms:.3f}")
        for baseline in baselines:
            baseline_ms = float(langs[baseline]["runtime"]["summary"]["mean_wall_ms"])
            if baseline_ms <= 0:
                row.append("n/a")
            else:
                row.append(f"{(vibe_ms / baseline_ms):.3f}")
        lines.append("| " + " | ".join(row) + " |")

    summary = report.get("summary", {})
    ratio_map = summary.get("geomean_vibelang_ratio_vs", {})
    lines.append("")
    lines.append("## Geomean Ratios")
    lines.append("")
    for baseline in baselines:
        ratio = summary.get(f"geomean_vibelang_ratio_vs_{baseline}", 0.0)
        if isinstance(ratio_map, dict) and baseline in ratio_map:
            ratio = ratio_map[baseline]
        lines.append(f"- vibelang_vs_{baseline}: `{float(ratio):.3f}`")
    lines.append("")
    lines.append(
        "Interpretation: ratio > 1.0 means VibeLang mean runtime is slower than the baseline; "
        "ratio < 1.0 means faster."
    )
    lines.append("")
    lines.append("## Fairness Notes")
    lines.append("")
    lines.append(
        "- Native AOT languages (VibeLang/C/Rust/Go) are compared in the same suite, "
        "while Python/TypeScript are interpreter/JIT-oriented baselines."
    )
    lines.append(
        "- Channel and scheduler semantics differ by runtime implementation; "
        "cross-language ratios in concurrency cases should be interpreted with this caveat."
    )
    lines.append(
        "- Host context matters: WSL2 and native Linux can differ on scheduler and timing behavior."
    )
    lines.append("")

    if baseline_report is not None:
        baseline_languages = report_languages(baseline_report)
        if baseline_languages != languages:
            fail(
                "baseline results language mismatch: "
                f"expected {languages}, got {baseline_languages}"
            )
        baseline_cases = {str(case["id"]): case for case in baseline_report["cases"]}
        current_cases = {str(case["id"]): case for case in report["cases"]}
        if list(baseline_cases.keys()) != list(current_cases.keys()):
            fail("baseline results case list mismatch")
        lines.append("## Delta vs Baseline")
        lines.append("")
        lines.append(f"- baseline: `{baseline_label or 'provided baseline'}`")
        baseline_generated = baseline_report.get("generated_at_utc", "unknown")
        lines.append(f"- baseline_generated_at_utc: `{baseline_generated}`")
        lines.append("")
        lines.append("### Geomean Delta (Vibe/Baseline Ratios)")
        lines.append("")
        lines.append("| baseline | before | after | delta_abs | delta_pct |")
        lines.append("| --- | ---: | ---: | ---: | ---: |")
        for baseline in baselines:
            before = geomean_ratio_for(baseline_report, baseline)
            after = geomean_ratio_for(report, baseline)
            delta_abs = after - before
            delta_pct = (delta_abs / before) * 100.0 if before != 0.0 else 0.0
            lines.append(
                f"| {baseline} | {before:.3f} | {after:.3f} | {delta_abs:.3f} | {delta_pct:.2f}% |"
            )
        lines.append("")
        lines.append("### Per-Case Vibe Runtime Delta")
        lines.append("")
        lines.append("| case | vibe_before_ms | vibe_after_ms | delta_abs_ms | delta_pct |")
        lines.append("| --- | ---: | ---: | ---: | ---: |")
        for case_id in baseline_cases:
            before = float(
                baseline_cases[case_id]["languages"]["vibelang"]["runtime"]["summary"]["mean_wall_ms"]
            )
            after = float(
                current_cases[case_id]["languages"]["vibelang"]["runtime"]["summary"]["mean_wall_ms"]
            )
            delta_abs = after - before
            delta_pct = (delta_abs / before) * 100.0 if before != 0.0 else 0.0
            lines.append(
                f"| {case_id} | {before:.3f} | {after:.3f} | {delta_abs:.3f} | {delta_pct:.2f}% |"
            )
        lines.append("")
    if budget_result is not None:
        lines.append("## Budget Gate")
        lines.append("")
        status = str(budget_result.get("status", "unknown"))
        lines.append(f"- status: `{status}`")
        violations = budget_result.get("violations", [])
        warnings = budget_result.get("warnings", [])
        if isinstance(violations, list) and violations:
            lines.append("- violations:")
            for violation in violations:
                lines.append(f"  - {violation}")
        if isinstance(warnings, list) and warnings:
            lines.append("- warnings:")
            for warning in warnings:
                lines.append(f"  - {warning}")
        if (not isinstance(violations, list) or not violations) and (
            not isinstance(warnings, list) or not warnings
        ):
            lines.append("- no budget issues detected")
        lines.append("")
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--results",
        default="reports/benchmarks/_DEPRECATED_cross_lang/latest/results.json",
        help="Results JSON path relative to repo root.",
    )
    parser.add_argument(
        "--baseline-results",
        default=None,
        help="Optional baseline results JSON path relative to repo root for delta section.",
    )
    parser.add_argument(
        "--budget-file",
        default="reports/benchmarks/_DEPRECATED_cross_lang/analysis/performance_budgets.json",
        help="Optional budget config JSON path relative to repo root.",
    )
    parser.add_argument(
        "--enforce-budgets",
        action="store_true",
        help="Fail validation if budget violations are detected.",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[2]
    results_path = repo_root / args.results
    if not results_path.exists():
        fail(f"missing results file: {results_path}")

    report = json.loads(results_path.read_text())
    if report.get("format") != "vibe-cross-language-benchmarks-v1":
        fail("results format mismatch")

    baseline_report = None
    baseline_label = None
    if args.baseline_results:
        baseline_path = repo_root / args.baseline_results
        if not baseline_path.exists():
            fail(f"missing baseline results file: {baseline_path}")
        baseline_report = json.loads(baseline_path.read_text())
        if baseline_report.get("format") != "vibe-cross-language-benchmarks-v1":
            fail("baseline results format mismatch")
        baseline_label = str(baseline_path.relative_to(repo_root))

    profile = report.get("profile")
    if profile not in {"quick", "full"}:
        fail("profile must be `quick` or `full`")

    run_cfg = report.get("run_config")
    if not isinstance(run_cfg, dict):
        fail("run_config section missing")
    warmup_runs = expect_int(run_cfg, "warmup_runs", "run_config", minimum=0)
    measured_runs = expect_int(run_cfg, "measured_runs", "run_config", minimum=1)

    env = report.get("environment")
    if not isinstance(env, dict):
        fail("environment section missing")
    if not isinstance(env.get("cpu_model"), str):
        fail("environment.cpu_model must be a string")
    if not isinstance(env.get("cpu_governor"), str):
        fail("environment.cpu_governor must be a string")
    expect_int(env, "logical_cpus", "environment", minimum=0)
    expect_int(env, "physical_cores", "environment", minimum=0)
    expect_int(env, "memory_total_kb", "environment", minimum=0)
    expect_int(env, "swap_total_kb", "environment", minimum=0)
    source_revisions = env.get("source_revisions")
    if not isinstance(source_revisions, dict):
        fail("environment.source_revisions must be an object")
    if not isinstance(source_revisions.get("repo_git_revision"), str):
        fail("environment.source_revisions.repo_git_revision must be a string")

    cases = report.get("cases")
    if not isinstance(cases, list) or len(cases) == 0:
        fail("cases must be a non-empty list")

    expected_languages = report.get("languages")
    if not isinstance(expected_languages, list) or not expected_languages:
        first_case = cases[0] if cases else {}
        first_case_languages = first_case.get("languages", {})
        if isinstance(first_case_languages, dict):
            expected_languages = list(first_case_languages.keys())
        else:
            fail("unable to determine language list from report")
    if "vibelang" not in expected_languages:
        fail("expected languages must include `vibelang`")
    for case in cases:
        case_id = case.get("id", "<unknown>")
        context = f"case[{case_id}]"
        expected_ops = expect_int(case, "expected_ops", context, minimum=1)
        languages = case.get("languages")
        if not isinstance(languages, dict):
            fail(f"{context}.languages must be an object")
        language_keys = list(languages.keys())
        if language_keys != expected_languages:
            fail(
                f"{context}.languages order/content mismatch. "
                f"expected {expected_languages}, got {language_keys}"
            )
        for language in expected_languages:
            if language not in languages:
                fail(f"{context}.languages missing `{language}`")
            entry = languages[language]
            compile_entry = entry.get("compile")
            runtime_entry = entry.get("runtime")
            result_entry = entry.get("result")
            if not isinstance(compile_entry, dict):
                fail(f"{context}.{language}.compile must be an object")
            if not isinstance(runtime_entry, dict):
                fail(f"{context}.{language}.runtime must be an object")
            if not isinstance(result_entry, dict):
                fail(f"{context}.{language}.result must be an object")

            if expect_int(compile_entry, "exit_code", f"{context}.{language}.compile") != 0:
                fail(f"{context}.{language}.compile.exit_code must be 0")
            expect_int(
                compile_entry,
                "binary_size_bytes",
                f"{context}.{language}.compile",
                minimum=1,
            )

            warmups = runtime_entry.get("warmups")
            runs = runtime_entry.get("runs")
            summary = runtime_entry.get("summary")
            if not isinstance(warmups, list):
                fail(f"{context}.{language}.runtime.warmups must be a list")
            if not isinstance(runs, list):
                fail(f"{context}.{language}.runtime.runs must be a list")
            if len(warmups) != warmup_runs:
                fail(
                    f"{context}.{language}.runtime.warmups expected {warmup_runs}, got {len(warmups)}"
                )
            if len(runs) != measured_runs:
                fail(
                    f"{context}.{language}.runtime.runs expected {measured_runs}, got {len(runs)}"
                )
            if not isinstance(summary, dict):
                fail(f"{context}.{language}.runtime.summary must be an object")
            expect_number(summary, "mean_wall_ms", f"{context}.{language}.runtime.summary", minimum=0.0)
            expect_number(summary, "p95_wall_ms", f"{context}.{language}.runtime.summary", minimum=0.0)
            expect_number(summary, "p99_wall_ms", f"{context}.{language}.runtime.summary", minimum=0.0)
            expect_number(summary, "mad_wall_ms", f"{context}.{language}.runtime.summary", minimum=0.0)
            expect_number(summary, "rsd_wall_pct", f"{context}.{language}.runtime.summary", minimum=0.0)
            expect_number(
                summary,
                "mean_max_rss_kb",
                f"{context}.{language}.runtime.summary",
                minimum=0.0,
            )

            ops = expect_int(result_entry, "ops", f"{context}.{language}.result", minimum=0)
            if ops != expected_ops:
                fail(
                    f"{context}.{language}.result.ops mismatch: expected {expected_ops}, got {ops}"
                )
            expect_int(result_entry, "checksum", f"{context}.{language}.result")

        parity = case.get("parity")
        if not isinstance(parity, dict):
            fail(f"{context}.parity must be an object")
        if parity.get("checksum_match") is not True:
            fail(f"{context}.parity.checksum_match must be true")
        if parity.get("ops_match") is not True:
            fail(f"{context}.parity.ops_match must be true")

    summary_section = report.get("summary")
    if not isinstance(summary_section, dict):
        fail("summary section missing")
    ratio_map = summary_section.get("geomean_vibelang_ratio_vs")
    if not isinstance(ratio_map, dict):
        fail("summary.geomean_vibelang_ratio_vs must be an object")
    for language in expected_languages:
        if language == "vibelang":
            continue
        key = f"geomean_vibelang_ratio_vs_{language}"
        expect_number(summary_section, key, "summary", minimum=0.0)
        if language not in ratio_map:
            fail(f"summary.geomean_vibelang_ratio_vs missing `{language}`")
        ratio_value = ratio_map.get(language)
        if not isinstance(ratio_value, (int, float)):
            fail(f"summary.geomean_vibelang_ratio_vs.{language} must be a number")
        if float(ratio_value) < 0.0:
            fail(f"summary.geomean_vibelang_ratio_vs.{language} must be >= 0")

    budget_result = None
    budget_path = repo_root / args.budget_file if args.budget_file else None
    if budget_path is not None and budget_path.exists():
        budgets = json.loads(budget_path.read_text())
        if not isinstance(budgets, dict):
            fail("budget file must contain a JSON object")
        violations, warnings = evaluate_budget_gate(report, budgets, baseline_report)
        rerun_gate = budgets.get("rerun_policy", {})
        enforce_rerun_gate = (
            isinstance(rerun_gate, dict)
            and bool(rerun_gate.get("enforce_rerun_gate", False))
        )
        status = "pass" if not violations else "fail"
        budget_result = {
            "status": status,
            "budget_file": str(budget_path.relative_to(repo_root)),
            "violations": violations,
            "warnings": warnings,
        }
        if args.enforce_budgets and violations:
            fail("budget violations detected: " + "; ".join(violations))
        if args.enforce_budgets and enforce_rerun_gate and warnings:
            fail("rerun gate triggered: " + "; ".join(warnings))
        for warning in warnings:
            print(f"budget warning: {warning}")

    summary_md = build_summary_markdown(
        report,
        baseline_report=baseline_report,
        baseline_label=baseline_label,
        budget_result=budget_result,
    )
    summary_path = results_path.with_name("summary.md")
    summary_path.write_text(summary_md + "\n")

    # Keep profile and latest summaries synchronized when validating latest.
    cross_lang_root = results_path.parents[1]
    latest_results = cross_lang_root / "latest" / "results.json"
    latest_summary = cross_lang_root / "latest" / "summary.md"
    profile_name = report["profile"]
    profile_summary = cross_lang_root / profile_name / "summary.md"
    if results_path == latest_results and profile_summary.exists():
        profile_summary.write_text(summary_md + "\n")
    if results_path != latest_results:
        latest_summary.write_text(summary_md + "\n")

    print("cross-language benchmark validation passed")
    print(f"wrote {summary_path}")


if __name__ == "__main__":
    main()

