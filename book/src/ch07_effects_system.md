# Chapter 7: The Effects System

Most programming languages treat side effects as invisible. A function that
reads a file, allocates memory, and sends a network request looks the same in
its signature as a function that adds two numbers. VibeLang rejects this
opacity. Every function must declare the kinds of side effects it performs, and
the compiler enforces these declarations transitively through the entire call
graph.

This chapter explains why VibeLang tracks effects, how the system works, what
each effect means, and how to use effect information to write safer, more
testable, and more performant code.

---

## 7.1 What Are Effects?

A **side effect** is anything a function does beyond computing a return value
from its inputs. Reading from the console is an effect. Writing to a file is an
effect. Allocating heap memory is an effect. Spawning a concurrent task is an
effect.

In most languages, you discover effects by reading the implementation — tracing
through every function call to see if anything "reaches out" to the world. This
is tedious for humans and nearly impossible for tools to do reliably without
whole-program analysis.

VibeLang takes a different approach: functions declare their effects explicitly,
and the compiler verifies the declarations are accurate.

### Why Track Effects?

Three practical reasons:

1. **Code review speed.** A reviewer can look at a function's effect declarations
   and immediately know its operational footprint. A function with no effects is
   pure — it cannot corrupt state, fail due to I/O errors, or behave
   nondeterministically. A function with `@effect net` touches the network. This
   information is available without reading the implementation.

2. **Test isolation.** Functions with no effects can be tested with simple
   input/output assertions. Functions with `@effect io` or `@effect net` need
   mocking or integration test infrastructure. Effect declarations tell your
   test framework what kind of test setup is required.

3. **Optimization safety.** The compiler can apply aggressive optimizations to
   pure functions (memoization, reordering, dead code elimination) that would be
   unsound for effectful functions. Effect declarations give the optimizer
   precise information about what transformations are safe.

### The Effect Vocabulary

VibeLang defines six base effects:

| Effect | Meaning |
|---|---|
| `alloc` | Heap allocation (creating `List`, `Map`, `Str` concatenation, etc.) |
| `mut_state` | Shared mutable state (modifying a value visible outside the function) |
| `io` | Console or file I/O (`println`, `read_file`, `write_file`) |
| `net` | Network operations (HTTP requests, socket communication) |
| `concurrency` | Concurrent operations (`go`, `chan`, `select`) |
| `nondet` | Nondeterministic operations (random numbers, current time) |

These six effects cover the major categories of "things a function can do to the
outside world." They are intentionally coarse-grained — the goal is to provide
useful signal at review time, not to build a full algebraic effect system.

---

## 7.2 Declaring Effects with `@effect`

### Syntax

```vibe
@effect effect_name
```

Effect declarations are contract annotations and follow the same placement rules
as all other annotations: they appear at the top of the function body, before
any executable statements.

### Single Effect

```vibe
pub greet(name: Str) -> Int {
    @intent "print a greeting to the console"
    @effect io

    println("Hello, " + name + "!")
    0
}
```

### Multiple Effects

A function may declare as many effects as it needs, each on its own line:

```vibe
pub fetch_and_cache(url: Str, mut cache: Map<Str, Str>) -> Result<Str, NetError> {
    @intent "fetch URL content and store it in the cache"
    @effect net
    @effect alloc
    @effect mut_state

    response := http_get(url)?
    body := response.body()
    cache.insert(url, body)
    ok(body)
}
```

### No Effects: Pure Functions

A function with no `@effect` declarations is **pure**. It computes a result
from its inputs and does nothing else:

```vibe
pub add(a: i64, b: i64) -> i64 {
    a + b
}

pub max(a: i64, b: i64) -> i64 {
    if a >= b { a } else { b }
}

pub factorial(n: i64) -> i64 {
    @require n >= 0
    @ensure . >= 1

    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
```

None of these functions allocate, mutate shared state, perform I/O, or do
anything nondeterministic. They are pure, and the absence of `@effect`
declarations communicates this clearly.

---

## 7.3 Effect Transitivity

This is the most important rule in VibeLang's effect system:

> **If function A calls function B, and B declares `@effect X`, then A must also
> declare `@effect X` (or a superset that includes X).**

Effects propagate upward through the call graph. You cannot hide an effect by
wrapping it in another function.

### Example: Transitivity in Action

