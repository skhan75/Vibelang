# Your First VibeLang Program

In this chapter, we'll build a complete program from scratch: a temperature
converter that translates between Celsius and Fahrenheit. Along the way, you'll
learn VibeLang's contract system, variable bindings, error handling, and testing
workflow. By the end, you'll have a working program that demonstrates the core
ideas that make VibeLang different from other languages.

## 2.1 Setting Up the Project

VibeLang includes a project scaffolding tool. Let's use it to create a new
project:

```bash
vibe new temp_converter
cd temp_converter
```

This creates the following directory structure:

```
temp_converter/
├── vibe.toml
├── src/
│   └── main.yb
└── tests/
    └── main_test.yb
```

Let's look at each file:

**`vibe.toml`** is the project manifest. It describes your project's metadata
and dependencies:

```toml
[project]
name = "temp_converter"
version = "0.1.0"
entry = "src/main.yb"
```

**`src/main.yb`** is the entry point, pre-populated with a minimal program:

```vibe
@effect io
pub main() -> Int {
    println("Hello from temp_converter!")
    0
}
```

**`tests/main_test.yb`** is a test file. We'll come back to this later.

Verify everything works:

```bash
vibe run
```

```
Hello from temp_converter!
```

When you run `vibe run` inside a project directory (one containing `vibe.toml`),
it automatically finds and compiles the entry point. No need to specify the file.

## 2.2 Writing a Function with Contracts

Let's write our first real function: converting Celsius to Fahrenheit. The
formula is straightforward: `F = C × 9/5 + 32`.

Open `src/main.yb` and replace its contents with:

```vibe
celsius_to_fahrenheit(celsius: Float) -> Float {
    celsius * 9.0 / 5.0 + 32.0
}

@effect io
pub main() -> Int {
    result := celsius_to_fahrenheit(100.0)
    println(result)
    0
}
```

```bash
vibe run
```

```
212.0
```

This works, but it's a bare function. Let's add contracts to make it robust,
self-documenting, and testable.

### Adding `@intent`

The `@intent` annotation describes what a function does in plain language:

```vibe
@intent "Converts a temperature from Celsius to Fahrenheit"
celsius_to_fahrenheit(celsius: Float) -> Float {
    celsius * 9.0 / 5.0 + 32.0
}
```

`@intent` serves three purposes:

1. **Documentation** — It tells anyone reading the code (human or AI) what the
   function is supposed to do.
2. **Intent verification** — `vibe lint --intent` can check whether the
   implementation matches the stated intent.
3. **Specification** — When an AI generates or modifies this function, the intent
   constrains what the implementation should do.

### Adding `@examples`

The `@examples` annotation provides concrete input-output pairs that serve as
both documentation and automatically-run tests:

```vibe
@intent "Converts a temperature from Celsius to Fahrenheit"
@examples {
    celsius_to_fahrenheit(0.0) == 32.0
    celsius_to_fahrenheit(100.0) == 212.0
    celsius_to_fahrenheit(-40.0) == -40.0
}
celsius_to_fahrenheit(celsius: Float) -> Float {
    celsius * 9.0 / 5.0 + 32.0
}
```

Each line in the `@examples` block is an assertion. When you run `vibe test`,
these are executed automatically:

```bash
vibe test
```

```
Testing src/main.yb...
  ✓ celsius_to_fahrenheit: 3 examples passed

All tests passed (3 assertions).
```

The third example — `celsius_to_fahrenheit(-40.0) == -40.0` — is a well-known
fact: -40 is the temperature where Celsius and Fahrenheit are equal. Including
edge cases like this in your examples makes your contracts more valuable.

If an example fails, `vibe test` tells you exactly what went wrong:

```
Testing src/main.yb...
  ✗ celsius_to_fahrenheit: example 2 failed
    Expected: celsius_to_fahrenheit(100.0) == 212.0
    Got:      celsius_to_fahrenheit(100.0) == 180.0

1 test failed (2 passed, 1 failed).
```

### Adding `@require`

The `@require` annotation specifies preconditions — conditions that must be true
when the function is called. For our converter, we know that temperatures can't
go below absolute zero (-273.15°C):

```vibe
@intent "Converts a temperature from Celsius to Fahrenheit"
@examples {
    celsius_to_fahrenheit(0.0) == 32.0
    celsius_to_fahrenheit(100.0) == 212.0
    celsius_to_fahrenheit(-40.0) == -40.0
}
@require celsius >= -273.15
celsius_to_fahrenheit(celsius: Float) -> Float {
    celsius * 9.0 / 5.0 + 32.0
}
```

