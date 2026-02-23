# V1 GA Go/No-Go Checklist (Strict)

Date: 2026-02-23
Candidate: `v1.0.0-rc1-dryrun-local`

## Hard Go Criteria

- [x] `P0` technical gates are implemented and locally validated.
- [x] `P1` exceptions are documented and approved.
- [x] GA evidence bundle artifacts exist (`phase10_13_exit_audit`, freeze manifest, GA announcement).
- [ ] Hosted RC run URLs are attached (not `local://` placeholders).
- [ ] Hosted release workflows for candidate are linked in checklist/dashboard.
- [ ] Public release payload is published (tag + release notes + signed artifacts).

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

# 1) trigger hosted gates and packaged release
gh workflow run "v1-release-gates.yml" --ref "$RELEASE_REF"
gh workflow run "v1-packaged-release.yml" --ref "$RELEASE_REF"
gh workflow run "release-notes-automation.yml" --ref "$RELEASE_REF"

# 2) fetch latest hosted run URLs (use these URLs in evidence docs)
gh run list --workflow "v1-release-gates.yml" --limit 2 --json databaseId,url,conclusion,headBranch
gh run list --workflow "v1-packaged-release.yml" --limit 2 --json databaseId,url,conclusion,headBranch
gh run list --workflow "release-notes-automation.yml" --limit 2 --json databaseId,url,conclusion,headBranch

# 3) replace placeholder run links in hosted RC inputs JSON
export RC1_URL="https://github.com/<org>/<repo>/actions/runs/<run-id-1>"
export RC2_URL="https://github.com/<org>/<repo>/actions/runs/<run-id-2>"
jq --arg rc1 "$RC1_URL" --arg rc2 "$RC2_URL" \
  '.cycles[0].run_link = $rc1 | .cycles[1].run_link = $rc2' \
  reports/v1/hosted_rc_cycle_inputs.json > /tmp/hosted_rc_cycle_inputs.json
mv /tmp/hosted_rc_cycle_inputs.json reports/v1/hosted_rc_cycle_inputs.json

# 4) regenerate GA evidence after URLs are updated
python3 tooling/release/collect_ga_promotion_evidence.py
python3 tooling/release/validate_ga_promotion_evidence.py

# 5) refresh release-notes and pilot evidence artifacts
python3 tooling/release/generate_release_notes.py
python3 tooling/release/validate_release_notes.py
python3 tooling/phase14/collect_pilot_program_metrics.py
python3 tooling/phase14/validate_pilot_program_metrics.py

# 6) final local verification pass before publishing
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