```vibe
pub write_log(message: Str) -> Int {
    @effect io

    println("[LOG] " + message)
    0
}

pub process_item(item: Item) -> Result<Output, ProcessError> {
    @intent "transform item and log the result"
    @effect io       // Required because we call write_log, which has @effect io
    @effect alloc

    result := transform(item)
    write_log("Processed item: " + item.id)
    ok(result)
}
```

`process_item` calls `write_log`, which declares `@effect io`. Therefore
`process_item` must also declare `@effect io`. If it does not, the compiler
rejects the code.

### Compiler Error: Missing Transitive Effect

```vibe
pub process_item(item: Item) -> Result<Output, ProcessError> {
    @intent "transform item and log the result"
    @effect alloc
    // Missing: @effect io

    result := transform(item)
    write_log("Processed item: " + item.id)  // write_log has @effect io
    ok(result)
}
```

```
error[E0401]: missing effect declaration
 --> src/process.yb:7:5
  |
7 |     write_log("Processed item: " + item.id)
  |     ^^^^^^^^^ this call requires `@effect io`
  |
  = note: `write_log` (src/log.yb:2) declares `@effect io`
  = note: effects are transitive: callers must declare all effects of their callees
  = help: add `@effect io` to `process_item`
```

The error message tells you:
- Which call introduced the undeclared effect
- Where the callee declares the effect
- Why the declaration is required (transitivity)
- How to fix it

### Deep Transitivity

Transitivity applies through arbitrarily deep call chains:

```vibe
pub low_level_write(data: Str) -> Int {
    @effect io
    write_stdout(data)
    0
}

pub format_and_write(value: i64) -> Int {
    @effect io      // Required: calls low_level_write
    @effect alloc   // Required: string formatting allocates
    low_level_write(to_str(value))
}

pub report_metrics(metrics: List<i64>) -> Int {
    @effect io      // Required: calls format_and_write → low_level_write
    @effect alloc   // Required: calls format_and_write which allocates
    for m in metrics {
        format_and_write(m)
    }
    0
}
```

`report_metrics` does not directly call `low_level_write`, but it calls
`format_and_write`, which calls `low_level_write`, which has `@effect io`.
The `io` effect propagates through the entire chain.

### Why Transitivity Matters

Without transitive enforcement, a developer could wrap any effectful operation
in a "clean-looking" function and call it from supposedly pure code:

```vibe
// Without transitivity, this would hide the I/O effect
pub sneaky_log(msg: Str) -> Int {
    // No @effect declaration — looks pure!
    println(msg)  // But actually performs I/O
    0
}

pub "pure_computation"(x: i64) -> i64 {
    sneaky_log("computing...")  // Caller thinks this is pure
    x * 2
}
```

VibeLang's compiler catches this. `println` has `@effect io`, so `sneaky_log`
must declare `@effect io`, and any caller of `sneaky_log` must also declare
`@effect io`. The effect cannot be hidden.

---

## 7.4 Each Effect Explained

### `alloc` — Heap Allocation

The `alloc` effect indicates that a function allocates memory on the heap. This
includes creating new `List`, `Map`, or `Str` values through concatenation or
builder operations.

```vibe
pub build_greeting(parts: List<Str>) -> Str {
    @intent "join name parts into a greeting string"
    @effect alloc

    result := "Hello, "
    for part in parts {
        result = result + part + " "
    }
    result.trim()
}
```

Why track allocation? Because allocation has performance implications. In a
tight loop, unexpected allocations can cause latency spikes. By declaring
`@effect alloc`, you signal to reviewers and profiling tools that this function
touches the heap.

Functions that only operate on primitive values passed by value (integers,
booleans, floats) and do not create new compound values typically do not need
`@effect alloc`.

### `mut_state` — Shared Mutable State

The `mut_state` effect indicates that a function modifies state visible outside
its own scope — typically a mutable reference passed as a parameter or a module-
level mutable variable.

```vibe
pub increment_counter(mut counter: Counter) -> Int {
    @intent "increment the counter by one"
    @effect mut_state

    counter.value = counter.value + 1
    counter.value
}
```

The `mut_state` effect is a signal that this function has *side effects on its
arguments*. Calling it twice with the same counter produces different results.
This matters for reasoning about concurrency, testing, and caching.

Note that creating and modifying a *local* mutable variable does not require
`@effect mut_state` — only mutation visible to the caller counts:

```vibe
pub sum(values: List<i64>) -> i64 {
    // No @effect mut_state needed: `total` is local
    mut total := 0
    for v in values {
        total = total + v
    }
    total
}
```

### `io` — Console and File I/O

The `io` effect covers all interactions with the console and file system:

```vibe
pub save_config(config: Config, path: Str) -> Result<Unit, IoError> {
    @intent "serialize config and write it to the given file path"
    @effect io
    @effect alloc

    content := serialize_toml(config)
    write_file(path, content)?
    ok(unit)
}
```

Functions with `@effect io` are inherently harder to test in isolation because
they depend on external state (file system contents, terminal availability).
The effect declaration makes this dependency explicit.

### `net` — Network Operations

The `net` effect covers any network communication — HTTP requests, TCP/UDP
sockets, DNS lookups:

```vibe
pub fetch_user(api_url: Str, user_id: Str) -> Result<User, ApiError> {
    @intent "fetch user profile from the remote API"
    @effect net
    @effect alloc

    url := api_url + "/users/" + user_id
    response := http_get(url)?
    parse_user(response.body())
}
```

Network operations are nondeterministic (they can fail, time out, or return
different results at different times), slow relative to local computation, and
depend on external systems. The `net` effect flags all of these concerns.

### `concurrency` — Concurrent Operations

The `concurrency` effect indicates that a function spawns tasks, creates
channels, or uses `select`:

```vibe
pub parallel_sum(chunks: List<List<i64>>) -> i64 {
    @intent "sum all chunks in parallel and return the total"
    @effect concurrency
    @effect alloc

    ch := chan(len(chunks))

    for chunk in chunks {
        go {
            ch.send(sum(chunk))
        }
    }

    mut total := 0
    mut received := 0
    for received < len(chunks) {
        total = total + ch.recv()
        received = received + 1
    }
    total
}
```

The `concurrency` effect tells reviewers that this function introduces
parallelism, which has implications for ordering, synchronization, and resource
usage.

### `nondet` — Nondeterministic Operations

The `nondet` effect covers operations whose results are not determined solely
by their inputs: random number generation, current time, UUIDs, etc.

```vibe
pub generate_id() -> Str {
    @intent "generate a unique identifier"
    @effect nondet
    @effect alloc

    uuid_v4()
}

pub timestamp_now() -> i64 {
    @intent "return the current Unix timestamp in milliseconds"
    @effect nondet

    current_time_ms()
}
```

Functions with `@effect nondet` cannot be meaningfully memoized or cached
because they return different values on each call. This effect is particularly
important for testing — you almost always want to inject deterministic
substitutes for nondeterministic operations in tests.

---

## 7.5 Pure Functions

A function with no `@effect` declarations is **pure**. Pure functions have
several valuable properties:

1. **Deterministic:** Same inputs always produce the same output.
2. **Referentially transparent:** A call can be replaced with its result without
   changing program behavior.
3. **Trivially testable:** No setup, no mocking, no teardown. Pass inputs, check
   output.
4. **Safe to parallelize:** No shared state means no data races.
5. **Optimizable:** The compiler can memoize, reorder, or eliminate pure calls.

```vibe
pub distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    @intent "compute Euclidean distance between two 2D points"
    @examples {
        distance(0.0, 0.0, 3.0, 4.0) => 5.0
        distance(1.0, 1.0, 1.0, 1.0) => 0.0
    }
    @ensure . >= 0.0

    dx := x2 - x1
    dy := y2 - y1
    sqrt(dx * dx + dy * dy)
}
```

This function is pure. It takes four numbers and returns a number. It does not
allocate (assuming `sqrt` operates on primitives), does not perform I/O, does
not touch the network, and does not depend on randomness. You can call it a
million times with the same arguments and always get the same result.

### Designing for Purity

A powerful design pattern is to separate pure computation from effectful
operations. Push effects to the edges of your program and keep the core logic
pure:

```vibe
// Pure: computes the discount
pub compute_discount(price: i64, tier: CustomerTier) -> i64 {
    @intent "calculate discount amount based on customer tier"
    @require price >= 0
    @ensure . >= 0
    @ensure . <= price

    match tier {
        CustomerTier.Gold     => price * 20 / 100,
        CustomerTier.Silver   => price * 10 / 100,
        CustomerTier.Standard => 0,
    }
}

// Effectful: applies the discount and saves the result
pub apply_and_save_discount(mut order: Order) -> Result<Unit, IoError> {
    @intent "compute and apply discount, then persist the updated order"
    @effect mut_state
    @effect io
    @effect alloc

    discount := compute_discount(order.total, order.customer.tier)
    order.discount = discount
    order.final_total = order.total - discount
    save_order(order)?
    ok(unit)
}
```