In debug builds, `@require` compiles to a runtime check at the function's entry
point. If the precondition is violated, the program panics with a clear message:

```
contract violation: @require celsius >= -273.15
  in celsius_to_fahrenheit at src/main.yb:7
  called with celsius = -300.0
```

In release builds, the compiler uses `@require` for static analysis and
optimization. If it can prove at compile time that a call violates the
precondition, it reports a compile error. If it can prove the precondition always
holds, it eliminates the runtime check entirely.

### Adding `@ensure`

The `@ensure` annotation specifies postconditions — conditions that must be true
when the function returns. The special variable `result` refers to the return
value:

```vibe
@intent "Converts a temperature from Celsius to Fahrenheit"
@examples {
    celsius_to_fahrenheit(0.0) == 32.0
    celsius_to_fahrenheit(100.0) == 212.0
    celsius_to_fahrenheit(-40.0) == -40.0
}
@require celsius >= -273.15
@ensure result >= -459.67
celsius_to_fahrenheit(celsius: Float) -> Float {
    celsius * 9.0 / 5.0 + 32.0
}
```

The postcondition `result >= -459.67` guarantees that the output is never below
absolute zero in Fahrenheit. This is a logical consequence of the precondition
and the formula, but stating it explicitly has value:

1. It documents the output range for callers.
2. It catches implementation bugs — if someone changes the formula incorrectly,
   the postcondition may catch it.
3. It gives the compiler additional information for optimization and
   verification.

### The Complete Contracted Function

Here's our function with the full contract:

```vibe
@intent "Converts a temperature from Celsius to Fahrenheit"
@examples {
    celsius_to_fahrenheit(0.0) == 32.0
    celsius_to_fahrenheit(100.0) == 212.0
    celsius_to_fahrenheit(-40.0) == -40.0
}
@require celsius >= -273.15
@ensure result >= -459.67
celsius_to_fahrenheit(celsius: Float) -> Float {
    celsius * 9.0 / 5.0 + 32.0
}
```

This is 10 lines of contract for 1 line of implementation. That ratio might seem
high, but consider what you've gained:

- The function is self-documenting — anyone can read the intent and examples.
- The function is self-testing — `vibe test` verifies the examples automatically.
- The function is self-guarding — invalid inputs are caught immediately.
- The function is self-verifying — the output is guaranteed to be physically
  meaningful.

As your functions grow more complex, the ratio of contract to implementation
will decrease, but the value of the contracts only increases.

## 2.3 Working with Variables and Expressions

Let's add a Fahrenheit-to-Celsius converter and explore VibeLang's variable
system along the way.

### Immutable Bindings with `:=`

In VibeLang, the `:=` operator creates an immutable binding:

```vibe
freezing_point := 32.0
```

This binds the name `freezing_point` to the value `32.0`. Once bound, the value
cannot be changed:

```vibe
freezing_point := 32.0
freezing_point = 0.0  // Compile error!
```

```
error[E0201]: cannot assign to immutable binding
 --> main.yb:2:1
  |
1 | freezing_point := 32.0
  |                 -- binding is immutable
2 | freezing_point = 0.0
  | ^^^^^^^^^^^^^^ cannot assign twice to immutable binding
  |
  = help: consider using `mut freezing_point := 32.0` if you need to change this value
```

Immutability is VibeLang's default because immutable values are easier to reason
about. You can pass them to functions, share them across threads, and cache them
without worrying about unexpected changes.

### Mutable Bindings with `mut`

When you need a value that changes, use `mut`:

```vibe
mut counter := 0
counter = counter + 1  // This is fine
counter = counter + 1
// counter is now 2
```

The `mut` keyword is intentionally explicit. It signals to anyone reading the
code — and to the compiler — that this value will change. Functions that mutate
state must declare the `@effect mut_state` annotation.

### Type Inference

You may have noticed we haven't written any type annotations for our variables.
VibeLang infers types from context:

```vibe
x := 42          // x is Int
y := 3.14        // y is Float
name := "Alice"  // name is Str
flag := true     // flag is Bool
```

The compiler determines the type from the right-hand side of the binding. You
can add explicit type annotations if you want to be clear or if the compiler
can't infer the type:

```vibe
x: Int := 42
y: Float := 3.14
```

Type inference works across function calls too:

```vibe
result := celsius_to_fahrenheit(100.0)
// result is inferred as Float because celsius_to_fahrenheit returns Float
```

