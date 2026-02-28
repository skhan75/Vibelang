# Chapter 3: Core Syntax and Semantics

This chapter covers the foundational syntax and semantic rules of VibeLang. By
the end, you will understand how source files are organized, how variables and
bindings work, why VibeLang is expression-oriented, and how evaluation order
guarantees deterministic behavior.

These are the rules you will use in every VibeLang program you write. Take the
time to internalize them — they inform everything that follows.

## 3.1 Source File Structure

Every VibeLang source file uses the `.yb` extension. The compiler recognizes
only this extension; attempting to compile a `.vb` or `.txt` file will produce
an error.

A typical source file has three regions, in order:

1. **Module declaration** (optional)
2. **Import declarations**
3. **Top-level declarations** (functions, types, constants)

Here is a minimal but complete source file:

```vibe
// file: math_utils.yb

module math_utils

import std.io

pub square(x: Int) -> Int {
  x * x
}

pub main() -> Int {
  @effect io
  println(square(7))
  0
}
```

### Module Declarations

A module declaration names the logical unit that this file belongs to. If
omitted, the compiler infers the module name from the filename (without the
`.yb` extension).

```vibe
module my_project.utils
```

Module names use dot-separated paths. This mirrors the directory structure: a
file at `src/my_project/utils.yb` would naturally belong to module
`my_project.utils`.

### Import Declarations

Imports bring names from other modules into scope:

```vibe
import std.io
import std.math.{abs, max, min}
import my_project.models.User
```

You can import an entire module, specific names from a module, or a single
type. Imports must appear before any function or type declarations.

VibeLang does not have wildcard imports (no `import std.math.*`). This is a
deliberate choice: every name in scope should be traceable to a specific import.
When you read a VibeLang file, you always know where a name came from.

### File Naming Conventions

VibeLang enforces a simple convention:

- Source files use `.yb` extension
- Filenames should be `snake_case`
- One primary module per file (though a module can span multiple files)
- Test files are suffixed with `_test.yb`

```bash
src/
  main.yb
  config.yb
  http_server.yb
  http_server_test.yb
```

### Top-Level Declarations

At the top level of a file, you can declare:

- Functions (with `pub` for public visibility)
- Type definitions
- Constants
- Contract-annotated functions

You cannot write bare expressions at the top level. All executable code lives
inside function bodies.

```vibe
// This is valid at the top level
const MAX_RETRIES: Int = 5

pub retry_count() -> Int {
  MAX_RETRIES
}

// This is NOT valid at the top level — bare expression
// MAX_RETRIES + 1    // error: expected declaration, found expression
```

## 3.2 Variables and Bindings

VibeLang uses the term "binding" rather than "variable" because the default
behavior is immutable: a binding attaches a name to a value, and that
attachment cannot change.

### Immutable Bindings with `:=`

The `:=` operator creates a new binding:

```vibe
pub main() -> Int {
  @effect io

  name := "VibeLang"
  version := 1
  pi := 3.14159

  println(name)
  println(version)
  0
}
```

These bindings are **immutable by default**. Once `name` is bound to
`"VibeLang"`, it cannot be reassigned. This is not a suggestion or a
convention — it is enforced by the compiler.

What happens if you try to reassign an immutable binding?

```vibe
pub main() -> Int {
  count := 10
  count = 20    // error!
  0
}
```

The compiler produces a clear error:

```
error[E0301]: cannot assign to immutable binding `count`
 --> main.yb:3:3
  |
2 |   count := 10
  |   ----- binding declared as immutable here
3 |   count = 20
  |   ^^^^^^^^^^ cannot assign twice to immutable binding
  |
help: consider making this binding mutable: `mut count := 10`
```

This error is intentional and central to VibeLang's design. Immutability by
default means:

- **Fewer bugs**: you cannot accidentally change a value that other code depends
  on.
- **Easier reasoning**: when you see `x := 42`, you know `x` is `42` for the
  rest of that scope. Period.
- **Better optimization**: the compiler can make stronger assumptions about
  values that never change.
- **Clearer intent**: when you *do* use `mut`, it signals to every reader that
  this value is expected to change — pay attention.

### Mutable Bindings with `mut`

When you genuinely need a value to change, use `mut`:

