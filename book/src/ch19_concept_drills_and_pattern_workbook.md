# Chapter 19: Concept Drills and Pattern Workbook

This chapter is a practical workbook. It is intentionally long and hands-on.
Use it to turn conceptual understanding into muscle memory.

You can read this chapter in three ways:

- sequentially as a full training track,
- selectively by topic (types, loops, contracts, concurrency, etc.),
- as a team workshop reference for onboarding and review calibration.

## 19.1 How to Use This Workbook

For each drill:

1. copy the snippet into a scratch module,
2. run `vibe check`, then `vibe test`,
3. add one `@intent` and one `@ensure`,
4. rerun with `vibe lint . --intent --changed`,
5. write down what changed in behavior confidence.

The goal is not speed. The goal is to build deterministic intuition.

### Why This Chapter Uses Many Lists

This workbook intentionally uses checklists and compact prompts because it is
designed for repeated practice sessions. However, each drill should be read as a
full engineering exercise, not a short coding puzzle. For every bullet prompt,
you should ask:

- what semantics does this exercise stress?
- what boundary risks is it exposing?
- which contracts or effects best capture intended behavior?
- how would this change behave in CI and release profiles?

If you apply that lens, the drill format becomes a deep learning tool rather
than a shallow checklist.

---

## 19.2 Drill A: Keywords and Lexical Clarity

### Task

Write one function that uses:

- `for`,
- `if/else`,
- `return`,
- `const` and `mut`.

### Starter

```txt
pub normalize(xs: List<i64>) -> List<i64> {
  @intent "map negative numbers to zero and keep order"
  @ensure len(.) == len(xs)
  @effect alloc

  mut out := []
  for x in xs {
    if x < 0 {
      out.append(0)
    } else {
      out.append(x)
    }
  }
  out
}
```

### Reflection

- Did you keep keyword usage readable?
- Are there any ambiguous short variable names?
- Does the intent line still match implementation after edits?

---

## 19.3 Drill B: Literals and Numeric Safety

### Task

Use:

- integer literals,
- suffixed literals,
- float literals,
- duration literals in a `select` case.

### Starter

```txt
pub config_defaults() -> f64 {
  retries := 3
  _max_batch: u32 := 1024u32
  threshold: f64 := 0.75
  _window := 5s

  if retries > 0 && threshold > 0.5 {
    threshold
  } else {
    0.0
  }
}
```

### Reflection

- Which values should be explicit-width at API boundaries?
- Where can default literal inference stay ergonomic?

---

## 19.4 Drill C: String and Text Behavior

### Task

Practice:

- escape handling,
- byte-length assumptions,
- deterministic string concatenation.

### Starter

```txt
pub render_header(name: Str) -> Str {
  @intent "produce single-line escaped header"
  @ensure len(.) >= len(name)
  @effect alloc

  "user=\"" + name + "\"\n"
}
```

### Extension

Add examples for:

- empty name,
- ASCII name,
- non-ASCII name.

Focus on behavior guarantees, not locale assumptions.

---

## 19.5 Drill D: Loops and Control-Flow Shape

### Task

Implement the same behavior three ways:

1. `for`,
2. `while`,
3. `repeat`.

### Starter target behavior

"Count how many values are non-negative."

### `for` solution

```txt
pub count_non_negative_for(xs: List<i64>) -> i64 {
  mut n := 0i64
  for x in xs {
    if x >= 0 {
      n = n + 1
    }
  }
  n
}
```

### Reflection

- Which control form reads best for this problem?
- Which form is easiest to review for off-by-one errors?

---

## 19.6 Drill E: Contracts as Design Tools

### Task

Start with a plain function, then add:

- `@intent`,
- `@examples`,
- `@require`,
- `@ensure`.

### Starter

```txt
pub bounded_add(a: i64, b: i64, maxv: i64) -> i64 {
  a + b
}
```

### Suggested contracted form

```txt
pub bounded_add(a: i64, b: i64, maxv: i64) -> i64 {
  @intent "add two numbers and clamp to maxv"
  @examples {
    bounded_add(1, 2, 10) => 3
    bounded_add(8, 5, 10) => 10
  }
  @require maxv >= 0
  @ensure . <= maxv
  @effect alloc

  sum := a + b
  if sum > maxv { maxv } else { sum }
}
```

### Reflection

- Which contract line gave the highest review value?
- Which line would most likely catch future drift?

---

## 19.7 Drill F: Effect Budgeting

### Task

Take one function and classify each operation under effect categories:

