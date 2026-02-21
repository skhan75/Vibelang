## Summary

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
- [ ] link to successful `v1-release-gates.yml` run
