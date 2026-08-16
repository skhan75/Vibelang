# Release Evidence Bundle

Date: 2026-02-17. Scope narrowed 2026-08-16.

The release procedure itself lives in [RELEASING.md](../../RELEASING.md) at the
repository root, which is the only document that lists the steps. This file now
covers one thing: the evidence a release candidate has to link.

## Required Evidence Bundle

Every release candidate should include links to:

- `.yb` parity report (`reports/phase6/source_extension_migration.md`)
- self-host conformance report (`reports/phase6/self_hosting_conformance.md`)
- metrics artifacts under `reports/phase6/metrics/`
- support matrix and known limitations docs
- v1 readiness dashboard (`reports/v1/readiness_dashboard.md`)
- v1 release candidate checklist (`reports/v1/release_candidate_checklist.md`)
- self-host readiness report (`reports/v1/selfhost_readiness.md`)
- spec readiness report (`reports/v1/spec_readiness.md`)
- install independence report (`reports/v1/install_independence.md`)
- distribution readiness report (`reports/v1/distribution_readiness.md`)
- phase8 CI evidence index (`reports/v1/phase8_ci_evidence.md`)
- phase8 closeout summary (`reports/v1/phase8_closeout_summary.md`)

## Required Operational Docs

- `docs/release/rc_process.md`
- `docs/release/rollback_playbook.md`
- `docs/release/release_blocker_policy.md`
- `docs/release/known_limitations_gate.md`
- `docs/support/issue_triage_sla.md`
- `docs/privacy/telemetry_statement.md`