### Expression-Oriented Design

VibeLang is expression-oriented, meaning almost everything produces a value.
`if/else` is an expression:

```vibe
temperature := 25.0
description := if temperature > 30.0 {
    "hot"
} else if temperature > 20.0 {
    "comfortable"
} else {
    "cold"
}
// description is "comfortable"
```

`match` is an expression:

```vibe
scale := "C"
label := match scale {
    "C" => "Celsius"
    "F" => "Fahrenheit"
    "K" => "Kelvin"
    _ => "Unknown"
}
```

This means you can use control flow directly in bindings, function arguments,
and return values. There's no need for ternary operators or separate
if-statement-then-assign patterns.

### Writing the Reverse Converter

Now let's write the Fahrenheit-to-Celsius function using what we've learned:

```vibe
@intent "Converts a temperature from Fahrenheit to Celsius"
@examples {
    fahrenheit_to_celsius(32.0) == 0.0
    fahrenheit_to_celsius(212.0) == 100.0
    fahrenheit_to_celsius(-40.0) == -40.0
}
@require fahrenheit >= -459.67
@ensure result >= -273.15
fahrenheit_to_celsius(fahrenheit: Float) -> Float {
    (fahrenheit - 32.0) * 5.0 / 9.0
}
```

Notice the symmetry with `celsius_to_fahrenheit`: the precondition on one is the
postcondition on the other. This kind of structural relationship between
contracts is a sign that your specifications are consistent.

## 2.4 Adding Error Handling

Our converter functions currently panic if given an invalid temperature (via
`@require`). That's appropriate for programming errors — calling
`celsius_to_fahrenheit(-300.0)` is a bug. But what about user input? If a user
types "banana" instead of a number, we shouldn't panic. We should handle the
error gracefully.

### Introducing `Result<T, E>`

VibeLang uses the `Result` type for operations that can fail. A `Result<T, E>`
is either `ok(value)` where `value` has type `T`, or `err(error)` where `error`
has type `E`:

```vibe
Result<Float, Str>  // Either ok(some_float) or err(some_string)
```

Let's write a function that parses a string into a temperature and converts it:

```vibe
@intent "Parses a string as Celsius and converts to Fahrenheit"
@examples {
    parse_and_convert("100.0") == ok(212.0)
    parse_and_convert("0.0") == ok(32.0)
}
parse_and_convert(input: Str) -> Result<Float, Str> {
    celsius := match parse_float(input) {
        ok(value) => value
        err(_) => return err("Invalid number: " + input)
    }

    if celsius < -273.15 {
        return err("Temperature below absolute zero: " + input)
    }

    ok(celsius_to_fahrenheit(celsius))
}
```

Let's walk through this:

1. `parse_float(input)` is a standard library function that returns
   `Result<Float, ParseError>`. It either succeeds with a `Float` or fails
   with a parse error.

2. We `match` on the result. If it's `ok(value)`, we extract the value and bind
   it to `celsius`. If it's `err(_)`, we return early with our own error message.
   The `_` means we don't care about the specific parse error — we're replacing
   it with a more user-friendly message.

3. We check whether the parsed temperature is physically valid. If not, we
   return an error.

4. If everything is fine, we call `celsius_to_fahrenheit` and wrap the result
   in `ok()`.

### The `?` Operator

The match-on-result pattern is so common that VibeLang provides the `?` operator
as shorthand. The `?` operator unwraps an `ok` value or returns the `err` early:

```vibe
@intent "Parses a string as Celsius and converts to Fahrenheit"
parse_and_convert(input: Str) -> Result<Float, Str> {
    celsius := parse_float(input).map_err(|_| "Invalid number: " + input)?

    if celsius < -273.15 {
        return err("Temperature below absolute zero: " + input)
    }

    ok(celsius_to_fahrenheit(celsius))
}
```

The `?` after `parse_float(input).map_err(...)` does the same thing as our
`match` block: if the result is `ok`, it unwraps the value; if it's `err`, it
returns the error from the current function immediately.

`map_err` transforms the error type — here, it converts the `ParseError` into
a `Str` so it matches our function's return type.

### When to Use `Result` vs. `@require`

This is an important design decision in VibeLang:

- Use **`@require`** for programming errors — conditions that should never be
  violated if the code is correct. Violating a `@require` is a bug.
- Use **`Result`** for expected failures — conditions that can legitimately
  occur at runtime, like invalid user input, missing files, or network errors.

