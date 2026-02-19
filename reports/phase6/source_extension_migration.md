# Phase 6.0 Source Extension Migration Report

Date: 2026-02-17

## Scope

This report captures Phase 6.0 migration from `.vibe` to `.yb` across:

- source discovery and changed-file detection,
- metadata roots (`index`, `artifacts`, `cache`) defaults,
- CLI/indexer/LSP behavior,
- parity tests and CI workflow gates.

Policy references:

- `docs/policy/source_extension_policy_v1x.md`
- `docs/migrations/v1_0_source_extension_transition.md`

## Implemented Migration Behavior

### Source Extensions

- Canonical source extension is now `.yb`.
- Legacy `.vibe` remains supported in the same command flows.
- Dual-extension support is implemented in:
  - CLI source collection and recursive scanning,
  - git changed-file detection for `lint --changed`,
  - indexer watcher hashing and change detection.

### Metadata Roots

- Default metadata root moved from `.vibe/` to `.yb/`.
- Default roots now resolve to:
  - `.yb/index`
  - `.yb/artifacts`
  - `.yb/cache` (documented path)
- Legacy bootstrap compatibility is implemented:
  - if `.yb/index/index_v1.json` is missing and `.vibe/index/index_v1.json` exists,
    the snapshot is copied into `.yb/index` before index open.

### Safety Guard

- Directory scans now fail fast if both `foo.vibe` and `foo.yb` are present in the same directory.
- Rationale: avoid deterministic artifact clobbering for `<stem>.o`, `<stem>`, and `<stem>.debug.map`.

## Verification Evidence

### Local Verification Commands

- `cargo fmt --all`
- `cargo test -p vibe_indexer`
- `cargo test -p vibe_lsp`
- `cargo test -p vibe_cli --test phase2_native`
- `cargo test -p vibe_cli --test phase4_indexer`
- `cargo test -p vibe_cli --test phase5_intent_lint`
- `cargo test -p vibe_cli --test frontend_fixtures`
- `cargo clippy --workspace --all-targets -- -D warnings`

### Result

- All listed commands passed.
- Added migration-focused tests passed, including:
  - `.yb` source indexing/building/running,
  - mixed-extension directory support when stems differ,
  - conflict guard when stems collide across `.vibe`/`.yb`,
  - legacy `.vibe/index` bootstrap into `.yb/index`.

## CI Gate Updates

Updated workflows to include `.vibe` + `.yb` parity smoke and/or `.yb` metadata path expectations:

- `.github/workflows/phase1-frontend.yml`
- `.github/workflows/phase2-native.yml`
- `.github/workflows/phase3-concurrency.yml`
- `.github/workflows/phase4-indexer-lsp.yml`
- `.github/workflows/phase5-ai-sidecar.yml`
- `.github/workflows/phase6-extension-parity.yml`

### Command Parity Matrix

| Command | `.vibe` | `.yb` | Evidence |
| --- | --- | --- | --- |
| `check` | PASS | PASS | phase1 + phase6 extension parity workflow |
| `build` | PASS | PASS | phase2 + phase6 extension parity workflow |
| `run` | PASS | PASS | phase2/phase3 + phase6 extension parity workflow |
| `test` | PASS | PASS | phase2 + phase6 extension parity workflow |
| `lint --intent` | PASS | PASS | phase5 + phase6 extension parity workflow |
| `index` | PASS | PASS | phase4 + phase6 extension parity workflow |
| `lsp` | PASS | PASS | phase6 extension parity lsp smoke |

## Known Limitations

- Same-directory same-stem dual files are intentionally rejected to prevent artifact collisions.
- Historical Phase 1-5 reports may still reference `.vibe` in frozen evidence text; migration keeps these as historical artifacts unless they are active runbooks.

## Deprecation Path

- Release N: dual support (`.yb` canonical, `.vibe` legacy).
- Release N+1: optional warning for new `.vibe` creation in scaffolding/docs.
- Release N+2: evaluate `.vibe` removal only if adoption and parity gates remain green.

Removal gates are policy-bound and must hold for 3 consecutive releases:

- `.yb` adoption ratio >= 90%
- dual-extension parity pass rate >= 99%
- zero open critical migration regressions

