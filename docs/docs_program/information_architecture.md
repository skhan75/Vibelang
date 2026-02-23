# VibeLang Documentation Information Architecture

Date: 2026-02-22

## Purpose

Define a stable docs information architecture for onboarding, day-to-day
engineering, and deep implementation work.

## Primary Surfaces

1. **Tutorials / Book**
   - Goal: progressive onboarding and practical workflows.
   - Source: `book/src/`.
2. **Language and Runtime Reference**
   - Goal: normative spec and guarantees.
   - Source: `docs/spec/`.
3. **Tooling Guides**
   - Goal: command usage, CI integration, operational workflows.
   - Source: `docs/cli/`, `docs/ide/`, `docs/release/`, `docs/package/`.
4. **Operational Governance**
   - Goal: security, support, incident, release controls.
   - Source: `docs/security/`, `docs/support/`, `docs/policy/`.
5. **Internals**
   - Goal: compiler/runtime/indexer/LSP architecture and contributor workflows.
   - Source: `compiler/`, `runtime/`, and internals chapter in `book/`.

## User Journeys

- **New team onboarding**
  - Start in book chapters 1-4, then tooling chapter 7.
- **Feature implementer**
  - Use `docs/spec/` + chapter 10 internals.
- **Release/operations owner**
  - Use `docs/release/`, `docs/security/`, `docs/support/`.

## Navigation Rules

- Book chapters reference normative docs but never supersede them.
- Any normative behavior change must update:
  1. `docs/spec/*` rule,
  2. `docs/spec/spec_coverage_matrix.md`,
  3. relevant book/tutorial snippet.
