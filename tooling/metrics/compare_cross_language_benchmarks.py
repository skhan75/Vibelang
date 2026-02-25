#!/usr/bin/env python3
import argparse
import json
import time
from pathlib import Path


def fail(message: str) -> None:
    raise SystemExit(f"cross-language benchmark delta failed: {message}")


def load_report(path: Path) -> dict:
    if not path.exists():
        fail(f"missing results file: {path}")
    report = json.loads(path.read_text())
    if report.get("format") != "vibe-cross-language-benchmarks-v1":
        fail(f"unsupported report format in {path}")
    cases = report.get("cases")
    if not isinstance(cases, list) or not cases:
        fail(f"`cases` section is missing or empty in {path}")
    return report


def report_languages(report: dict) -> list[str]:
    languages = report.get("languages")
    if isinstance(languages, list) and languages:
        return [str(language) for language in languages]
    first_case = report["cases"][0]
    case_languages = first_case.get("languages", {})
    if not isinstance(case_languages, dict) or not case_languages:
        fail("unable to determine language list")
    return list(case_languages.keys())


def case_index(report: dict) -> dict[str, dict]:
    out: dict[str, dict] = {}
    for case in report["cases"]:
        case_id = str(case.get("id", ""))
        if not case_id:
            fail("encountered case with missing `id`")
        out[case_id] = case
    return out


def geomean_ratio_map(report: dict, baselines: list[str]) -> dict[str, float]:
    summary = report.get("summary", {})
    ratio_map = summary.get("geomean_vibelang_ratio_vs", {})
    out: dict[str, float] = {}
    for baseline in baselines:
        value: float | int | None = None
        if isinstance(ratio_map, dict):
            raw = ratio_map.get(baseline)
            if isinstance(raw, (int, float)):
                value = raw
        if value is None:
            raw = summary.get(f"geomean_vibelang_ratio_vs_{baseline}")
            if isinstance(raw, (int, float)):
                value = raw
        if value is None:
            fail(f"missing geomean ratio for baseline `{baseline}`")
        out[baseline] = float(value)
    return out


def ratio_delta(before: float, after: float) -> dict[str, float]:
    delta_abs = after - before
    delta_pct = 0.0
    if before != 0.0:
        delta_pct = (delta_abs / before) * 100.0
    return {
        "before": before,
        "after": after,
        "delta_abs": delta_abs,
        "delta_pct": delta_pct,
    }


def build_case_delta(
    case_id: str,
    baseline_case: dict,
    candidate_case: dict,
    languages: list[str],
) -> dict:
    baseline_langs = baseline_case["languages"]
    candidate_langs = candidate_case["languages"]
    means_by_language: dict[str, dict[str, float]] = {}
    for language in languages:
        baseline_mean = float(baseline_langs[language]["runtime"]["summary"]["mean_wall_ms"])
        candidate_mean = float(candidate_langs[language]["runtime"]["summary"]["mean_wall_ms"])
        means_by_language[language] = ratio_delta(baseline_mean, candidate_mean)

    ratio_by_baseline: dict[str, dict[str, float]] = {}
    base_vibe = means_by_language["vibelang"]["before"]
    cand_vibe = means_by_language["vibelang"]["after"]
    for language in languages:
        if language == "vibelang":
            continue
        base_baseline = means_by_language[language]["before"]
        cand_baseline = means_by_language[language]["after"]
        if base_baseline <= 0.0 or cand_baseline <= 0.0:
            fail(f"non-positive mean runtime for case `{case_id}` language `{language}`")
        ratio_by_baseline[language] = ratio_delta(base_vibe / base_baseline, cand_vibe / cand_baseline)

    cand_summary = candidate_langs["vibelang"]["runtime"]["summary"]
    cand_mean = float(cand_summary["mean_wall_ms"])
    cand_stddev = float(cand_summary["stddev_wall_ms"])
    cand_p95 = float(cand_summary["p95_wall_ms"])
    noise_flags = {
        "high_rsd": (cand_stddev / cand_mean) > 0.15 if cand_mean > 0.0 else False,
        "high_p95_to_mean": (cand_p95 / cand_mean) > 1.5 if cand_mean > 0.0 else False,
    }

    return {
        "case_id": case_id,
        "means_by_language": means_by_language,
        "vibelang_ratio_by_baseline": ratio_by_baseline,
        "candidate_noise_flags": noise_flags,
    }


