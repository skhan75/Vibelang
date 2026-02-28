# Chapter 11: Concurrency with `go`, `chan`, and `select`

Concurrency is where VibeLang diverges most sharply from languages that treat
parallelism as an afterthought. VibeLang's concurrency model is inspired by Go's
goroutines and channels but adds compile-time safety that Go lacks: the compiler
verifies that values crossing task boundaries are safe to send, that concurrent
functions declare their effects, and that channel protocols are used correctly.

This chapter is the longest in the book for good reason. Concurrency bugs are
among the hardest to find and fix. Understanding VibeLang's model deeply — not
just the syntax, but *why* each rule exists — will save you hours of debugging.

## 11.1 VibeLang's Concurrency Model

### 11.1.1 Go-Inspired, Compiler-Enforced

VibeLang's concurrency draws from Go's proven model: lightweight tasks
communicate through typed channels. But where Go relies on runtime conventions
and programmer discipline, VibeLang enforces safety at compile time:

| Feature                  | Go                        | VibeLang                          |
|--------------------------|---------------------------|-----------------------------------|
| Lightweight tasks        | goroutines                | `go` tasks                        |
| Communication            | channels                  | typed `Chan<T>` channels          |
| Multiplexing             | `select`                  | `select` with `after`/`closed`    |
| Data race prevention     | race detector (runtime)   | sendability checks (compile time) |
| Effect tracking          | none                      | `@effect concurrency`             |
| Scheduler                | M:N                       | M:N                               |

The key difference is the last two rows. VibeLang catches data races and missing
concurrency annotations *before your program runs*.

### 11.1.2 The M:N Scheduler

VibeLang uses an M:N scheduler: M lightweight tasks are multiplexed onto N
operating system threads. This means:

- **Tasks are cheap.** Creating a task allocates a small stack (a few KB), not
  an OS thread (typically 1–8 MB). You can spawn thousands or millions of tasks.
- **Tasks are preemptively scheduled.** The runtime can suspend a task at
  well-defined points (channel operations, function calls) and run another.
- **N is typically the number of CPU cores.** The runtime creates a thread pool
  sized to the hardware, then schedules tasks across those threads.

You don't control which thread a task runs on, and you shouldn't need to. The
scheduler handles that. What you control is the *structure* of your concurrent
program: which tasks exist, how they communicate, and when they finish.

### 11.1.3 Why Structured Concurrency Matters

Unstructured concurrency — where any code can spawn a background thread that
outlives its caller — leads to resource leaks, orphaned tasks, and
impossible-to-reproduce bugs. VibeLang encourages structured concurrency through
several mechanisms:

- Tasks spawned with `go` can be joined to wait for their completion.
- Channels provide explicit communication paths that are visible in the code.
- The `@effect concurrency` annotation makes concurrent code visible in function
  signatures.
- The compiler rejects code that sends unsafe values across task boundaries.

The result: when you read a VibeLang function's signature, you know whether it
spawns tasks, and the type system guarantees that data flows safely between them.

## 11.2 Spawning Tasks with `go`

### 11.2.1 Basic Syntax

The `go` keyword spawns a new lightweight task that runs concurrently:

```vibe
pub main() -> Int {
    @effect concurrency

    go print("hello from a task")

    0
}
```

The expression after `go` runs in a new task. The spawning task continues
immediately — it does not wait for the spawned task to finish.

### 11.2.2 What Happens When You Spawn

When the runtime encounters `go expr`:

1. It allocates a lightweight task with a small initial stack.
2. It captures any values that `expr` references from the enclosing scope.
3. It places the task on the scheduler's run queue.
4. The spawning task continues to the next statement immediately.
5. At some point, a worker thread picks up the new task and executes `expr`.

The order in which tasks execute is non-deterministic. You must not rely on a
spawned task running before or after any particular statement in the spawning
task. Channels are the mechanism for coordinating order.

### 11.2.3 The `@effect concurrency` Requirement

Any function that uses `go`, `chan`, or `select` must declare the concurrency
effect:

```vibe
pub process_parallel(items: List<Int>) -> Int {
    @effect concurrency
    @effect alloc

    ch := chan(10)

    go {
        for item in items {
            ch.send(item * 2)
        }
        ch.close()
    }

    mut total := 0
    for result in ch {
        total = total + result
    }
    total
}
```

