# Chapter 14: The Compiler — How VibeLang Works

Understanding how the compiler transforms your source code into a native binary
makes you a better VibeLang programmer. When you know what happens at each
stage, error messages make more sense, performance characteristics become
predictable, and you can use the compiler's inspection tools to debug subtle
issues.

---

## 14.1 The Compilation Pipeline

VibeLang compiles ahead-of-time (AOT) to native machine code through a series
of transformations:

```
  Source Code (.yb)
        |
        v
  [ Lexer ]              vibe_lexer
        |
        |  Token Stream
        v
  [ Parser ]             vibe_parser
        |
        |  AST
        v
  [ Type Checker ]       vibe_types
        |
        |  Typed AST
        v
  [ HIR Lowering ]       vibe_hir
        |
        |  High-Level IR
        v
  [ MIR Lowering ]       vibe_mir
        |
        |  Mid-Level IR
        v
  [ Code Gen ]           vibe_codegen (Cranelift)
        |
        |  Machine Code
        v
  [ Native Binary ]
```

Each stage has a corresponding Rust crate. The `vibe_diagnostics` crate provides
error reporting across all stages, and `vibe_cli` ties everything into the
`vibe` command.

| Stage | Input | Output | Responsibility |
|---|---|---|---|
| Lexer | Source text | Tokens | Character-level processing, literal parsing |
| Parser | Tokens | AST | Syntactic structure, expression precedence |
| Type Checker | AST | Typed AST | Type inference, effect checking, contracts |
| HIR Lowering | Typed AST | HIR | Desugaring, contract lowering |
| MIR Lowering | HIR | MIR | Optimization, effect analysis, control flow |
| Code Gen | MIR | Native code | Instruction selection, register allocation |

This separation makes the compiler easier to maintain and test. Each stage can
be inspected independently with `vibe ast`, `vibe hir`, and `vibe mir`.

---

## 14.2 Lexing

The lexer reads raw source text and produces a stream of tokens — the smallest
meaningful units of the language.

### How Source Code Becomes Tokens

```vibe
pub add(a: Int, b: Int) -> Int {
    a + b
}
```

The lexer produces:

```
KW_PUB  "pub"     IDENT  "add"     LPAREN "("
IDENT   "a"       COLON  ":"       IDENT  "Int"
COMMA   ","       IDENT  "b"       COLON  ":"
IDENT   "Int"     RPAREN ")"       ARROW  "->"
IDENT   "Int"     LBRACE "{"       IDENT  "a"
PLUS    "+"       IDENT  "b"       RBRACE "}"
```

Each token carries its text, position (line and column), and category. Position
information flows through every subsequent stage, enabling precise error
messages.

### Token Categories

**Keywords:** `pub`, `mut`, `if`, `else`, `for`, `while`, `repeat`, `match`,
`case`, `default`, `go`, `select`, `return`, `break`, `continue`, `import`,
`module`, `type`, `true`, `false`

**Identifiers:** Any name that isn't a keyword.

**Literals:** Integer (`42`, `1_000_000`), float (`3.14`), string (`"hello"`),
duration (`5s`, `100ms`).

**Operators:** `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`,
`&&`, `||`, `!`, `?`, `:=`, `=`, `->`, `..`, `..=`, `|`

**Annotations:** `@intent`, `@examples`, `@require`, `@ensure`, `@effect`

**Delimiters:** `(`, `)`, `{`, `}`, `[`, `]`, `,`, `:`, `.`

### Using `vibe ast` to Inspect

```bash
vibe ast --tokens src/main.yb
```

The lexer recovers from errors by skipping invalid characters, allowing the
parser to report additional errors in the same compilation pass:

```
error[E0001]: invalid character
 --> main.yb:3:10
  |
3 |     x := 42$
  |            ^ unexpected character `$`
```

---

## 14.3 Parsing

The parser takes the token stream and builds an Abstract Syntax Tree (AST) — a
tree that represents the syntactic structure of your program.

### Tokens to AST

For the `add` function, the parser produces (simplified):

```
FunctionDecl
 ├─ visibility : Public
 ├─ name       : "add"
 ├─ params
 │   ├─ Param { name: "a", type: Int }
 │   └─ Param { name: "b", type: Int }
 ├─ return_type: Int
 └─ body
     └─ BinaryExpr
         ├─ op   : Add
         ├─ left : Ident("a")
         └─ right: Ident("b")
```

### Expression-Oriented Parsing

VibeLang's expression-oriented design means `if/else` is always parsed as an
expression. Whether it's used as a statement or a value depends on context —
the type checker resolves this later.

### Operator Precedence

