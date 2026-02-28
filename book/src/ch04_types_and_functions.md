# Chapter 4: Types and Functions

This chapter covers VibeLang's type system and function model in depth. You will
learn how types keep programs safe without drowning you in annotations, how
functions are declared and composed, and how to define your own types with
structs and enums.

Types and functions are the two primary building blocks of every VibeLang
program. Understanding them well is the foundation for everything that follows.

## 4.1 The Type System Philosophy

VibeLang's type system is guided by two principles that are often in tension:

1. **Safety**: catch errors at compile time, not at runtime.
2. **Ergonomics**: don't force the programmer to write types everywhere.

The resolution is **local type inference with explicit public boundaries**.
Inside function bodies, the compiler figures out types for you. At module
boundaries — public function signatures, type definitions, constants — you
write types explicitly.

This split is intentional:

- **Inside functions**, you are the primary reader. Inference reduces noise and
  lets you focus on logic.
- **At boundaries**, other people (and tools) are the readers. Explicit types
  serve as documentation, contracts, and stable interfaces.

The result: VibeLang code reads cleanly in function bodies while remaining
self-documenting at API surfaces.

### Static Typing

VibeLang is statically typed. Every value has a type known at compile time.
There is no `any` type, no implicit coercion between unrelated types, and no
runtime type discovery. If the compiler accepts your program, every operation
is type-safe.

```vibe
pub main() -> Int {
  x := 42         // x is Int at compile time
  y := "hello"    // y is Str at compile time

  // x + y         // error: cannot add Int and Str
  0
}
```

```
error[E0401]: mismatched types in binary operation
 --> main.yb:5:3
  |
5 |   x + y
  |   ^ - - `y` has type `Str`
  |   |
  |   `x` has type `Int`
  |
note: operator `+` is not defined for `Int` + `Str`
help: convert explicitly if intended: `to_str(x) + y`
```

## 4.2 Primitive Types

VibeLang provides a small set of primitive types that cover the vast majority
of everyday programming needs.

### Int

`Int` is the default integer type, backed by a 64-bit signed integer (`i64`).
It can represent values from -9,223,372,036,854,775,808 to
9,223,372,036,854,775,807.

```vibe
pub main() -> Int {
  age := 30
  population := 8_000_000_000    // underscores for readability
  negative := -42
  0
}
```

VibeLang uses `Int` as the default because 64-bit integers handle virtually all
practical integer needs without overflow surprises. You don't need to think
about whether your loop counter will overflow at 2 billion — it won't.

### Float

`Float` is the default floating-point type, backed by a 64-bit IEEE 754
double-precision value (`f64`).

```vibe
pub main() -> Int {
  pi := 3.14159265358979
  temperature := -40.0
  tiny := 0.000_001
  0
}
```

Floating-point literals must contain a decimal point. `3.0` is a `Float`;
`3` is an `Int`. This distinction is always unambiguous.

### Bool

`Bool` has exactly two values: `true` and `false`.

```vibe
pub main() -> Int {
  active := true
  found := false
  ready := 10 > 5    // comparison produces Bool
  0
}
```

VibeLang does not have "truthy" or "falsy" values. `0`, `""`, and `[]` are
not `false`. Only `Bool` values can be used in conditions:

```vibe
pub main() -> Int {
  count := 0

  // if count { ... }    // error: expected Bool, found Int

  if count > 0 {    // correct: explicit comparison
    // ...
  }
  0
}
```