- alloc,
- io,
- net,
- mut_state,
- concurrency,
- nondet.

### Exercise pattern

1. write the function without effects,
2. add precise effects only,
3. run check/lint and inspect diagnostics,
4. remove one effect and observe what changes.

### Reflection

- Did effect declarations reveal hidden behavior?
- Did they simplify reviewer understanding?

---

## 19.8 Drill G: `Result` and Error Channel Discipline

### Task

Refactor a panic-prone function into explicit `Result<T,E>` flow.

### Starter idea

"Parse and validate order quantity."

```txt
pub parse_qty(raw: Str) -> Result<i64, QtyError> {
  n := parse_i64(raw)?
  if n <= 0 {
    err(QtyError.non_positive())
  } else {
    ok(n)
  }
}
```

### Reflection

- Is every failure mode represented?
- Is the error channel stable for callers?
- Is `?` used where propagation is truly intended?

---

## 19.9 Drill H: Containers and Mutation Boundaries

### Task

Write one function that:

- mutates a list locally,
- returns an immutable output shape,
- uses contracts to bound result size.

```txt
pub dedupe_non_negative(xs: List<i64>) -> List<i64> {
  @intent "keep first occurrence of each non-negative value"
  @ensure len(.) <= len(xs)
  @effect alloc

  mut out := []
  mut seen := {}
  for x in xs {
    if x >= 0 && !seen.contains(x) {
      seen.set(x, 1)
      out.append(x)
    }
  }
  out
}
```

If your current runtime profile has map feature limits, simplify `seen` behavior
while preserving the intent/contract structure.

---

## 19.10 Drill I: Channel-Based Work Partitioning

### Task

Create a two-stage pipeline:

- producer sends tasks,
- workers process and send results,
- aggregator collects fixed count.

### Starter skeleton

```txt
worker(jobs, out) -> Int {
  @effect concurrency
  x := jobs.recv()
  out.send(x * x)
  0
}

pub run_batch(tasks: List<Int>) -> List<Int> {
  @intent "process tasks concurrently and collect all results"
  @ensure len(.) == len(tasks)
  @effect alloc
  @effect concurrency

  jobs := chan(1024)
  out := chan(1024)

  repeat cpu_count() {
    go worker(jobs, out)
  }
  for t in tasks {
    jobs.send(t)
  }
  jobs.close()
  out.take(len(tasks))
}
```

### Reflection

- Where does ownership transfer happen?
- Where are visibility guarantees established?

---

## 19.11 Drill J: `select`, Timeout, and Shutdown Semantics

### Task

Implement a loop that:

- handles incoming messages,
- emits timeout heartbeat,
- shuts down cleanly on channel close.

### Starter

```txt
pub loop_once(inbox) -> Int {
  @effect concurrency
  @effect io

  select {
    case msg := inbox.recv() =>
      println(msg)
    case after 1s =>
      println("heartbeat")
    case closed inbox =>
      println("shutdown")
    case default =>
      println("idle")
  }
  0
}
```

### Reflection

- Do you need `default`?
- Does `default` create unintended busy-loop behavior?
- Is timeout unit appropriate for workload?

---

## 19.12 Drill K: Module API Hardening

### Task

Define one small module with:

- one public type,
- one public function,
- private helpers,
- intent/contracts on public function.

### Pattern

```txt
module app.auth

pub type Session { ... }

pub create_session(user: User) -> Result<Session, AuthError> {
  @intent "create session for authenticated user"
  @require user.id != none
  @ensure .is_ok()
  @effect alloc
  ...
}
```

### Reflection

- Is your public API minimal and clear?
- Are internals truly private?

---

## 19.13 Drill L: Deterministic Release Slice

### Task

For one module change, run this sequence and archive outputs:

```bash
vibe check src/module.yb
vibe test src/
vibe lint . --intent --changed
vibe build src/module.yb --profile release
```

Then answer:

- Which step caught the first meaningful issue?
- Did diagnostics remain stable on rerun?
- Did release-profile behavior differ in expected ways?

---

## 19.14 Pattern Catalog: High-Signal Templates

Use these templates repeatedly.

### Template A: Public API function

```txt
pub fn_name(args...) -> Result<T, E> {
  @intent "..."
  @examples { ... }
  @require ...
  @ensure ...
  @effect ...
  ...
}
```

### Template B: Concurrent worker

```txt
worker(in, out) -> Int {
  @effect concurrency
  ...
}
```

### Template C: Deterministic transformer