| Precedence | Operators | Associativity |
|---|---|---|
| Highest | `!`, `-` (unary) | Right |
| | `*`, `/`, `%` | Left |
| | `+`, `-` | Left |
| | `..`, `..=` | None |
| | `==`, `!=`, `<`, `>`, `<=`, `>=` | None |
| | `&&` | Left |
| | `||` | Left |
| | `?` (postfix) | Left |
| Lowest | `:=`, `=` | Right |

### Error Recovery

The parser skips tokens until it finds a synchronization point (closing brace,
top-level newline, or declaration keyword), allowing it to report multiple
errors per pass:

```
error[E0101]: expected `)` after parameter list
 --> main.yb:1:25
  |
1 | pub add(a: Int, b: Int -> Int {
  |                        ^^ expected `)`, found `->`

error[E0102]: type mismatch
 --> main.yb:3:5
  |
3 |     "not a number"
  |     ^^^^^^^^^^^^^^ expected `Int`, found `Str`

Found 2 errors.
```

Inspect the AST with `vibe ast src/main.yb`.

---

## 14.4 Type Checking

The type checker is the most complex stage. It produces a typed AST where every
expression has a known type, every effect is verified, and every contract is
validated.

### Type Inference Algorithm

VibeLang uses bidirectional type inference:

**Top-down (checking):** When the expected type is known, push it into
subexpressions. In `x: Float := 3.14`, the declared type `Float` flows down.

**Bottom-up (synthesis):** When the expected type is unknown, infer from the
expression. In `x := 3 + 4`, the checker infers `Int` from the operands.

The two modes work together. In `result := celsius_to_fahrenheit(100.0)`, the
checker synthesizes the return type `Float` and uses it as the type of `result`.

### Assignability Rules

VibeLang's type system is nominal — types are distinguished by name:

1. A value of type `T` is assignable to a binding of type `T` (identity).
2. `ok(T)` and `err(E)` are assignable to `Result<T, E>`.
3. Integer literals default to `Int`; decimal literals default to `Float`.
4. Generic types match when type parameters match: `List<Int>` is not
   `List<Float>`.

### Effect Checking and Transitivity

The type checker walks the call graph:

1. For each function call, look up the callee's declared effects.
2. Verify the caller declares all of the callee's effects.
3. Report missing effects as errors.

This is transitive: if `A` calls `B` calls `C`, and `C` declares `@effect io`,
then both `B` and `A` must declare `@effect io`. The checker also warns about
over-declared effects that aren't actually used.

### Contract Validation

The type checker validates contracts:

1. **Type checking:** `@require x > 0` requires `x` to support `>` with `Int`.
2. **Purity checking:** Contract expressions must be pure — no effects allowed.
3. **Return reference:** `.` (or `result`) in `@ensure` must match the return
   type.
4. **Example validation:** `@examples` must type-check against the signature.

---

## 14.5 HIR and MIR

### High-Level IR (HIR)

The HIR is a desugared, normalized version of the typed AST:

**Contract lowering:** `@require` becomes explicit runtime checks (dev/test) or
static assertions (release):

```
// Before (typed AST):
@require x >= 0
sqrt(x: Float) -> Float { ... }

// After (HIR, dev profile):
sqrt(x: Float) -> Float {
    if !(x >= 0) { contract_violation("@require x >= 0", "math.yb", 1) }
    ...
}
```

**Desugaring:** Syntactic sugar is expanded:

- `for item in collection { ... }` → iterator protocol calls
- `x?` → `match x { ok(v) => v, err(e) => return err(e) }`
- `a..b` → range constructor call

### Mid-Level IR (MIR)

The MIR is designed for optimization and analysis:

**Control flow graph:** Functions become graphs of basic blocks connected by
branches, making control flow analysis straightforward.

**Optimization passes:**

- Dead code elimination
- Constant folding and propagation
- Function inlining (small pure functions)
- Loop-invariant code motion
- Common subexpression elimination

**Contract optimization (release mode):**

- Eliminate redundant checks when preconditions are provably satisfied
- Hoist invariant checks out of loops
- Merge overlapping pre/postcondition checks

### Inspecting HIR and MIR

```bash
vibe hir src/main.yb    # Show desugared IR with contract checks
vibe mir src/main.yb    # Show optimized control flow graph
```

Comparing HIR and MIR reveals which optimizations the compiler applied:

```
fn guarded_sqrt(x: Float) -> Float {
  bb0:
    %0 = gte x, 0.0
    branch %0, bb1, bb_contract_fail
  bb_contract_fail:
    call contract_violation("@require x >= 0.0", "math.yb", 2)
    unreachable
  bb1:
    %1 = call sqrt(x)
    return %1
}
```