If you forget the annotation, the compiler tells you:

```
error[E0701]: function uses concurrency primitives but missing `@effect concurrency`
 --> parallel.yb:1:1
  |
1 | pub process_parallel(items: List<Int>) -> Int {
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing effect annotation
  |
5 |     ch := chan(10)
  |            ------- `chan` requires `@effect concurrency`
  |
  = help: add `@effect concurrency` to the function body
```

This is not bureaucracy — it's a feature. When you see a function *without*
`@effect concurrency`, you know it doesn't spawn tasks or use channels. This
makes reasoning about program behavior dramatically easier.

### 11.2.4 Tasks with Return Values

A `go` expression returns a task handle that you can join to get the result:

```vibe
pub main() -> Int {
    @effect concurrency

    task := go {
        heavy_computation(1000)
    }

    other_work()

    result := task.join()
    result
}
```

`task.join()` blocks the current task until the spawned task completes, then
returns its result. This is how you wait for concurrent work to finish and
retrieve its output.

### 11.2.5 Spawning Multiple Tasks

A common pattern is spawning several tasks and collecting their results:

```vibe
pub parallel_sum(chunks: List<List<Int>>) -> Int {
    @effect concurrency
    @effect alloc

    mut tasks : List<Task<Int>> := []

    for chunk in chunks {
        t := go {
            mut sum := 0
            for n in chunk {
                sum = sum + n
            }
            sum
        }
        tasks.append(t)
    }

    mut total := 0
    for t in tasks {
        total = total + t.join()
    }
    total
}
```

Each chunk is processed in its own task, and the main task collects the partial
sums. The `join()` calls ensure all tasks complete before we return.

## 11.3 Channels (`Chan<T>`)

### 11.3.1 Creating Channels

Channels are typed conduits for sending values between tasks. Create one with
`chan(capacity)`:

```vibe
messages := chan(10)
```

This creates a `Chan<Int>` (or whatever type the compiler infers from usage)
with a buffer capacity of 10. The capacity determines how many values can be
buffered before a sender blocks.

You can also specify the type explicitly:

```vibe
ch : Chan<Str> := chan(5)
```

### 11.3.2 Sending Values

Use `.send(value)` to put a value into a channel:

```vibe
ch := chan(3)
ch.send(42)
ch.send(43)
ch.send(44)
```

If the channel's buffer is full, `.send()` blocks the current task until another
task receives a value, freeing space. This is **backpressure** — a fast producer
is automatically slowed to match a slow consumer.

### 11.3.3 Receiving Values

Use `.recv()` to take a value from a channel:

```vibe
ch := chan(3)
ch.send(42)

value := ch.recv()
```

`value` is `42`. If the channel is empty, `.recv()` blocks the current task
until a value is available.

This blocking behavior is fundamental to channel-based concurrency. It means
tasks naturally synchronize: a consumer waits for a producer, and a producer
waits for a consumer (when the buffer is full). No explicit locks or condition
variables are needed.

### 11.3.4 Closing Channels

Use `.close()` to signal that no more values will be sent:

```vibe
ch := chan(10)
ch.send(1)
ch.send(2)
ch.close()
```

After closing:

- **Receiving** from a closed channel returns any remaining buffered values.
  Once the buffer is drained, further `.recv()` calls return a sentinel or can
  be detected with `select { case closed ch => ... }`.
- **Sending** to a closed channel is a runtime error:

```
runtime error: send on closed channel
 --> producer.yb:5:1
```

- **Closing** an already-closed channel is also a runtime error:

```
runtime error: close of closed channel
 --> cleanup.yb:8:1
```

The close operation is a one-way signal: it tells consumers "the producer is
done." This is essential for clean shutdown patterns.

### 11.3.5 Iterating Over a Channel

You can use `for ... in` to receive all values from a channel until it's closed:

```vibe
pub main() -> Int {
    @effect concurrency
    @effect alloc

    ch := chan(10)

    go {
        for i in [1, 2, 3, 4, 5] {
            ch.send(i)
        }
        ch.close()
    }

    mut total := 0
    for value in ch {
        total = total + value
    }

    total
}
```

The `for value in ch` loop receives values one at a time. When the channel is
closed and its buffer is drained, the loop exits. This is the cleanest way to
consume a stream of values.

