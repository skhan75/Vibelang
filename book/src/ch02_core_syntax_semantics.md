# Chapter 2: Core Syntax and Semantics

This chapter defines how VibeLang code is structured and how it executes at a
high level. If Chapter 1 gave you the "why," Chapter 2 gives you the "shape."

## 2.1 The Source-of-Truth Rule

VibeLang has a strict precedence model for language truth:

1. the current grammar file in `docs/spec/`
2. normative model docs (`type_system`, `numeric_model`, concurrency/memory/ABI)
3. `docs/spec/syntax.md` and `docs/spec/semantics.md`
4. examples/tutorial content

When writing production code or tooling integrations, always anchor decisions to
the grammar and normative docs first.

## 2.2 File Structure

A VibeLang file may contain:

1. optional module declaration,
2. zero or more imports,
3. zero or more declarations.

Example:

```txt
module app.math

import std.math

pub topK(xs: List<i64>, k: i64) -> List<i64> {
  @intent "k largest numbers sorted descending"
  @examples {
    topK([3,1,2], 2) => [3,2]
  }
  @require k >= 0
  @ensure len(.) <= len(xs)
  @effect alloc

  xs.sort_desc().take(k)
}
```

Observe that contracts and intent are part of function body structure, not
detached metadata.

## 2.3 Declaration Basics

### Type Declarations

```txt
pub type Account {
  id: Str
  balance: i64
}
```

### Function Declarations

```txt
name(arg1: T1, arg2: T2) -> T3 { ... }
pub async fetch(url: Str) -> Result<Str, Error> { ... }
```

Important defaults:

- parameters are immutable unless explicitly mutable,
- explicit `return` is optional if a final tail expression provides the value.

## 2.4 Statement and Expression Surface

VibeLang blends expression-oriented style with explicit statements.

Core statement/control forms:

- `if` / `else`
- `for item in iterable`
- `while condition`
- `repeat count`
- `match expr { ... }`
- `break` / `continue`
- `return`

Core expression classes:

- identifiers and literals,
- unary/binary operators,
- call expressions,
- member/index access,
- postfix `?` propagation,
- `await` unary expression,
- contract-only forms `.` and `old(expr)` in postconditions.

## 2.5 Deterministic Evaluation Order

A critical semantic guarantee is deterministic evaluation order:

- function arguments evaluate left-to-right,
- call chains evaluate left-to-right,
- binary operations evaluate left-to-right.

This predictability is especially valuable in effectful code and when debugging
subtle failures. In high-scale CI, deterministic ordering also keeps diagnostics
stable and easier to diff.

## 2.6 Execution Model

In the current language/toolchain model:

- source is compiled ahead-of-time to native binaries,
- runtime services (scheduler, channels, GC) are explicit semantic participants,
- deterministic input + deterministic environment should produce deterministic
  behavior, unless explicit nondeterministic behavior is introduced.

This gives VibeLang a systems-like deployment profile without forcing teams into
systems-level boilerplate everywhere.

## 2.7 Effects as Semantic Boundaries

VibeLang treats side effects as first-class semantic boundaries.

The base effect vocabulary includes:

- `alloc`
- `mut_state`
- `io`
- `net`
- `concurrency`
- `nondet`

Effects are not decorative. They drive analysis, diagnostics, and intent drift
checks, and they guide reviewers about what a function can do.

## 2.8 Profile-Aware Semantics

Semantics are profile-aware:

- **dev/test** profiles emphasize diagnostics and strict checks,
- **release** profile emphasizes optimized native output while preserving
  deterministic contracts defined by policy.

VibeLang explicitly resists hidden profile surprises. Any profile-specific
difference should be explicit and auditable.

## 2.9 Concurrency and Async as Language Surface

Concurrency is not bolted on:

- `go expr` for runtime tasks,
- `thread expr` for explicit OS-thread boundary semantics,
- `select` for coordinated waits,
- `await` for async suspension points.

All of these tie back into sendability, memory visibility, and deterministic
diagnostics.

## 2.10 Undefined Behavior Policy (Safe Surface)

The safe surface aims to avoid undefined behavior. Unsafe operations are
restricted to explicit audited boundaries. This allows VibeLang to preserve a
high-confidence model for most application code.

## 2.11 Design Consequences

From a software architecture perspective, this syntax/semantics combination
changes how teams structure code:

- contracts can be colocated with implementation instead of living in separate
  docs,
- effect boundaries are visible in function signatures/bodies,
- concurrency primitives are explicit where behavior matters,
- release confidence improves because language constructs map cleanly to
  toolchain checks.

## 2.12 From Syntax to Behavior: A Worked Interpretation

When you read a VibeLang function, do not read it as "tokens only." Read it in
layers:

1. declaration layer (signature, visibility, return shape),
2. contract/effect layer (intent, invariants, side-effect envelope),
3. executable logic layer (control flow and expression order).

For example, if two functions have identical executable bodies but one has
stronger contracts and effect declarations, they are not equivalent from a
maintenance perspective. The second function carries more semantic intent and
therefore yields better reviewer understanding, better lint guidance, and better
long-term change safety.

This is why VibeLang syntax and semantics are deliberately taught together:
syntax choices influence real operational behavior.

---

Next: Chapter 3 dives into types, functions, and error channels in depth.
