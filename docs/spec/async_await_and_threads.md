# VibeLang Async/Await and Thread Model (v1.0 Target)

Status: normative target.

## Goals

- First-class asynchronous programming model.
- Explicit thread-boundary semantics for systems workloads.
- Deterministic error/cancellation behavior.

## Async Function Semantics

Async function declaration:

```txt
pub async fetchUser(id: i64) -> Result<User, Error> {
  ...
}
```

Rules:

- Async functions execute as resumable state machines.
- Calling async function yields awaitable value/future-like object.
- Async function body may suspend only at explicit `await` points.

## Await Semantics

`await expr`:

- evaluates `expr` to awaitable value
- suspends current async task until completion
- resumes with completed value or propagated error

`await` outside async context is compile-time error unless explicitly allowed by
top-level runtime rule.

## Structured Async Scope

V1 target favors structured async:

- Child async tasks should be joined/cancelled by parent scope.
- Detached async tasks require explicit detach API.
- Scope exit must not leak unjoined child tasks by default.

## Thread Model

### `go` vs `thread`

- `go`:
  - runtime task abstraction
  - scheduled by runtime worker pool
  - cheaper than OS-thread per task
- `thread`:
  - explicit OS-thread execution boundary
  - used for thread-affine operations or blocking native integrations

### Thread Affinity

- APIs requiring thread affinity must document it explicitly.
- Crossing thread boundary may require sendable ownership transfer.

## Interop Between Async and Threads

- Awaitable completion may be fulfilled from different thread than caller.
- Runtime scheduler must marshal continuations safely back into async context.
- Thread-to-async handoff must establish memory visibility guarantees.

## Cancellation and Timeouts

- Async tasks accept cancellation signals cooperatively.
- Cancellation observed at `await` and other suspension points.
- Timeout behavior SHOULD be expressed via async timeout helpers or `select`
  over `after` events.

## Error Propagation

- Awaiting failed async result propagates error through `Result` or raises
  deterministic task failure according to API contract.
- Unobserved async failure policy must be deterministic (logged, escalated, or
  both) and configurable by profile.

## Sendability Across Async/Thread Boundaries

- Values captured by async tasks and moved across thread boundaries must satisfy
  sendability constraints.
- Non-sendable captures across boundary are compile-time errors in safe mode.

## Determinism Requirements

- Async/task diagnostics must be deterministic and reproducible.
- Runtime ordering of concurrently completed tasks is only deterministic where
  synchronization/ordering constraints are explicit.

## Deferred Notes

- Advanced async generators/streams syntax is deferred unless accepted by
  decision log.
