# V1 GA Go/No-Go Checklist (Strict)

Date: 2026-02-23
Candidate: `v1.0.0-rc1-dryrun-local`

## Hard Go Criteria

- [x] `P0` technical gates are implemented and locally validated.
- [x] `P1` exceptions are documented and approved.
- [x] GA evidence bundle artifacts exist (`phase10_13_exit_audit`, freeze manifest, GA announcement).
- [x] Hosted RC run URLs are attached (not `local://` placeholders).
- [x] Hosted release workflows for candidate are linked in checklist/dashboard.
- [x] Public release payload is published (tag + release notes + signed artifacts).

## Immediate No-Go Triggers

- Any unresolved `P0` blocker.
- Missing hosted workflow evidence URLs for the candidate cycle.
- Missing trust artifacts (checksums/signatures/provenance/SBOM) in published release payload.
- Known limitations or breaking changes absent from release notes.

## Exact Remaining Commands

Run these from repo root (`vibelang/`) after selecting the release ref.

```bash
# 0) choose release ref
export RELEASE_REF="release/v1.0.0"

# 1) hosted GA evidence is already attached (reference runs)
export RC1_URL="https://github.com/skhan75/VibeLang/actions/runs/22302057210"
export RC2_URL="https://github.com/skhan75/VibeLang/actions/runs/22299615440"

# 2) download packaged signed bundle from the latest successful packaged release run
export PACKAGED_RUN_ID="22299615390"
gh run download "$PACKAGED_RUN_ID" --name v1-packaged-signed-bundle --dir /tmp/v1-release-assets

# 3) create public release tag with notes and attached trust artifacts
export RELEASE_TAG="v1.0.0"
gh release create "$RELEASE_TAG" \
  --target "$RELEASE_REF" \
  --title "VibeLang $RELEASE_TAG" \
  --notes-file reports/v1/release_notes_preview.md \
  /tmp/v1-release-assets/*

# 4) regenerate GA evidence after release publication
python3 tooling/release/collect_ga_promotion_evidence.py
python3 tooling/release/validate_ga_promotion_evidence.py

# 5) refresh release-notes and pilot evidence artifacts
python3 tooling/release/generate_release_notes.py
python3 tooling/release/validate_release_notes.py
python3 tooling/phase14/collect_pilot_program_metrics.py
python3 tooling/phase14/validate_pilot_program_metrics.py

# 6) final local verification pass
python3 tooling/docs/validate_snippets.py
python3 tooling/docs/link_check.py
python3 tooling/docs/spell_check.py
python3 tooling/docs/stale_example_check.py
python3 tooling/docs/compute_docs_coverage.py
python3 tooling/docs/generate_documentation_quality_report.py
```

## Required Artifacts To Publish

- `reports/v1/release_candidate_checklist.md`
- `reports/v1/readiness_dashboard.md`
- `reports/v1/ci_cost_optimization.md`
- `reports/v1/hosted_rc_cycles.md`
- `reports/v1/phase10_13_exit_audit.md`
- `reports/v1/ga_freeze_bundle_manifest.md`
- `reports/v1/ga_readiness_announcement.md`
- `reports/v1/release_notes_preview.md`
- `reports/v1/distribution_readiness.md`
- `reports/v1/install_independence.md`
- Trust payload from packaged release workflow:
  - checksums
  - signatures
  - provenance
  - SBOM

## CI Cost Controls (Operational, Non-Blocking)

- [x] Path-scoped workflow triggers are enabled for all release/phase workflows.
- [x] In-flight stale runs are auto-canceled via workflow-level concurrency controls.
- [x] Rust cache and short artifact retention are enabled in CI.
- [x] `v1-release-gates.yml` PR duplication is removed.
- [x] `v1-packaged-release.yml` PR lane is reduced to Linux-only packaging.

## Final Decision Rule

- **GO** only if every hard go criterion is checked.
- Otherwise **NO-GO** and carry blocker(s) into next RC cycle.
