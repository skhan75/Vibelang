# Summary

Describe why this change exists and what user-facing behavior changed.

## Checklist IDs (required)

- [ ] IDs addressed from `docs/checklists/features_and_optimizations.md` (example: `A-02`, `C-01`)

## Acceptance Evidence (required)

- [ ] Evidence/report path(s) proving acceptance criteria are met
- [ ] If examples are impacted: list exact files moved from fail -> pass

## Test Plan

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] relevant `cargo test` commands
- [ ] Regression tests added/updated for the checklist ID(s)

## Release-Gate Reports (required for release PRs)

If this PR is a release PR (for example, branch starts with `release/`), include:

- [ ] `reports/v1/readiness_dashboard.md`
- [ ] `reports/v1/release_candidate_checklist.md`
- [ ] `reports/v1/install_independence.md`
- [ ] `reports/v1/distribution_readiness.md`
- [ ] `reports/v1/phase8_ci_evidence.md`
- [ ] `reports/v1/phase8_closeout_summary.md`
- [ ] `reports/v1/selfhost_m2_readiness.md`
- [ ] `reports/v1/selfhost_m3_expansion.md`
- [ ] `reports/v1/selfhost_readiness.md`
- [ ] `docs/selfhost/m4_transition_criteria.md`
- [ ] `docs/release/selfhost_transition_playbook.md`
- [ ] artifact link: `v1-selfhost-m3-shadow`
- [ ] artifact link: `v1-selfhost-m4-rc-cycle`
- [ ] link to successful `v1-release-gates.yml` run