The pure `compute_discount` is trivial to test. The effectful
`apply_and_save_discount` is harder to test but is thin — it orchestrates
effects around the pure core.

### Using Pure Functions in Contracts

Contract expressions (`@require`, `@ensure`) should themselves be pure. The
compiler enforces this: you cannot call an effectful function inside a contract
annotation.

```vibe
pub process(data: Data) -> Output {
    @require is_valid(data)     // is_valid must be pure
    @ensure is_normalized(.)    // is_normalized must be pure

    // ...
}
```

If `is_valid` had `@effect io`, the compiler would reject its use in `@require`:

```
error[E0402]: effectful function in contract expression
 --> src/process.yb:2:14
  |
2 |     @require is_valid(data)
  |              ^^^^^^^^^^^^^^ `is_valid` has `@effect io`
  |
  = note: contract expressions must be pure (no effects)
  = help: refactor `is_valid` to be pure, or use a pure validation function
```

This restriction exists because contracts must be deterministic and
side-effect-free. A precondition that performs I/O would make contract checking
unpredictable and could cause side effects during verification.

---

## 7.6 Effects and Testing

Effect declarations directly inform your testing strategy.

### Pure Functions: Unit Tests

Functions with no effects need only simple assertions:

```vibe
test "compute_discount gold tier" {
    assert_eq(compute_discount(1000, CustomerTier.Gold), 200)
}

test "compute_discount zero price" {
    assert_eq(compute_discount(0, CustomerTier.Gold), 0)
}
```

No setup. No mocking. No teardown. Pure functions are the easiest code to test.

### `@effect io` / `@effect net`: Integration Tests or Mocks

Functions with I/O or network effects need either real infrastructure or mocked
boundaries:

```vibe
test "fetch_user returns parsed user" {
    // Option 1: Use a test server
    server := start_test_server()
    server.register_response("/users/123", user_json)

    result := fetch_user(server.url(), "123")
    assert_eq(result, ok(expected_user))

    server.stop()
}
```

The `@effect net` declaration on `fetch_user` tells you immediately that this
function needs network infrastructure in tests. Without the effect declaration,
you would discover this only by reading the implementation or watching the test
fail.

### `@effect nondet`: Deterministic Substitutes

Functions with `@effect nondet` are the hardest to test because their output
varies. The standard approach is to inject deterministic substitutes:

```vibe
pub create_session(user: User, id_generator: IdGenerator) -> Session {
    @intent "create a new session for the user with a unique ID"
    @effect nondet
    @effect alloc

    Session {
        id: id_generator.next(),
        user_id: user.id,
        created_at: current_time_ms(),
    }
}

test "create_session uses provided ID generator" {
    gen := fixed_id_generator("test-session-001")
    user := User { id: "user-42" }
    session := create_session(user, gen)
    assert_eq(session.id, "test-session-001")
}
```

### Effect-Based Test Classification

A useful team convention is to classify tests by the effects of the code they
exercise:

| Effects | Test type | Speed | Infrastructure |
|---|---|---|---|
| None | Unit test | Fast | None |
| `alloc` | Unit test | Fast | None |
| `mut_state` | Unit test | Fast | Setup/teardown |
| `io` | Integration test | Medium | File system |
| `net` | Integration test | Slow | Network/mocks |
| `concurrency` | Concurrency test | Variable | Synchronization |
| `nondet` | Unit test with injection | Fast | Deterministic stubs |

This classification helps teams organize their test suites and set appropriate
timeouts and parallelism levels.

---

## 7.7 Effects and Performance

Effect declarations carry implicit performance information that helps you reason
about cost before profiling.

### What Each Effect Implies

| Effect | Performance implication |
|---|---|
| `alloc` | Heap allocation pressure; potential GC pauses or allocator contention |
| `mut_state` | Possible cache invalidation; ordering constraints |
| `io` | Syscall overhead; potential blocking |
| `net` | Latency (milliseconds to seconds); failure modes |
| `concurrency` | Task spawn overhead; synchronization costs |
| `nondet` | Typically cheap, but prevents memoization |

### Effect-Aware Optimization

When optimizing a hot path, start by examining the effect declarations:

1. **Remove unnecessary effects.** If a function declares `@effect alloc` but
   could be rewritten to avoid allocation, that is often the highest-impact
   optimization.

2. **Push effects outward.** Move effectful operations to the caller and keep
   the inner computation pure. Pure inner loops are easier for the compiler to
   optimize.

3. **Batch effects.** Instead of performing I/O inside a loop, collect results
   and perform one I/O operation after the loop.

