# Unsafe Block Review Policy

Date: 2026-02-22

## Purpose

Require explicit and auditable review for every unsafe escape hatch used in
VibeLang code.

## Required Marker Contract

Unsafe scopes must use:

```txt
// @unsafe begin: <reason>
// @unsafe review: <ticket-or-change-id>
// @unsafe end
```

`@unsafe review:` is mandatory and must reference a reviewable change record.

## Review Requirements

For each unsafe block, review must document:

- why safe alternatives were insufficient,
- scope boundary and invariants that must hold,
- failure modes and mitigation,
- test coverage validating the unsafe path.

## CI Enforcement

`vibe build` emits `<stem>.unsafe.audit.json` and fails when:

- begin/review/end markers are malformed,
- review marker is missing,
- nested or unclosed unsafe scopes exist.

Release gates must include an unsafe-governance job that validates:

- unsafe audit artifact generation,
- violation failure behavior,
- stable report structure.

## Release Evidence

Release candidates must attach:

- unsafe governance CI artifact summary,
- any unsafe audit reports produced by candidate builds,
- linked review references for all unsafe blocks in release scope.
