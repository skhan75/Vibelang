# Chapter 12: Ownership, Sendability, and Memory

Chapter 11 showed you *how* to write concurrent programs with `go`, `chan`, and
`select`. This chapter explains *why* those programs are safe. VibeLang's memory
model, ownership rules, and sendability checks form an interlocking system that
prevents data races, dangling references, and memory corruption — all without
requiring you to manually manage memory.

Understanding this chapter deeply will change how you think about concurrent
program design. The rules aren't arbitrary restrictions; they're the minimum set
of constraints needed to guarantee safety in a concurrent, garbage-collected
language.

## 12.1 VibeLang's Memory Model

### 12.1.1 Concurrent Generational Garbage Collector

VibeLang uses a concurrent generational garbage collector (GC). Let's unpack
each word:

- **Garbage collector.** The runtime automatically reclaims memory that is no
  longer reachable. You never call `free()` or `delete`. You never worry about
  use-after-free bugs.
- **Generational.** The GC divides objects into generations based on age. Most
  objects die young (temporary strings, intermediate lists), so the GC
  frequently scans the young generation (fast, small) and rarely scans the old
  generation (slow, large). This makes typical GC pauses very short.
- **Concurrent.** The GC runs alongside your program's tasks, not in
  stop-the-world pauses. While the GC does occasionally need brief pauses to
  maintain consistency, these are measured in microseconds, not milliseconds.

### 12.1.2 No Manual Memory Management

Unlike C, C++, or Rust, VibeLang does not expose manual memory management to the
programmer. There is no `malloc`, no `free`, no lifetime annotations, no borrow
checker in the Rust sense.

This is a deliberate trade-off:

| Approach               | Safety        | Performance    | Complexity     |
|------------------------|---------------|----------------|----------------|
| Manual (C/C++)         | Programmer    | Maximum        | High           |
| Ownership (Rust)       | Compile-time  | Near-maximum   | High           |
| GC (VibeLang)          | Runtime       | Very good      | Low            |

VibeLang chooses the GC approach because it dramatically reduces the learning
curve and eliminates entire categories of bugs (use-after-free, double-free,
dangling pointers) while still achieving excellent performance for the vast
majority of programs.

### 12.1.3 Immutability by Default

VibeLang's most important memory safety feature is also its simplest:
**immutability by default.** When you write:

```vibe
name := "Alice"
scores := [90, 85, 92]
config := {"timeout": 5000}
```

These bindings are immutable. You cannot reassign them, and the data they point
to cannot be modified. This has profound consequences for concurrency:

- **Immutable data can be freely shared between tasks.** If no one can modify
  it, there are no data races.
- **The GC can reason about immutable data more efficiently.** Immutable objects
  in the old generation rarely need scanning.
- **Code is easier to reason about.** When you see an immutable binding, you
  know its value at any point in the program.

Mutation requires an explicit `mut`:

```vibe
mut counter := 0
counter = counter + 1
```

This explicitness makes mutation visible and auditable. In a code review, you
can quickly scan for `mut` to find all points of mutation.

## 12.2 Ownership and Borrowing

### 12.2.1 Value Semantics vs Reference Semantics

VibeLang uses **value semantics** for primitive types and **reference semantics**
for heap-allocated types (strings, lists, maps). Understanding the difference is
critical:

```vibe
a := 42
b := a
```

For integers, `b` gets a *copy* of `a`'s value. Modifying one (if it were
mutable) would not affect the other.

```vibe
xs := [1, 2, 3]
ys := xs
```

For lists, `ys` gets a reference to the same underlying data as `xs`. But
because both bindings are immutable, this sharing is safe — neither can modify
the shared data.

### 12.2.2 How VibeLang Differs from Rust

Rust uses an ownership and borrowing system with lifetime annotations to
guarantee memory safety at compile time without a GC. VibeLang takes a different
path:

| Concept              | Rust                          | VibeLang                       |
|----------------------|-------------------------------|--------------------------------|
| Memory reclamation   | Ownership + drop              | Garbage collector              |
| Sharing              | Borrowing with lifetimes      | Immutable sharing (free)       |
| Mutation             | `&mut` exclusive reference    | `mut` binding                  |
| Concurrency safety   | `Send`/`Sync` traits          | Sendability checks             |
| Learning curve       | Steep (borrow checker)        | Gentle (GC + immutability)     |

VibeLang's approach is simpler: immutable data is freely shareable, mutable data
is restricted at task boundaries, and the GC handles reclamation. You don't need
to think about lifetimes, borrowing rules, or drop order.

### 12.2.3 Move Semantics for Channel Sends