Before optimization:

```vibe
pub process_all(items: List<Item>) -> Int {
    @effect io
    @effect alloc

    for item in items {
        result := transform(item)
        println("Processed: " + result.summary())  // I/O in every iteration
    }
    0
}
```

After optimization:

```vibe
pub process_all(items: List<Item>) -> Int {
    @effect io
    @effect alloc

    summaries := items.map(|item| {
        result := transform(item)
        result.summary()
    })

    for s in summaries {
        println("Processed: " + s)
    }
    0
}
```

Both versions have the same effects, but the second separates computation from
I/O, which may allow the compiler to optimize the `map` operation more
aggressively.

### The Cost of No Effects

Pure functions are the cheapest to call. The compiler knows they have no side
effects and can:

- **Memoize** repeated calls with the same arguments
- **Reorder** calls when the result is not immediately needed
- **Eliminate** calls whose results are never used
- **Inline** small pure functions without worrying about side effect duplication

This is why designing for purity is not just an aesthetic choice — it has direct
performance benefits.

---

## 7.8 Common Patterns and Pitfalls

### Pattern: Effect Narrowing

When refactoring, try to narrow the effect set. Removing an effect from a
function is always safe for callers (they declared the effect because of you,
and now they might be able to remove it too):

```vibe
// Before: allocates a new list
pub double_all(items: List<i64>) -> List<i64> {
    @effect alloc
    items.map(|x| { x * 2 })
}

// After: modifies in place (if the API supports it)
pub double_all(mut items: List<i64>) -> List<i64> {
    @effect mut_state
    for i in 0..len(items) {
        items[i] = items[i] * 2
    }
    items
}
```

The effect changed from `alloc` to `mut_state`. Depending on the context, this
might be a better trade-off. The point is that effect declarations make these
trade-offs visible.

### Pitfall: Over-Declaring Effects

Declaring effects you do not actually use is not a compiler error, but it is
misleading:

```vibe
pub add(a: i64, b: i64) -> i64 {
    @effect io      // Misleading: this function does no I/O
    @effect net     // Misleading: this function does no networking

    a + b
}
```

`vibe lint` warns about over-declared effects:

```
warning[W0402]: unused effect declaration
 --> src/math.yb:2:5
  |
2 |     @effect io
  |     ^^^^^^^^^^ `add` does not perform any `io` operations
  |
  = help: remove this effect declaration if the function is pure
```

### Pitfall: Forgetting Transitive Effects After Refactoring

When you add a new call to an effectful function, you must update the caller's
effects. This is easy to forget during refactoring, but the compiler catches it:

```vibe
pub summarize(data: List<i64>) -> Str {
    @effect alloc
    // After refactoring, someone adds a debug log:
    println("Summarizing " + to_str(len(data)) + " items")  // @effect io needed!
    // ...
}
```

```
error[E0401]: missing effect declaration
 --> src/summary.yb:4:5
  |
4 |     println("Summarizing " + to_str(len(data)) + " items")
  |     ^^^^^^^ this call requires `@effect io`
  |
  = help: add `@effect io` to `summarize`
```

---

## 7.9 Summary

VibeLang's effect system makes the invisible visible:

- **Six base effects** (`alloc`, `mut_state`, `io`, `net`, `concurrency`,
  `nondet`) cover the major categories of side effects in real programs.
- **Explicit declarations** via `@effect` tell readers and tools what a function
  does to the outside world, without reading the implementation.
- **Transitive enforcement** by the compiler ensures that effects cannot be
  hidden by wrapping them in intermediate functions. If any function in your call
  chain performs I/O, every caller up to the entry point must acknowledge it.
- **Pure functions** (no effects) are the easiest to test, reason about, and
  optimize. Designing for purity at the core with effects at the edges produces
  programs that are both correct and fast.
- **Testing strategy** follows directly from effect declarations: pure functions
  get unit tests, I/O functions get integration tests, nondeterministic functions
  get deterministic substitutes.
- **Performance reasoning** starts with effects: removing unnecessary effects,
  pushing effects outward, and batching effectful operations are the first steps
  in optimization.

The effect system works in concert with the contract system from Chapter 6.
Together, they give every function a complete behavioral specification: what it
does (intent), how it behaves on specific inputs (examples), what it requires
(preconditions), what it guarantees (postconditions), and what it does to the
world (effects).

The next chapter covers error handling with `Result` — how VibeLang represents,
propagates, and recovers from expected runtime failures.
