#!/usr/bin/env python3
import argparse
import json
import re
import time
from pathlib import Path
from typing import Any


def fail(message: str) -> None:
    raise SystemExit(f"third-party benchmark delta generation failed: {message}")


def is_commit_sha(ref: str) -> bool:
    return bool(re.fullmatch(r"[0-9a-fA-F]{40}", ref.strip()))


def expect_format(report: dict[str, Any], label: str) -> None:
    if report.get("format") != "vibe-third-party-benchmarks-v1":
        fail(f"{label} format mismatch")


def validate_publication_report(report: dict[str, Any], label: str) -> None:
    tooling_raw = report.get("tooling", {})
    tooling = tooling_raw if isinstance(tooling_raw, dict) else {}
    publication_raw = report.get("publication", {})
    publication = publication_raw if isinstance(publication_raw, dict) else {}
    preflight_raw = report.get("preflight", {})
    preflight = preflight_raw if isinstance(preflight_raw, dict) else {}

    if not bool(tooling.get("publication_mode", False)):
        fail(f"{label} is not marked as publication_mode")
    if not bool(tooling.get("docker_enabled", False)):
        fail(f"{label} docker_enabled must be true")
    plbci_ref = str(tooling.get("plbci_ref", "")).strip()
    if not is_commit_sha(plbci_ref):
        fail(f"{label} plbci_ref must be a pinned 40-char commit SHA")
    if str(preflight.get("status", "")).strip() != "ok":
        fail(f"{label} preflight.status must be `ok`")
    if str(publication.get("mode", "")).strip() != "strict":
        fail(f"{label} publication.mode must be `strict`")


def collect_runtime_ratio_map(report: dict[str, Any]) -> dict[str, float]:
    runtime = report.get("runtime", {})
    comparisons = runtime.get("comparisons", {}) if isinstance(runtime, dict) else {}
    out: dict[str, float] = {}
    if isinstance(comparisons, dict):
        for baseline, row in comparisons.items():
            if not isinstance(row, dict):
                continue
            ratio = float(row.get("geomean_vibelang_ratio", 0.0))
            if ratio > 0.0:
                out[str(baseline)] = ratio
    return out


def collect_compile_ratio_map(report: dict[str, Any]) -> dict[str, float]:
    compile_section = report.get("compile", {})
    comparisons = compile_section.get("comparisons", {}) if isinstance(compile_section, dict) else {}
    out: dict[str, float] = {}
    if isinstance(comparisons, dict):
        for baseline, row in comparisons.items():
            if not isinstance(row, dict):
                continue
            ratio = float(row.get("vibelang_cold_ratio", 0.0))
            if ratio > 0.0:
                out[str(baseline)] = ratio
    return out


def build_delta(
    baseline: dict[str, Any],
    candidate: dict[str, Any],
    baseline_label: str,
    candidate_label: str,
) -> dict[str, Any]:
    runtime_before = collect_runtime_ratio_map(baseline)
    runtime_after = collect_runtime_ratio_map(candidate)
    compile_before = collect_compile_ratio_map(baseline)
    compile_after = collect_compile_ratio_map(candidate)

    runtime_delta: dict[str, dict[str, float]] = {}
    for baseline_lang in sorted(set(runtime_before).union(runtime_after)):
        before = float(runtime_before.get(baseline_lang, 0.0))
        after = float(runtime_after.get(baseline_lang, 0.0))
        delta_abs = after - before
        delta_pct = (delta_abs / before * 100.0) if before > 0.0 else 0.0
        runtime_delta[baseline_lang] = {
            "before": before,
            "after": after,
            "delta_abs": delta_abs,
            "delta_pct": delta_pct,
        }

    compile_delta: dict[str, dict[str, float]] = {}
    for baseline_lang in sorted(set(compile_before).union(compile_after)):
        before = float(compile_before.get(baseline_lang, 0.0))
        after = float(compile_after.get(baseline_lang, 0.0))
        delta_abs = after - before
        delta_pct = (delta_abs / before * 100.0) if before > 0.0 else 0.0
        compile_delta[baseline_lang] = {
            "before": before,
            "after": after,
            "delta_abs": delta_abs,
            "delta_pct": delta_pct,
        }

    now_epoch = int(time.time())
    now_utc = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(now_epoch))
    stamp = time.strftime("%Y%m%d_%H%M%SZ", time.gmtime(now_epoch))
    return {
        "format": "vibe-third-party-benchmark-delta-v1",
        "generated_at_epoch_s": now_epoch,
        "generated_at_utc": now_utc,
        "timestamp_id": stamp,
        "baseline_results": baseline_label,
        "candidate_results": candidate_label,
        "runtime_geomean_delta": runtime_delta,
        "compile_cold_delta": compile_delta,
    }