def build_markdown(delta_report: dict) -> str:
    lines: list[str] = []
    lines.append("# Cross-Language Benchmark Delta Report")
    lines.append("")
    lines.append(f"- generated_at_utc: `{delta_report['generated_at_utc']}`")
    lines.append(f"- baseline_results: `{delta_report['baseline_results']}`")
    lines.append(f"- candidate_results: `{delta_report['candidate_results']}`")
    lines.append(f"- baseline_profile: `{delta_report['baseline_profile']}`")
    lines.append(f"- candidate_profile: `{delta_report['candidate_profile']}`")
    lines.append("")
    lines.append("## Geomean Delta (Vibe/Baseline Ratios)")
    lines.append("")
    lines.append("| baseline | before | after | delta_abs | delta_pct |")
    lines.append("| --- | ---: | ---: | ---: | ---: |")
    for baseline, row in delta_report["geomean_delta"].items():
        lines.append(
            f"| {baseline} | {row['before']:.3f} | {row['after']:.3f} | "
            f"{row['delta_abs']:.3f} | {row['delta_pct']:.2f}% |"
        )
    lines.append("")
    lines.append("Interpretation: for these ratios, lower is better. Negative delta means improvement.")
    lines.append("")

    lines.append("## Per-Case Vibe Runtime Delta")
    lines.append("")
    lines.append("| case | vibe_before_ms | vibe_after_ms | delta_abs_ms | delta_pct |")
    lines.append("| --- | ---: | ---: | ---: | ---: |")
    for case in delta_report["cases"]:
        vibe = case["means_by_language"]["vibelang"]
        lines.append(
            f"| {case['case_id']} | {vibe['before']:.3f} | {vibe['after']:.3f} | "
            f"{vibe['delta_abs']:.3f} | {vibe['delta_pct']:.2f}% |"
        )
    lines.append("")

    lines.append("## Noisy Case Signals (Candidate Run)")
    lines.append("")
    noisy = []
    for case in delta_report["cases"]:
        flags = case["candidate_noise_flags"]
        if flags["high_rsd"] or flags["high_p95_to_mean"]:
            noisy.append((case["case_id"], flags))
    if not noisy:
        lines.append("- none")
    else:
        for case_id, flags in noisy:
            markers = []
            if flags["high_rsd"]:
                markers.append("high_rsd")
            if flags["high_p95_to_mean"]:
                markers.append("high_p95_to_mean")
            lines.append(f"- `{case_id}`: {', '.join(markers)}")
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--baseline-results",
        required=True,
        help="Baseline results JSON path relative to repo root.",
    )
    parser.add_argument(
        "--candidate-results",
        default="reports/benchmarks/_DEPRECATED_cross_lang/latest/results.json",
        help="Candidate results JSON path relative to repo root.",
    )
    parser.add_argument(
        "--output-root",
        default="reports/benchmarks/_DEPRECATED_cross_lang/analysis/deltas",
        help="Output directory relative to repo root.",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[2]
    baseline_path = repo_root / args.baseline_results
    candidate_path = repo_root / args.candidate_results
    baseline = load_report(baseline_path)
    candidate = load_report(candidate_path)

    baseline_langs = report_languages(baseline)
    candidate_langs = report_languages(candidate)
    if baseline_langs != candidate_langs:
        fail(f"language mismatch baseline={baseline_langs} candidate={candidate_langs}")
    if "vibelang" not in baseline_langs:
        fail("reports must include `vibelang`")
    baselines = [language for language in baseline_langs if language != "vibelang"]

    baseline_cases = case_index(baseline)
    candidate_cases = case_index(candidate)
    if list(baseline_cases.keys()) != list(candidate_cases.keys()):
        fail("case-id mismatch between baseline and candidate reports")

    geomean_before = geomean_ratio_map(baseline, baselines)
    geomean_after = geomean_ratio_map(candidate, baselines)
    geomean_delta = {
        baseline_name: ratio_delta(geomean_before[baseline_name], geomean_after[baseline_name])
        for baseline_name in baselines
    }

    case_deltas = [
        build_case_delta(
            case_id=case_id,
            baseline_case=baseline_cases[case_id],
            candidate_case=candidate_cases[case_id],
            languages=baseline_langs,
        )
        for case_id in baseline_cases
    ]

    generated_epoch = int(time.time())
    generated_utc = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(generated_epoch))
    timestamp = time.strftime("%Y%m%d_%H%M%SZ", time.gmtime(generated_epoch))
    report = {
        "format": "vibe-cross-language-benchmark-delta-v1",
        "generated_at_epoch_s": generated_epoch,
        "generated_at_utc": generated_utc,
        "baseline_results": str(baseline_path.relative_to(repo_root)),
        "candidate_results": str(candidate_path.relative_to(repo_root)),
        "baseline_profile": baseline.get("profile", "unknown"),
        "candidate_profile": candidate.get("profile", "unknown"),
        "languages": baseline_langs,
        "geomean_delta": geomean_delta,
        "cases": case_deltas,
    }

    out_dir = repo_root / args.output_root
    out_dir.mkdir(parents=True, exist_ok=True)
    json_path = out_dir / f"{timestamp}_delta.json"
    md_path = out_dir / f"{timestamp}_delta.md"
    latest_json = out_dir / "latest_delta.json"
    latest_md = out_dir / "latest_delta.md"
    json_payload = json.dumps(report, indent=2) + "\n"
    md_payload = build_markdown(report) + "\n"
    json_path.write_text(json_payload)
    md_path.write_text(md_payload)
    latest_json.write_text(json_payload)
    latest_md.write_text(md_payload)

    print(f"wrote {json_path}")
    print(f"wrote {md_path}")
    print(f"wrote {latest_json}")
    print(f"wrote {latest_md}")


if __name__ == "__main__":
    main()
