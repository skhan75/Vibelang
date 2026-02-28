# Appendix B: Operators and Symbols

This appendix is a complete reference for every operator and special symbol in
VibeLang, including precedence rules, descriptions, and examples.

---

## B.1 Operator Precedence Table

Operators listed from highest precedence (binds tightest) to lowest. All binary
operators are left-associative unless noted.

| Precedence | Operator(s)            | Name                    | Associativity |
|:----------:|------------------------|-------------------------|:-------------:|
| 1          | `.`                    | Member access            | Left          |
| 2          | `()` `[]`              | Call / Index             | Left          |
| 3          | `?`                    | Error propagation        | Postfix       |
| 4          | `!`  `-` (unary)       | Logical NOT / Negation   | Prefix        |
| 5          | `*` `/` `%`            | Multiply / Divide / Mod  | Left          |
| 6          | `+` `-`                | Add / Subtract           | Left          |
| 7          | `..`                   | Range                    | None          |
| 8          | `<` `>` `<=` `>=`      | Comparison               | Left          |
| 9          | `==` `!=`              | Equality                 | Left          |
| 10         | `&&`                   | Logical AND              | Left          |
| 11         | `\|\|`                 | Logical OR               | Left          |
| 12         | `\|>`                  | Pipe                     | Left          |
| 13         | `:=` `=`               | Binding / Assignment     | Right         |

When in doubt, use parentheses.

---

## B.2 Arithmetic Operators

### `+` — Addition

Adds two numeric values. Also concatenates strings.

```vibe
sum := 3 + 4            // 7
greeting := "hi" + "!"  // "hi!"
```

### `-` — Subtraction / Negation

Binary subtraction or unary negation.

```vibe
diff := 10 - 3   // 7
neg := -42        // -42
```

### `*` — Multiplication

```vibe
area := width * height
```

### `/` — Division

Integer division truncates toward zero. Division by zero is a runtime error.

```vibe
half := 10 / 2      // 5
truncated := 7 / 3  // 2
```

### `%` — Remainder

Returns the remainder of integer division. Result sign matches the dividend.

```vibe
remainder := 17 % 5   // 2
is_even := n % 2 == 0
```

---

## B.3 Comparison Operators

All comparison operators return `Bool`.

### `==` / `!=` — Equality

```vibe
if status == 200 { println("OK") }
if name != "" { println(name) }
```

### `<` / `>` / `<=` / `>=` — Ordering

```vibe
if temperature < 0 { println("freezing") }
if score > high_score { high_score = score }
if retries <= MAX_RETRIES { attempt(request) }
if age >= 18 { println("eligible") }
```

---

## B.4 Logical Operators

Logical operators work exclusively on `Bool` values. `&&` and `||`
short-circuit.

### `&&` — Logical AND

Returns `true` only if both operands are `true`.

```vibe
if is_valid && is_authorized { proceed() }
```

### `||` — Logical OR

Returns `true` if either operand is `true`.

```vibe
if is_admin || is_owner { grant_access() }
```

### `!` — Logical NOT

Unary prefix. Returns the boolean inverse.

```vibe
if !found { println("not found") }
```

---

## B.5 Assignment Operators

### `:=` — Binding

Creates a new variable binding (immutable by default, mutable with `mut`).

```vibe
name := "VibeLang"
mut counter := 0
```

### `=` — Assignment

Reassigns a value to an existing mutable binding.

```vibe
mut x := 10
x = 20
```

Using `=` on an immutable binding is a compile-time error.

---

## B.6 Pipe Operator

### `|>` — Pipe

Passes the left expression as the first argument to the function on the right.
Enables readable left-to-right data transformation chains.

```vibe
result := raw_input
  |> trim()
  |> parse_int()
  |> validate()
```

Equivalent without pipe: `validate(parse_int(trim(raw_input)))`.

---

## B.7 Range Operator

### `..` — Range

Creates a half-open range (start inclusive, end exclusive). Used in `for` loops
and slice operations.

```vibe
for i in 0..5 {
  println(i.to_str())  // 0, 1, 2, 3, 4
}
```