When you send a value through a channel, VibeLang uses **move semantics**: the
value is transferred from the sender to the receiver. After sending, the sender
should not use the value:

```vibe
pub main() -> Int {
    @effect concurrency
    @effect alloc

    ch := chan(1)
    data := [1, 2, 3, 4, 5]

    ch.send(data)

    sum := 0
    for n in data {
        sum = sum + n
    }

    0
}
```

```
error[E0802]: use of moved value `data`
  --> move.yb:11:14
   |
8  |     ch.send(data)
   |             ---- value moved here (sent to channel)
...
11 |     for n in data {
   |              ^^^^ value used after move
   |
   = help: if you need to use `data` after sending, create a copy first
```

Why move semantics? Because after you send a value to another task through a
channel, that other task now owns it. If the sender could also continue using
it, you'd have two tasks accessing the same mutable data — a data race.

The fix is to copy the data before sending if you need to keep using it:

```vibe
pub main() -> Int {
    @effect concurrency
    @effect alloc

    ch := chan(1)
    data := [1, 2, 3, 4, 5]
    data_copy := data.clone()

    ch.send(data_copy)

    mut sum := 0
    for n in data {
        sum = sum + n
    }

    0
}
```

### 12.2.4 Capture Rules for `go` Tasks

When a `go` block references variables from the enclosing scope, it *captures*
them. The capture rules are:

1. **Immutable bindings** are captured by sharing. Since they can't be modified,
   sharing is safe.
2. **Mutable bindings** cannot be captured. The compiler rejects this to prevent
   data races.
3. **Channel handles** are always capturable — they're designed for cross-task
   use.

```vibe
pub main() -> Int {
    @effect concurrency

    name := "Alice"
    ch := chan(1)
    mut count := 0

    go {
        print("Hello, " + name)
        ch.send(42)
    }

    go {
        count = count + 1
    }

    0
}
```

```
error[E0801]: cannot capture mutable binding `count` in `go` block
  --> capture.yb:12:5
   |
6  |     mut count := 0
   |     --- mutable binding declared here
...
12 |     go {
   |     ^^ `go` block captures `count`
13 |         count = count + 1
   |         ^^^^^ mutation in spawned task
```

The first `go` block is fine: it captures `name` (immutable) and `ch` (channel
handle). The second `go` block is rejected because it captures `mut count`.

## 12.3 Sendability Rules

### 12.3.1 What Makes a Value Sendable

A value is **sendable** if it can safely cross a task boundary — via `go`
capture or `channel.send()`. The compiler checks sendability at every task
boundary.

Sendable types:

| Type                    | Sendable? | Reason                                    |
|-------------------------|-----------|-------------------------------------------|
| `Int`, `Bool`           | Yes       | Primitive, copied by value                |
| `Str`                   | Yes       | Immutable, safe to share                  |
| `List<T>` (immutable)   | Yes*      | If `T` is sendable                        |
| `Map<K,V>` (immutable)  | Yes*      | If `K` and `V` are sendable               |
| `Chan<T>`               | Yes       | Designed for cross-task communication      |
| `Result<T,E>`           | Yes*      | If `T` and `E` are sendable               |
| Mutable binding         | No        | Could cause data races                    |

*Sendability is recursive: a `List<List<Int>>` is sendable because `List<Int>`
is sendable because `Int` is sendable.

### 12.3.2 Compile-Time Sendability Checks

The compiler performs sendability analysis at every point where a value crosses
a task boundary:

```vibe
pub main() -> Int {
    @effect concurrency
    @effect alloc

    ch : Chan<List<Int>> := chan(1)

    data := [1, 2, 3]
    ch.send(data)

    0
}
```

This compiles successfully: `data` is an immutable `List<Int>`, which is
sendable.

### 12.3.3 Values That Cannot Cross Task Boundaries

The primary non-sendable category is mutable bindings and values derived from
them in ways that preserve mutability:

```vibe
pub main() -> Int {
    @effect concurrency

    mut items := [1, 2, 3]

    go {
        items.append(4)
    }

    0
}
```

```
error[E0801]: cannot capture mutable binding `items` in `go` block
  --> send_mut.yb:5:5
   |
3  |     mut items := [1, 2, 3]
   |     --- mutable binding declared here
5  |     go {
   |     ^^ `go` block captures `items`
6  |         items.append(4)
   |         ^^^^^ mutation in spawned task
   |
   = note: mutable bindings cannot cross task boundaries
   = help: send the data through a channel, or make an immutable copy
```

## 12.4 Shared Mutable State

### 12.4.1 Why Shared Mutable State Is Dangerous

