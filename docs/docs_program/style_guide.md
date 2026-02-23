# Documentation Style Guide

Date: 2026-02-22

## Tone

- Technical and direct.
- Explain why/when, not only what.
- Prefer deterministic language for normative behavior (`MUST`, `SHOULD`, `MAY`).

## Examples

- Provide runnable snippets where practical.
- Prefer `.yb` syntax for new examples.
- Include expected output when it improves debugging.

## Conventions

- Commands in fenced `bash` blocks.
- Vibe source snippets in fenced `vibe` blocks for snippet CI execution.
- Cross-reference canonical docs instead of duplicating normative text.

## Glossary And Terms

- Reuse terms from `docs/spec/spec_glossary.md`.
- Avoid introducing synonyms for established terms in spec docs.

## Update Rules

- Behavior-changing PRs must update docs in same change.
- New language/tooling surfaces must include:
  - reference update,
  - tutorial/book touchpoint,
  - snippet coverage where feasible.
