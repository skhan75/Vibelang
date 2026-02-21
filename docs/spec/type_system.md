# VibeLang Type System (v1.0 Target)

Status: normative target.

## Goals

- Static safety with low-noise authoring.
- Deterministic type checking.
- Explicit behavior at API and concurrency boundaries.

## Type Categories

### Primitive Types

- `Bool`
- `Str`
- Signed integers: `i8`, `i16`, `i32`, `i64`, `isize`
- Unsigned integers: `u8`, `u16`, `u32`, `u64`, `usize`
- Floating point: `f32`, `f64`

### Compound Types

- `List<T>`
- `Map<K,V>`
- `Result<T,E>`
- user-defined `type` declarations
- channel types: `Chan<T>`

### Optional Types

- Canonical syntax: `T?`
- `none` is the empty optional value.

`Option<T>` may be used as an explanatory alias in documentation, but syntax
and parser contracts use `T?`.

## Binding and Inference Rules

- `x := expr` infers type from `expr`.
- `mut x := expr` infers mutable binding type from `expr`.
- `const x: T = expr` requires `expr` assignable to `T`.
- Public API functions SHOULD use explicit parameter and return types.

Inference MUST be deterministic and independent of file traversal order.

## Assignability

`S` assignable to `T` when one of the following holds:

1. `S` and `T` are identical.
2. numeric widening coercion is permitted by `docs/spec/numeric_model.md`.
3. `S` is `none` and `T` is optional (`U?`).
4. all generic arguments are pairwise assignable under variance rules.

### Generic Variance Baseline

- `List<T>` invariant in `T`
- `Map<K,V>` invariant in both `K` and `V`
- `Result<T,E>` invariant in both `T` and `E`
- `Chan<T>` invariant in `T`

## Conversion and Cast Policy

- Implicit coercions are intentionally narrow and deterministic.
- Potentially lossy conversions MUST be explicit.
- Cross-domain conversions (`Str` to numeric, numeric to `Bool`, etc.) require
  explicit conversion functions or casts.

## Function Types and Calls

- Arguments are checked left-to-right.
- Arity mismatch is a compile-time error.
- Generic function instantiation may be explicit or inferred.
- Call-site inference MUST produce the same specialization for same input types.

## Optional and Result Interaction

- Optional and `Result` are distinct and not implicitly interchangeable.
- `?` is valid only for `Result<T,E>` unless extended by explicit language rule.
- Unwrapping optional values requires explicit handling (`match` / helper APIs).

## Pattern and Match Typing

- All reachable `match` arms must produce assignable result types.
- Non-exhaustive `match` on non-optional/non-enum-like domains requires
  `default`.
- Pattern literal types must be compatible with scrutinee type.

## Type Identity and Name Resolution

- Qualified names (`module.Type`) resolve deterministically.
- Type aliases (if introduced) MUST not change runtime layout unexpectedly.
- Generic instantiations of same definition and arguments are type-equal.

## Concurrency Type Constraints

- Values crossing `go`, `thread`, or async task boundaries MUST satisfy
  sendability rules in `docs/spec/ownership_sendability.md`.
- `Chan<T>` payload type determines channel transfer safety constraints.

## Type Errors

Type checker diagnostics MUST provide:

- stable diagnostic code
- primary source span
- expected vs actual type
- optional fix guidance

## Deferred and Out-Of-Scope Notes

- Trait/interface-based ad-hoc polymorphism is deferred unless explicitly
  accepted by decision log.
- Higher-kinded types are out-of-scope for v1 target.
