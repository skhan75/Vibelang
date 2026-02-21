# VibeLang Memory Model and GC Spec (v1.0 Target)

Status: normative target.

## Scope

- Defines memory visibility guarantees for safe language surface.
- Defines GC behavior contracts relevant to correctness and performance.

## Memory Ordering Model

### Program Order

- Within a single task/thread, operations are observed in source order except
  where language/runtime explicitly permits reordering without changing
  observable semantics.

### Happens-Before Relations

Happens-before is established by:

- channel send -> corresponding receive
- task/thread join completion
- explicit synchronization primitives (mutex/atomic APIs, if present)
- async await completion handoff

If operation A happens-before B, writes visible at A must be visible at B.

### Data Race Rule

- Unsynchronized concurrent read/write or write/write to same mutable location is
  invalid in safe mode.

## Ownership and Visibility

- Value transfer across boundary follows sendability rules.
- Ownership transfer and synchronization jointly determine visibility.
- Borrowed/shared references in concurrent contexts require synchronization-safe
  contract.

## GC Model

### Baseline

- Automatic garbage collection is part of runtime model.
- Collector is concurrent/generational by target design.

### Correctness Guarantees

- GC must preserve language-level object reachability semantics.
- No user-visible use-after-free in safe surface.
- Finalization order is not guaranteed unless explicitly specified by API.

### Pause and Throughput Contract

- GC pause behavior and throughput targets are profile/runtime contracts and must
  be published in release notes/metrics docs.
- Runtime may expose observability counters for GC events.

## Allocation Semantics

- Allocation is explicit semantic effect (`@effect alloc`).
- Allocation failure behavior must be deterministic and documented (error/panic
  policy).
- Container/string growth semantics must align with allocation policy.

## Interactions With Async/Threads

- GC synchronization with task/thread transitions must preserve safety.
- Object reachability across async suspension and thread handoff is required.

## Unsafe/Internal Boundaries

- Compiler/runtime internals may use unsafe memory operations.
- Unsafe internals must not violate safe-surface memory guarantees.

## Determinism Requirements

- Memory-model diagnostics and race-check diagnostics must be deterministic.
- GC telemetry format must be stable across runs for identical workload and
  configuration, excluding timing fields.

## Deferred Notes

- Real-time hard pause guarantees are out-of-scope for v1 target.
