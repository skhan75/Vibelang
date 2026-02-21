# Phase 8 CI Evidence Capture

Date: 2026-02-21

## Objective

Track evidence capture for Phase 8 closure workflows:

- `.github/workflows/v1-cli-ux.yml`
- `.github/workflows/v1-packaged-release.yml`
- `.github/workflows/v1-release-gates.yml`

## Remote Workflow URL Capture Status

- Attempted GitHub API lookup for recent workflow runs:
  - `v1-release-gates.yml`
  - `v1-cli-ux.yml`
  - `v1-packaged-release.yml`
- Result: API returned `404` in this execution environment, so direct run URL
  capture is not available from here.

## Local Workflow-Equivalent Evidence

### v1-cli-ux equivalent commands

- `cargo test -p vibe_cli --test cli_help_snapshots`
- `cargo test -p vibe_cli --test cli_version`

Status: pass.

### v1-release-gates subset equivalent commands

- `cargo test -p vibe_cli --test phase2_native deterministic_build_binary_and_metadata`
- `cargo test -p vibe_cli --test frontend_fixtures phase7_basic_and_intermediate_matrix`
- `cargo test -p vibe_cli --test frontend_fixtures phase7_frontend_outputs_are_deterministic`
- `cargo test -p vibe_cli --test phase7_v1_tightening`

Status: pass.

### v1-packaged-release Linux equivalent flow

Executed local packaged-install simulation with reproducibility manifest compare:

- build locked release binary
- package `vibe-x86_64-unknown-linux-gnu.tar.gz`
- generate checksum file
- build + compare checksum manifest
- extract package and run:
  - `vibe --version`
  - `vibe run hello.yb`

Status: pass.

Local artifact links:

- `/tmp/phase8_packaged_local/dist/vibe-x86_64-unknown-linux-gnu.tar.gz`
- `/tmp/phase8_packaged_local/dist/checksums-x86_64-unknown-linux-gnu.txt`
- `/tmp/phase8_packaged_local/current-manifest.json`
- `/tmp/phase8_packaged_local/repro-report.md`
- `/tmp/phase8_packaged_local/version.txt`
- `/tmp/phase8_packaged_local/run-output.txt`

## Follow-Up for Full CI Closure

- Attach first successful GitHub workflow run URLs and hosted artifact links
  once available from the repository CI environment.

## Phase 8 Exit-Criteria Mapping

| Exit Criterion | Evidence | Current Status |
| --- | --- | --- |
| Fresh machine without Rust/Cargo can install and run from packaged artifacts | `reports/v1/install_independence.md`, local packaged-run artifacts under `/tmp/phase8_packaged_local`, workflow wiring in `.github/workflows/v1-packaged-release.yml` | partial (local-pass + workflow-wired; first hosted cross-platform CI URLs pending) |
| `vibe --help` and `vibe --version` are stable and regression-tested | tests `cli_help_snapshots`, `cli_version`, docs `docs/cli/help_manual.md`, `docs/cli/version_output.md`, workflow `.github/workflows/v1-cli-ux.yml` | local-pass (hosted CI URL pending) |
| Packaged artifacts are signed/checksummed/policy-compliant for tier-1 | workflow jobs `packaged_reproducibility`, `sign_attest_and_sbom`, docs `docs/release/distribution_security.md`, report `reports/v1/distribution_readiness.md` | partial (pipeline wired; first hosted signed artifact cycle URL pending) |
