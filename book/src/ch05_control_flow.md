# Chapter 5: Control Flow

This chapter covers every control flow construct in VibeLang: conditionals,
loops, pattern matching, and the `select` statement for concurrency. You will
learn not just the syntax, but how VibeLang's expression-oriented design makes
control flow more powerful and less error-prone than in most languages.

By the end of this chapter, you will be able to write complex branching and
iteration logic idiomatically, using expressions to minimize mutable state and
maximize clarity.

## 5.1 Conditional Expressions with `if/else`

The `if/else` construct is VibeLang's primary branching mechanism. Unlike most
languages, `if/else` in VibeLang is an **expression** — it produces a value.

### Basic `if/else`

```vibe
pub main() -> Int {
  @effect io

  temperature := 25

  if temperature > 30 {
    println("It's hot outside")
  } else {
    println("It's comfortable")
  }

  0
}
```

The condition must be a `Bool`. VibeLang has no concept of "truthy" or "falsy"
values — you cannot use an integer, string, or list as a condition:

```vibe
pub main() -> Int {
  items := [1, 2, 3]

  // if items { ... }    // error: expected Bool, found List<Int>
  if items.len() > 0 {   // correct: explicit comparison
    // process items
  }
  0
}
```

```
error[E0501]: mismatched types in condition
 --> main.yb:4:6
  |
4 |   if items {
  |      ^^^^^ expected `Bool`, found `List<Int>`
  |
note: VibeLang requires explicit boolean conditions
help: did you mean `if items.len() > 0 {`?
```

This strictness is intentional. In languages with truthy values, the meaning
of `if x` depends on the type of `x`: it might check for null, zero, empty,
or something else entirely. VibeLang eliminates this ambiguity — you always
write exactly the condition you mean.

### `if/else` as Expression

Because `if/else` is an expression, you can bind its result to a variable:

```vibe
pub main() -> Int {
  @effect io

  age := 25

  category := if age < 13 {
    "child"
  } else if age < 18 {
    "teenager"
  } else if age < 65 {
    "adult"
  } else {
    "senior"
  }

  println("Category: " + category)
  0
}
```

When used as an expression, both branches must produce the same type. The
compiler enforces this:

```vibe
// This will not compile
result := if condition {
  42
} else {
  "not a number"
}
```

```
error[E0502]: `if` and `else` branches have incompatible types
 --> main.yb:2:12
  |
2 | result := if condition {
  |           ^^ branches must have the same type
3 |   42
  |   -- type `Int`
5 |   "not a number"
  |   -------------- type `Str`
```

When `if/else` is used as an expression, the `else` branch is **required**.
Without it, the compiler cannot determine what value to produce when the
condition is false:

```vibe
// Error: if-expression requires else branch
// value := if x > 0 { x }

// Correct
value := if x > 0 { x } else { 0 }
```

```
error[E0503]: `if` expression missing `else` branch
 --> main.yb:2:11
  |
2 |   value := if x > 0 { x }
  |            ^^^^^^^^^^^^^^^ missing `else` branch
  |
note: when `if` is used as an expression, `else` is required
      so the expression always produces a value
```

When `if` is used as a statement (not bound to a variable), the `else` branch
is optional:

```vibe
pub main() -> Int {
  @effect io

  score := 95
  if score >= 90 {
    println("Excellent!")    // no else needed
  }
  0
}
```

### Chained `if/else if/else`

For multiple conditions, chain `else if` branches:

```vibe
pub http_status_text(code: Int) -> Str {
  if code == 200 {
    "OK"
  } else if code == 201 {
    "Created"
  } else if code == 301 {
    "Moved Permanently"
  } else if code == 400 {
    "Bad Request"
  } else if code == 404 {
    "Not Found"
  } else if code == 500 {
    "Internal Server Error"
  } else {
    "Unknown Status"
  }
}
```

For many branches like this, consider using `match` instead (covered in
Section 5.3). Match is often clearer when you are comparing a single value
against multiple possibilities.

### Nested Conditions

Conditions can be nested, though deeply nested code is usually a sign that
the logic should be restructured:

```vibe
pub classify_triangle(a: Float, b: Float, c: Float) -> Str {
  if a <= 0.0 || b <= 0.0 || c <= 0.0 {
    "invalid"
  } else if a == b && b == c {
    "equilateral"
  } else if a == b || b == c || a == c {
    "isosceles"
  } else {
    "scalene"
  }
}
```

## 5.2 Loops

VibeLang provides three loop constructs, each designed for a specific use case:
`for` for iteration, `while` for condition-based repetition, and `repeat` for
counted repetition.

### `for` Loops

The `for` loop iterates over any iterable value:

```vibe
pub main() -> Int {
  @effect io

  names := ["Alice", "Bob", "Charlie"]

  for name in names {
    println("Hello, " + name)
  }

  0
}
```

The loop variable (`name`) is immutable and scoped to the loop body. A new
binding is created for each iteration.

#### Iterating Over Ranges

VibeLang supports range expressions for numeric iteration:

```vibe
pub main() -> Int {
  @effect io

  // 0 to 4 (exclusive upper bound)
  for i in 0..5 {
    println(i)
  }

  // 1 to 10 (inclusive upper bound)
  for i in 1..=10 {
    println(i)
  }

  0
}
```

- `0..5` produces values 0, 1, 2, 3, 4 (exclusive end)
- `1..=10` produces values 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 (inclusive end)

#### Iterating Over Maps

When iterating over a map, each iteration yields a key-value pair:

```vibe
pub main() -> Int {
  @effect io

  scores := {"Alice": 95, "Bob": 87, "Charlie": 92}

  for name, score in scores {
    println(name + ": " + to_str(score))
  }

  0
}
```

#### Iterating with Index

Use the two-variable form to get both the index and the value:

```vibe
pub main() -> Int {
  @effect io

  colors := ["red", "green", "blue"]

  for i, color in colors {
    println(to_str(i) + ": " + color)
  }
  // Output:
  // 0: red
  // 1: green
  // 2: blue

  0
}
```

### `while` Loops

The `while` loop repeats as long as a condition is true:

```vibe
pub main() -> Int {
  @effect io

  mut count := 1
  while count <= 10 {
    println(count)
    count = count + 1
  }

  0
}
```

The condition is checked before each iteration. If it is false initially, the
body never executes.

A practical example — reading until a sentinel value:

```vibe
pub sum_until_zero(numbers: List<Int>) -> Int {
  mut total := 0
  mut i := 0

  while i < numbers.len() {
    n := numbers.get(i)
    if n == 0 {
      break
    }
    total = total + n
    i = i + 1
  }

  total
}
```

### `repeat` Loops

The `repeat` loop executes a body a fixed number of times. This is unique to
VibeLang and exists because "do something N times" is an extremely common
pattern that deserves its own syntax:

```vibe
pub main() -> Int {
  @effect io

  repeat 5 {
    println("VibeLang!")
  }

  0
}
```

This prints "VibeLang!" five times. The count must be a non-negative integer.

`repeat` is cleaner than the alternatives when you don't need an index:

```vibe
// With repeat (clean, intent is clear)
repeat 3 {
  send_heartbeat()
}

// Without repeat (more noise for the same result)
for _ in 0..3 {
  send_heartbeat()
}
```

When you do need the iteration number, use `for` with a range instead.

### `break` and `continue`

`break` exits the innermost loop immediately. `continue` skips to the next
iteration:

```vibe
pub first_negative(numbers: List<Int>) -> Int {
  mut result := 0

  for n in numbers {
    if n >= 0 {
      continue    // skip non-negative numbers
    }
    result = n
    break         // found it, stop looking
  }

  result
}
```

### Loop Labels

When you have nested loops, `break` and `continue` apply to the innermost
loop by default. Labels let you target an outer loop:

```vibe
pub find_in_matrix(matrix: List<List<Int>>, target: Int) -> Bool {
  mut found := false

  @outer for row in matrix {
    for item in row {
      if item == target {
        found = true
        break @outer    // break out of BOTH loops
      }
    }
  }

  found
}
```

Labels are declared with `@label_name` before the loop keyword and referenced
with `break @label_name` or `continue @label_name`.

A more complex example with `continue` on an outer loop:

```vibe
pub process_batches(batches: List<List<Int>>) -> List<Int> {
  @effect alloc

  mut results: List<Int> := []

  @batch_loop for batch in batches {
    // Skip empty batches
    if batch.len() == 0 {
      continue @batch_loop
    }

    for item in batch {
      if item < 0 {
        // Negative item invalidates the entire batch
        continue @batch_loop
      }
      results.append(item)
    }
  }

  results
}
```

Without labels, the `continue` inside the inner loop would only skip the
current item, not the entire batch. The label makes the intent explicit: "skip
the rest of this batch and move to the next one."

## 5.3 Pattern Matching with `match`

The `match` expression is VibeLang's most powerful control flow construct. It
examines a value and executes the first branch whose pattern matches.

### Basic Match Syntax

```vibe
pub day_type(day: Str) -> Str {
  match day {
    case "Monday" => "start of work week"
    case "Friday" => "end of work week"
    case "Saturday" => "weekend"
    case "Sunday" => "weekend"
    default => "midweek"
  }
}
```

Each `case` specifies a pattern and a body separated by `=>`. The body can be
a single expression or a block.

### Matching on Values

Match works with any type that supports equality:

```vibe
pub describe_exit_code(code: Int) -> Str {
  match code {
    case 0 => "success"
    case 1 => "general error"
    case 2 => "misuse of shell command"
    case 126 => "command not executable"
    case 127 => "command not found"
    case 130 => "interrupted (Ctrl+C)"
    default => "unknown exit code: " + to_str(code)
  }
}
```

### Matching on Enum Variants

This is where `match` truly shines. Combined with enums, it provides type-safe
branching with data extraction:

```vibe
pub type Result<T, E> {
  Ok(value: T)
  Err(error: E)
}

pub handle_result(r: Result<Int, Str>) -> Str {
  match r {
    case Ok(value) => "Success: " + to_str(value)
    case Err(msg) => "Error: " + msg
  }
}
```

Enum matching can destructure nested data:

```vibe
pub type Expr {
  Literal(value: Int)
  Add(left: Expr, right: Expr)
  Mul(left: Expr, right: Expr)
  Neg(inner: Expr)
}

pub eval(expr: Expr) -> Int {
  match expr {
    case Literal(v) => v
    case Add(l, r) => eval(l) + eval(r)
    case Mul(l, r) => eval(l) * eval(r)
    case Neg(inner) => -eval(inner)
  }
}

pub to_string(expr: Expr) -> Str {
  match expr {
    case Literal(v) => to_str(v)
    case Add(l, r) => "(" + to_string(l) + " + " + to_string(r) + ")"
    case Mul(l, r) => "(" + to_string(l) + " * " + to_string(r) + ")"
    case Neg(inner) => "(-" + to_string(inner) + ")"
  }
}
```

### The `default` Case

`default` matches anything not covered by previous cases:

```vibe
pub priority_label(level: Int) -> Str {
  match level {
    case 1 => "critical"
    case 2 => "high"
    case 3 => "medium"
    default => "low"
  }
}
```

Use `default` when:

- Matching on open-ended types like `Int` or `Str` where you cannot enumerate
  all values
- You want to group "everything else" into a single handler

Avoid `default` when:

- Matching on enums where you want exhaustiveness checking to catch new
  variants

### Exhaustiveness Checking

When matching on an enum without a `default` case, the compiler verifies that
every variant is covered:

```vibe
pub type Permission {
  Read
  Write
  Execute
  Admin
}

pub can_modify(perm: Permission) -> Bool {
  match perm {
    case Read => false
    case Write => true
    case Execute => false
    // missing: Admin
  }
}
```

```
error[E0540]: non-exhaustive match expression
 --> auth.yb:9:3
  |
9 |   match perm {
  |   ^^^^^^^^^^ pattern `Admin` not covered
  |
help: ensure all variants are handled:
  |     case Admin => /* ... */
  |
note: alternatively, add a `default` case to handle remaining variants
```

This is one of the most valuable safety features in VibeLang. When you add a
new variant to an enum — say, `Permission` gains a `SuperAdmin` variant — the
compiler immediately flags every `match` that needs updating. You cannot
accidentally forget to handle the new case.

This is why idiomatic VibeLang prefers explicit cases over `default` for enums:
you trade a bit of verbosity for compile-time safety when the enum evolves.

### Match as Expression

`match` is an expression and can be used anywhere a value is expected:

```vibe
pub main() -> Int {
  @effect io

  status := 404

  message := match status {
    case 200 => "OK"
    case 404 => "Not Found"
    case 500 => "Server Error"
    default => "Unknown"
  }

  println(message)
  0
}
```

You can use match inline in function calls:

```vibe
pub main() -> Int {
  @effect io

  level := 3

  println("Priority: " + match level {
    case 1 => "HIGH"
    case 2 => "MEDIUM"
    default => "LOW"
  })

  0
}
```

### Match with Guards

Patterns can include guard conditions for more precise matching:

```vibe
pub classify_number(n: Int) -> Str {
  match n {
    case 0 => "zero"
    case x if x > 0 && x <= 10 => "small positive"
    case x if x > 10 => "large positive"
    case x if x < 0 && x >= -10 => "small negative"
    default => "large negative"
  }
}
```

Guards are evaluated after the pattern matches. They add conditions that
cannot be expressed by the pattern alone.

### Multi-Pattern Cases

A single case can match multiple patterns:

```vibe
pub is_vowel(ch: Str) -> Bool {
  match ch {
    case "a" | "e" | "i" | "o" | "u" => true
    case "A" | "E" | "I" | "O" | "U" => true
    default => false
  }
}

pub is_weekend(day: Str) -> Bool {
  match day {
    case "Saturday" | "Sunday" => true
    default => false
  }
}
```

The `|` operator separates alternative patterns within a single case. The body
executes if any of the patterns match.

## 5.4 The `select` Statement (Preview)

VibeLang includes a `select` statement for multiplexing over channel
operations. This is a concurrency primitive covered in depth in Chapter 11, but
a brief preview here shows how it fits into the control flow landscape.

### What `select` Does

`select` waits on multiple channel operations and executes the branch for
whichever operation completes first:

```vibe
pub handle_messages(data_ch: Chan<Str>, quit_ch: Chan<Bool>) -> Str {
  @effect concurrency

  select {
    case msg := data_ch.recv() => {
      "received: " + msg
    }
    case _ := quit_ch.recv() => {
      "shutting down"
    }
    case after 5s => {
      "timed out after 5 seconds"
    }
  }
}
```

### Select Cases

`select` supports three kinds of cases:

- **`case x := ch.recv()`** — receive from a channel
- **`case after duration`** — timeout after a duration
- **`case closed ch`** — detect when a channel is closed

```vibe
pub monitor(events: Chan<Event>, health: Chan<Status>) -> Str {
  @effect concurrency

  select {
    case event := events.recv() => {
      process_event(event)
      "processed event"
    }
    case status := health.recv() => {
      log_status(status)
      "logged status"
    }
    case after 30s => {
      "no activity for 30 seconds"
    }
    case closed events => {
      "event stream ended"
    }
  }
}
```

Duration literals like `5s`, `100ms`, `2m` are built into VibeLang. They make
timeout expressions readable without requiring imports or conversions.

`select` is covered fully in Chapter 11. The key point for now: it is a
control flow construct, like `match`, but for concurrent channel operations
rather than value patterns.

## 5.5 Expression-Oriented Control Flow

VibeLang's expression-oriented design means that `if/else` and `match` are not
just control flow — they are value-producing expressions. This has profound
implications for how you write code.

### Assigning Complex Logic to Variables

Instead of declaring a mutable variable and assigning to it in multiple
branches, compute the value directly:

```vibe
// Statement-oriented style (works, but not idiomatic)
pub ticket_price_v1(age: Int, is_member: Bool) -> Float {
  mut price := 0.0
  if age < 12 {
    price = 5.0
  } else if age >= 65 {
    price = 7.0
  } else {
    price = 12.0
  }
  if is_member {
    price = price * 0.8
  }
  price
}

// Expression-oriented style (idiomatic VibeLang)
pub ticket_price_v2(age: Int, is_member: Bool) -> Float {
  base := if age < 12 {
    5.0
  } else if age >= 65 {
    7.0
  } else {
    12.0
  }

  if is_member { base * 0.8 } else { base }
}
```

The second version has no mutable state. Each value is computed once and bound
immutably. The data flow is clear: `base` is determined by age, then the final
price is determined by membership.

### Reducing Mutable State

Expression-oriented control flow is the primary tool for reducing `mut` usage.
Every time you would write "declare mut, then assign in branches," ask whether
you can use an `if/else` or `match` expression instead.

```vibe
// Before: mutable state
pub format_size_v1(bytes: Int) -> Str {
  mut result := ""
  mut value := bytes
  mut unit := "B"

  if value >= 1_073_741_824 {
    value = value / 1_073_741_824
    unit = "GB"
  } else if value >= 1_048_576 {
    value = value / 1_048_576
    unit = "MB"
  } else if value >= 1024 {
    value = value / 1024
    unit = "KB"
  }

  result = to_str(value) + " " + unit
  result
}

// After: expression-oriented
pub format_size_v2(bytes: Int) -> Str {
  pair := if bytes >= 1_073_741_824 {
    [bytes / 1_073_741_824, "GB"]
  } else if bytes >= 1_048_576 {
    [bytes / 1_048_576, "MB"]
  } else if bytes >= 1024 {
    [bytes / 1024, "KB"]
  } else {
    [bytes, "B"]
  }

  to_str(pair.get(0)) + " " + pair.get(1)
}
```

### Composing Expressions

Because control flow constructs are expressions, they compose naturally:

```vibe
pub describe_weather(temp: Float, humidity: Float, wind: Float) -> Str {
  comfort := match true {
    case _ if temp > 35.0 => "dangerously hot"
    case _ if temp > 28.0 => "hot"
    case _ if temp > 18.0 => "comfortable"
    case _ if temp > 5.0 => "cool"
    default => "cold"
  }

  moisture := if humidity > 80.0 {
    " and humid"
  } else if humidity < 20.0 {
    " and dry"
  } else {
    ""
  }

  wind_note := if wind > 50.0 {
    " with strong winds"
  } else if wind > 20.0 {
    " with moderate wind"
  } else {
    ""
  }

  comfort + moisture + wind_note
}
```

Each piece of the description is computed independently as an expression, then
composed at the end with string concatenation. No mutable state, no temporary
variables that get reassigned.

### When Mutable State Is Still Appropriate

Expression-oriented style is not always the best choice. Some algorithms are
naturally stateful:

```vibe
pub running_average(values: List<Float>) -> List<Float> {
  @effect alloc

  mut sum := 0.0
  mut averages: List<Float> := []

  for i, v in values {
    sum = sum + v
    averages.append(sum / to_float(i + 1))
  }

  averages
}
```

Here, `sum` accumulates across iterations and `averages` grows with each step.
Trying to eliminate `mut` would make this code harder to read, not easier.

The guideline: use expressions when they make the logic clearer. Use `mut` when
the algorithm is inherently stateful. Don't force one style where the other
fits better.

## 5.6 Putting It Together

Here is a complete program that exercises every control flow construct from
this chapter:

```vibe
//! A simple command processor that demonstrates VibeLang control flow.

module command_processor

import std.io

pub type Command {
  Help
  Greet(name: Str)
  Repeat(message: Str, count: Int)
  Quit
}

pub parse_command(input: Str) -> Command {
  match input {
    case "help" => Help
    case "quit" => Quit
    default => {
      if input.starts_with("greet ") {
        Greet(input.slice(6, input.len()))
      } else if input.starts_with("repeat ") {
        parts := input.slice(7, input.len())
        Repeat(parts, 3)
      } else {
        Help
      }
    }
  }
}

pub execute(cmd: Command) -> Str {
  match cmd {
    case Help => {
      "Commands: help, greet <name>, repeat <msg>, quit"
    }
    case Greet(name) => {
      prefix := if name.len() > 0 { "Hello" } else { "Hey" }
      prefix + ", " + name + "!"
    }
    case Repeat(msg, count) => {
      mut output := ""
      repeat count {
        output = output + msg + "\n"
      }
      output
    }
    case Quit => "Goodbye!"
  }
}

pub main() -> Int {
  @effect io

  commands := ["help", "greet Alice", "repeat VibeLang", "quit"]

  for input in commands {
    cmd := parse_command(input)
    result := execute(cmd)
    println("> " + input)
    println(result)
    println("")
  }

  0
}
```

This program demonstrates:

- Enum types with variants carrying data
- `match` for dispatching on enum variants
- `if/else` as expression (the `prefix` binding in `Greet`)
- `repeat` for counted iteration
- `for` for iterating over a list
- Expression-oriented style with tail expressions
- Mutable state where appropriate (building `output` in `Repeat`)

## 5.7 Summary

This chapter covered VibeLang's control flow constructs:

- **`if/else`** is an expression. Conditions must be `Bool` — no truthy/falsy.
  When used as an expression, both branches must produce the same type and
  `else` is required.
- **`for`** iterates over lists, ranges, maps, and other iterables. The loop
  variable is immutable and scoped to the loop body.
- **`while`** repeats while a condition is true. The condition is checked before
  each iteration.
- **`repeat`** executes a body a fixed number of times. Use it when you don't
  need an index.
- **`break`** exits a loop; **`continue`** skips to the next iteration. Both
  support labels for targeting outer loops in nested structures.
- **`match`** examines a value against patterns. It supports value matching,
  enum destructuring, guards, multi-patterns with `|`, and the `default`
  catch-all. The compiler checks exhaustiveness for enum matches.
- **`select`** multiplexes over channel operations (preview — full coverage in
  Chapter 11).
- **Expression-oriented control flow** lets you compute values directly with
  `if/else` and `match`, reducing mutable state and making data flow explicit.

The combination of expression-oriented design and exhaustive pattern matching
gives VibeLang a control flow model that is both concise and safe. You write
less code, and the compiler catches more mistakes.

---

Next: Chapter 6 introduces contracts and intent-driven development.
