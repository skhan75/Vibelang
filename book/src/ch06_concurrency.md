# Chapter 6: Concurrency (`go`, `chan`, `select`, cancellation)

Concurrency in VibeLang is explicit by design. You do not "accidentally become
concurrent." You mark boundaries and reason about value movement directly.

This chapter covers the practical model and how to write safe, deterministic
concurrent code.

## 6.1 Core Primitives

VibeLang’s concurrency surface includes:

- `go expr` - spawn runtime task,
- `thread expr` - explicit OS-thread boundary,
- `Chan<T>` - typed channels,
- `select` - wait over multiple readiness events,
- `after` and `closed` select patterns,
- cooperative cancellation semantics.

## 6.2 Task Lifecycle Mental Model

A task typically moves through:

1. created,
2. runnable,
3. running,
4. suspended (waiting),
5. completed/failed/cancelled.

Understanding lifecycle is useful for debugging throughput and deadlock-like
symptoms.

## 6.3 Worker Pool Example

```txt
worker(jobs, out) -> Int {
  @effect concurrency
  job := jobs.recv()
  out.send(job * job)
  0
}

pub runAll(tasks: List<Int>) -> List<Int> {
  @intent "process all tasks concurrently"
  @effect alloc
  @effect concurrency
  @ensure len(.) == len(tasks)

  jobs := chan(1024)
  out := chan(1024)

  repeat cpu_count() {
    go worker(jobs, out)
  }

  for task in tasks {
    jobs.send(task)
  }
  jobs.close()

  out.take(len(tasks))
}
```

Key points:

- clear task spawn boundary,
- explicit channel topology,
- deterministic output-size contract.

## 6.4 Channel Semantics

From the spec model:

- `chan(capacity)` defines buffered channel with deterministic capacity,
- `capacity = 0` models rendezvous behavior,
- send on closed channel is deterministic runtime error,
- close on already-closed channel is deterministic runtime error,
- send/recv pairs establish synchronization boundaries.

## 6.5 `select` Semantics

```txt
select {
  case msg := inbox.recv() =>
    handle(msg)
  case after 5s =>
    on_timeout()
  case closed inbox =>
    on_closed()
  case default =>
    on_idle()
}
```

Important behavior:

- if multiple cases are ready, runtime follows deterministic fairness policy,
- `default` fires immediately when no case is ready,
- timeout and closed-channel behavior are explicit and testable.

## 6.6 Cancellation

Cancellation is cooperative:

- blocking operations should observe cancellation and unblock,
- parent/child propagation policy should be explicit,
- failures in structured scopes should propagate predictably.

When designing concurrent APIs, always define:

- who owns cancellation,
- when children are joined,
- what happens to partial work on cancellation.

## 6.7 Concurrency + Effects

Functions performing concurrent behavior should declare `@effect concurrency`.
If concurrent code mutates shared state via synchronized pathways, `@effect
mut_state` may also be relevant.

This dual visibility helps reviewers detect mismatches between behavior and
declarations.

## 6.8 Concurrency + Sendability

Any value crossing boundaries (`go`, `thread`, channel send, async-thread
handoff) must satisfy sendability constraints.

Common design fix when diagnostics fail:

- move ownership instead of sharing mutable aliases,
- transfer immutable snapshots across channels,
- centralize mutation behind one owning task.

## 6.9 Failure Propagation Strategy

Define failure strategy up front:

- Are child task failures fatal to parent scope?
- Are failures collected and aggregated?
- Are detached tasks allowed?

If the answer is "it depends," encode that policy explicitly in API shape and
tests.

## 6.10 Determinism and Concurrency

Concurrency does not remove determinism requirements. It changes where ordering
must be explicit.

Deterministic behavior requires:

- clear synchronization points,
- stable error classes and diagnostics,
- explicit channel/timeout semantics,
- avoiding hidden races.

## 6.11 Debugging Checklist for Concurrent Code

When behavior looks unstable:

1. verify sendability diagnostics first,
2. inspect channel close/send ordering,
3. validate timeout units (`5s`, `100ms`, etc.),
4. check for unsynchronized shared mutation,
5. confirm cancellation handling at blocking points.

## 6.12 Clarification: Concurrency Does Not Mean Nondeterminism Everywhere

A frequent misconception is that once code is concurrent, behavior is inherently
"random." VibeLang's model is more precise. Some ordering is intentionally
unspecified unless synchronized, but many behaviors remain deterministic:
diagnostic classes, boundary checks, channel semantics, and profile policy.

In other words, concurrency increases the number of possible execution
interleavings, but it does not eliminate the language's deterministic contract.
The goal is to make variability explicit and controlled through synchronization,
timeouts, and ownership movement, rather than accidental and opaque.

## 6.13 Chapter Checklist

You should now be able to:

- design channel-based pipelines with explicit ownership movement,
- write and reason about `select` behavior,
- model cancellation propagation,
- align effects with concurrency behavior,
- avoid common race and lifecycle pitfalls.

---

Next: Chapter 7 covers the CLI and day-to-day development workflow.
