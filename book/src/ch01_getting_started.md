# Getting Started

This chapter covers everything you need to go from zero to running your first
VibeLang program. We'll install the toolchain, write a "Hello, World!" program,
dissect every line of it, and explore the tools you'll use daily.

## 1.1 Installing VibeLang

VibeLang compiles ahead-of-time (AOT) to native machine code. The compiler is
written in Rust, so you'll need a Rust toolchain to build from source. If you
prefer a prebuilt binary, packaged installs are available for major platforms.

### Installing from Source

If you have Rust installed (via [rustup](https://rustup.rs/)), building VibeLang
from source is straightforward:

```bash
git clone https://github.com/skhan75/VibeLang.git
cd VibeLang
cargo build --release -p vibe_cli
```

This produces the `vibe` binary in `target/release/`. Add it to your PATH:

```bash
export PATH="$PWD/target/release:$PATH"
```

To make this permanent, add the export line to your shell profile (`~/.bashrc`,
`~/.zshrc`, or equivalent).

Verify the installation:

```bash
vibe --version
```

You should see output like:

```
vibe 1.1.0
```

### Packaged Install (Linux)

Prebuilt binaries are published for major platforms. For the canonical install
guide, follow the official documentation:

```bash
# See the latest install guide:
# https://www.thevibelang.org/documentation
```

If you are reading the book from a checkout of the repository, you can also use
the in-repo platform guides (including verification steps):

- `docs/install/linux.md`
- `docs/install/macos.md`
- `docs/install/windows.md`

### Packaged Install (macOS)

See the official documentation (and/or `docs/install/macos.md` when reading from
this repository checkout).

### Packaged Install (Windows)

See the official documentation (and/or `docs/install/windows.md` when reading
from this repository checkout).

### Editor Support

VibeLang has syntax highlighting and language server support for major editors.
Look for the "VibeLang" extension in your editor's marketplace. The language
server provides real-time type checking, contract validation, and effect
tracking as you type.

## 1.2 Hello, World!

Let's write the simplest possible VibeLang program. Create a new directory for
your project and a source file:

```bash
mkdir hello
cd hello
```

Create a file called `main.yb` with the following contents:

```vibe
@effect io
pub main() -> Int {
    println("Hello, World!")
    0
}
```

Now compile and run it:

```bash
vibe run main.yb
```

You should see:

```
Hello, World!
```

Congratulations — you've just run your first VibeLang program. Now let's
understand every single piece of it.

### Line-by-Line Breakdown

```vibe
@effect io
```

This is an **effect annotation**. It tells the compiler that the `main` function
performs I/O — specifically, it writes to standard output via `println`. VibeLang
requires you to declare every side effect a function performs. If you removed
this line, the compiler would refuse to compile the program:

```
error[E0301]: undeclared effect `io`
 --> main.yb:2:5
  |
2 |     println("Hello, World!")
  |     ^^^^^^^ this call performs `io`
  |
  = help: add `@effect io` before the function signature
```

This might seem strict, but it's one of VibeLang's most powerful features. By
looking at any function's annotations, you can immediately tell whether it reads
files, makes network requests, allocates memory, or mutates state. Pure functions
— those with no effect annotations — are guaranteed to produce the same output
for the same input, every time.

```vibe
pub main() -> Int {
```

This line declares the program's entry point. Let's break it down:

- **`pub`** — This function is public. In VibeLang, `pub` controls visibility
  across module boundaries. The `main` function must be `pub` because the
  runtime needs to call it from outside your module.

- **`main`** — The function name. VibeLang uses `main` as the entry point, just
  like C, Go, and Rust. Every executable VibeLang program must have exactly one
  `pub main` function.

- **`()`** — The parameter list. `main` takes no arguments. (In a later chapter,
  we'll see how to access command-line arguments through the standard library.)

- **`-> Int`** — The return type. `main` returns an `Int`, which is VibeLang's
  default integer type (a 64-bit signed integer, equivalent to `i64` in Rust or
  `int64` in Go). The return value becomes the program's exit code.

- **`{`** — Opens the function body.

```vibe
    println("Hello, World!")
```

This calls the built-in `println` function, which prints a string followed by
a newline to standard output. `println` is one of VibeLang's few built-in
functions — most functionality lives in the standard library.

Notice there's no semicolon at the end of this line. VibeLang is expression-
oriented, and statements are separated by newlines. Semicolons are optional and
only needed if you want to put multiple expressions on one line.

```vibe
    0
```

This is the return value. In VibeLang, the last expression in a function body
is its return value — there's no `return` keyword needed (though one exists for
early returns). The value `0` is returned to the operating system as the exit
code, where `0` conventionally means "success."

```vibe
}
```

Closes the function body.

### Why `main` Returns `Int`

If you've used Python or JavaScript, you might wonder why `main` returns an
integer instead of nothing. The reason is Unix convention: every process returns
an exit code to its parent. An exit code of `0` means success; any other value
indicates an error. Shell scripts, CI systems, and orchestration tools all rely
on exit codes to determine whether a command succeeded.

VibeLang makes this explicit. Instead of having the runtime silently return `0`
for you, you choose what to return. This becomes useful when your program needs
to signal different kinds of failures:

```vibe
@effect io
pub main() -> Int {
    result := do_work()
    match result {
        ok(_) => 0
        err(_) => 1
    }
}
```

We'll cover `match`, `Result`, and error handling in detail in Chapter 2 and
Chapter 8.

## 1.3 Understanding the Anatomy of a VibeLang Program

Now that you've seen a complete program, let's formalize the structure. Every
VibeLang program is built from these components:

### Function Signatures

A function signature in VibeLang has this general form:

```vibe
@intent "Description of what this function does"
@require condition
@ensure condition
@effect effect1, effect2
pub function_name(param1: Type1, param2: Type2) -> ReturnType {
    // body
}
```

Not all of these are required. The minimal function is:

```vibe
add(a: Int, b: Int) -> Int {
    a + b
}
```

This function is private (no `pub`), has no contracts (no `@intent`, `@require`,
`@ensure`), has no effects (it's a pure function), takes two `Int` parameters,
and returns their sum.

### Return Types

Every function in VibeLang has an explicit return type declared with `->`. There
is no implicit `void` — if a function doesn't return a meaningful value, it
returns the unit type `()`:

```vibe
@effect io
log_message(msg: Str) -> () {
    println(msg)
}
```

The compiler enforces that the function body's final expression matches the
declared return type. If they don't match, you get a clear error:

```
error[E0102]: type mismatch
 --> math.yb:3:5
  |
1 | add(a: Int, b: Int) -> Int {
  |                         --- expected `Int`
2 |     result := a + b
3 |     println(result)
  |     ^^^^^^^^^^^^^^^ found `()`
  |
  = help: `println` returns `()`, but this function expects `Int`
  = help: did you mean to return `result` on the last line?
```

### Effect Declarations

The `@effect` annotation lists every side effect a function performs. VibeLang
tracks six categories of effects:

| Effect        | Meaning                                      |
|---------------|----------------------------------------------|
| `io`          | Reads from or writes to the outside world (files, stdout, stdin) |
| `net`         | Makes network requests                       |
| `alloc`       | Allocates heap memory                        |
| `mut_state`   | Mutates shared or global state               |
| `concurrency` | Spawns goroutines or uses channels           |
| `nondet`      | Non-deterministic behavior (random numbers, timestamps) |

A function can declare multiple effects:

```vibe
@effect io, net
fetch_page(url: Str) -> Result<Str, Error> {
    // ...
}
```

If a function calls another function that has effects, the caller must also
declare those effects. Effects propagate up the call chain. The compiler
verifies this statically — you cannot accidentally perform an undeclared effect.

This is one of VibeLang's key safety guarantees. When you see a function with
no `@effect` annotation, you know it's pure. You can call it in any context,
cache its results, run it in parallel, and test it without mocking anything.

### Modules

Every `.yb` file is implicitly a module. You can make this explicit with a
`module` declaration at the top of the file:

```vibe
module math

pub add(a: Int, b: Int) -> Int {
    a + b
}
```

Other files import it with:

```vibe
import math

@effect io
pub main() -> Int {
    result := math.add(2, 3)
    println(result)
    0
}
```

We'll cover modules in depth in Chapter 10.

## 1.4 The vibe Toolchain

VibeLang ships with a single binary — `vibe` — that handles compilation,
testing, formatting, and more. Here are the commands you'll use most often.

### `vibe check` — Type Check Without Building

```bash
vibe check main.yb
```

This runs the full type checker, contract validator, and effect checker without
producing a binary. It's the fastest way to verify your code is correct:

```
Checking main.yb...
  ✓ Types verified
  ✓ Contracts validated
  ✓ Effects consistent
All checks passed.
```

If there are errors, `vibe check` reports them all at once rather than stopping
at the first one:

```
Checking math.yb...

error[E0102]: type mismatch
 --> math.yb:5:5
  |
5 |     "not a number"
  |     ^^^^^^^^^^^^^^ expected `Int`, found `Str`

error[E0301]: undeclared effect `io`
 --> math.yb:4:5
  |
4 |     println(result)
  |     ^^^^^^^ this call performs `io`

Found 2 errors.
```

Use `vibe check` liberally. It's designed to be fast enough to run on every
save — many editors with VibeLang support do this automatically.

### `vibe build` — Compile to Native Binary

```bash
vibe build main.yb
```

This compiles your program to a native executable. By default, it produces a
debug build with contract checks enabled:

```
Compiling main.yb...
  ✓ Built: ./main (debug, 2.1 MB)
```

For a release build with optimizations and contract checks compiled to static
assertions where possible:

```bash
vibe build --release main.yb
```

```
Compiling main.yb (release)...
  ✓ Built: ./main (release, 856 KB)
```

The resulting binary is a standalone native executable with no runtime
dependencies. You can copy it to any compatible machine and run it directly.

### `vibe run` — Build and Execute

```bash
vibe run main.yb
```

This is shorthand for `vibe build main.yb && ./main`. It compiles the program
and immediately runs it:

```
Hello, World!
```

You can pass arguments to your program after a `--` separator:

```bash
vibe run main.yb -- --verbose input.txt
```

### `vibe test` — Run Tests

```bash
vibe test main.yb
```

This runs all tests in the file, including `@examples` from contract
annotations. We'll use this extensively starting in Chapter 2:

```
Testing main.yb...
  ✓ celsius_to_fahrenheit: 3 examples passed
  ✓ fahrenheit_to_celsius: 3 examples passed
  ✓ test_absolute_zero: passed

All 3 tests passed (7 assertions).
```

`vibe test` discovers test functions (functions whose names start with `test_`)
and `@examples` annotations automatically. No test runner configuration needed.

### `vibe fmt` — Format Code

```bash
vibe fmt main.yb
```

This reformats your code to VibeLang's canonical style. There's one style, and
it's not configurable — this eliminates style debates and ensures all VibeLang
code looks the same:

```
Formatted main.yb (2 changes)
```

To format all `.yb` files in a directory:

```bash
vibe fmt .
```

To check formatting without modifying files (useful in CI):

```bash
vibe fmt --check .
```

### `vibe lint --intent` — Verify Intent Alignment

```bash
vibe lint --intent main.yb
```

This is VibeLang's most distinctive tool. It runs two layers of analysis:

1. **Local heuristic checks** (always run, no setup needed) — detect missing
   `@intent` on public functions (I5001), vague intent text (I5002), effect
   mismatches (I5003), and missing `@examples` (I5004).

2. **AI-powered semantic drift detection** (requires an Anthropic API key) —
   uses Claude to compare each function's `@intent` against its implementation
   and flag semantic drift (W0801).

```
Linting main.yb (intent analysis)...

warning[W0801]: possible intent drift in `abs`
 --> math.yb:8:1
  |
1 | @intent "Returns the absolute value of a number"
  |          ---------------------------------------- stated intent
  ...
8 |     if n < 0 { n } else { -n }
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ implementation negates positive numbers
  |
  = help: did you mean `if n < 0 { -n } else { n }`?

Found 1 warning.
```

This is particularly valuable when reviewing AI-generated code. The intent
serves as a specification, and `vibe lint --intent` checks whether the
implementation matches.

#### Setting Up the AI Sidecar (Optional)

VibeLang uses a **Bring Your Own Key (BYOK)** model. There is no centralized
VibeLang API proxy — all LLM traffic goes directly from your machine to the
Anthropic API. You control your own account, billing, and privacy.

**Option 1: Environment variable (recommended)**

```bash
export ANTHROPIC_API_KEY="sk-ant-api03-..."
```

Add this to your `~/.bashrc`, `~/.zshrc`, or CI secrets for persistence.

**Option 2: Global config file**

Create `~/.config/vibe/sidecar.toml`:

```toml
api_key = "sk-ant-api03-..."
model = "claude-sonnet-4-20250514"
endpoint = "https://api.anthropic.com"
```

This is set once per machine and never committed to version control.

**Option 3: Project config (for non-secret settings only)**

Add a `[sidecar]` section to your project's `vibe.toml`:

```toml
[sidecar]
enabled = true
mode = "hybrid"
model = "claude-sonnet-4-20250514"
redact_strings = true
```

Do **not** put your API key in `vibe.toml` — the CLI will warn you if it
detects one there.

**No key? No problem.** Without an API key, `vibe lint --intent` still runs all
local heuristic checks. AI features are silently skipped — no error, no degraded
exit code. Local checks are the baseline; AI is the upgrade.

**Running with AI enabled:**

```bash
export ANTHROPIC_API_KEY="sk-ant-api03-..."
vibe lint . --intent --mode hybrid
```

## 1.5 Your Development Workflow

Here's the typical workflow when writing VibeLang code:

### The Check-Run-Test-Format Loop

1. **Write code** in your editor.
2. **`vibe check`** — catch type errors, effect violations, and contract issues
   immediately. This is your fastest feedback loop.
3. **`vibe run`** — execute the program and observe behavior.
4. **`vibe test`** — run all tests and `@examples` to verify correctness.
5. **`vibe fmt`** — format your code before committing.

In practice, steps 2 and 3 often happen automatically if your editor runs
`vibe check` on save. You'll spend most of your time in the write → check cycle,
occasionally running the program and tests.

### How VibeLang Catches Errors at Compile Time

VibeLang's compiler is deliberately strict. It catches entire categories of bugs
before your program ever runs:

**Type errors** — You can't pass a `Str` where an `Int` is expected:

```vibe
add(a: Int, b: Int) -> Int {
    a + b
}

@effect io
pub main() -> Int {
    result := add("hello", 42)  // Compile error!
    0
}
```

```
error[E0102]: type mismatch
 --> main.yb:7:19
  |
7 |     result := add("hello", 42)
  |                   ^^^^^^^ expected `Int`, found `Str`
```

**Effect violations** — You can't perform I/O in a function that doesn't
declare it:

```vibe
pure_add(a: Int, b: Int) -> Int {
    println(a + b)  // Compile error!
    a + b
}
```

```
error[E0301]: undeclared effect `io`
 --> math.yb:2:5
  |
2 |     println(a + b)
  |     ^^^^^^^ this call performs `io`
  |
  = help: add `@effect io` before the function signature
  = note: or remove the `println` call to keep this function pure
```

**Contract violations** (caught at compile time when possible):

```vibe
@require n >= 0
sqrt(n: Float) -> Float {
    // ...
}

@effect io
pub main() -> Int {
    result := sqrt(-1.0)  // Compile error!
    0
}
```

```
error[E0401]: contract violation
 --> main.yb:8:15
  |
1 | @require n >= 0
  |          ------ required precondition
  ...
8 |     result := sqrt(-1.0)
  |                    ^^^^ `-1.0` violates `n >= 0`
```

These compile-time checks mean that when your program does compile, you can have
high confidence it's correct — not just type-safe, but semantically aligned with
your stated intent.

## 1.6 Summary

In this chapter, you:

- Installed the VibeLang toolchain
- Wrote, compiled, and ran your first VibeLang program
- Learned the anatomy of a VibeLang program: function signatures, return types,
  effect declarations, and the module system
- Explored the `vibe` toolchain: `check`, `build`, `run`, `test`, `fmt`, and
  `lint --intent`
- Understood VibeLang's compile-time safety guarantees: type checking, effect
  tracking, and contract validation

In the next chapter, we'll build a real program — a temperature converter — and
explore VibeLang's contract system, error handling, and testing in depth.

---

*Continue to [Chapter 2: Your First VibeLang Program](ch02_first_program.md)*
