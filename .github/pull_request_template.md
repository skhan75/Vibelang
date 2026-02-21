# Summary

Describe why this change exists and what user-facing behavior changed.

## Test Plan

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] relevant `cargo test` commands

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
