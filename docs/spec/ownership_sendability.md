# VibeLang Ownership and Sendability (v1.0 Target)

Status: normative target; implementation maturity varies by feature area.

## Goals

- Enforce safe cross-boundary value movement.
- Prevent obvious data races in safe language surface.
- Keep model understandable without mandatory lifetime syntax.

## Boundary Types

Sendability checks apply at:

- `go` task spawn boundaries
- `thread` OS-thread boundaries
- async task capture/handoff boundaries
- channel send boundaries

## Sendability Rules

### Baseline Sendable Categories

- Primitive scalars (`Bool`, integer, float)
- `Str`
- `List<T>` where `T` is sendable
- `Result<T,E>` where `T` and `E` are sendable
- `Chan<T>` handles

### Baseline Non-Sendable Categories

- values with unknown/inferred-unknown dynamic layout
- containers including non-sendable members
- explicitly thread-affine handles/resources

`Map<K,V>` sendability depends on key/value sendability and deterministic runtime
support status for map transfer.

## Capture and Move Semantics

- Capturing immutable values by copy/move across boundary is allowed when
  sendable.
- Capturing mutable aliases into concurrent contexts is rejected unless wrapped
  in explicit synchronization abstraction.
- Compiler diagnostics must identify capture source and boundary type.

## Shared Mutable State Rules

- Unsynchronized shared mutable writes in concurrent contexts are invalid in safe
  mode.
- Member/field assignment in concurrent contexts requires synchronization
  primitive evidence or explicit unsafe boundary.
- Runtime synchronization APIs establish legal mutation boundaries.

## Channel Transfer Semantics

- Sending through channel transfers ownership visibility to receiver.
- Receiver observes sender writes that happened-before send.
- Post-send mutation legality depends on copy vs move semantics of sent value and
  API contract.

## Async and Thread Interaction

- Async tasks and thread tasks use same sendability checks when values cross
  task/thread boundaries.
- Await suspension/resume across threads preserves ownership invariants.
- Non-sendable future captures across thread handoff are compile-time errors in
  safe mode.

## Effect Coupling

Ownership diagnostics and effects are coupled:

- concurrency behavior should align with `@effect concurrency`
- shared mutable writes in concurrent flows should align with `@effect mut_state`
- transitive calls propagate effect expectations

## Unsafe Escape Hatches

- Explicit unsafe escape hatches may exist for advanced interop.
- Unsafe blocks must be isolated and auditable.
- Unsafe use does not relax deterministic diagnostics requirements.

## Determinism Requirements

- Same source and configuration must produce same ownership/sendability
  diagnostics.
- Diagnostic ordering and severity mapping must be stable.

## Deferred Notes

- Full Rust-like borrow checker and lifetime annotations remain deferred.
- Complete static race elimination across all dynamic patterns remains deferred.
