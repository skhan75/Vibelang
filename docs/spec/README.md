# VibeLang Specification Suite

Status: draft normative suite for production-readiness hardening.

This directory contains the language and runtime specification artifacts for
VibeLang. The goal is to provide a single auditable source of truth for syntax,
semantics, runtime behavior, and release-gated language guarantees.

## How To Read This Spec

- Start with this file for structure and source-of-truth rules.
- Read normative docs before explanatory docs.
- Treat grammar and model docs as authoritative over examples.
- When docs conflict, follow the precedence order below.

## Source-Of-Truth Precedence

1. `docs/spec/grammar_v1_0.ebnf`
2. Normative model docs (`type_system`, `numeric_model`, memory/concurrency/ABI)
3. `docs/spec/syntax.md` and `docs/spec/semantics.md`
4. Examples and samples

Phase-1 bootstrap freeze artifacts remain available for traceability:

- `docs/spec/grammar_v0_1.ebnf`
- `docs/spec/phase1_resolved_decisions.md`

## Spec Taxonomy

### Normative

- `syntax.md` - language surface forms and statement/expr syntax policy.
- `semantics.md` - high-level semantic model and profile behavior.
- `grammar_v1_0.ebnf` - parser-level grammar source of truth.
- `type_system.md` - type categories, inference, assignability, coercions.
- `numeric_model.md` - integer/float representation and arithmetic rules.
- `mutability_model.md` - binding/assignment mutability and const semantics.
- `strings_and_text.md` - text encoding, escapes, indexing/slicing semantics.
- `containers.md` - dynamic `Str`/`List`/`Map` semantics and guarantees.
- `control_flow.md` - loop/control semantics, termination, and branch behavior.
- `concurrency_and_scheduling.md` - `go`/channel/select runtime rules.
- `async_await_and_threads.md` - async model and thread model definitions.
- `ownership_sendability.md` - safe transfer/capture rules across boundaries.
- `unsafe_escape_hatches.md` - unsafe marker syntax, scope boundaries, and audit contract.
- `memory_model_and_gc.md` - happens-before and GC guarantees.
- `cost_model.md` - copies/allocations/concurrency cost model and release expectations.
- `error_model.md` - `Result`, `?`, contract failure, panic/trap semantics.
- `module_and_visibility.md` - module/import/export and visibility semantics.
- `abi_and_ffi.md` - ABI layout/calling conventions/interop boundaries.
- `spec_coverage_matrix.md` - traceability from rule to tests/deferreds.

### Explanatory

- `contracts.md`, `intents.md`, `examples.md` - intent/contract authoring and
  guidance.
- `syntax_samples.yb`, `syntax_samples.vibe` - illustrative sample files.
- `spec_glossary.md` - canonical terminology.
- `spec_decision_log.md` - decision records with rationale and status.

## Conformance Language

- `MUST` / `MUST NOT`: mandatory for conformance.
- `SHOULD` / `SHOULD NOT`: strong recommendation; deviations require rationale.
- `MAY`: optional behavior.

## Compatibility And Versioning

- v0.1 freeze docs are historical references.
- v1.0 docs in this directory define production target behavior.
- Breaking syntax/semantic changes require:
  1. decision log update,
  2. grammar update,
  3. coverage matrix update,
  4. checklist and readiness report update.

## Release-Gate Integration

Spec quality is release-gated by:

- `tooling/spec/validate_spec_consistency.py`
- `tooling/spec/validate_spec_coverage.py`
- `v1-release-gates.yml` job `spec_integrity_gate`

## Known Scope Boundaries

- This suite defines language and runtime behavior.
- Standard library APIs are covered where needed for semantic guarantees.
- Performance claims are constrained to explicit guarantees, not benchmark
  marketing.
