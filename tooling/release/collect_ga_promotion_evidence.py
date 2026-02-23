#!/usr/bin/env python3
import hashlib
import json
from pathlib import Path


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(path: Path) -> None:
    if not path.exists():
        raise SystemExit(f"missing required GA evidence input: {path}")


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    reports_v1 = repo_root / "reports" / "v1"
    reports_v1.mkdir(parents=True, exist_ok=True)

    hosted_inputs = reports_v1 / "hosted_rc_cycle_inputs.json"
    require(hosted_inputs)
    raw = json.loads(hosted_inputs.read_text())
    cycles = raw.get("cycles", [])
    if not isinstance(cycles, list) or len(cycles) < 2:
        raise SystemExit("hosted RC cycle evidence requires at least two cycles")
    for cycle in cycles:
        if cycle.get("status") != "pass":
            raise SystemExit(f"hosted RC cycle not passing: {cycle}")
        if not cycle.get("run_link"):
            raise SystemExit(f"hosted RC cycle missing run_link: {cycle}")

    phase_exit_evidence = [
        "reports/v1/selfhost_readiness.md",
        "reports/v1/selfhost_m2_readiness.md",
        "reports/v1/selfhost_m3_expansion.md",
        "reports/v1/spec_readiness.md",
        "reports/v1/dynamic_containers_conformance.md",
        "reports/v1/phase11_containers_text_readiness.md",
        "reports/v1/phase11_async_thread_readiness.md",
        "reports/v1/phase11_module_composition_readiness.md",
        "reports/v1/phase12_stdlib_readiness.md",
        "reports/v1/phase12_package_ecosystem_readiness.md",
        "reports/v1/phase12_qa_ecosystem_readiness.md",
        "reports/phase13/debugging_workflow.md",
        "reports/phase13/observability_primitives.md",
        "reports/phase13/crash_repro_sample.md",
        "reports/docs/documentation_quality.md",
    ]
    audit_rows: list[dict[str, str]] = []
    for rel in phase_exit_evidence:
        path = repo_root / rel
        require(path)
        audit_rows.append({"artifact": rel, "status": "present"})

    freeze_manifest_inputs = [
        "reports/v1/distribution_readiness.md",
        "reports/v1/install_independence.md",
        "reports/v1/security_response_exercise.md",
        "reports/v1/release_notes_preview.md",
        "reports/v1/readiness_dashboard.md",
        "reports/v1/release_candidate_checklist.md",
        "reports/v1/hosted_rc_cycle_inputs.json",
    ]
    checksums: dict[str, str] = {}
    for rel in freeze_manifest_inputs:
        path = repo_root / rel
        require(path)
        checksums[rel] = sha256_file(path)

    hosted_json = {
        "format": "v1-hosted-rc-cycles-v1",
        "cycle_count": len(cycles),
        "cycles": cycles,
    }
    (reports_v1 / "hosted_rc_cycles.json").write_text(json.dumps(hosted_json, indent=2) + "\n")
    (reports_v1 / "hosted_rc_cycles.md").write_text(
        "# Consecutive Hosted RC Cycles\n\n"
        f"- cycle_count: {len(cycles)}\n"
        + "\n".join(
            f"- `{cycle['cycle_id']}`: `{cycle['run_link']}` ({cycle['status']})"
            for cycle in cycles
        )
        + "\n"
    )

    phase_audit_json = {"format": "v1-phase10-13-exit-audit-v1", "artifacts": audit_rows}
    (reports_v1 / "phase10_13_exit_audit.json").write_text(
        json.dumps(phase_audit_json, indent=2) + "\n"
    )
    (reports_v1 / "phase10_13_exit_audit.md").write_text(
        "# Phase 10-13 Exit Audit\n\n"
        + "\n".join(f"- {row['artifact']}: {row['status']}" for row in audit_rows)
        + "\n"
    )

    freeze_manifest = {
        "format": "v1-ga-freeze-manifest-v1",
        "checksums_sha256": checksums,
    }
    (reports_v1 / "ga_freeze_bundle_manifest.json").write_text(
        json.dumps(freeze_manifest, indent=2) + "\n"
    )
    (reports_v1 / "ga_freeze_bundle_manifest.md").write_text(
        "# GA Freeze Bundle Manifest\n\n"
        + "\n".join(f"- `{path}`: `{digest}`" for path, digest in checksums.items())
        + "\n"
    )

    (reports_v1 / "ga_readiness_announcement.md").write_text(
        "# VibeLang GA Readiness Announcement\n\n"
        "- status: ready-for-ga\n"
        "- decision_date: 2026-02-22\n"
        "- hosted_rc_cycles: `reports/v1/hosted_rc_cycles.md`\n"
        "- phase_exit_audit: `reports/v1/phase10_13_exit_audit.md`\n"
        "- freeze_manifest: `reports/v1/ga_freeze_bundle_manifest.md`\n\n"
        "## Support and Limitations Matrix\n\n"
        "| Dimension | Source | Status |\n"
        "| --- | --- | --- |\n"
        "| Tier target support | `docs/targets/support_matrix.md` | Active |\n"
        "| LTS/support windows | `docs/support/lts_support_windows.md` | Active |\n"
        "| Compatibility guarantees | `docs/policy/compatibility_guarantees.md` | Active |\n"
        "| Known limitations | `docs/release/known_limitations_gate.md` | Accepted + published |\n"
        "| Breaking changes communication | `reports/v1/release_notes_preview.md` | Automated |\n"
    )

    print(f"wrote {reports_v1 / 'ga_readiness_announcement.md'}")


if __name__ == "__main__":
    main()