```
error[E0402]: mismatched types
 --> main.yb:4:6
  |
4 |   if count {
  |      ^^^^^ expected `Bool`, found `Int`
  |
note: VibeLang does not have truthy/falsy values
help: use an explicit comparison: `if count != 0 {`
```

This strictness prevents a common class of bugs. In languages with truthy
values, `if items` might mean "if items is not null", "if items is not empty",
or "if items is not zero" depending on the type. In VibeLang, you always write
the condition you mean.

### Str

`Str` is VibeLang's string type. Strings are UTF-8 encoded, immutable
sequences of characters.

```vibe
pub main() -> Int {
  @effect io

  greeting := "Hello, world!"
  empty := ""
  multiline := "line one\nline two"

  println(greeting.len())     // prints: 13
  println(greeting + " 🎉")   // concatenation with +
  0
}
```

String concatenation uses the `+` operator. Strings are compared with `==` and
`!=` for equality, and `<`, `>` for lexicographic ordering.

### Sized Integer Types

When you need precise control over integer size — for binary protocols, FFI,
or memory-sensitive code — VibeLang provides sized variants:

| Type | Size | Range |
|---|---|---|
| `i8` | 8-bit signed | -128 to 127 |
| `i16` | 16-bit signed | -32,768 to 32,767 |
| `i32` | 32-bit signed | -2,147,483,648 to 2,147,483,647 |
| `i64` | 64-bit signed | -(2^63) to 2^63 - 1 |
| `isize` | Pointer-width signed | Platform-dependent |
| `u8` | 8-bit unsigned | 0 to 255 |
| `u16` | 16-bit unsigned | 0 to 65,535 |
| `u32` | 32-bit unsigned | 0 to 4,294,967,295 |
| `u64` | 64-bit unsigned | 0 to 2^64 - 1 |
| `usize` | Pointer-width unsigned | Platform-dependent |

You can create sized integers with type annotations or literal suffixes:

```vibe
pub main() -> Int {
  a: i32 = 42          // type annotation
  b := 42i32           // literal suffix
  c := 255u8           // unsigned byte
  d := 1_000_000i64    // explicit i64 (same as Int)
  0
}
```

### Sized Float Types

| Type | Size | Precision |
|---|---|---|
| `f32` | 32-bit | ~7 decimal digits |
| `f64` | 64-bit | ~15 decimal digits |

```vibe
pub main() -> Int {
  precise := 3.14159265358979    // f64 (default Float)
  compact := 3.14f32             // f32, less precision, less memory
  0
}
```

Use `f32` when memory or bandwidth matters (graphics, large arrays of
coordinates). Use `f64` (the default `Float`) for everything else.

### Numeric Literals and Suffixes

VibeLang supports several literal formats:

```vibe
pub main() -> Int {
  // Integer literals
  decimal := 1_000_000
  hex := 0xFF
  octal := 0o77
  binary := 0b1010_1100

  // Float literals
  standard := 3.14
  scientific := 1.5e10
  negative_exp := 2.5e-3

  // Suffixed literals
  small := 42i8
  unsigned := 100u32
  single := 1.0f32

  0
}
```

Underscores in numeric literals are purely visual separators. The compiler
ignores them: `1_000_000` and `1000000` are identical.

### Type Widening and Narrowing

VibeLang allows **widening** conversions (smaller type to larger type)
implicitly, but **forbids narrowing** conversions (larger to smaller) without
an explicit cast:

```vibe
pub main() -> Int {
  small: i32 = 42
  big: i64 = small       // OK: i32 widens to i64

  // big2: i32 = big     // error: cannot narrow i64 to i32
  0
}
```

```
error[E0410]: cannot implicitly narrow `i64` to `i32`
 --> main.yb:5:17
  |
5 |   big2: i32 = big
  |               ^^^ value has type `i64`
  |
note: narrowing may lose data; use an explicit conversion
help: `big2: i32 = i32(big)` — this may truncate the value
```

The widening rules follow a natural hierarchy:

- `i8` → `i16` → `i32` → `i64`
- `u8` → `u16` → `u32` → `u64`
- `f32` → `f64`
- `i32` → `f64` (integer to float, when no precision is lost)

Signed-to-unsigned conversions always require an explicit cast because the
semantic meaning changes (negative values become large positive values).

## 4.3 Type Inference

VibeLang's type inference works **locally** within function bodies. The
compiler examines the right-hand side of a binding and determines the type
without you writing it.

### How Inference Works

The compiler uses a constraint-based inference algorithm. When you write:

```vibe
x := 42
```

The compiler sees the literal `42`, determines it is an `Int`, and assigns
type `Int` to `x`. When you write:

```vibe
y := if condition { 1 } else { 2 }
```

The compiler infers that both branches produce `Int`, so `y` is `Int`.

Inference flows forward through expressions:

```vibe
pub example() -> Int {
  a := 10              // Int
  b := 20              // Int
  c := a + b           // Int (because Int + Int = Int)
  d := c > 15          // Bool (because Int > Int = Bool)
  c
}
```

### When Explicit Types Are Required

1. **Public function parameters and return types** — always:

```vibe
// Required: explicit types on public API
pub calculate_tax(income: Float, rate: Float) -> Float {
  income * rate
}
```

2. **Empty collections** — the compiler cannot infer the element type:

```vibe
// Ambiguous: what type of list?
// items := []    // error: cannot infer element type

items: List<Int> := []    // OK: explicitly typed
```

3. **Numeric ambiguity** — when a literal could be multiple types:

```vibe
// If a function accepts i32, the literal needs guidance
send_packet(42i32)    // suffix disambiguates
```

### Inference Does Not Cross Function Boundaries

VibeLang deliberately limits inference to within a single function. The
compiler never infers the type of a public function's parameters or return
from how the function is called:

```vibe
// The compiler will NOT look at call sites to infer types
// pub add(a, b) { a + b }    // error: parameters need types

pub add(a: Int, b: Int) -> Int {
  a + b    // return type could be inferred for private functions
}
```

This boundary is intentional. If inference crossed function boundaries, changing
how you call a function could change its type signature, which could silently
break other callers. By requiring explicit types at boundaries, VibeLang
ensures that each function's contract is self-contained and stable.

## 4.4 Functions

Functions are the primary unit of abstraction in VibeLang. They are also the
unit to which contracts attach — every `@intent`, `@require`, `@ensure`, and
`@effect` annotation belongs to a function.

### Declaration Syntax

A function declaration has this shape:

```
[pub] name(param1: Type1, param2: Type2) -> ReturnType {
  body
}
```

The simplest function:

```vibe
pub identity(x: Int) -> Int {
  x
}
```

### Parameters

Parameters are always immutable within the function body. You cannot reassign
a parameter:

```vibe
pub double(x: Int) -> Int {
  // x = x * 2    // error: cannot assign to function parameter `x`
  x * 2
}
```

```
error[E0420]: cannot assign to function parameter `x`
 --> math.yb:3:3
  |
1 | pub double(x: Int) -> Int {
  |            - parameter declared here
3 |   x = x * 2
  |   ^^^^^^^^^ parameters are immutable
  |
help: bind a new local variable: `result := x * 2`
```

If you need a modified copy, bind a new variable:

```vibe
pub process(input: Str) -> Str {
  cleaned := input.trim()
  cleaned
}
```

### Return Types

Every public function must declare its return type. Private functions can omit
the return type and let the compiler infer it, but explicit return types are
recommended for clarity:

```vibe
// Public: return type required
pub area(width: Float, height: Float) -> Float {
  width * height
}

// Private: return type optional but recommended
format_name(first: Str, last: Str) -> Str {
  first + " " + last
}
```

### Public vs Private

The `pub` keyword controls visibility:

- `pub` functions are accessible from other modules.
- Functions without `pub` are private to the current module.

```vibe
module geometry

// Other modules can call this
pub circle_area(radius: Float) -> Float {
  PI * radius * radius
}

// Only this module can call this
const PI: Float = 3.14159265358979

validate_radius(r: Float) -> Bool {
  r > 0.0
}
```

The visibility rule is simple: if it's part of your module's public API, mark
it `pub`. Everything else stays private. This creates a clear boundary between
interface and implementation.

### Multiple Parameters

Functions can take any number of parameters:

```vibe
pub clamp(value: Int, low: Int, high: Int) -> Int {
  if value < low {
    low
  } else if value > high {
    high
  } else {
    value
  }
}
```

### Functions as the Unit of Contracts

In VibeLang, contracts are attached to functions, not to arbitrary code blocks.
This is a preview of Chapter 6, but it's important to understand the connection
now:

```vibe
pub divide(numerator: Float, denominator: Float) -> Float {
  @intent "safely divide two numbers, returning 0 for division by zero"
  @require denominator != 0.0
  @ensure . >= 0.0 || . < 0.0    // result is a valid float

  numerator / denominator
}
```

Every annotation — `@intent`, `@require`, `@ensure`, `@effect` — belongs to
the function it appears in. Functions are the natural boundary for specifying
behavior, preconditions, postconditions, and effects.

## 4.5 The Return Value

VibeLang offers two ways to return a value from a function: tail expressions
and the `return` keyword.

### Tail Expressions

The last expression in a function body is its return value. No keyword needed:

```vibe
pub add(a: Int, b: Int) -> Int {
  a + b
}

pub greeting(name: Str) -> Str {
  "Welcome, " + name + "!"
}

pub classify(score: Int) -> Str {
  if score >= 90 {
    "excellent"
  } else if score >= 70 {
    "good"
  } else {
    "needs improvement"
  }
}
```

Tail expressions are the idiomatic way to return values. They make the data
flow explicit: the value of the function is the value of its body expression.

### Explicit `return`

The `return` keyword exits the function immediately with a value:

```vibe
pub find_index(items: List<Int>, target: Int) -> Int {
  mut i := 0
  for item in items {
    if item == target {
      return i    // found it — exit early
    }
    i = i + 1
  }
  -1    // not found — tail expression for default
}
```

### When to Use Each

Use **tail expressions** for the normal return path — the value the function
produces when everything goes as expected.

Use **`return`** for early exits — guard clauses, error conditions, or search
results found before the end of a loop.

```vibe
pub process_order(order: Order) -> Result<Receipt, Str> {
  // Guard clauses with early return
  if order.items.len() == 0 {
    return Err("empty order")
  }
  if order.total < 0 {
    return Err("negative total")
  }

  // Normal path with tail expression
  receipt := generate_receipt(order)
  Ok(receipt)
}
```

Mixing both in one function is fine and common. The key is consistency: early
exits use `return`, the main result uses a tail expression.

### Why VibeLang Prefers Tail Expressions

Tail expressions align with VibeLang's expression-oriented design. When a
function body is a single expression (possibly a complex one with `if/else` or
`match`), the function's purpose is immediately clear: it transforms inputs
into an output.

Compare:

```vibe
// Tail expression: the function IS this expression
pub max(a: Int, b: Int) -> Int {
  if a >= b { a } else { b }
}

// Explicit return: works, but adds noise for simple cases
pub max(a: Int, b: Int) -> Int {
  if a >= b {
    return a
  }
  return b
}
```

The first version reads as a definition: "max of a and b is a if a >= b,
otherwise b." The second reads as a procedure: "check if a >= b, if so return
a, otherwise return b." Both are correct, but the first is more declarative.

## 4.6 Function Composition

Real programs are built by composing small functions into larger behaviors.

### Calling Functions from Functions

```vibe
pub celsius_to_fahrenheit(c: Float) -> Float {
  c * 1.8 + 32.0
}

pub is_boiling(celsius: Float) -> Bool {
  celsius_to_fahrenheit(celsius) >= 212.0
}

pub water_state(celsius: Float) -> Str {
  if celsius <= 0.0 {
    "solid"
  } else if is_boiling(celsius) {
    "gas"
  } else {
    "liquid"
  }
}
```

Each function is small, focused, and testable. `water_state` builds on
`is_boiling`, which builds on `celsius_to_fahrenheit`.

### Building Abstractions

Functions let you name and reuse computations:

```vibe
pub distance(x1: Float, y1: Float, x2: Float, y2: Float) -> Float {
  dx := x2 - x1
  dy := y2 - y1
  sqrt(dx * dx + dy * dy)
}

pub is_nearby(x1: Float, y1: Float, x2: Float, y2: Float, threshold: Float) -> Bool {
  distance(x1, y1, x2, y2) < threshold
}

pub find_nearest(
  points: List<Point>,
  origin_x: Float,
  origin_y: Float
) -> Point {
  @require points.len() > 0

  mut nearest := points.get(0)
  mut best_dist := distance(origin_x, origin_y, nearest.x, nearest.y)

  for p in points {
    d := distance(origin_x, origin_y, p.x, p.y)
    if d < best_dist {
      nearest = p
      best_dist = d
    }
  }
  nearest
}
```

### Effect Propagation Through Call Chains

When a function calls another function that has effects, those effects
propagate upward. If `read_file` has `@effect io`, then any function that
calls `read_file` must also declare `@effect io`:

```vibe
pub read_config(path: Str) -> Str {
  @effect io
  read_file(path)
}

pub load_settings() -> Settings {
  @effect io, alloc
  raw := read_config("/etc/app.conf")
  parse_settings(raw)
}
```

This propagation is checked by the compiler. If you call an `io` function
without declaring `@effect io`, you get an error:

```
error[E0430]: undeclared effect `io`
 --> config.yb:8:10
  |
7 | pub load_settings() -> Settings {
  |     ------------- this function does not declare effect `io`
8 |   raw := read_config("/etc/app.conf")
  |          ^^^^^^^^^^^ `read_config` requires effect `io`
  |
help: add `@effect io` to `load_settings`
```

Effect propagation is covered in depth in Chapter 7. The key point here is
that functions are the boundary where effects are declared and checked.

## 4.7 Type Declarations

VibeLang lets you define custom types to model your domain.

### Struct Types

A struct groups related data under a single name:

```vibe
type Point {
  x: Float
  y: Float
}

type User {
  name: Str
  email: Str
  age: Int
  active: Bool
}
```

### Creating Instances

Create a struct instance by providing values for all fields:

```vibe
pub main() -> Int {
  @effect io

  origin := Point { x: 0.0, y: 0.0 }
  user := User {
    name: "Alice",
    email: "alice@example.com",
    age: 30,
    active: true
  }

  println(user.name)    // prints: Alice
  println(origin.x)     // prints: 0.0
  0
}
```

### Accessing Fields

Use dot notation to access fields:

```vibe
pub full_name(user: User) -> Str {
  user.first_name + " " + user.last_name
}

pub distance_from_origin(p: Point) -> Float {
  sqrt(p.x * p.x + p.y * p.y)
}
```

### Type Visibility

Types follow the same visibility rules as functions:

```vibe
// Public type: accessible from other modules
pub type ApiResponse {
  status: Int
  body: Str
}

// Private type: only used within this module
type InternalState {
  buffer: List<u8>
  position: usize
}
```

## 4.8 Enums and Match

Enums define a type that can be one of several variants. Combined with pattern
matching, they are one of VibeLang's most powerful features.

### Enum Declarations

```vibe
pub type Direction {
  North
  South
  East
  West
}

pub type Color {
  Red
  Green
  Blue
  Custom(r: u8, g: u8, b: u8)
}

pub type Shape {
  Circle(radius: Float)
  Rectangle(width: Float, height: Float)
  Triangle(base: Float, height: Float)
}
```

Enum variants can be simple labels (`North`, `Red`) or carry data
(`Circle(radius: Float)`).

### Pattern Matching with `match`

The `match` expression examines a value and executes the branch whose pattern
matches:

```vibe
pub describe_direction(dir: Direction) -> Str {
  match dir {
    case North => "heading north"
    case South => "heading south"
    case East => "heading east"
    case West => "heading west"
  }
}
```

For variants with data, the match can bind the inner values:

```vibe
pub area(shape: Shape) -> Float {
  match shape {
    case Circle(r) => 3.14159 * r * r
    case Rectangle(w, h) => w * h
    case Triangle(b, h) => 0.5 * b * h
  }
}
```

### Exhaustiveness Checking

The compiler ensures that every `match` covers all possible variants. If you
forget one, the compiler tells you:

```vibe
pub to_string(dir: Direction) -> Str {
  match dir {
    case North => "N"
    case South => "S"
    case East => "E"
    // missing: West
  }
}
```

```
error[E0440]: non-exhaustive match
 --> nav.yb:2:3
  |
2 |   match dir {
  |   ^^^^^^^^^ pattern `West` not covered
  |
help: add the missing case:
  |     case West => /* ... */
  |
note: match expressions must be exhaustive — all variants of
      `Direction` must be handled
```

Exhaustiveness checking is one of the most valuable features of enums with
match. When you add a new variant to an enum, the compiler finds every match
expression that needs updating. This makes refactoring safe: you cannot
accidentally forget to handle a new case.

### The `default` Case

When you don't need to handle every variant individually, use `default`:

```vibe
pub is_primary(color: Color) -> Bool {
  match color {
    case Red => true
    case Green => true
    case Blue => true
    default => false
  }
}
```

`default` matches anything not covered by previous cases. Use it sparingly —
explicit cases are usually better because they benefit from exhaustiveness
checking. If you add a new variant, a `default` case will silently swallow it,
while explicit cases would produce a compiler error prompting you to decide.

### Match as Expression

Like `if/else`, `match` is an expression. You can bind its result:

```vibe
pub main() -> Int {
  @effect io

  shape := Circle(5.0)

  description := match shape {
    case Circle(r) => "circle with radius " + to_str(r)
    case Rectangle(w, h) => to_str(w) + "x" + to_str(h) + " rectangle"
    case Triangle(b, h) => "triangle (base=" + to_str(b) + ")"
  }

  println(description)
  0
}
```

All branches must produce the same type, just like `if/else` branches.

### Nested Patterns

Patterns can be nested to match complex structures:

```vibe
pub type Expr {
  Num(value: Int)
  Add(left: Expr, right: Expr)
  Mul(left: Expr, right: Expr)
}

pub eval(expr: Expr) -> Int {
  match expr {
    case Num(v) => v
    case Add(l, r) => eval(l) + eval(r)
    case Mul(l, r) => eval(l) * eval(r)
  }
}
```

This is a classic use of enums and pattern matching: defining a recursive data
structure (an expression tree) and processing it with a recursive function.
The compiler guarantees you handle every variant, and the pattern matching
binds the inner data for you.

## 4.9 Putting It Together

Here is a complete program that uses types, functions, enums, and match:

```vibe
//! A simple shape calculator.

module shapes

import std.io

const PI: Float = 3.14159265358979

pub type Shape {
  Circle(radius: Float)
  Rectangle(width: Float, height: Float)
  Triangle(base: Float, height: Float)
}

pub area(shape: Shape) -> Float {
  @intent "compute the area of a geometric shape"
  @ensure . >= 0.0

  match shape {
    case Circle(r) => PI * r * r
    case Rectangle(w, h) => w * h
    case Triangle(b, h) => 0.5 * b * h
  }
}

pub perimeter(shape: Shape) -> Float {
  @intent "compute the perimeter of a geometric shape"
  @ensure . >= 0.0

  match shape {
    case Circle(r) => 2.0 * PI * r
    case Rectangle(w, h) => 2.0 * (w + h)
    case Triangle(b, h) => {
      hypotenuse := sqrt(b * b + h * h)
      b + h + hypotenuse
    }
  }
}

pub describe(shape: Shape) -> Str {
  name := match shape {
    case Circle(_) => "Circle"
    case Rectangle(_, _) => "Rectangle"
    case Triangle(_, _) => "Triangle"
  }

  name + ": area=" + to_str(area(shape)) +
    ", perimeter=" + to_str(perimeter(shape))
}

pub main() -> Int {
  @effect io

  shapes := [
    Circle(5.0),
    Rectangle(4.0, 6.0),
    Triangle(3.0, 4.0)
  ]

  for s in shapes {
    println(describe(s))
  }

  0
}
```

This program demonstrates:

- Custom type declarations with `type`
- Enum variants carrying data
- Pattern matching in `match` expressions
- Functions composing other functions (`describe` calls `area` and `perimeter`)
- Constants at module level
- Contract annotations on functions
- Expression-oriented style throughout

## 4.10 Summary

This chapter covered VibeLang's type system and function model:

- **Static typing with local inference**: types are checked at compile time,
  inferred within functions, and explicit at public boundaries.
- **Primitive types**: `Int` (i64), `Float` (f64), `Bool`, `Str`, plus sized
  variants for precise control.
- **Numeric literals** support decimal, hex, octal, binary, scientific
  notation, and type suffixes.
- **Type widening** is implicit (safe); **narrowing** requires explicit
  conversion.
- **Functions** are declared with parameter types and return types. `pub`
  controls visibility. Parameters are immutable.
- **Tail expressions** are the idiomatic return mechanism; `return` is for
  early exits.
- **Struct types** group related fields. **Enum types** define variants.
- **Pattern matching** with `match` destructures enums and is checked for
  exhaustiveness by the compiler.
- **Functions are the unit of contracts**: `@intent`, `@require`, `@ensure`,
  and `@effect` attach to functions.

These building blocks — types, functions, structs, enums, and match — are the
vocabulary you will use to express every VibeLang program. The chapters ahead
build on this foundation with control flow, contracts, effects, and
concurrency.

---

Next: Chapter 5 covers control flow in depth.
