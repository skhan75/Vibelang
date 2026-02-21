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

- **Resolved**: [grammar_v0_1.ebnf](grammar_v0_1.ebnf) is the parser reference for Phase 1 implementation.
- **Rule**: syntax changes in Phase 1 require explicit checklist + spec updates before parser modifications.

## Compatibility Appendix: v0.1 Freeze vs v1 Target

This appendix clarifies how historical v0.1 freeze decisions map to the v1
production-target specification documents.

### A. Grammar Source

- v0.1 parser freeze reference remains `docs/spec/grammar_v0_1.ebnf`.
- v1 normative target grammar is `docs/spec/grammar_v1_0.ebnf`.
- No v1 syntax claims should be added only to v0.1 grammar.

### B. Control-Flow Completeness

- v0.1 docs previously mentioned `match`/`break`/`continue` with partial parser
  coverage.
- v1 target grammar makes these forms explicit and normative.

### C. Optional Types

- v0.1 wording used `Option<T>` style informally.
- v1 syntax canonicalizes optional typing as `T?` with `none` literal.

### D. Contract Placement

- v0.1 and v1 are aligned: contract annotations remain top-of-function-body
  before executable statements.
- v1 additionally requires deterministic diagnostics for invalid placement.

### E. Decision Governance

For changes beyond this compatibility map:

1. update `docs/spec/spec_decision_log.md`,
2. update `docs/spec/grammar_v1_0.ebnf`,
3. update `docs/spec/spec_coverage_matrix.md`,
4. update release readiness evidence.