---

## 14.6 Code Generation

### The Cranelift Backend

VibeLang uses Cranelift for code generation:

1. **Compilation speed.** Significantly faster than LLVM, improving the
   edit-compile-run cycle.
2. **Safety.** Written in Rust with memory safety guarantees.
3. **Determinism.** Produces deterministic output for the same input — essential
   for reproducible builds.
4. **Good-enough optimization.** While not matching LLVM's peak, Cranelift
   produces code fast enough for the vast majority of applications.

### AOT Compilation to Native Code

The output is a standalone native executable:

```bash
vibe build src/main.yb
file ./main
```

```
./main: ELF 64-bit LSB executable, x86-64
```

The binary includes your compiled code, the VibeLang runtime (scheduler, GC,
channels), standard library functions, and contract check stubs (dev/test).

### Platform Support

| Platform | Architecture | Status |
|---|---|---|
| Linux | x86_64, aarch64 | Supported |
| macOS | x86_64, aarch64 | Supported |
| Windows | x86_64 | Supported |

Cross-compilation: `vibe build --target x86_64-linux src/main.yb`

---

## 14.7 Determinism Engineering

VibeLang guarantees: same source + same flags + same compiler version = same
binary.

**No timestamps in binaries.** No build timestamps, hostnames, or
environment-dependent data.

**Deterministic iteration order.** Hash maps use a fixed seed; sets iterate in
insertion order.

**Stable symbol names.** Generated symbols use deterministic naming based on
source position, not memory addresses.

**Deterministic optimization.** Cranelift's passes produce the same output
regardless of host platform.

**Stable diagnostics.** The same source with the same errors produces identical
diagnostic output in the same order.

Verify reproducibility:

```bash
vibe build --release src/main.yb -o main_a
vibe build --release src/main.yb -o main_b
sha256sum main_a main_b
```

Identical hashes confirm the guarantee holds.

---

## 14.8 The Runtime

The compiled binary includes the VibeLang runtime: task scheduling, channel
communication, and garbage collection.

### M:N Scheduler

M lightweight tasks are multiplexed onto N OS threads:

- **Thread pool** defaults to CPU core count (override with `VIBE_THREADS`)
- **Task creation** is cheap: a few KB of stack, no OS thread
- **Preemptive scheduling** at channel operations, function calls, and loop
  back-edges
- **Work stealing** distributes tasks across threads when queues are imbalanced

### Channel Implementation

Channels are bounded, lock-free queues:

- **Send** enqueues or parks the sender until space opens
- **Receive** dequeues or parks the receiver until a value arrives
- **Close** prevents further sends and signals waiting receivers
- **Fairness:** Waiting receivers are woken in FIFO order
- **Happens-before:** Every send-receive pair establishes memory visibility

### The Garbage Collector

Concurrent generational GC:

**Young generation (nursery):** Bump-pointer allocation, stop-the-world minor
GC (< 1ms), surviving objects promoted to old generation.

**Old generation:** Concurrent mark-sweep, application threads run alongside
GC, brief pauses only for root scanning (< 10ms).

| Variable | Default | Description |
|---|---|---|
| `VIBE_GC_NURSERY_SIZE` | 4MB | Young generation size |
| `VIBE_GC_HEAP_MAX` | 1GB | Maximum heap size |
| `VIBE_GC_THREADS` | 2 | GC worker threads |

For most programs, defaults work well. Tune only after profiling.

---

## 14.9 Summary

The VibeLang compiler transforms source code into native binaries through a
carefully designed pipeline:

- **Lexing** converts source text into tokens with position information for
  error reporting.
- **Parsing** builds an AST with error recovery to report multiple issues per
  compilation.
- **Type checking** performs bidirectional inference, verifies effects
  transitively, and validates contracts for correctness and purity.
- **HIR lowering** desugars constructs and lowers contracts into runtime checks
  or static assertions.
- **MIR lowering** builds control flow graphs and runs optimization passes.
- **Code generation** uses Cranelift for fast, deterministic native code
  production.
- **Determinism engineering** ensures reproducible builds through deterministic
  data structures and stable symbol naming.
- **The runtime** provides an M:N scheduler, bounded channels with
  happens-before guarantees, and a concurrent generational GC.

Understanding this pipeline helps you write better code: you know why the
compiler catches certain errors, how contracts are enforced, what optimizations
apply, and how the runtime manages concurrent tasks.

---

Next: Chapter 15 explores the `vibe` toolchain in depth — every command, flag,
and workflow.