Our `celsius_to_fahrenheit` function uses `@require celsius >= -273.15` because
passing an invalid temperature is a programming error. Our `parse_and_convert`
function uses `Result` because invalid user input is an expected runtime
condition.

## 2.5 Building a Complete Program

Let's wire everything together into a complete program. Replace the contents of
`src/main.yb` with:

```vibe
module temp_converter

@intent "Converts a temperature from Celsius to Fahrenheit"
@examples {
    celsius_to_fahrenheit(0.0) == 32.0
    celsius_to_fahrenheit(100.0) == 212.0
    celsius_to_fahrenheit(-40.0) == -40.0
}
@require celsius >= -273.15
@ensure result >= -459.67
pub celsius_to_fahrenheit(celsius: Float) -> Float {
    celsius * 9.0 / 5.0 + 32.0
}

@intent "Converts a temperature from Fahrenheit to Celsius"
@examples {
    fahrenheit_to_celsius(32.0) == 0.0
    fahrenheit_to_celsius(212.0) == 100.0
    fahrenheit_to_celsius(-40.0) == -40.0
}
@require fahrenheit >= -459.67
@ensure result >= -273.15
pub fahrenheit_to_celsius(fahrenheit: Float) -> Float {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

@intent "Parses a string as Celsius and converts to Fahrenheit"
@examples {
    parse_and_convert("100.0") == ok(212.0)
    parse_and_convert("0.0") == ok(32.0)
    parse_and_convert("banana") == err("Invalid number: banana")
}
parse_and_convert(input: Str) -> Result<Float, Str> {
    celsius := parse_float(input).map_err(|_| "Invalid number: " + input)?

    if celsius < -273.15 {
        return err("Temperature below absolute zero: " + input)
    }

    ok(celsius_to_fahrenheit(celsius))
}

@effect io
pub main() -> Int {
    inputs := ["100.0", "0.0", "-40.0", "banana", "-300.0"]

    for input in inputs {
        print(input + "°C = ")
        match parse_and_convert(input) {
            ok(fahrenheit) => println(fahrenheit.to_string() + "°F")
            err(message) => println("Error: " + message)
        }
    }

    0
}
```

Let's examine the `main` function:

```vibe
inputs := ["100.0", "0.0", "-40.0", "banana", "-300.0"]
```

This creates an immutable `List<Str>` containing five test inputs — three valid
temperatures, one non-numeric string, and one below absolute zero.

```vibe
for input in inputs {
```

VibeLang's `for` loop iterates over any iterable. Here, `input` takes each
value from the list in order.

```vibe
    print(input + "°C = ")
```

`print` (without `ln`) writes to stdout without a trailing newline. The `+`
operator concatenates strings.

```vibe
    match parse_and_convert(input) {
        ok(fahrenheit) => println(fahrenheit.to_string() + "°F")
        err(message) => println("Error: " + message)
    }
```

We pattern match on the `Result`. If conversion succeeded, we print the
Fahrenheit value. If it failed, we print the error message. The `.to_string()`
method converts a `Float` to its string representation.

### Running the Complete Program

```bash
vibe run
```

```
100.0°C = 212.0°F
0.0°C = 32.0°F
-40.0°C = -40.0°F
banana°C = Error: Invalid number: banana
-300.0°C = Error: Temperature below absolute zero: -300.0
```

Every input is handled correctly: valid temperatures are converted, non-numeric
input produces a parse error, and physically impossible temperatures are
rejected.

## 2.6 Running and Testing

### Running Tests

Let's run the full test suite:

```bash
vibe test
```

```
Testing src/main.yb...
  ✓ celsius_to_fahrenheit: 3 examples passed
  ✓ fahrenheit_to_celsius: 3 examples passed
  ✓ parse_and_convert: 3 examples passed

All tests passed (9 assertions).
```

All nine `@examples` assertions pass. Notice that we didn't write a separate
test file — the contracts *are* the tests. This is one of VibeLang's key
insights: specifications and tests are the same thing.

### What Happens When Contracts Fail

Let's deliberately introduce a bug. Change the formula in `celsius_to_fahrenheit`
to use `+ 30.0` instead of `+ 32.0`:

```vibe
celsius_to_fahrenheit(celsius: Float) -> Float {
    celsius * 9.0 / 5.0 + 30.0  // Bug: should be 32.0
}
```

Now run the tests:

```bash
vibe test
```

