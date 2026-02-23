# VibeLang Unsafe Escape Hatch Contract (v1.0)

Status: normative policy for low-level escape hatches in v1.

## Purpose

Define the only allowed unsafe escape-hatch surface in v1 and keep it auditable.

## Scope Boundaries

- Unsafe escape hatches are allowed only for low-level runtime/interop boundaries.
- Unsafe markers must not appear in ordinary application logic when a safe
  equivalent exists.
- Unsafe use does not waive determinism, diagnostics ordering, or release-gate
  requirements.

## Marker Syntax (v1 policy)

Unsafe scope must be delimited with comment markers:

```txt
// @unsafe begin: <reason>
// @unsafe review: <ticket-or-change-id>
// @unsafe end
```

Rules:

- `@unsafe begin:` requires a non-empty reason.
- `@unsafe review:` is mandatory between begin/end.
- `@unsafe end` must match a prior begin marker.
- Nested unsafe blocks are rejected.

## Required Review Path

- Every unsafe block must link to explicit review evidence via
  `@unsafe review:`.
- Review references must map to a tracked change/review in release governance.
- See `docs/release/unsafe_review_policy.md` for process requirements.

## Build-Time Audit Contract

- `vibe build` must emit an unsafe audit artifact per build:
  - `<artifact-stem>.unsafe.audit.json`
- The report lists:
  - file path
  - begin/end lines
  - reason
  - review reference
- Any unsafe marker policy violation fails the build.

## Determinism Requirements

- For identical source and toolchain, unsafe audit output must be byte-stable.
- Violation messages and ordering must remain deterministic.