```vibe
pub main() -> Int {
  @effect io

  mut counter := 0
  counter = counter + 1
  counter = counter + 1
  counter = counter + 1

  println(counter)    // prints: 3
  0
}
```

The `mut` keyword is placed before the binding name. It tells both the compiler
and the reader: "this value will be reassigned."

A practical example — accumulating a sum:

```vibe
pub sum_list(numbers: List<Int>) -> Int {
  @effect alloc

  mut total := 0
  for n in numbers {
    total = total + n
  }
  total
}
```

Without `mut`, the loop body would fail to compile because `total` could not be
reassigned.

### When to Use `mut`

Use `mut` when:

- You are accumulating a result across iterations (sums, counts, building
  strings)
- You need to track state that changes over time (counters, flags)
- An algorithm requires in-place updates

Avoid `mut` when:

- You can express the computation as a chain of expressions
- You can use `if/else` or `match` to select a value directly
- The value is only assigned once

VibeLang's culture strongly favors immutable bindings. If you find yourself
reaching for `mut` frequently, consider whether the logic can be restructured.

### Constants with `const`

Constants are compile-time values that never change and are available at the
module level:

```vibe
const MAX_CONNECTIONS: Int = 100
const DEFAULT_TIMEOUT_MS: Int = 5000
const PI: Float = 3.14159265358979
const APP_NAME: Str = "MyService"
```

Constants differ from immutable bindings in important ways:

| Property | `const` | Immutable binding (`:=`) |
|---|---|---|
| Scope | Module-level | Block-level |
| Evaluation | Compile-time | Runtime |
| Type annotation | Required | Optional (inferred) |
| Naming convention | `UPPER_SNAKE_CASE` | `lower_snake_case` |

The compiler evaluates `const` values at compile time. This means the
right-hand side must be a constant expression — no function calls, no runtime
values:

```vibe
const GOOD: Int = 42 * 2          // OK: constant expression
const BAD: Int = compute_value()  // error: not a constant expression
```

```
error[E0305]: constant expression required
 --> config.yb:2:20
  |
2 | const BAD: Int = compute_value()
  |                  ^^^^^^^^^^^^^^^^ function calls are not allowed in constant expressions
```

### Type Annotations vs Inference

VibeLang has local type inference. Within function bodies, the compiler can
usually determine the type from the right-hand side of a binding:

```vibe
pub main() -> Int {
  x := 42           // inferred as Int
  name := "hello"   // inferred as Str
  pi := 3.14        // inferred as Float
  active := true    // inferred as Bool
  0
}
```

You can always add an explicit type annotation after the binding name:

```vibe
pub main() -> Int {
  x: Int := 42
  name: Str := "hello"
  ratio: Float := 0.75
  0
}
```

Explicit annotations are required in certain contexts:

- **Public function signatures**: parameters and return types must always be
  annotated. This is a hard rule — public APIs are the contract boundary, and
  types are part of that contract.
- **Ambiguous literals**: when the compiler cannot determine which numeric type
  you want.
- **Empty collections**: `items: List<Int> := []` needs the type because `[]`
  alone is ambiguous.

```vibe
// Public function: types required on parameters and return
pub add(a: Int, b: Int) -> Int {
  a + b
}

// Private function: return type can be inferred
greet(name: Str) -> Str {
  "Hello, " + name
}
```

If you omit the return type on a public function, the compiler tells you:

```
error[E0310]: public function `process` requires explicit return type
 --> api.yb:5:1
  |
5 | pub process(data: Str) {
  |     ^^^^^^^ add return type annotation
  |
help: add `-> ReturnType` after the parameter list
```

### Shadowing

VibeLang permits shadowing: you can declare a new binding with the same name as
an existing one in the same scope. The new binding "shadows" the old one — the
old value still exists but is no longer accessible by that name.

```vibe
pub main() -> Int {
  @effect io

  x := 5
  println(x)        // prints: 5

  x := x * 2
  println(x)        // prints: 10

  x := "now a string"
  println(x)        // prints: now a string

  0
}
```

Shadowing is different from mutation. With `mut`, you change the value stored in
the same binding. With shadowing, you create an entirely new binding that
happens to have the same name. The type can even change, as shown above.

Why allow shadowing? Consider a common pattern — transforming a value through
several stages:

