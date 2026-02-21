# Phase 8.6 + Phase 9 Development Plan

Date: 2026-02-21

## Purpose

Provide a concrete, execution-ready plan for:

- Phase `8.6` Linux installer compatibility follow-up
- Phase `9.1` through `9.4` progressive self-host transition

This plan is the implementation companion to `docs/development_checklist.md`.

## Scope

In scope:

- Linux packaged installer ABI compatibility hardening (`8.6`)
- M2 tooling self-host expansion (`9.1`)
- M3 frontend shadow expansion (`9.2`)
- M4 default switch strategy with rollback (`9.3`)
- Governance, ownership, and release gate wiring (`9.4`)

Out of scope:

- New language-surface features unrelated to self-host transition
- Broad target-matrix expansion beyond current tier priorities, except where needed to prove compatibility gates

## Program Principles

- Determinism first: host vs self-host comparisons must be byte-for-byte where specified.
- Fail closed: any parity drift, missing artifact, or unmet threshold blocks promotion.
- Reversible promotion: every default-path change must keep a tested fallback path.
- Evidence-driven closure: checklist items close only with linked CI/report artifacts.

## Delivery Order

1. **Wave A**: Phase `8.6` compatibility stabilization
2. **Wave B**: Phase `9.1` M2 tooling parity lanes
3. **Wave C**: Phase `9.2` M3 shadow expansion + performance budgets
4. **Wave D**: Phase `9.3` M4 graduation + RC default-switch cycle
5. **Wave E**: Phase `9.4` governance + blocking release gate integration

`9.3` must not begin promotion until `9.1` and `9.2` parity thresholds are stable.

---

## Phase 8.6 Plan: Linux Installer Compatibility Follow-Up

### Objective

Ensure Linux packaged installs run on common Ubuntu/WSL baselines (glibc `2.35+`) without manual workarounds.

### Workstream 8.6-A: Compatibility Policy and Baseline Definition

- Define minimum supported Linux runtime ABI baseline in docs (`glibc` floor and distro examples).
- Capture representative validation environments:
  - `ubuntu-22.04` (WSL/common enterprise baseline)
  - `ubuntu-latest` (forward compatibility)
- Decide release policy:
  - primary: `x86_64-unknown-linux-gnu` built against baseline-compatible environment
  - fallback: static `musl` installer path (if baseline compatibility cannot be guaranteed)

### Workstream 8.6-B: Packaging Pipeline Adjustments

- Update `.github/workflows/v1-packaged-release.yml` Linux packaging lane to build with baseline-compatible host/toolchain.
- Keep checksum/signature/provenance/SBOM flow unchanged and deterministic.
- If `musl` fallback is enabled:
  - add explicit artifact naming and docs path
  - add verification and install-smoke lane parity with gnu path

### Workstream 8.6-C: Blocking Compatibility Lanes

- Add blocking install-smoke checks for packaged Linux artifact on:
  - `ubuntu-22.04`
  - `ubuntu-latest`
- Verify both:
  - `vibe --version`
  - `vibe run hello.yb`
- Assert no dynamic loader ABI errors (for example, missing `GLIBC_*` symbols).

### Workstream 8.6-D: Documentation and Evidence

- Update:
  - `docs/install/linux.md` (minimum runtime requirements + fallback path)
  - `reports/v1/install_independence.md` (compatibility lane evidence)
  - `reports/v1/distribution_readiness.md` (artifact compatibility status)
- Add entry in `docs/targets/limitations_register.md` if residual caveats remain.

### Exit Conditions (8.6)

- Linux packaged artifact passes install-smokes on both baseline and latest Ubuntu lanes.
- Installer docs include explicit runtime requirements and fallback instructions.
- Checklist `8.6` bullets link to CI/report evidence and are marked complete.

---

## Phase 9.1 Plan: M2 Self-Host Expansion (Tooling Components)

### Objective

Promote select tooling components from seed-only self-hosting to repeatable, release-gated parity execution.

### Target Components

- Docs formatter component(s)
- Diagnostics formatter component(s)

### Implementation Tasks

- Publish component contracts and boundaries:
  - `docs/selfhost/m2_formatter_diagnostics_contract.md`
- Build parity harnesses that compare host and self-host outputs across canonical fixture corpus.
- Add deterministic repeat-run tests for each M2 component:
  - same input, same toolchain, identical output across repeated runs.
- Add CI jobs for M2 parity and determinism (candidate name: `selfhost_m2_gate`).
- Publish readiness report:
  - `reports/v1/selfhost_m2_readiness.md`

### Exit Conditions (9.1)

- M2 component outputs are byte-identical to host outputs across agreed fixtures.
- Determinism repeat-run tests pass in CI for consecutive runs.
- M2 readiness report includes scope, pass rates, and unresolved drift items.