```txt
pub transform(xs: List<i64>) -> List<i64> {
  @intent "..."
  @ensure len(.) <= len(xs)
  @effect alloc
  ...
}
```

### Template D: Boundary wrapper

```txt
pub safe_boundary(...) -> Result<..., BoundaryError> {
  @intent "..."
  @require ...
  @ensure ...
  ...
}
```

---

## 19.15 Capstone Exercise: Intent-Governed Batch Service

### Objective

Build a tiny service module that:

- accepts a batch input,
- validates preconditions,
- processes items concurrently,
- returns deterministic aggregated output,
- exposes stable error channel,
- includes examples and effects.

### Required elements

- at least one `@examples` block,
- at least one `select` with timeout or closed handling,
- `@effect concurrency` and `@effect alloc`,
- one explicit module boundary (`pub` API + private helper).

### Suggested success criteria

- all local checks pass,
- intent lint is clean or only low-confidence warnings,
- output shape is contract-verified.

---

## 19.16 Team Workshop Mode

Run this chapter as a team calibration session:

1. each engineer solves one drill independently,
2. pair-review with "intent + effects + boundaries" lens,
3. compare styles for readability and determinism confidence,
4. standardize project-level conventions.

This quickly aligns teams on what "high-quality VibeLang" looks like.

---

## 19.17 Final Workbook Checklist

After completing this chapter, you should be able to:

- author clear keyword/literal/control-flow heavy code,
- convert behavior statements into executable contracts,
- model effects as explicit engineering boundaries,
- design safe concurrent pipelines with ownership awareness,
- use the CLI and release sequence as a deterministic confidence ladder.

---

## 19.18 Quick Reference Tables

### Keywords by Category

| Category | Keywords |
| --- | --- |
| Declarations | `module`, `import`, `pub`, `type` |
| Concurrency/Async | `go`, `thread`, `async`, `await`, `select`, `after`, `closed` |
| Control flow | `if`, `else`, `for`, `while`, `repeat`, `match`, `case`, `default`, `break`, `continue`, `return` |
| Bindings | `const`, `mut` |
| Core values | `true`, `false`, `none` |

### Literal Forms

| Literal family | Examples |
| --- | --- |
| Integer | `42`, `42i32`, `42u64` |
| Float | `3.14`, `0.25f32`, `1.0f64` |
| String | `"hello\nworld"` |
| Char | `'x'` |
| List | `[1, 2, 3]` |
| Map | `{"a": 1, "b": 2}` |
| Duration | `5ms`, `1s`, `2m`, `1h` |

### Contract Annotations

| Annotation | Purpose |
| --- | --- |
| `@intent` | States expected behavior outcome |
| `@examples` | Encodes executable examples |
| `@require` | Asserts entry preconditions |
| `@ensure` | Asserts exit invariants |
| `@effect` | Declares side-effect categories |

---

## 19.19 Control-Flow Choice Matrix

Choose control forms intentionally:

| Need | Prefer |
| --- | --- |
| Iterate over collection in defined order | `for item in iterable` |
| Iterate while condition changes each cycle | `while` |
| Iterate fixed count known up front | `repeat` |
| Multi-branch value dispatch | `match` |
| Event-driven wait over channels/timeouts | `select` |

Practical rule:

- If the loop is data-driven, prefer `for`.
- If the loop is state-machine driven, prefer `while`.
- If the loop is bounded by a fixed number, prefer `repeat`.

For service loops, combine:

- `select`,
- explicit timeout units (`after`),
- explicit shutdown path (`closed`),
- cancellation-aware boundaries.

---

## 19.20 Release-Readiness Drill Template

Use this template for serious feature branches:

### Step 1: Local language integrity

```bash
vibe check src/
vibe test src/
```

### Step 2: Intent and behavior drift

```bash
vibe lint . --intent --changed
```

### Step 3: Release profile confidence

```bash
vibe build src/main.yb --profile release
```

### Step 4: Evidence notes (human-readable)

Record:

- what changed semantically,
- what contracts/examples were added or tightened,
- which effect declarations changed and why,
- whether any boundary/concurrency assumptions changed.

### Step 5: Team review prompts

Ask reviewers:

1. Does intent match implementation?
2. Are contracts high signal and minimal?
3. Are effect declarations accurate?
4. Are concurrency boundaries and ownership transfers clear?
5. Is release behavior deterministic and explainable?

This review style keeps VibeLang’s design philosophy active in day-to-day work.

---

This completes the extended book edition. Use the chapter set as both a learning
track and a production reference baseline for VibeLang engineering.
