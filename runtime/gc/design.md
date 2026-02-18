# VibeLang GC Design (v0.1)

## Goals

VibeLang uses an automatic **concurrent generational GC** with these goals:

- Low pause times for interactive/service workloads
- High throughput for allocation-heavy code
- Predictable integration with concurrent runtime
- Minimal syntax burden for users

## Non-Goals

- Hard real-time guarantees in v0.1
- User-managed memory APIs for normal application code

## Heap Model

Heap is split into:

- **Nursery (young generation)** for short-lived objects
- **Mature generation** for promoted survivors
- **Large object space** for oversized allocations

Each worker thread/task has a fast allocation context (TLAB-like local bump region).

## Collection Strategy

## Young Generation

- Stop-the-world minor collection
- Copying collector from nursery to survivor/promoted spaces
- Very short pauses due to small young heap

## Mature Generation

- Concurrent mark with mostly concurrent sweep
- Snapshot-at-the-beginning style invariants
- Bounded stop-the-world points for root snapshot and phase transitions

## Large Objects

- Allocated in dedicated region
- Marked concurrently
- Reclaimed by sweep/free-list policy

## Write and Read Barriers

## Write Barrier (required)

- Card marking for old-to-young pointers
- Enables precise remembered sets for minor collections

## Read Barrier (v0.1)

- Not required in baseline design
- Deferred for future moving mature compaction modes

## Root Set

Roots include:

- Task stacks and registers at safepoints
- Global/static references
- Runtime handles (channels, scheduler queues, timers)
- Native interop pinned handles

## Safepoint Model

Safepoints inserted at:

- Function prologues
- Loop back-edges
- Allocation slow paths
- Blocking runtime operations (`recv`, `select`, I/O waits)

Compiler emits metadata for precise stack maps.

## Object Layout

Each heap object header includes:

- Type metadata pointer
- Mark bits / generation bits
- Optional forwarding pointer during evacuation

Layout prioritizes cache alignment and quick type access.

## Promotion Policy

- Objects surviving N minor collections are promoted.
- N tunable; default 2 in v0.1.
- Adaptive nursery sizing based on survival rate and pause budget.

## Finalization and Resource Safety

V0.1 policy:

- Finalizers are discouraged for critical logic.
- Explicit `defer`/scope cleanup remains preferred for non-memory resources.
- GC can run finalizers on dedicated finalizer queue thread.

## Compiler Integration Points

Compiler must provide:

- Precise stack maps per safepoint
- Type pointer maps for object fields
- Barrier insertion on pointer stores
- Escape analysis hints for stack allocation opportunities (future optimization)

## Runtime Integration Points

Runtime provides:

- Global heap manager
- Per-worker allocation contexts
- Stop-the-world coordinator
- Concurrent marking workers
- Metrics/tracing interface

## Telemetry and Tuning

Expose metrics:

- Allocation rate
- Minor/major GC frequency
- Pause time histogram (p50/p95/p99)
- Promotion rate
- Live heap size over time

Target envelopes for v0.1:

- p95 pause under 10 ms for service-style benchmark profile
- p99 pause under 25 ms on reference hardware

## Failure Modes and Mitigations

- **Excessive promotion pressure**
  - Mitigation: grow nursery and tune survivor threshold.
- **Long major-cycle latency**
  - Mitigation: increase concurrent marker workers and enforce phase pacing.
- **Barrier overhead too high**
  - Mitigation: optimize barrier fast paths and card table locality.

## Testing Strategy

- Deterministic stress tests with forced GC intervals
- Soak tests under mixed allocation patterns
- Concurrency race checks around safepoint and root capture transitions
- Contract-level tests for `@effect alloc` consistency with observed allocations
