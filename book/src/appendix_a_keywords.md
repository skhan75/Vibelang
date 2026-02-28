# Appendix A: Keyword Reference

This appendix lists every reserved keyword in VibeLang, grouped by category.
Each entry includes syntax, a brief description, a minimal example, and a chapter
reference. Reserved keywords cannot be used as identifiers.

---

## A.1 Declaration Keywords

### `pub`

Marks a declaration as publicly visible outside its module. Without `pub`,
declarations are module-private.

```vibe
pub greet(name: Str) -> Str {
  "Hello, " + name
}
```

**Chapter:** 10 — Modules and Packages

### `type`

Declares a named structured type (record).

```vibe
pub type Point { x: Float, y: Float }
```

**Chapter:** 4 — Types and Functions

### `mut`

Declares a mutable binding. Without `mut`, all bindings are immutable.

```vibe
mut counter := 0
counter = counter + 1
```

**Chapter:** 3 — Core Syntax and Semantics

### `const`

Declares a compile-time constant. The value must be a literal or constant
expression.

```vibe
const MAX_RETRIES := 5
```

**Chapter:** 3 — Core Syntax and Semantics

---

## A.2 Control Flow Keywords

### `if` / `else`

Conditional branching. `if` is an expression — it produces a value. The condition
must be `Bool`.

```vibe
status := if score >= 90 { "pass" } else { "fail" }
```

**Chapter:** 5 — Control Flow

### `for` / `in`

Iterates over elements of a collection or range. `in` separates the loop
variable from the iterable.

```vibe
for name in names {
  println(name)
}
```

**Chapter:** 5 — Control Flow

### `while`

Loops while a boolean condition remains true.

```vibe
mut n := 10
while n > 0 {
  n = n - 1
}
```

**Chapter:** 5 — Control Flow

### `repeat`

Executes a block a fixed number of times.

```vibe
repeat 3 {
  println("hello")
}
```

**Chapter:** 5 — Control Flow

### `match`

Pattern matching expression. Matches a value against patterns and executes the
corresponding branch. Must be exhaustive.

```vibe
message := match status {
  200 => "OK"
  404 => "Not Found"
  _ => "Unknown"
}
```

**Chapter:** 5 — Control Flow

### `case`

Introduces a branch inside a `select` statement.

```vibe
select {
  case msg := <-ch => println(msg)
  case after 5000 => println("timeout")
}
```

**Chapter:** 11 — Concurrency

### `return`

Exits the current function early with a value. Optional when the function body
ends with a tail expression.

```vibe
pub abs(n: Int) -> Int {
  if n < 0 { return -n }
  n
}
```

**Chapter:** 4 — Types and Functions

### `break`

Exits the innermost enclosing loop immediately.

```vibe
for item in items {
  if item == target { break }
}
```

**Chapter:** 5 — Control Flow

### `continue`

Skips the rest of the current loop iteration and proceeds to the next one.

```vibe
for n in 0..10 {
  if n % 2 == 0 { continue }
  println(n.to_str())
}
```

**Chapter:** 5 — Control Flow

---

## A.3 Concurrency Keywords

### `go`

Spawns a new lightweight task that runs concurrently. Requires
`@effect concurrency`.

```vibe
go process_request(req)
```

**Chapter:** 11 — Concurrency

### `chan`

Creates a typed channel for inter-task communication.

```vibe
ch := chan(10)
ch <- "hello"
msg := <-ch
```

**Chapter:** 11 — Concurrency

### `select`

Waits on multiple channel operations simultaneously. Executes the first branch
that becomes ready.

```vibe
select {
  case val := <-data_ch => handle(val)
  case err := <-err_ch => report(err)
}
```

**Chapter:** 11 — Concurrency

### `after`

Timeout branch inside `select`. Fires after the specified milliseconds if no
other case is ready.

```vibe
select {
  case msg := <-ch => println(msg)
  case after 5000 => println("timed out")
}
```

**Chapter:** 11 — Concurrency

### `closed`

Channel-closed branch inside `select`. Fires when the specified channel has been
closed by the sender.

```vibe
select {
  case val := <-ch => process(val)
  case closed ch => println("done")
}
```

**Chapter:** 11 — Concurrency

### `default`

Non-blocking fallback in `select`. Executes immediately if no other case is
ready.

```vibe
select {
  case msg := <-ch => println(msg)
  case default => println("nothing ready")
}
```

**Chapter:** 11 — Concurrency

### `async` / `await`

`async` declares an asynchronous function. `await` suspends execution until the
awaited operation completes. `await` is only valid inside `async` functions.

```vibe
pub async fetch_page(url: Str) -> Result<Str, Error> {
  @effect net
  response := await http_get(url)
  response.body()
}
```

**Chapter:** 11 — Concurrency

### `thread`

Spawns an OS-level thread for CPU-bound or blocking work that should not occupy
the lightweight task scheduler.

```vibe
handle := thread compute_heavy_result(data)
result := handle.join()
```

**Chapter:** 11 — Concurrency

---

## A.4 Literal Keywords

### `true` / `false`

Boolean literals. Type is `Bool`.

```vibe
enabled := true
done := false
```

**Chapter:** 3 — Core Syntax and Semantics

### `none`

Represents the absence of a value in an optional type (`T?`).

```vibe
pub find(items: List<Str>, target: Str) -> Str? {
  for item in items {
    if item == target { return item }
  }
  none
}
```

**Chapter:** 8 — Error Handling with Result

---

## A.5 Module Keywords

### `module`

Declares the module that the current file belongs to. Must appear at the top of
the file.

```vibe
module app.services.auth
```

**Chapter:** 10 — Modules and Packages

### `import`

Brings declarations from another module into scope.

```vibe
import std.io
import std.json
```

**Chapter:** 10 — Modules and Packages

---

## A.6 Quick Reference Table

| Keyword    | Category    | Purpose                                      |
|------------|-------------|----------------------------------------------|
| `pub`      | Declaration | Public visibility modifier                   |
| `type`     | Declaration | Structured type definition                   |
| `mut`      | Declaration | Mutable binding modifier                     |
| `const`    | Declaration | Compile-time constant                        |
| `if`       | Control     | Conditional expression                       |
| `else`     | Control     | Alternative branch                           |
| `for`      | Control     | Collection/range iteration                   |
| `in`       | Control     | Loop variable separator                      |
| `while`    | Control     | Condition-based loop                         |
| `repeat`   | Control     | Fixed-count loop                             |
| `match`    | Control     | Pattern matching expression                  |
| `case`     | Control     | Branch in `select`                           |
| `return`   | Control     | Early function exit                          |
| `break`    | Control     | Exit innermost loop                          |
| `continue` | Control     | Skip to next iteration                       |
| `go`       | Concurrency | Spawn lightweight task                       |
| `chan`      | Concurrency | Create typed channel                         |
| `select`   | Concurrency | Multiplex channel operations                 |
| `after`    | Concurrency | Timeout branch in `select`                   |
| `closed`   | Concurrency | Channel-closed branch in `select`            |
| `default`  | Concurrency | Non-blocking fallback in `select`            |
| `async`    | Concurrency | Asynchronous function declaration            |
| `await`    | Concurrency | Suspend until async operation completes      |
| `thread`   | Concurrency | Spawn OS-level thread                        |
| `true`     | Literal     | Boolean true                                 |
| `false`    | Literal     | Boolean false                                |
| `none`     | Literal     | Absent optional value                        |
| `module`   | Module      | File module declaration                      |
| `import`   | Module      | Bring external declarations into scope       |
