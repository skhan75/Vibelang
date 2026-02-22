# Phase 13.1 Baseline Contract

Date: 2026-02-22

## Scope lock

Phase 13.1 implementation scope is locked to the checklist items in
`docs/development_checklist.md` section `13.1 IDE and Workflow Maturity`:

1. Upgrade LSP from diagnostics-focused to productivity-focused
2. Add large-workspace performance benchmarks for indexer/LSP paths
3. Add editor/CI consistency checks for local-vs-CI behavior
4. Publish IDE setup guides and recommended workflows

## Baseline implementation state

- LSP transport is currently line-stdio command JSON in `crates/vibe_lsp/src/lib.rs`.
- Existing LSP features are diagnostics, definition, references, and hover metadata.
- Formatter exists in `crates/vibe_fmt/src/lib.rs` and is available through `vibe fmt`.
- Grammar source of truth exists in `docs/spec/grammar_v1_0.ebnf`.
- Fixture corpus is available in `compiler/tests/fixtures/`.
- No `editor-support/vscode` package or `docs/ide` setup guide existed at baseline.

## Guardrails and assumptions

- Parser, typechecker, and indexer remain the authoritative semantic engines.
- Editor-facing features are adapters layered on existing compiler/index data.
- JSON-RPC transport migration keeps a temporary legacy fallback path.
- Formatter behavior in editor must be byte-equivalent to `vibe fmt`.
- Phase 13.1 checklist items are checked only after evidence artifacts exist.

## Completion evidence expectations

The following artifacts are required before 13.1 checklist closure:

- `editor-support/vscode` extension package and language assets
- JSON-RPC LSP protocol tests and CI lane
- Large-workspace benchmark report with explicit thresholds
- Editor-vs-CLI diagnostics consistency report
- IDE setup documentation for VS Code/Cursor

