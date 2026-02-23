# Chapter 15: Ownership, Sendability, and Memory Model

This chapter combines three tightly linked concerns:

- ownership and movement across execution boundaries,
- sendability constraints in concurrency/async/thread flows,
- memory visibility and GC guarantees.

Together, they define what "safe concurrent code" means in VibeLang’s model.

## 15.1 Why This Chapter Matters

Many correctness bugs in distributed/high-throughput systems are not syntax bugs.
They are boundary bugs:

- values moved unsafely across tasks,
- hidden shared mutation,
- unclear visibility ordering,
- weak failure/cancellation semantics.

VibeLang addresses these through explicit sendability and memory-model contracts.

## 15.2 Boundary Types

Sendability checks apply at:

- `go` spawn boundaries,
- `thread` boundaries,
- async capture/handoff boundaries,
- channel send boundaries.

If data crosses one of these boundaries, safety constraints must be satisfied.

## 15.3 Baseline Sendable Categories

Sendable by baseline:

- primitive scalars,
- `Str`,
- `List<T>` where `T` is sendable,
- `Result<T,E>` where both are sendable,
- `Chan<T>` handles.

`Map<K,V>` sendability depends on key/value sendability and runtime support
constraints.

## 15.4 Common Non-Sendable Cases

Typical non-sendable patterns:

- values with unknown dynamic layout,
- containers holding non-sendable members,
- explicitly thread-affine resources moved across boundaries.

Compiler diagnostics should identify both capture source and boundary type.

## 15.5 Capture and Move Guidance

Practical rules:

- prefer moving immutable snapshots across boundaries,
- avoid sharing mutable aliases unless synchronized by design,
- centralize mutable ownership where possible.

This leads to simpler debugging and fewer race-like failures.

## 15.6 Shared Mutable State Rules

In safe mode:

- unsynchronized shared mutable writes are invalid,
- synchronization primitives define legal mutation boundaries,
- concurrent mutation requires explicit design, not assumption.

## 15.7 Channel Transfer Visibility

Channel transfer is not only a queue operation; it is also a visibility boundary.
Send/receive establishes happens-before for transferred value semantics.

That gives you a deterministic reasoning surface for producer/consumer flows.

## 15.8 Memory Ordering Essentials

Memory-model highlights:

- per-thread/task program order is preserved under language semantics,
- happens-before arises through channel transfer, joins, await completion handoff,
  and explicit synchronization APIs,
- data races in unsynchronized mutable access are invalid in safe surface.

## 15.9 GC and Safety Guarantees

The runtime memory model includes automatic GC (concurrent/generational design
target). Safe-surface guarantees include:

- no user-visible use-after-free in safe code,
- reachability semantics preserved,
- deterministic failure classes and diagnostics.

Finalization order is not guaranteed unless an API explicitly defines it.

## 15.10 Allocation Effect and Memory Reasoning

Allocation is an explicit semantic effect (`@effect alloc`). This helps teams:

- identify memory-heavy functions quickly,
- reason about pressure during code review,
- connect profile behavior to runtime observability.

## 15.11 Async/Thread Interaction

Async and thread models share sendability constraints at boundaries. If an async
capture may cross threads, it must satisfy boundary safety requirements.

Design implication:

- treat async capture sets as explicit API design concerns,
- avoid implicit closure capture of mutable shared objects unless synchronized.

## 15.12 Pattern: Ownership-Centered Pipeline

A robust concurrent pattern:

1. parse/validate input in one stage,
2. transfer immutable task payloads over channel,
3. aggregate results in a single owner stage,
4. publish final immutable output.

This pattern minimizes aliasing risk and simplifies reasoning.

## 15.13 Pattern: Controlled Shared State

When shared mutable state is unavoidable:

- encapsulate mutation behind one synchronized abstraction,
- expose narrow APIs,
- document effect expectations (`mut_state`, `concurrency`),
- test failure and cancellation behavior explicitly.

## 15.14 Diagnostics You Should Expect

High-quality ownership/memory diagnostics should include:

- stable diagnostic code,
- clear boundary context (`go`/`thread`/channel/async),
- capture source span,
- concise remediation hints.

Stable diagnostics are especially important for CI and AI-assisted workflows.

## 15.15 Common Mistakes

1. capturing mutable outer bindings into spawned tasks,
2. treating channel usage as a substitute for all synchronization without
   understanding ownership movement,
3. assuming GC removes the need for explicit memory/order reasoning,
4. ignoring sendability errors as "tooling noise" instead of design feedback.

## 15.16 Practical Review Checklist

For concurrent PRs:

- what values cross boundaries?
- are they sendable?
- where is ownership transferred?
- where is shared mutation synchronized?
- which operations establish happens-before?
- are failure/cancellation paths explicit?

If any answer is vague, the design likely needs tightening.

## 15.17 Clarification: GC Does Not Eliminate Ownership Thinking

Some readers assume that automatic GC means ownership and boundary analysis are
mostly irrelevant. In concurrent systems, the opposite is true. GC manages
reclamation, but it does not automatically solve boundary safety, sendability,
happens-before visibility, or unsynchronized mutation hazards.

VibeLang combines GC convenience with explicit boundary rules so teams can reason
about correctness and performance without manual memory-management burden.

## 15.18 Chapter Checklist

You should now be able to:

- reason about boundary safety in concurrency and async code,
- apply sendability constraints as design tools,
- understand memory visibility/happens-before essentials,
- connect allocation/memory behavior with effect declarations.

---

Next: Chapter 16 covers ABI/FFI and unsafe boundary governance.
