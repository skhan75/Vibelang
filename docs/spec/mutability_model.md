# VibeLang Mutability Model (v1.0 Target)

Status: normative target.

## Principles

- Immutability by default.
- Mutation is explicit and auditable.
- Concurrency boundaries require stricter mutation rules.

## Binding Forms

- Immutable inferred binding:
  - `x := expr`
- Mutable inferred binding:
  - `mut x := expr`
- Immutable constant binding:
  - `const x: T = expr`

## Reassignment Rules

- Reassignment (`=`) is allowed only for mutable bindings and mutable fields.
- Reassigning immutable or `const` binding is compile-time error.
- Rebinding by shadowing (`x := ...` in inner scope) is legal and distinct from
  reassignment.

## Parameter Mutability

- Function parameters are immutable by default.
- Mutable parameters must be explicit (`mut arg: T`) if reassignment is allowed.
- Parameter mutability does not bypass ownership/sendability constraints.

## Field and Index Mutation

- `obj.field = expr` requires:
  1. mutable receiver binding or mutable reference context
  2. field declared mutable (if type declaration differentiates mutability)
- `list[i] = expr` requires mutable list binding and valid index semantics.
- `map[key] = value` requires mutable map binding.

## Const Semantics

- `const` values are immutable for program lifetime in their scope.
- `const` initializers MUST be compile-time evaluable.
- Taking mutable references to `const` values is illegal.

## Borrow/Reference Baseline

V1 target keeps reference model simple:

- No user-facing lifetime annotations required.
- Mutation through aliases in concurrent contexts is constrained by
  sendability/ownership checks.
- Runtime synchronization primitives are required where shared mutation is
  allowed.

## Concurrency Interaction

- Mutable shared writes in concurrent contexts require explicit synchronization.
- Unsynchronized shared mutable writes are diagnostics errors in safe mode.
- Mutation across `go`/`thread`/async boundaries must satisfy
  `docs/spec/ownership_sendability.md`.

## Contracts and Mutability

- `@require` and `@ensure` are pure-expression contexts by default.
- Contract expressions cannot perform mutation.
- `old(expr)` snapshots read-only entry-time values.

## Diagnostics Requirements

Mutability diagnostics SHOULD include:

- whether operation was reassignment or mutation
- immutable binding/field/index target details
- concurrency context if relevant
- deterministic code and span

## Deferred Notes

- Fine-grained interior mutability primitives are deferred unless explicitly
  accepted in decision log.