### 11.3.6 Bounded Channels and Backpressure

The channel capacity is not just a buffer size — it's a design decision about
how your system handles load:

- **`chan(0)`** — a rendezvous channel. Every send blocks until a receiver is
  ready, and every receive blocks until a sender is ready. This provides the
  tightest synchronization.
- **`chan(1)`** — minimal buffering. The producer can get one step ahead of the
  consumer.
- **`chan(N)`** — the producer can get up to N steps ahead. This smooths out
  temporary speed differences between producer and consumer.

Choosing the right capacity is a design trade-off:

| Capacity | Latency    | Throughput | Memory   | Coupling     |
|----------|------------|------------|----------|--------------|
| 0        | Lowest     | Lowest     | Minimal  | Tightest     |
| 1–10     | Low        | Moderate   | Low      | Moderate     |
| 100–1000 | Moderate   | High       | Moderate | Loose        |
| 10000+   | Higher     | Highest    | High     | Very loose   |

A good starting point is a small buffer (10–100) and adjusting based on
profiling. Unbounded channels don't exist in VibeLang by design — they hide
memory leaks and remove backpressure.

## 11.4 The Worker Pool Pattern

### 11.4.1 Why Worker Pools

Many concurrent programs follow the same shape: a set of jobs arrives, and you
want to process them using a fixed number of workers. This bounds resource usage
(you don't spawn millions of tasks) while still achieving parallelism.

### 11.4.2 Fan-Out: Distributing Work

Fan-out means one producer sends jobs to multiple consumers:

```vibe
pub worker(id: Int, jobs: Chan<Int>, results: Chan<Int>) -> Int {
    @effect concurrency

    for job in jobs {
        result := job * job
        results.send(result)
    }
    0
}

pub main() -> Int {
    @effect concurrency
    @effect alloc

    num_workers := 4
    num_jobs := 20

    jobs := chan(num_jobs)
    results := chan(num_jobs)

    mut i := 0
    for _ in 0..num_workers {
        go worker(i, jobs, results)
        i = i + 1
    }

    mut j := 0
    for _ in 0..num_jobs {
        jobs.send(j)
        j = j + 1
    }
    jobs.close()

    mut total := 0
    mut received := 0
    for received < num_jobs {
        total = total + results.recv()
        received = received + 1
    }

    total
}
```

When `jobs.close()` is called, each worker's `for job in jobs` loop eventually
exits. The workers compete to receive from the same channel — the runtime
distributes jobs fairly among them.

### 11.4.3 Fan-In: Collecting Results

Fan-in is the reverse: multiple producers send to a single results channel. In
the example above, all workers send to `results`. The main task receives from
`results` until it has collected all expected outputs.

### 11.4.4 Complete Working Example: Parallel Score Processing

Here's a complete program that uses a worker pool to process exam scores in
parallel:

```vibe
module score_processor

pub process_score(score: Int) -> Map<Str, Int> {
    @effect alloc

    mut result : Map<Str, Int> := {}
    result.set("original", score)
    result.set("curved", score + 5)

    if score >= 90 {
        result.set("grade", 4)
    } else if score >= 80 {
        result.set("grade", 3)
    } else if score >= 70 {
        result.set("grade", 2)
    } else {
        result.set("grade", 1)
    }

    result
}

pub score_worker(
    id: Int,
    jobs: Chan<Int>,
    results: Chan<Map<Str, Int>>
) -> Int {
    @effect concurrency
    @effect alloc

    for score in jobs {
        processed := process_score(score)
        results.send(processed)
    }
    0
}

pub main() -> Int {
    @effect concurrency
    @effect alloc

    scores := [85, 92, 78, 95, 88, 67, 73, 91, 82, 76, 94, 89]
    num_workers := 4

    jobs := chan(scores.len())
    results := chan(scores.len())

    mut w := 0
    for _ in 0..num_workers {
        go score_worker(w, jobs, results)
        w = w + 1
    }

    for score in scores {
        jobs.send(score)
    }
    jobs.close()

    mut total_curved := 0
    mut count := 0
    for count < scores.len() {
        result := results.recv()
        total_curved = total_curved + result.get("curved")
        count = count + 1
    }

    avg := total_curved / scores.len()
    print("Average curved score: " + avg.to_str())

    0
}
```

This program demonstrates the full worker pool pattern: jobs channel for input,
results channel for output, multiple workers processing in parallel, and a
collector aggregating results.

## 11.5 Multiplexing with `select`

### 11.5.1 The Problem `select` Solves

Sometimes a task needs to wait on multiple channels simultaneously. Without
`select`, you'd have to poll channels in a loop, wasting CPU cycles. `select`
lets you wait efficiently until *any* of several channel operations is ready.

### 11.5.2 Basic `select` Syntax

```vibe
select {
    case msg := ch1.recv() => {
        print("received from ch1: " + msg.to_str())
    }
    case msg := ch2.recv() => {
        print("received from ch2: " + msg.to_str())
    }
}
```

The `select` statement evaluates all cases simultaneously. Whichever channel has
a value ready first wins. If multiple channels are ready, one is chosen
non-deterministically.

### 11.5.3 Timeout with `case after`

The `after` case triggers if no other case is ready within the specified
duration:

```vibe
select {
    case msg := ch.recv() => {
        handle_message(msg)
    }
    case after 5s => {
        print("timed out waiting for message")
    }
}
```

Duration literals use suffixes: `ms` for milliseconds, `s` for seconds, `m` for
minutes. Examples: `100ms`, `5s`, `2m`.

Timeouts are essential for building robust systems. Without them, a `recv()` on
a channel whose producer has crashed will block forever. With `after`, you can
detect the problem and take corrective action.

### 11.5.4 Shutdown Detection with `case closed`

The `closed` case triggers when a channel has been closed and its buffer is
empty:

```vibe
pub consumer(ch: Chan<Int>) -> Int {
    @effect concurrency

    mut running := true
    mut total := 0

    for running {
        select {
            case value := ch.recv() => {
                total = total + value
            }
            case closed ch => {
                running = false
            }
        }
    }

    total
}
```

This pattern is how you build consumers that process values until the producer
signals completion by closing the channel.

### 11.5.5 Non-Blocking with `default`

The `default` case executes immediately if no other case is ready:

```vibe
select {
    case msg := ch.recv() => {
        process(msg)
    }
    case default => {
        do_other_work()
    }
}
```

This turns a blocking `recv()` into a non-blocking poll. Use it sparingly — a
tight loop with `default` consumes CPU. It's appropriate when you have
productive work to do while waiting.

### 11.5.6 Combining Cases

A real-world `select` often combines multiple channels, timeouts, and shutdown
detection:

```vibe
pub event_loop(
    commands: Chan<Str>,
    metrics: Chan<Int>,
    shutdown: Chan<Bool>
) -> Int {
    @effect concurrency

    mut running := true
    mut processed := 0

    for running {
        select {
            case cmd := commands.recv() => {
                execute_command(cmd)
                processed = processed + 1
            }
            case metric := metrics.recv() => {
                record_metric(metric)
            }
            case _ := shutdown.recv() => {
                print("shutdown signal received")
                running = false
            }
            case after 30s => {
                print("no activity for 30 seconds")
            }
        }
    }

    processed
}
```

## 11.6 Sendability

### 11.6.1 What the Compiler Checks

When you send a value across a task boundary — via `go` capture or
`channel.send()` — the compiler verifies that the value is **sendable**. A
sendable value is one that can safely exist in two tasks without causing data
races.

### 11.6.2 Sendable Types

The following types are always sendable:

- **Primitive types:** `Int`, `Bool`, `Str` (strings are immutable, so sharing
  is safe).
- **Immutable collections:** an immutable `List<Int>` or `Map<Str, Int>` can be
  safely shared because no task can modify it.
- **Channel handles:** `Chan<T>` is sendable — that's the whole point of
  channels.

### 11.6.3 Non-Sendable Values

A value is non-sendable if sharing it between tasks could cause a data race:

- **Mutable bindings captured by `go`.** If a `go` block captures a `mut`
  binding from the enclosing scope, both the spawning task and the spawned task
  could mutate it simultaneously.

```vibe
pub main() -> Int {
    @effect concurrency

    mut counter := 0

    go {
        counter = counter + 1
    }

    counter = counter + 1

    0
}
```

```
error[E0801]: cannot capture mutable binding `counter` in `go` block
 --> race.yb:6:5
  |
4 |     mut counter := 0
  |     --- mutable binding declared here
5 |
6 |     go {
  |     ^^ `go` block captures `counter`
7 |         counter = counter + 1
  |         ^^^^^^^ mutation in spawned task
  |
  = note: mutable bindings cannot cross task boundaries
  = help: use a channel to communicate the value instead
```

This error prevents a data race. The fix is to use a channel:

```vibe
pub main() -> Int {
    @effect concurrency

    ch := chan(1)

    go {
        ch.send(1)
    }

    increment := ch.recv()

    0
}
```

### 11.6.4 Designing Sendable Data

To write concurrent code smoothly, design your data to be sendable:

1. **Prefer immutable values.** Immutable data is always safe to share.
2. **Use channels for communication.** Instead of sharing mutable state, send
   values through channels.
3. **Copy before sending.** If you need to send data that's currently mutable,
   create an immutable copy first.
4. **Keep task-local state local.** Mutable state that only one task uses
   doesn't need to be sendable.

## 11.7 Common Concurrency Patterns

### 11.7.1 Producer-Consumer

The simplest concurrent pattern: one task produces data, another consumes it.

```vibe
pub producer(ch: Chan<Int>) -> Int {
    @effect concurrency

    mut i := 0
    for i < 100 {
        ch.send(i * i)
        i = i + 1
    }
    ch.close()
    0
}

pub consumer(ch: Chan<Int>) -> Int {
    @effect concurrency

    mut sum := 0
    for value in ch {
        sum = sum + value
    }
    sum
}

pub main() -> Int {
    @effect concurrency

    ch := chan(20)

    go producer(ch)
    result := consumer(ch)

    result
}
```

The channel buffer (20) lets the producer get ahead of the consumer, improving
throughput. When the producer closes the channel, the consumer's `for` loop
exits naturally.

### 11.7.2 Pipeline (Chain of Channels)

A pipeline connects stages where each stage's output feeds the next stage's
input:

```vibe
pub generate(out: Chan<Int>) -> Int {
    @effect concurrency
    mut i := 1
    for i <= 50 {
        out.send(i)
        i = i + 1
    }
    out.close()
    0
}

pub square(input: Chan<Int>, out: Chan<Int>) -> Int {
    @effect concurrency
    for n in input {
        out.send(n * n)
    }
    out.close()
    0
}

pub filter_even(input: Chan<Int>, out: Chan<Int>) -> Int {
    @effect concurrency
    for n in input {
        if n % 2 == 0 {
            out.send(n)
        }
    }
    out.close()
    0
}

pub main() -> Int {
    @effect concurrency
    @effect alloc

    ch1 := chan(10)
    ch2 := chan(10)
    ch3 := chan(10)

    go generate(ch1)
    go square(ch1, ch2)
    go filter_even(ch2, ch3)

    mut sum := 0
    for value in ch3 {
        sum = sum + value
    }

    sum
}
```

Data flows: `generate → square → filter_even → main`. Each stage runs
concurrently. The pipeline naturally handles backpressure — if `filter_even` is
slow, `square` blocks on its `out.send()`, which in turn slows `generate`.

### 11.7.3 Timeout and Retry

Combining channels with `select` and `after` enables timeout-and-retry logic:

```vibe
pub fetch_with_retry(
    request_ch: Chan<Str>,
    response_ch: Chan<Int>,
    max_retries: Int
) -> Result<Int, Str> {
    @effect concurrency

    mut attempts := 0

    for attempts < max_retries {
        request_ch.send("fetch_data")

        select {
            case result := response_ch.recv() => {
                Ok(result)
            }
            case after 3s => {
                attempts = attempts + 1
                print("attempt " + attempts.to_str() + " timed out, retrying...")
            }
        }
    }

    Err("all " + max_retries.to_str() + " attempts failed")
}
```

Each attempt waits up to 3 seconds for a response. If the timeout fires, it
retries. After exhausting all retries, it returns an error.

### 11.7.4 Graceful Shutdown

A robust concurrent system needs to shut down cleanly — finishing in-progress
work, closing channels, and joining all tasks:

```vibe
pub graceful_shutdown(
    workers: List<Task<Int>>,
    jobs: Chan<Int>,
    shutdown: Chan<Bool>
) -> Int {
    @effect concurrency

    jobs.close()

    shutdown.send(true)

    mut total := 0
    for worker in workers {
        total = total + worker.join()
    }

    total
}
```

The pattern is:

1. Close the jobs channel so workers stop receiving new work.
2. Signal shutdown through a dedicated channel.
3. Join all worker tasks to wait for in-progress work to complete.
4. Aggregate results.

## 11.8 Concurrency and Contracts

### 11.8.1 Effect Tracking for Concurrent Code

The `@effect concurrency` annotation is part of VibeLang's effect system
(Chapter 7). It serves multiple purposes:

- **Documentation.** Readers know immediately that a function involves
  concurrent behavior.
- **Composition checking.** A function without `@effect concurrency` cannot call
  a function that has it — the compiler enforces this.
- **Testing.** Test harnesses can treat concurrent functions differently,
  applying timeouts or running them with deterministic schedulers.

### 11.8.2 Contracts on Concurrent Functions

You can use `@require` and `@ensure` on concurrent functions just like any
other:

```vibe
pub parallel_map(
    items: List<Int>,
    num_workers: Int
) -> List<Int> {
    @effect concurrency
    @effect alloc
    @require items.len() > 0, "need at least one item"
    @require num_workers > 0, "need at least one worker"
    @ensure .len() == items.len()

    jobs := chan(items.len())
    results := chan(items.len())

    mut w := 0
    for w < num_workers {
        go {
            for job in jobs {
                results.send(job * 2)
            }
        }
        w = w + 1
    }

    for item in items {
        jobs.send(item)
    }
    jobs.close()

    mut output : List<Int> := []
    mut count := 0
    for count < items.len() {
        output.append(results.recv())
        count = count + 1
    }

    output
}
```

The `@ensure .len() == items.len()` postcondition guarantees that the parallel
map produces exactly as many outputs as inputs — a critical correctness property
that the contract system can verify.

### 11.8.3 Testing Concurrent Code

Testing concurrent code requires care because task scheduling is
non-deterministic. VibeLang's testing framework provides tools to help:

```vibe
module tests.parallel

import score_processor

pub test_parallel_map() -> Int {
    @effect concurrency
    @effect alloc

    input := [1, 2, 3, 4, 5]
    result := parallel_map(input, 2)

    assert(result.len() == 5, "should produce 5 results")

    mut sorted_result := result
    sorted_result.sort_desc()

    expected := [10, 8, 6, 4, 2]
    mut i := 0
    for i < expected.len() {
        assert(
            sorted_result.get(i) == expected.get(i),
            "mismatch at index " + i.to_str()
        )
        i = i + 1
    }

    0
}
```

Because the worker pool may process items in any order, the test sorts the
results before comparing. This is a standard technique for testing concurrent
code: verify the *set* of results, not their order.

## 11.9 Summary

VibeLang's concurrency model gives you the power of lightweight tasks and
channels with the safety of compile-time checks:

- **`go expr`** spawns a lightweight task on the M:N scheduler. Tasks are cheap
  — spawn thousands without worry.

- **`chan(capacity)`** creates a bounded, typed channel. Channels are the
  primary communication mechanism between tasks. Bounded capacity provides
  natural backpressure.

- **`.send()` and `.recv()`** transfer values between tasks through channels.
  Both can block, which provides implicit synchronization.

- **`.close()`** signals that no more values will be sent. Consumers detect
  this via `for ... in` loops or `select { case closed ch => ... }`.

- **`select`** multiplexes across multiple channels, with support for timeouts
  (`after`), shutdown detection (`closed`), and non-blocking polls (`default`).

- **Sendability** is checked at compile time. The compiler prevents you from
  sharing mutable state across task boundaries, eliminating data races before
  your program runs.

- **`@effect concurrency`** marks functions that use concurrent primitives,
  making concurrency visible in function signatures and enabling the compiler to
  enforce effect discipline.

The patterns in this chapter — producer-consumer, worker pools, pipelines,
timeout-and-retry, graceful shutdown — form the building blocks of real
concurrent systems. In the next chapter, we'll go deeper into ownership,
sendability, and VibeLang's memory model to understand *why* these patterns are
safe.