def to_markdown(delta: dict[str, Any]) -> str:
    lines: list[str] = []
    lines.append("# Third-Party Benchmark Delta Report")
    lines.append("")
    lines.append(f"- generated_at_utc: `{delta.get('generated_at_utc', 'unknown')}`")
    lines.append(f"- baseline_results: `{delta.get('baseline_results', 'unknown')}`")
    lines.append(f"- candidate_results: `{delta.get('candidate_results', 'unknown')}`")
    lines.append("")
    lines.append("## Runtime Geomean Delta (VibeLang/Baseline)")
    lines.append("")
    lines.append("| baseline | before | after | delta_abs | delta_pct |")
    lines.append("| --- | ---: | ---: | ---: | ---: |")
    runtime_delta = delta.get("runtime_geomean_delta", {})
    if isinstance(runtime_delta, dict):
        for baseline, row in runtime_delta.items():
            if not isinstance(row, dict):
                continue
            lines.append(
                f"| {baseline} | {float(row.get('before', 0.0)):.3f} | "
                f"{float(row.get('after', 0.0)):.3f} | "
                f"{float(row.get('delta_abs', 0.0)):.3f} | "
                f"{float(row.get('delta_pct', 0.0)):.2f}% |"
            )
    lines.append("")
    lines.append("Interpretation: negative delta is improvement (ratio got smaller).")
    lines.append("")

    lines.append("## Compile Cold Delta (VibeLang/Baseline)")
    lines.append("")
    lines.append("| baseline | before | after | delta_abs | delta_pct |")
    lines.append("| --- | ---: | ---: | ---: | ---: |")
    compile_delta = delta.get("compile_cold_delta", {})
    if isinstance(compile_delta, dict):
        for baseline, row in compile_delta.items():
            if not isinstance(row, dict):
                continue
            lines.append(
                f"| {baseline} | {float(row.get('before', 0.0)):.3f} | "
                f"{float(row.get('after', 0.0)):.3f} | "
                f"{float(row.get('delta_abs', 0.0)):.3f} | "
                f"{float(row.get('delta_pct', 0.0)):.2f}% |"
            )
    lines.append("")
    lines.append("Interpretation: negative delta is improvement (ratio got smaller).")
    lines.append("")
    return "\n".join(lines)


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n")


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content + "\n")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--baseline-results",
        required=True,
        help="Baseline results path relative to repo root.",
    )
    parser.add_argument(
        "--candidate-results",
        default="reports/benchmarks/third_party/full/results.json",
        help="Candidate results path relative to repo root.",
    )
    parser.add_argument(
        "--output-dir",
        default="reports/benchmarks/third_party/analysis/deltas",
        help="Output directory for delta artifacts.",
    )
    parser.add_argument(
        "--publication-mode",
        action="store_true",
        help="Require baseline/candidate to be strict publication reports.",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[2]
    baseline_path = repo_root / args.baseline_results
    candidate_path = repo_root / args.candidate_results
    output_dir = repo_root / args.output_dir

    if not baseline_path.exists():
        fail(f"baseline results missing: {baseline_path}")
    if not candidate_path.exists():
        fail(f"candidate results missing: {candidate_path}")

    baseline = json.loads(baseline_path.read_text())
    candidate = json.loads(candidate_path.read_text())
    expect_format(baseline, "baseline")
    expect_format(candidate, "candidate")
    if args.publication_mode:
        validate_publication_report(baseline, "baseline")
        validate_publication_report(candidate, "candidate")
        baseline_ref = str(
            (baseline.get("tooling", {}) if isinstance(baseline.get("tooling", {}), dict) else {}).get(
                "plbci_ref", ""
            )
        ).strip()
        candidate_ref = str(
            (candidate.get("tooling", {}) if isinstance(candidate.get("tooling", {}), dict) else {}).get(
                "plbci_ref", ""
            )
        ).strip()
        if baseline_ref != candidate_ref:
            fail(
                "publication-mode requires baseline and candidate to use identical "
                f"plbci_ref values (baseline={baseline_ref}, candidate={candidate_ref})"
            )

    delta = build_delta(
        baseline=baseline,
        candidate=candidate,
        baseline_label=str(baseline_path.relative_to(repo_root)),
        candidate_label=str(candidate_path.relative_to(repo_root)),
    )
    stamp = str(delta["timestamp_id"])
    json_path = output_dir / f"{stamp}_delta.json"
    md_path = output_dir / f"{stamp}_delta.md"
    latest_json = output_dir / "latest_delta.json"
    latest_md = output_dir / "latest_delta.md"

    write_json(json_path, delta)
    write_text(md_path, to_markdown(delta))
    write_json(latest_json, delta)
    write_text(latest_md, to_markdown(delta))
    print(f"wrote {json_path}")
    print(f"wrote {md_path}")
    print(f"wrote {latest_json}")
    print(f"wrote {latest_md}")


if __name__ == "__main__":
    main()