---

## Phase 9.2 Plan: M3 Expansion (Compiler Frontend Slices in Shadow Mode)

### Objective

Expand M3 from starter slice to multiple frontend slices with enforced shadow parity and bounded overhead.

### Target Slices

- Parser diagnostics normalization
- Type diagnostic ordering
- Selected MIR formatting/normalization paths

### Implementation Tasks

- Implement self-host shadow paths for each slice while keeping host path authoritative.
- Introduce host + self-host dual-run comparison in CI:
  - block on parity drift
  - retain detailed diff artifacts for debugging
- Define and enforce shadow overhead budgets:
  - latency ceiling
  - memory overhead ceiling
- Publish evidence report:
  - `reports/v1/selfhost_m3_expansion.md`

### Exit Conditions (9.2)

- All selected M3 slices pass shadow parity checks in CI.
- Performance budgets are measured and enforced with blocking thresholds.
- M3 report includes parity trends and overhead metrics.

---

## Phase 9.3 Plan: M4 Transition Gate (Default Switch Strategy)

### Objective

Safely promote at least one self-host component to default path with proven rollback and RC-cycle evidence.

### Implementation Tasks

- Define promotion criteria:
  - `docs/selfhost/m4_transition_criteria.md`
- Add explicit per-component fallback toggles (host path instantly recoverable).
- Choose first promotion candidate (recommendation: smallest high-signal diagnostics component).
- Execute at least one RC cycle with promoted default:
  - no parity regressions
  - no unresolved production-impact incidents
- Publish transition playbook:
  - `docs/release/selfhost_transition_playbook.md`

### Exit Conditions (9.3)

- One or more components run self-host by default in RC without parity drift.
- Rollback path is documented, tested, and fast to activate.
- Promotion decision and evidence are recorded in RC reports.

---

## Phase 9.4 Plan: Governance, Ownership, and Reporting

### Objective

Operationalize self-host transition with clear owners, blocking gates, and repeatable release governance.

### Implementation Tasks

- Extend `reports/v1/selfhost_readiness.md` with component-level matrix:
  - component
  - current mode (host/shadow/default-selfhost)
  - parity counters
  - owner/signoff state
- Publish ownership matrix:
  - `docs/selfhost/component_ownership.md`
- Add blocking workflow gate:
  - `selfhost_transition_gate` in `.github/workflows/v1-release-gates.yml`
- Update PR/release template requirements for parity evidence attachments.
- Fold Phase `8.6` compatibility evidence into governance dashboard if still open at this stage.

### Exit Conditions (9.4)

- Self-host ownership and incident-response model is documented and active.
- `selfhost_transition_gate` blocks release on missing/failed parity evidence.
- Reports are release-auditable and linked from RC checklist.

---

## Cross-Phase Risks and Mitigations

### Risk 1: Parity Drift Under Real Workloads

- Mitigation: fixture expansion + shadow diff artifacts + blocking CI thresholds.

### Risk 2: Performance Regressions in Self-Host Paths

- Mitigation: explicit latency/memory budgets with trend tracking and hard fail gates.

### Risk 3: Irreversible Promotion

- Mitigation: mandatory fallback toggles and rollback drills before default switch.

### Risk 4: Linux Runtime ABI Drift in Packaged Installs

- Mitigation: baseline Ubuntu compatibility lane, documented runtime requirements, optional `musl` fallback artifact.

## Evidence Map (Checklist Alignment)

| Checklist Scope | Primary Evidence Docs | Primary CI Gates |
| --- | --- | --- |
| 8.6 | `docs/install/linux.md`, `reports/v1/install_independence.md`, `reports/v1/distribution_readiness.md` | `v1-packaged-release.yml` Linux install-smoke compatibility lanes |
| 9.1 | `docs/selfhost/m2_formatter_diagnostics_contract.md`, `reports/v1/selfhost_m2_readiness.md` | `selfhost_m2_gate` |
| 9.2 | `reports/v1/selfhost_m3_expansion.md` | shadow parity gate(s) + performance budget jobs |
| 9.3 | `docs/selfhost/m4_transition_criteria.md`, `docs/release/selfhost_transition_playbook.md` | RC cycle evidence + promoted component parity lanes |
| 9.4 | `docs/selfhost/component_ownership.md`, `reports/v1/selfhost_readiness.md` | `selfhost_transition_gate` in `v1-release-gates.yml` |

## Definition of Done

This plan is complete when:

- All checklist items in `8.6` and `9.1`-`9.4` are checked with linked evidence.
- Phase 9 exit criteria are satisfied with blocking CI and report artifacts.
- Remaining open release blockers, if any, are explicitly outside `8.6`/`9.x` scope.