---

## B.8 Error Propagation Operator

### `?` — Propagate

Postfix on `Result<T, E>`. If the result is an error, the function returns it
immediately. If success, the inner value is extracted.

```vibe
pub read_config(path: Str) -> Result<Config, Error> {
  @effect io
  text := fs.read_text(path)?
  json.parse(text)?
}
```

Only valid inside functions that return `Result<T, E>`.

---

## B.9 Special Symbols

### `->` — Return Type Arrow

Separates a function's parameter list from its return type.

```vibe
pub add(a: Int, b: Int) -> Int { a + b }
```

### `=>` — Match Arm

Separates a pattern from its body in `match` and `select` branches.

```vibe
match direction {
  "north" => move(0, 1)
  "south" => move(0, -1)
  _ => stay()
}
```

### `:` — Type Annotation

Separates a name from its type in declarations, parameters, and struct fields.

```vibe
pub type Config { host: Str, port: Int }
```

### `,` — Separator

Separates items in arguments, list literals, map entries, and type parameters.

```vibe
result := add(3, 4)
items := [1, 2, 3]
```

### `.` — Member Access

Accesses a field or method on a value.

```vibe
length := name.len()
```

### `@` — Annotation Prefix

Introduces a contract annotation: `@intent`, `@examples`, `@require`, `@ensure`,
or `@effect`.

```vibe
pub clamp(val: Int, lo: Int, hi: Int) -> Int {
  @intent "constrain val to [lo, hi]"
  @require lo <= hi
  @ensure . >= lo && . <= hi
  if val < lo { lo } else if val > hi { hi } else { val }
}
```

### `//` — Line Comment

Everything after `//` on a line is ignored by the compiler.

### `{ }` — Block Delimiters

Enclose function bodies, control flow branches, type definitions, and match arms.

### `( )` — Grouping / Call

Group sub-expressions or delimit function parameters and call arguments.

### `[ ]` — List / Index

Create list literals or index into a list.

```vibe
items := [10, 20, 30]
first := items[0]
```

---

## B.10 Annotation Keywords

| Annotation   | Purpose                                    | Chapter |
|--------------|--------------------------------------------|---------|
| `@intent`    | Natural-language description of behavior   | 6       |
| `@examples`  | Input/output examples for verification     | 6       |
| `@require`   | Precondition that callers must satisfy     | 6       |
| `@ensure`    | Postcondition the function guarantees      | 6       |
| `@effect`    | Side effect declaration                    | 7       |

---

## B.11 Quick Reference Card

| Symbol | Name                 | Example                    |
|--------|----------------------|----------------------------|
| `+`    | Add / Concatenate    | `a + b`                    |
| `-`    | Subtract / Negate    | `a - b`, `-x`             |
| `*`    | Multiply             | `a * b`                    |
| `/`    | Divide               | `a / b`                    |
| `%`    | Remainder            | `a % b`                    |
| `==`   | Equal                | `a == b`                   |
| `!=`   | Not equal            | `a != b`                   |
| `<`    | Less than            | `a < b`                    |
| `>`    | Greater than         | `a > b`                    |
| `<=`   | Less or equal        | `a <= b`                   |
| `>=`   | Greater or equal     | `a >= b`                   |
| `&&`   | Logical AND          | `a && b`                   |
| `\|\|` | Logical OR          | `a \|\| b`                 |
| `!`    | Logical NOT          | `!a`                       |
| `:=`   | Binding              | `x := 5`                   |
| `=`    | Assignment           | `x = 10`                   |
| `\|>`  | Pipe                 | `x \|> f()`                |
| `..`   | Range                | `0..10`                    |
| `?`    | Error propagation    | `result?`                  |
| `->`   | Return type          | `fn() -> Int`              |
| `=>`   | Match arm            | `x => body`                |
| `:`    | Type annotation      | `x: Int`                   |
| `.`    | Member access        | `obj.field`                |
| `@`    | Annotation prefix    | `@intent "..."`            |
| `//`   | Comment              | `// note`                  |