```vibe
pub parse_and_validate(raw: Str) -> Int {
  input := raw.trim()
  input := parse_i64(input)
  input := if input < 0 { 0 } else { input }
  input
}
```

Without shadowing, you would need a different name for each stage (`raw_input`,
`parsed_input`, `validated_input`), which adds noise without adding clarity.
Shadowing lets you express a pipeline of transformations on a single conceptual
value.

Shadowing does not affect the original binding in an outer scope:

```vibe
pub main() -> Int {
  @effect io

  x := 10
  if true {
    x := 99       // shadows outer x, only within this block
    println(x)    // prints: 99
  }
  println(x)      // prints: 10 — outer x is unchanged
  0
}
```

## 3.3 Expressions vs Statements

VibeLang is an **expression-oriented** language. This is one of its most
important design properties, and understanding it will change how you write
code.

### What Is an Expression?

An expression is a piece of code that **evaluates to a value**. In VibeLang,
almost everything is an expression:

- Literals: `42`, `"hello"`, `true`
- Arithmetic: `a + b`, `x * 2`
- Function calls: `square(5)`
- Blocks: `{ ... }`
- `if/else` constructs
- `match` constructs

### What Is a Statement?

A statement is a piece of code that **performs an action but does not produce a
value**. VibeLang has very few statements:

- Binding declarations: `x := 10`
- Mutable assignments: `counter = counter + 1`
- `import` declarations
- `module` declarations

The key distinction: **you cannot use a statement where an expression is
expected**.

```vibe
// This does NOT work — assignment is a statement, not an expression
// y := (x = 10)    // error: assignment is not an expression
```

```
error[E0320]: assignment is a statement, not an expression
 --> main.yb:3:8
  |
3 |   y := (x = 10)
  |         ^^^^^^ assignment does not produce a value
  |
note: VibeLang separates assignment (statement) from binding (declaration)
      to prevent accidental use of `=` where `==` was intended
```

This is a deliberate departure from C-family languages where `x = 10` is an
expression that returns `10`. VibeLang makes assignment a statement to prevent
an entire class of bugs where `=` is used instead of `==` in conditions.

### Blocks as Expressions

A block `{ ... }` is an expression. Its value is the value of its last
expression:

```vibe
pub main() -> Int {
  @effect io

  result := {
    a := 10
    b := 20
    a + b       // this is the value of the block
  }

  println(result)    // prints: 30
  0
}
```

The last line in a block — `a + b` — is the **tail expression**. It determines
the block's value. There is no `return` keyword needed here; the value flows
naturally.

### `if/else` as Expression

Because `if/else` is an expression, you can bind its result directly:

```vibe
pub main() -> Int {
  @effect io

  temperature := 35

  description := if temperature > 30 {
    "hot"
  } else if temperature > 20 {
    "warm"
  } else {
    "cool"
  }

  println(description)    // prints: hot
  0
}
```

Both branches must produce the same type. If they don't, the compiler reports a
type mismatch:

```
error[E0321]: `if` and `else` branches have incompatible types
 --> main.yb:5:19
  |
5 |   result := if condition {
  |             ^^ expected both branches to have the same type
6 |     42
  |     -- this branch has type `Int`
7 |   } else {
8 |     "hello"
  |     ------- this branch has type `Str`
  |
note: when using `if/else` as an expression, both branches must
      evaluate to the same type
```

## 3.4 Comments

### Line Comments

VibeLang uses `//` for line comments:

```vibe
// This is a comment
x := 42  // inline comment after code
```

Comments extend from `//` to the end of the line. There are no multi-line
block comments (`/* ... */`).

### Documentation Comments

Documentation comments use `///` and attach to the declaration that follows
them:

```vibe
/// Computes the factorial of a non-negative integer.
///
/// Returns 1 for input 0, as 0! = 1 by convention.
/// Panics if `n` is negative.
pub factorial(n: Int) -> Int {
  @require n >= 0
  if n <= 1 { 1 } else { n * factorial(n - 1) }
}
```

Documentation comments are extracted by `vibe doc` to generate API
documentation. They support a subset of markdown formatting for emphasis, code
spans, and lists.

The convention is:

- First line: a concise summary of what the function does.
- Subsequent lines: details, edge cases, and usage notes.
- Keep documentation comments close to the contract annotations — together they
  form the complete specification of a function's behavior.

