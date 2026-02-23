# Docs Engine And Repository Layout

Date: 2026-02-22

## Engine Choice

- Engine: **mdBook-style** source tree.
- Rationale:
  - simple Markdown-first authoring,
  - deterministic build pipeline,
  - straightforward CI snippet validation.

## Layout

- `book/book.toml` - book metadata/config.
- `book/src/SUMMARY.md` - chapter index and ordering.
- `book/src/ch*.md` - chapter sources.
- `docs/spec/` - normative specification suite.
- `docs/docs_program/` - docs governance and authoring policy.
- `reports/docs/` - release docs quality artifacts.

## Build And Validation Contract

- Book content is validated in CI via docs-quality workflow.
- Snippets in `book/src/` and selected reference docs must pass executable checks.
- Link/spell/stale-example checks are blocking for docs gate.
