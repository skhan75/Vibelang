# Linux Runtime Compatibility Policy (v1)

Date: 2026-02-21

## Purpose

Define the minimum Linux runtime compatibility contract for packaged `vibe`
artifacts so end-user installs remain stable on common Ubuntu/WSL baselines.

## Compatibility Baseline

- Primary Linux package target: `x86_64-unknown-linux-gnu`
- Minimum supported glibc baseline for the packaged GNU artifact: `2.35`
- Validation environments:
  - `ubuntu-22.04` (compatibility floor)
  - `ubuntu-latest` (forward compatibility lane)

## Release Rules

- The GNU Linux packaged artifact MUST be built in a baseline-compatible
  environment.
- The packaged Linux artifact MUST execute successfully in both validation
  environments with:
  - `vibe --version`
  - `vibe run hello.yb`
- Any detected runtime ABI drift (for example a `GLIBC_*` requirement above
  baseline) is a release blocker.

## CI Enforcement

- Workflow `.github/workflows/v1-packaged-release.yml`:
  - Linux package built on `ubuntu-22.04`
  - GLIBC symbol compatibility check step in Linux packaging lane
  - Install smoke jobs:
    - `install_smoke_linux` (`ubuntu-22.04`)
    - `install_smoke_linux_latest` (`ubuntu-latest`)
- Workflow `.github/workflows/v1-release-gates.yml`:
  - Blocking `linux_compatibility_gate` matrix lane on `ubuntu-22.04` and
    `ubuntu-latest`

## Fallback Strategy

If the GNU baseline cannot be maintained without unacceptable regressions, ship
an additional static Linux fallback artifact (`musl`) with equivalent signing,
checksum, provenance, and install-smoke evidence requirements.

## Evidence Sources

- `reports/v1/install_independence.md`
- `reports/v1/distribution_readiness.md`
- `reports/v1/phase8_ci_evidence.md`