### Module-Level Documentation

A `//!` comment at the top of a file documents the module itself:

```vibe
//! HTTP client utilities for making authenticated requests.
//!
//! This module provides a high-level API for interacting with
//! external services, handling retries and timeouts internally.

module http_client

import std.net
import std.io
```

## 3.5 The Expression-Oriented Design

VibeLang's expression-oriented design is not an accident — it is a deliberate
choice that shapes how you think about and write code.

### Why Expression-Oriented?

In statement-oriented languages (C, Java, Python), you write code as a sequence
of instructions that modify state. In expression-oriented languages, you write
code as compositions of values.

Consider computing an absolute value. In a statement-oriented style:

```vibe
// Statement-oriented style (works, but not idiomatic)
pub abs_value(x: Int) -> Int {
  mut result := 0
  if x < 0 {
    result = -x
  } else {
    result = x
  }
  result
}
```

In expression-oriented style:

```vibe
// Expression-oriented style (idiomatic VibeLang)
pub abs_value(x: Int) -> Int {
  if x < 0 { -x } else { x }
}
```

The expression-oriented version is shorter, has no mutable state, and makes
the data flow obvious. The value flows from the `if/else` expression directly
to the function's return.

### How It Affects Code Style

Expression-oriented design encourages:

1. **Fewer mutable variables**: instead of declaring a `mut` variable and
   assigning to it in branches, you compute the value directly.

2. **Smaller functions**: when every block is an expression, you naturally write
   functions that compute and return values rather than functions that
   manipulate state.

3. **Clearer data flow**: you can trace where a value comes from by following
   the expression tree, not by tracking assignments across lines.

4. **Better composability**: expressions compose naturally. You can nest
   `if/else` inside a function call, or use a `match` result as an argument.

```vibe
pub classify_and_format(score: Int) -> Str {
  "Grade: " + match score / 10 {
    case 10 => "A+"
    case 9 => "A"
    case 8 => "B"
    case 7 => "C"
    default => "F"
  }
}
```

### Tail Expressions as Return Values

The last expression in a function body is its return value. This is the **tail
expression** rule:

```vibe
pub double(x: Int) -> Int {
  x * 2    // tail expression — this is the return value
}

pub greet(name: Str) -> Str {
  "Hello, " + name    // tail expression
}

pub max_of_three(a: Int, b: Int, c: Int) -> Int {
  if a >= b && a >= c {
    a
  } else if b >= c {
    b
  } else {
    c
  }
  // the entire if/else chain is the tail expression
}
```

You can use `return` for early exits, but idiomatic VibeLang prefers tail
expressions for the normal return path:

```vibe
pub find_first_positive(numbers: List<Int>) -> Int {
  for n in numbers {
    if n > 0 {
      return n    // early exit — return is appropriate here
    }
  }
  -1    // tail expression for the default case
}
```

The guideline: use `return` for early exits and guard clauses; use tail
expressions for the main return path.

## 3.6 Evaluation Order

VibeLang guarantees a strict, deterministic evaluation order. This is not
merely a convenience — it is a core language property that supports
reproducibility and debugging.

### Left-to-Right Argument Evaluation

Function arguments are evaluated left to right:

```vibe
pub main() -> Int {
  @effect io

  // a() is called first, then b(), then c()
  result := combine(a(), b(), c())
  0
}
```

If `a()`, `b()`, and `c()` have side effects (printing, network calls), they
will always execute in that order. This is guaranteed by the language
specification, not left to the compiler's discretion.

Why does this matter? Consider:

```vibe
pub main() -> Int {
  @effect io, mut_state

  mut counter := 0

  next() -> Int {
    @effect mut_state
    counter = counter + 1
    counter
  }

  // Always evaluates to process(1, 2, 3)
  result := process(next(), next(), next())
  0
}
```

In languages with unspecified evaluation order (like C), the result of
`process(next(), next(), next())` is undefined — the compiler can evaluate
arguments in any order. In VibeLang, this always produces `process(1, 2, 3)`.

### Deterministic Branch Selection

Conditional expressions evaluate their condition first, then execute exactly
one branch:

```vibe
value := if should_compute() {
  expensive_calculation()    // only called if condition is true
} else {
  cached_result()            // only called if condition is false
}
```

