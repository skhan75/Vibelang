# VibeLang Semantics Spec (v1.0 Target)

## Status

- This document defines normative semantic behavior for v1 production target.
- Where implementation is still in progress, behavior is marked as deferred in
  `docs/spec/spec_coverage_matrix.md`.

## Semantic Priorities

- Determinism first
- Safety by default
- Native performance with explicit contracts
- AI-assisted workflows without AI-dependent correctness

## Execution Model

- Programs compile ahead-of-time to native binaries.
- Deterministic input + deterministic environment MUST produce deterministic
  behavior unless `nondet` effect is explicitly involved.
- Runtime services (scheduler, channels, GC) are explicit semantic participants.

## Type and Value Model

- The language is statically typed with local inference.
- Every expression has a compile-time type.
- Public API boundaries SHOULD be explicitly annotated.
- Optional values are explicit:
  - syntax-level canonical form: `T?`
  - empty optional literal: `none`

See:

- `docs/spec/type_system.md`
- `docs/spec/numeric_model.md`
- `docs/spec/strings_and_text.md`

## Bindings and Mutability

- `x := expr` creates a new binding.
- `mut x := expr` creates mutable binding.
- `const x = expr` creates immutable compile-time constant binding.
- Assignment (`=`) MUST be rejected on immutable bindings.
- Field/index mutation legality follows mutability + ownership rules.

See `docs/spec/mutability_model.md`.

## Function Semantics

- Function call argument evaluation is left-to-right.
- Explicit `return` exits function immediately.
- If no explicit `return` on final path, tail expression value is returned.
- Async functions return async-compatible values and can be suspended at
  `await` points.

## Error and Propagation Semantics

- Fallible operations use `Result<T,E>` model.
- `expr?` propagates failure to caller preserving error channel type.
- Contract failures follow profile policy (dev/test strict by default; release
  policy configurable).

See `docs/spec/error_model.md` and `docs/spec/contracts.md`.

## Control-Flow Semantics

- `if`, `for`, `while`, `repeat`, `match`, and `select` are structured forms.
- `break` and `continue` target nearest enclosing loop unless label is provided.
- `match` semantics include explicit default handling and deterministic arm
  selection.

See `docs/spec/control_flow.md`.

## Concurrency, Async, and Threads

- `go` spawns runtime-scheduled task units.
- `thread` schedules explicit OS-thread boundary execution.
- `await` suspends async execution until awaited value resolves.
- Channel send/recv form synchronization points.
- Cross-boundary value movement must satisfy sendability rules.

See:

- `docs/spec/concurrency_and_scheduling.md`
- `docs/spec/async_await_and_threads.md`
- `docs/spec/ownership_sendability.md`

## Effects and Determinism

`@effect` declarations describe side-effect classes used for static checks and
reasoning.

Vocabulary:

- `alloc`
- `mut_state`
- `io`
- `net`
- `concurrency`
- `nondet`

Rules:

- Observed effects not declared: diagnostic.
- Declared but unobserved effects: diagnostic (typically warning).
- `nondet` effect participation weakens deterministic guarantees by design.

## Memory and Visibility

- Visibility and ordering MUST follow language memory model.
- Channel sync and explicit runtime synchronization establish happens-before.
- Unsynchronized shared mutable access is invalid in safe surface.

See `docs/spec/memory_model_and_gc.md`.

## ABI and Module Semantics

- External boundaries use explicit ABI contracts.
- Module/import/visibility rules are deterministic and reject ambiguous
  resolution.

See:

- `docs/spec/abi_and_ffi.md`
- `docs/spec/module_and_visibility.md`

## Build Profile Semantics

### Dev/Test Profile

- Prioritize diagnostics and contract enforcement.
- Preserve deterministic debug metadata.

### Release Profile

- Prioritize optimized native codegen.
- Keep critical correctness checks; non-critical checks follow release policy.

## Undefined Behavior Policy

User-visible safe language surface aims to avoid undefined behavior.
Low-level unsafe operations are confined to runtime/compiler internals and MUST
not leak undefined behavior through safe APIs.
