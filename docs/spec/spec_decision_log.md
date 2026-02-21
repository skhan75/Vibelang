# VibeLang Spec Decision Log

This log records normative decisions for the production specification suite.

## Decision Format

- ID: Stable identifier.
- Status: `accepted`, `superseded`, `proposed`, or `deferred`.
- Scope: Grammar, semantics, runtime, tooling, or release policy.
- Decision: The chosen rule.
- Rationale: Why this was chosen.
- Impact: Compatibility and implementation implications.

## Decisions

### SPEC-001: Versioned Grammar Source Of Truth

- Status: accepted
- Scope: grammar/process
- Decision: `grammar_v1_0.ebnf` is the production grammar source of truth;
  `grammar_v0_1.ebnf` remains archived for historical freeze reference.
- Rationale: Preserve traceability while enabling production surface expansion.
- Impact: Any syntax change MUST update v1 grammar and coverage matrix.

### SPEC-002: Explicit Mutability Model

- Status: accepted
- Scope: syntax/semantics
- Decision: Bindings are immutable by default; mutable reassignment requires
  explicit mutable binding form.
- Rationale: Prevent accidental state mutation while preserving ergonomics.
- Impact: Assignment legality is determined by mutability model, not parser
  acceptance alone.

### SPEC-003: Deterministic Container Iteration Baseline

- Status: accepted
- Scope: runtime/containers
- Decision: `List` iteration order is insertion order; `Map` iteration order is
  deterministic for identical input and toolchain version.
- Rationale: Deterministic tests, reproducible CI behavior, and auditability.
- Impact: Runtime and codegen must preserve stable order guarantees.

### SPEC-004: Numeric Type Family Definition

- Status: accepted
- Scope: type system/numerics
- Decision: Include fixed-width signed/unsigned integer families and IEEE-754
  floats (`f32`, `f64`) with explicit overflow and NaN policy.
- Rationale: Production-grade portability and predictable low-level behavior.
- Impact: Type checker and constant folding rules must follow numeric model doc.

### SPEC-005: Control Flow Completeness

- Status: accepted
- Scope: grammar/semantics
- Decision: `break`, `continue`, and `match` are first-class control-flow forms
  with defined termination and exhaustiveness behavior.
- Rationale: Remove ambiguity and close syntax/semantics contradictions.
- Impact: Grammar, syntax, and semantics docs must be aligned.

### SPEC-006: Async/Await With Explicit Thread Model

- Status: accepted
- Scope: concurrency/runtime
- Decision: VibeLang defines `async`/`await` semantics and an explicit runtime
  thread model; async is not only advisory.
- Rationale: Modern language completeness and predictable concurrent execution.
- Impact: Runtime, ownership/sendability rules, and error propagation must
  include async/thread boundaries.

### SPEC-007: Memory Model And GC Contracts

- Status: accepted
- Scope: runtime/memory
- Decision: Publish normative happens-before and GC behavior contracts including
  synchronization points and profile-level guarantees.
- Rationale: Safety and performance must be reasoned about formally.
- Impact: Documentation and diagnostics must reference same memory contracts.

### SPEC-008: ABI/FFI Definition Before GA

- Status: accepted
- Scope: ABI/toolchain
- Decision: Calling convention, data layout constraints, and FFI safety rules
  must be documented before v1 production sign-off.
- Rationale: Interop correctness and portability require explicit ABI rules.
- Impact: Release readiness includes ABI spec review.

### SPEC-009: Spec Traceability As A Blocking Gate

- Status: accepted
- Scope: CI/release
- Decision: Spec consistency and coverage traceability are enforced by blocking
  CI gates and readiness reporting.
- Rationale: Prevent spec drift and undocumented behavior from entering release.
- Impact: `spec_integrity_gate` is part of release gating.

### SPEC-010: Deferred Features Must Be Explicit

- Status: accepted
- Scope: governance
- Decision: Any unimplemented behavior in normative docs MUST include explicit
  deferred status, target phase, and rationale.
- Rationale: Avoid false claims and maintain trustworthy specs.
- Impact: Coverage matrix includes deferred entries with owners/evidence.
