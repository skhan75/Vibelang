# VibeLang Semantics Spec (v0.1 Draft)

## Overview

VibeLang semantics prioritize:

- Predictable behavior
- Safe-by-default concurrency
- Compiler-analyzable contracts

Phase 1 ambiguity resolutions are frozen in `docs/spec/phase1_resolved_decisions.md`.

## Execution Model

- Ahead-of-time compiled native binaries.
- Deterministic execution for deterministic inputs, excluding explicitly nondeterministic effects.
- No implicit background runtime behavior outside declared runtime services (GC, scheduler, channel operations).

## Type System

## Static with Local Inference

- Every expression has a compile-time type.
- Local variables infer type from initializer.
- Public APIs should annotate parameter and return types.

## Core Categories

- Scalar: `Int`, `Float`, `Bool`, `Str`
- Aggregate: `List<T>`, `Map<K,V>`, user `type`
- Concurrency primitives: `chan<T>`
- Result model: `Result<T, E>`

## Nullability

V0.1 avoids implicit null. Optional values use explicit `Option<T>` style type.

## Assignment and Mutation

- `:=` introduces a new binding.
- `=` updates an existing mutable value.
- Mutation to shared state is legal only under rules enforced by runtime/concurrency checker.

## Function Semantics

- Functions are first-class values.
- Last expression returns when explicit `return` is absent.
- Parameter passing is by value or reference based on type category and compiler lowering rules.

## Error Semantics

- Fallible functions return `Result<T, E>`.
- `expr?` expands to "if error, return early with same error channel."
- `@require` failures return contract violations (debug/test builds) or configured failure mode (release policy).

## Contract Semantics

Contracts are part of semantic model, not comments.

- `@require` is evaluated at function entry.
- `@ensure` is evaluated at function exit with `.` bound to return value.
- `old(expr)` inside `@ensure` captures entry-time value snapshots.
- `@examples` produce executable tests; they do not change runtime semantics.
- `@intent` is metadata for human and tooling layers; compiler preserves it in semantic index.

## Evaluation Order

V0.1 defines left-to-right evaluation for:

- Argument lists
- Method chains
- Binary expressions

This avoids hidden reorderings in early versions and simplifies mental model.

## Control Flow

- `if`, `for`, `while`, `match`, and `select` are structured control forms.
- `break` and `continue` apply to nearest loop.
- `return` exits current function immediately.

## Concurrency and Memory Visibility

- `go` schedules a task on runtime scheduler.
- Channel `send`/`recv` operations define synchronization points.
- Values sent through channels are transferred according to runtime copy/move policy.
- Shared mutable state without synchronization is disallowed in safe mode.

## Effect Semantics

`@effect` declares function side effects. Effects are checked against operations observed during semantic analysis.

Initial v0.1 effect set:

- `alloc`
- `mut_state`
- `io`
- `net`
- `concurrency`
- `nondet`

Compiler policies:

- Missing declared effect when operation exists: compile error or warning based on profile.
- Declared effect not observed: warning (helps clean contracts over time).

## Determinism Policy

By default, functions are assumed deterministic unless:

- They declare `@effect nondet`
- They transitively call nondeterministic APIs

This policy enables stable generated tests and reproducible CI behavior.

## Build Profiles and Semantics

## Dev Profile

- More contract checks enabled at runtime.
- Faster compilation with moderate optimization.

## Release Profile

- Aggressive optimization.
- Contract checks configurable: keep critical checks, lower/remove non-critical checks by policy.

## Undefined Behavior Policy

V0.1 aims to eliminate user-visible undefined behavior in safe language surface.
Unsafe runtime/internal operations are confined to compiler/runtime implementation layers.