Short-circuit evaluation applies to logical operators:

- `a && b`: if `a` is `false`, `b` is not evaluated
- `a || b`: if `a` is `true`, `b` is not evaluated

```vibe
// safe_divide is never called if denominator is 0
result := denominator != 0 && safe_divide(numerator, denominator) > threshold
```

### Operator Precedence

VibeLang follows conventional operator precedence, from highest to lowest:

| Precedence | Operators | Associativity |
|---|---|---|
| 1 (highest) | Unary `-`, `!` | Right |
| 2 | `*`, `/`, `%` | Left |
| 3 | `+`, `-` | Left |
| 4 | `<`, `>`, `<=`, `>=` | Left |
| 5 | `==`, `!=` | Left |
| 6 | `&&` | Left |
| 7 (lowest) | `\|\|` | Left |

When in doubt, use parentheses. Explicit grouping is always clearer than
relying on precedence rules:

```vibe
// Clear
result := (a + b) * (c - d)

// Also clear, but relies on precedence
result := a + b * c    // multiplication first, then addition
```

### Why Deterministic Evaluation Matters

Deterministic evaluation order is part of VibeLang's broader commitment to
reproducibility. When you run the same code with the same inputs, you get the
same results — including the same side-effect ordering.

This matters for:

- **Debugging**: you can reproduce issues reliably.
- **Testing**: test results are deterministic.
- **Auditing**: you can reason about what a program did by reading the code.
- **AI-assisted development**: code generators can predict behavior without
  worrying about unspecified evaluation order.

In concurrent code (covered in Chapter 11), VibeLang provides separate
mechanisms for controlled non-determinism. But within a single thread of
execution, evaluation order is always deterministic.

## 3.7 Putting It Together

Let's write a small program that uses everything from this chapter:

```vibe
//! Temperature conversion utilities.

module temp_convert

import std.io

const FREEZING_F: Float = 32.0
const SCALE_FACTOR: Float = 1.8

/// Converts a Fahrenheit temperature to Celsius.
pub to_celsius(fahrenheit: Float) -> Float {
  (fahrenheit - FREEZING_F) / SCALE_FACTOR
}

/// Converts a Celsius temperature to Fahrenheit.
pub to_fahrenheit(celsius: Float) -> Float {
  celsius * SCALE_FACTOR + FREEZING_F
}

/// Returns a human-readable description of the temperature.
pub describe(celsius: Float) -> Str {
  if celsius < 0.0 {
    "freezing"
  } else if celsius < 10.0 {
    "cold"
  } else if celsius < 20.0 {
    "cool"
  } else if celsius < 30.0 {
    "comfortable"
  } else {
    "hot"
  }
}

pub main() -> Int {
  @effect io

  temps_f := [32.0, 50.0, 72.0, 98.6, 212.0]

  for f in temps_f {
    c := to_celsius(f)
    label := describe(c)
    println(f + "°F = " + c + "°C (" + label + ")")
  }

  0
}
```

This program demonstrates:

- Module declaration and imports
- Constants with explicit types
- Immutable bindings with type inference
- `if/else` used as an expression (in `describe`)
- Tail expressions as return values
- Documentation comments
- A `for` loop iterating over a list
- Expression-oriented style throughout

## 3.8 Summary

This chapter covered the core syntax and semantics of VibeLang:

- **Source files** use `.yb` extension and contain module declarations, imports,
  and top-level declarations in that order.
- **Bindings** are immutable by default (`x := value`). Use `mut` only when
  reassignment is necessary. Constants (`const`) are compile-time values.
- **Type inference** works locally within functions. Public API signatures
  require explicit type annotations.
- **Shadowing** lets you rebind a name, even changing its type, without
  mutation.
- **Expressions** produce values; **statements** do not. VibeLang is
  expression-oriented: blocks, `if/else`, and `match` are all expressions.
- **Tail expressions** are the idiomatic way to return values from functions
  and blocks.
- **Evaluation order** is deterministic: left-to-right for arguments,
  short-circuit for logical operators.

These rules are simple individually, but their combination produces a language
where code is predictable, readable, and easy to reason about — properties that
compound as programs grow.

---

Next: Chapter 4 covers types and functions in depth.