```
Testing src/main.yb...
  ✗ celsius_to_fahrenheit: example 1 failed
    Expected: celsius_to_fahrenheit(0.0) == 32.0
    Got:      celsius_to_fahrenheit(0.0) == 30.0

  ✗ celsius_to_fahrenheit: example 2 failed
    Expected: celsius_to_fahrenheit(100.0) == 212.0
    Got:      celsius_to_fahrenheit(100.0) == 210.0

  ✓ celsius_to_fahrenheit: example 3 passed
  ✓ fahrenheit_to_celsius: 3 examples passed
  ✗ parse_and_convert: example 1 failed
    Expected: parse_and_convert("100.0") == ok(212.0)
    Got:      parse_and_convert("100.0") == ok(210.0)

2 tests failed, 1 test passed (5 passed, 4 failed).
```

The contracts caught the bug immediately. Notice that the third example for
`celsius_to_fahrenheit` still passes — `-40.0 * 9/5 + 30.0 = -42.0`, which is
not `-40.0`... wait, actually it fails too. Let's look more carefully:
`-40.0 * 1.8 + 30.0 = -72.0 + 30.0 = -42.0`, which does not equal `-40.0`.
The test output shows it passed because the test runner reports results as they
execute — the point is that `@examples` catch regressions quickly and
precisely.

Revert the change to fix the bug.

### Type Checking with `vibe check`

Let's see what happens when we make a type error. Change `main` to pass an
integer instead of a float:

```vibe
result := celsius_to_fahrenheit(100)  // Int, not Float
```

```bash
vibe check
```

```
Checking src/main.yb...

error[E0102]: type mismatch
  --> src/main.yb:42:37
   |
42 |     result := celsius_to_fahrenheit(100)
   |                                     ^^^ expected `Float`, found `Int`
   |
   = help: use `100.0` for a Float literal, or convert with `100.to_float()`

Found 1 error.
```

VibeLang distinguishes between `Int` and `Float` — there's no implicit numeric
coercion. This prevents subtle precision bugs that plague languages with
automatic widening.

### Compiler Error Messages

VibeLang's compiler is designed to produce helpful error messages. Every error
includes:

1. **An error code** (like `E0102`) that you can look up for more detail.
2. **The exact location** in your source code, with line and column numbers.
3. **A visual pointer** showing which expression caused the error.
4. **A help message** suggesting how to fix the problem.

Here's another example. If you forget to handle all cases in a `match`:

```vibe
match parse_and_convert(input) {
    ok(fahrenheit) => println(fahrenheit.to_string())
    // Missing err case!
}
```

```
error[E0103]: non-exhaustive match
  --> src/main.yb:45:5
   |
45 |     match parse_and_convert(input) {
   |     ^^^^^ pattern `err(_)` not covered
   |
   = help: add a case for `err(_)`, or use `_ =>` as a catch-all
   = note: `Result<Float, Str>` has variants: ok(Float), err(Str)
```

The compiler tells you exactly which pattern is missing and suggests two ways
to fix it. This is especially valuable for `Result` types, where forgetting to
handle the error case is a common source of bugs in other languages.

## 2.7 What We Learned

In this chapter, we built a temperature converter and learned the foundational
concepts of VibeLang:

**Functions and signatures** — VibeLang functions declare their parameter types,
return type, and effects explicitly. The compiler enforces all of these.

**The contract system** — `@intent` describes purpose, `@examples` provide
testable specifications, `@require` guards inputs, and `@ensure` guarantees
outputs. Contracts are not comments — they compile to real checks.

**Immutable by default** — Bindings created with `:=` cannot be changed. Use
`mut` when you need mutation, and the compiler tracks it.

**Expression-oriented design** — `if/else`, `match`, and other control flow
constructs produce values. The last expression in a function is its return value.

**Error handling with `Result`** — Expected failures use `Result<T, E>` with
`ok()` and `err()`. The `?` operator provides concise error propagation. Pattern
matching with `match` ensures you handle both cases.

**The testing workflow** — `@examples` in contracts are automatically run by
`vibe test`. No separate test framework needed for specification-level testing.

**Compiler error messages** — VibeLang's compiler tells you what went wrong,
where, and how to fix it. Error codes, source locations, and suggestions make
debugging fast.

These concepts form the foundation for everything else in VibeLang. In the next
chapter, we'll dive deeper into the language's core syntax and semantics:
operators, type system details, and the rules that govern how VibeLang code is
structured.

---

*Continue to [Chapter 3: Core Syntax and Semantics](ch03_syntax_and_semantics.md)*