Shared mutable state is the root cause of most concurrency bugs. When two tasks
can both read and write the same memory location without synchronization, the
result depends on the order of execution — which is non-deterministic:

```
Task A: read counter  → gets 0
Task B: read counter  → gets 0
Task A: write counter ← sets 1
Task B: write counter ← sets 1    (should be 2!)
```

This is a **data race**: the final value of `counter` depends on which task's
write happens last. The program produces different results on different runs, or
even on different CPU cores.

VibeLang prevents this at compile time by forbidding mutable bindings from
crossing task boundaries.

### 12.4.2 The `mut_state` Effect

Functions that manage mutable state internally should declare the `mut_state`
effect to signal this to callers:

```vibe
pub accumulate(values: List<Int>) -> Int {
    @effect mut_state
    @effect alloc

    mut total := 0
    mut max_seen := 0

    for v in values {
        total = total + v
        if v > max_seen {
            max_seen = v
        }
    }

    total + max_seen
}
```

The `@effect mut_state` annotation doesn't change the compiler's behavior — it's
a documentation and composition signal. It tells callers "this function uses
internal mutable state" which is relevant for reasoning about determinism and
side effects.

### 12.4.3 Safe Patterns for Shared State

When multiple tasks need to coordinate around shared data, VibeLang provides
safe patterns:

**Pattern 1: Single owner with channel interface.**

One task owns the mutable state. Other tasks communicate with it through
channels:

```vibe
pub state_manager(
    requests: Chan<Str>,
    responses: Chan<Int>
) -> Int {
    @effect concurrency

    mut counter := 0

    for cmd in requests {
        if cmd == "increment" {
            counter = counter + 1
            responses.send(counter)
        } else if cmd == "get" {
            responses.send(counter)
        }
    }

    counter
}

pub main() -> Int {
    @effect concurrency

    requests := chan(10)
    responses := chan(10)

    go state_manager(requests, responses)

    requests.send("increment")
    val1 := responses.recv()

    requests.send("increment")
    val2 := responses.recv()

    requests.send("get")
    val3 := responses.recv()

    requests.close()

    0
}
```

Only `state_manager` touches `counter`. All other tasks interact through
channels. This eliminates data races by design.

**Pattern 2: Reduce via channels.**

Each task computes a partial result independently, then sends it to a collector:

```vibe
pub main() -> Int {
    @effect concurrency
    @effect alloc

    results := chan(4)

    go { results.send(compute_chunk_a()) }
    go { results.send(compute_chunk_b()) }
    go { results.send(compute_chunk_c()) }
    go { results.send(compute_chunk_d()) }

    mut total := 0
    mut received := 0
    for received < 4 {
        total = total + results.recv()
        received = received + 1
    }

    total
}
```

No shared mutable state exists. Each task has its own local computation, and
results flow through the channel.

### 12.4.4 Using Channels Instead of Shared Memory

VibeLang's philosophy can be summarized as:

> **Don't communicate by sharing memory; share memory by communicating.**

This principle, borrowed from Go, is enforced by VibeLang's compiler. You
*can't* share mutable memory between tasks (the compiler prevents it), so you
*must* communicate through channels. This constraint, while occasionally
inconvenient, eliminates the most common and hardest-to-debug class of
concurrency errors.

## 12.5 Memory Ordering and Happens-Before

### 12.5.1 What Is Happens-Before?

In a concurrent program, two events in different tasks have no inherent ordering.
The CPU, the OS scheduler, and the runtime can execute them in any order. A
**happens-before** relationship is a guarantee that one event is visible to
another — that the effects of event A are fully committed before event B
observes them.

Without happens-before guarantees, one task might read stale data that another
task has already updated. VibeLang establishes happens-before through specific
synchronization points.

### 12.5.2 Channels Establish Happens-Before

Every channel send-receive pair creates a happens-before relationship:

```
Task A: x := 42; ch.send(x)    happens-before    Task B: y := ch.recv()
```

After `ch.recv()` returns in Task B, Task B is guaranteed to see the value `42`
that Task A sent. This seems obvious, but it's a deep guarantee: it means all
memory effects in Task A *before* the send are visible to Task B *after* the
receive.

This is why channels are the primary synchronization mechanism. They don't just
transfer values — they transfer *visibility*.

### 12.5.3 Task Joins Establish Ordering

When you join a task, you establish a happens-before relationship between the
task's completion and the join point:

```vibe
task := go {
    compute_something()
}

result := task.join()
```

Everything that happened inside the `go` block is guaranteed to be visible after
`task.join()` returns. This means `result` reflects the complete computation,
not a partial or stale view.

### 12.5.4 Why This Matters for Correctness

Consider this pattern:

```vibe
pub main() -> Int {
    @effect concurrency
    @effect alloc

    ch := chan(1)

    go {
        data := expensive_computation()
        ch.send(data)
    }

    result := ch.recv()
    process(result)

    0
}
```

The happens-before guarantee from the channel ensures that `result` contains the
complete output of `expensive_computation()`. Without this guarantee, `process`
might see partially-constructed data — a bug that would be nearly impossible to
reproduce or debug.

VibeLang's memory model makes this guarantee automatic. You don't need to insert
memory barriers or use atomic operations. Channels and task joins handle it.

## 12.6 The Garbage Collector

### 12.6.1 Generational, Concurrent GC

VibeLang's GC uses a generational strategy with concurrent collection:

**Young generation (nursery):**
- Newly allocated objects start here.
- Collected frequently (every few milliseconds under load).
- Collection is fast because most young objects are already dead.
- Surviving objects are promoted to the old generation.

**Old generation:**
- Long-lived objects reside here.
- Collected infrequently.
- Collection runs concurrently with program execution.
- Only brief pauses are needed for root scanning.

### 12.6.2 GC Pauses and Performance

VibeLang's GC is designed for low-latency applications:

- **Young generation pauses:** typically < 1ms. These are frequent but fast.
- **Old generation pauses:** typically < 10ms. These are rare and mostly
  concurrent.
- **Worst-case pauses:** bounded by heap size and allocation rate, but
  generally well under 50ms even for large heaps.

For most applications, GC pauses are invisible. For latency-sensitive
applications (real-time systems, game loops), you can tune GC behavior through
runtime configuration.

### 12.6.3 When Allocation Happens (The `alloc` Effect)

The `alloc` effect marks functions that allocate heap memory:

```vibe
pub build_report(data: List<Int>) -> Map<Str, Int> {
    @effect alloc

    mut report : Map<Str, Int> := {}
    report.set("count", data.len())

    mut sum := 0
    for n in data {
        sum = sum + n
    }
    report.set("total", sum)
    report.set("average", sum / data.len())

    report
}
```

Why track allocation? Because allocation is the primary driver of GC pressure.
Functions that allocate heavily cause more frequent GC cycles. By marking these
functions with `@effect alloc`, you can:

1. **Identify hot allocation paths** during code review.
2. **Optimize selectively** — focus on reducing allocations in functions that
   run in tight loops.
3. **Compose safely** — the effect system ensures that a function claiming to be
   allocation-free actually is.

### 12.6.4 Memory Footprint Characteristics

VibeLang programs have predictable memory characteristics:

- **Strings** are compact (UTF-8 is space-efficient) but each concatenation
  allocates a new string.
- **Lists** use contiguous memory with amortized growth (doubling strategy).
  A list of N integers uses approximately `N * 8` bytes plus overhead.
- **Maps** use a hash table with open addressing. Memory usage is approximately
  `N * (key_size + value_size + overhead)` with a load factor around 0.75.
- **Channels** allocate a fixed buffer at creation time. A `chan(100)` of
  integers uses approximately `100 * 8` bytes.
- **Tasks** have a small initial stack (typically 4–8 KB) that grows on demand.
  Idle tasks consume minimal memory.

For most programs, you don't need to think about these details. But when
optimizing memory-intensive applications, understanding the footprint of each
data structure helps you make informed decisions.

## 12.7 Summary

VibeLang's ownership, sendability, and memory model work together to provide
safe concurrency without manual memory management:

- **The garbage collector** handles memory reclamation automatically. Its
  generational, concurrent design keeps pauses short and predictable.

- **Immutability by default** is the foundation of safety. Immutable data can be
  freely shared between tasks because no one can modify it.

- **Move semantics for channel sends** ensure that values transferred between
  tasks don't create aliasing. After sending, the sender can no longer use the
  value.

- **Capture rules for `go` blocks** prevent mutable bindings from crossing task
  boundaries. The compiler rejects code that would create data races.

- **Sendability checks** are recursive and automatic. The compiler verifies at
  every task boundary that the transferred values are safe.

- **Happens-before relationships** are established by channels and task joins,
  giving you deterministic visibility guarantees without manual memory barriers.

- **The `alloc` and `mut_state` effects** make allocation and mutation visible
  in function signatures, enabling informed optimization and safe composition.

Together, these mechanisms mean that if your VibeLang program compiles, it is
free from data races, use-after-free bugs, and memory corruption. The compiler
does the hard work so you can focus on your program's logic.

In the next chapter, we'll build on everything covered so far to explore
advanced patterns and real-world programs that combine collections, modules,
concurrency, and contracts into complete, production-ready applications.
