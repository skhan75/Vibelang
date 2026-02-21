# VibeLang Syntax Spec (v1.0 Target)

## Status

- Normative target grammar: `docs/spec/grammar_v1_0.ebnf`
- Archived parser freeze: `docs/spec/grammar_v0_1.ebnf`
- Historical ambiguity notes: `docs/spec/phase1_resolved_decisions.md`

This document defines language-level syntax. If this document conflicts with the
v1 grammar, the grammar wins.

## Design Goals

- Keep syntax low-noise and readable.
- Preserve deterministic parse behavior.
- Keep contracts and intent close to executable code.
- Support modern concurrency and async constructs explicitly.

## File Structure

A file may contain:

1. optional module declaration
2. zero or more imports
3. zero or more declarations (`type` and function declarations)

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

## Reserved Keywords

Reserved keywords in v1 grammar:

- `module`, `import`, `pub`, `type`
- `async`, `await`, `thread`, `go`
- `if`, `else`, `for`, `while`, `repeat`, `match`, `case`, `default`
- `select`, `after`, `closed`
- `return`, `break`, `continue`
- `const`, `mut`
- `true`, `false`, `none`

Contract annotations begin with `@` and are not identifiers.

## Declarations

### Type Declaration

```txt
pub type Account {
  id: Str
  balance: i64
}
```

### Function Declaration

```txt
name(arg1: T1, arg2: T2) -> T3 { ... }
pub async fetch(url: Str) -> Result<Str, Error> { ... }
```

Notes:

- `async` is optional; non-async functions are synchronous.
- Function parameters may be prefixed with `mut` to allow reassignment.
- `return` is optional when using tail-expression return.

## Bindings, Constants, and Assignment

- Mutable or immutable inferred binding:
  - `x := expr`
  - `mut x := expr`
- Constant binding:
  - `const limit: i64 = 1024`
- Assignment:
  - `x = expr`
  - `obj.field = expr`
  - `list[i] = expr`

Assignment validity is constrained by the mutability model in
`docs/spec/mutability_model.md`.

## Control Statements

Supported statements:

- `if` / `else`
- `for name in iterable`
- `while condition`
- `repeat count`
- `match expr { ... }`
- `break` / `continue` (optional labels)
- `return`

Example:

```txt
for item in items {
  if item == stopValue {
    break
  }
  if item < 0 {
    continue
  }
  process(item)
}
```

## Concurrency and Async Surface

### Task and Thread Forms

- `go expr` schedules concurrent work on runtime task scheduler.
- `thread expr` requests explicit OS-thread execution semantics.

### Select Form

```txt
select {
  case msg := inbox.recv() => handle(msg)
  case after 5s => on_timeout()
  case closed inbox => break
  case default => on_idle()
}
```

### Async and Await

`await` is a unary expression form:

```txt
pub async fetchAll(urls: List<Str>) -> Result<List<Str>, Error> {
  first := await fetch(urls[0])
  [first]
}
```

Detailed behavior is in `docs/spec/async_await_and_threads.md`.

## Expression Surface

Primary expression classes include:

- identifiers, literals, grouped expressions
- function/method calls
- member access and index access
- unary/binary operations
- postfix `?` propagation
- `await` unary form
- contract-only forms: `.` and `old(expr)`

Evaluation order is left-to-right for arguments, call chains, and binary ops.

## Literals

- Integer literals with optional suffixes:
  - `42`, `42i32`, `42u64`
- Float literals with optional suffixes:
  - `3.14`, `3.14f32`
- String literals:
  - `"hello\nworld"`
- Char literals:
  - `'x'`
- Boolean:
  - `true`, `false`
- List:
  - `[1, 2, 3]`
- Map:
  - `{"a": 1, "b": 2}`
- Duration literal:
  - `5ms`, `1s`, `2m`, `1h`

Exact numeric and text behavior is defined in:

- `docs/spec/numeric_model.md`
- `docs/spec/strings_and_text.md`

## Contracts and Intent Placement

Contracts MUST appear at function-body top before executable statements:

- `@intent "..."`, `@examples { ... }`, `@require`, `@ensure`, `@effect`

Illegal placement MUST produce deterministic diagnostics.

## Option-Like Nullability Syntax

V1 uses explicit optional typing via nullable suffix syntax:

- `Str?`
- `List<i64>?`

`none` is the optional empty-value literal.

`Option<T>` remains an explanatory alias in docs, but syntax-level canonical
form is `T?` in v1 grammar.

## Formatting and Readability Conventions

- Keep contracts grouped and ordered before executable code.
- Prefer one transformation per chain line for long pipelines.
- Keep line length around 100 chars.
