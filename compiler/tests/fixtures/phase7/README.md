# Phase 7 Fixture Taxonomy

This directory hosts the Phase 7.1 validation corpus.

## Progression Levels

- `basic/`: parser and type-system fundamentals.
- `intermediate/`: annotation/contracts/effect semantics.
- `advanced/`: end-to-end single-thread and concurrency programs.
- `stress/`: bounded stress and stability scenarios.

## Naming Convention

Use:

- `<domain>__<scenario>.yb` for source fixtures.
- `<domain>__<scenario>.diag` for expected diagnostics (when applicable).

Examples:

- `syntax__literals_and_comments.yb`
- `annotations__unknown_tag.diag`
- `concurrency__worker_pool.yb`

## Diagnostic Fixture Rules

- Keep `.diag` outputs deterministic and sorted.
- Include parser/type/effect/concurrency diagnostics exactly as emitted.
- Update goldens via `UPDATE_GOLDEN=1` only after reviewing diffs.
