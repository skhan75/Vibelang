# Phase 1 Resolved Decisions (Grammar and Semantics Freeze)

This appendix resolves open ambiguities called out in Phase 1 and acts as the implementation contract for Rust frontend work.

## Decision 1: Block Style

- **Resolved**: braces are mandatory for declarations and control flow in v0.1.
- **Rationale**: unambiguous parser behavior and predictable recovery.
- **Impact**: indentation/newline are non-semantic formatting concerns.

## Decision 2: Inference Boundaries

- **Resolved**: local bindings with `:=` use inference; explicit local types are optional.
- **Resolved**: public function parameter/return types remain optional in grammar for v0.1, but missing annotations are reported as style warnings by frontend diagnostics.
- **Rationale**: preserve ergonomic authoring while encouraging stable API contracts.

## Decision 3: Contract Placement

- **Resolved**: contract annotations must appear at the top of function bodies before executable statements.
- **Rationale**: deterministic lowering and predictable review conventions.

## Decision 4: Effect Vocabulary Stability

- **Resolved frozen set for Phase 1**:
  - `alloc`
  - `mut_state`
  - `io`
  - `net`
  - `concurrency`
  - `nondet`
- **Rule**: unknown effect names are compile errors in frontend checks.

## Decision 5: Dot Result and `old(...)`

- **Resolved**: `.` is valid only in `@ensure` expressions and refers to function result.
- **Resolved**: `old(expr)` is valid only in `@ensure`.
- **Rationale**: maintain clear pre/post-state semantics.

## Decision 6: Evaluation and Determinism

- **Resolved**: left-to-right evaluation for argument lists, method chains, and binary expressions.
- **Resolved**: parser/diagnostics output ordering must be deterministic for same input.

## Decision 7: Grammar Source of Truth

- **Resolved**: [docs/spec/grammar_v0_1.ebnf](docs/spec/grammar_v0_1.ebnf) is the parser reference for Phase 1 implementation.
- **Rule**: syntax changes in Phase 1 require explicit checklist + spec updates before parser modifications.
