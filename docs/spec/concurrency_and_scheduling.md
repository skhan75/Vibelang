# VibeLang Concurrency and Scheduling Spec (v1.0 Target)

Status: normative target.

## Concurrency Primitives

- `go expr`: spawn runtime task
- `thread expr`: spawn explicit OS-thread execution boundary
- `Chan<T>`: typed channels for message passing
- `select`: coordinated wait over channel and timeout events

## Scheduler Model

Baseline target model:

- Runtime uses M:N scheduling where many tasks multiplex over a bounded worker
  thread pool.
- Scheduler policy must be fair enough to avoid deterministic starvation for
  runnable tasks under bounded load assumptions.
- Scheduler implementation details may vary per target, but observable semantics
  must remain stable.

## Task Lifecycle

- Created: task exists and may be queued.
- Runnable: eligible for scheduling.
- Running: currently executing.
- Suspended: waiting on channel/await/sync event.
- Completed: normal completion.
- Failed: terminated with error/panic.
- Cancelled: terminated due to cancellation signal.

## Channels

### Channel Creation

- `chan(capacity)` returns buffered channel with deterministic capacity.
- Capacity `0` defines rendezvous channel semantics.

### Send/Receive

- `send` on open channel enqueues or blocks according to capacity state.
- `recv` on empty open channel blocks.
- `recv` on closed and drained channel follows closed-channel policy (explicit
  empty signal or deterministic error depending API form).

### Close

- Closing channel marks no-further-sends boundary.
- Sending to closed channel is deterministic runtime error.
- Closing already-closed channel is deterministic runtime error.

## Select Semantics

`select` chooses one ready case.

Rules:

- Evaluate case guards/patterns in deterministic order.
- If multiple cases are ready, selection uses deterministic fairness policy.
- `after` timeout cases become ready after specified duration.
- `default` case executes immediately when no other cases are ready.

## Cancellation

- Cancellation is cooperative.
- Blocking operations (`recv`, `select`, await points) must observe cancellation
  and unblock deterministically.
- Cancel propagation policy is defined by parent/child task relationship.

## Error Propagation

- Unhandled task failure propagates according to task join/wait API policy.
- Structured concurrency mode should propagate child failure to parent scope by
  default unless explicitly detached.

## Synchronization and Visibility

- Channel send/recv pair establishes happens-before for transferred value.
- Additional runtime primitives (mutex/atomic APIs, if provided) define
  synchronization boundaries in memory model.

## Determinism Constraints

- For deterministic inputs and scheduling policy, output ordering dependent on
  explicit concurrency races may vary unless synchronization constrains order.
- Runtime must preserve deterministic diagnostics and stable failure classes.

## Relationship To Async/Await

- `await` suspension points integrate with same scheduler event loop mechanics.
- Async tasks and `go` tasks share sendability and cancellation rules unless
  explicitly documented otherwise.
