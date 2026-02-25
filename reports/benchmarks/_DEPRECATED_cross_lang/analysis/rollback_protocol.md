# Benchmark Regression Rollback Protocol

## Trigger Conditions

Initiate rollback decision if any of the following hold after rerun policy is applied:

- geomean budget violation remains reproducible.
- hotspot case regression remains above threshold in two consecutive reruns.
- correctness parity is compromised (checksum or ops mismatch).

## Decision Flow

1. Confirm regression is reproducible on at least one authoritative run.
2. Identify change window and suspect component.
3. Choose one path:
   - fast rollback of suspected patch set, or
   - guarded disablement (feature toggle/fallback) if rollback is too broad.
4. Re-run quick + full benchmark profiles.
5. Publish delta evidence and incident closure note.

## Rollback Execution Checklist

- [ ] isolate commit range or patch files
- [ ] revert or disable with minimal blast radius
- [ ] run `cargo test -q -p vibe_runtime` and `cargo test -q -p vibe_cli`
- [ ] run quick benchmark profile and validate budgets
- [ ] run full benchmark profile and validate budgets
- [ ] regenerate delta report (`latest_delta.json`/`latest_delta.md`)

## Communication Requirements

- Add/update incident entry using `regression_triage_template.md`.
- Include before/after ratio tables for impacted cases.
- Record whether rollback is temporary or permanent.

## Exit Criteria

- no blocking budget violations
- no unresolved correctness issues
- updated analysis docs with evidence links
