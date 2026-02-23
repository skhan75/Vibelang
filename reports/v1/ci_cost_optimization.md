# V1 CI Cost Optimization Report

Date: 2026-02-23
Owner: Release/CI

## Goal

Reduce recurring GitHub Actions spend while preserving blocking release safety checks.

## Implemented Controls

- Added workflow-level concurrency cancellation to all workflows:
  - `group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}`
  - `cancel-in-progress: true`
- Narrowed branch triggers from `["**"]` to `["main", "release/**"]` for phase workflows.
- Added path filters to all workflows so unrelated changes no longer trigger full CI fan-out.
- Removed `pull_request` trigger from `v1-release-gates.yml` (kept `push` + `workflow_dispatch`).
- Optimized `v1-packaged-release.yml` PR behavior:
  - PR packaging lane now builds Linux target only.
  - Signing/provenance and multi-OS install smoke lanes are skipped on PRs and reserved for push/release runs.
- Added Rust dependency/build caching (`Swatinem/rust-cache@v2`) to Rust jobs.
- Reduced artifact retention footprint with `retention-days: 3` on upload steps.

## Workflows Updated

- `.github/workflows/phase1-frontend.yml`
- `.github/workflows/phase2-native.yml`
- `.github/workflows/phase3-concurrency.yml`
- `.github/workflows/phase4-indexer-lsp.yml`
- `.github/workflows/phase5-ai-sidecar.yml`
- `.github/workflows/phase6-extension-parity.yml`
- `.github/workflows/phase6-metrics.yml`
- `.github/workflows/phase6-portability.yml`
- `.github/workflows/phase7-language-validation.yml`
- `.github/workflows/phase7-readme-quality.yml`
- `.github/workflows/phase13-editor-ux.yml`
- `.github/workflows/v1-cli-ux.yml`
- `.github/workflows/docs-quality.yml`
- `.github/workflows/release-notes-automation.yml`
- `.github/workflows/v1-packaged-release.yml`
- `.github/workflows/v1-release-gates.yml`

## Expected Cost Impact

- Lower duplicate-run spend from canceled stale branch/PR runs.
- Lower PR spend by avoiding release-gates duplication and restricting packaged-release PR matrix.
- Lower compute minutes per Rust job due to dependency/cache reuse.
- Lower artifact storage cost via short retention defaults.

## Verification Commands

```bash
gh run list --limit 50 --json name,status,conclusion,headBranch,headSha
gh run list --workflow "v1-packaged-release.yml" --limit 10 --json databaseId,url,conclusion
gh run list --workflow "v1-release-gates.yml" --limit 10 --json databaseId,url,conclusion
```
