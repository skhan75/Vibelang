# Chapter 15: Toolchain Deep Dive

VibeLang ships as a single binary — `vibe` — that handles every phase of
development: checking, building, running, testing, formatting, linting,
documentation, language server support, package management, and compiler
inspection. This chapter covers every command in depth.

---

## 15.1 The `vibe` CLI

### Architecture: One Binary, Many Commands

The `vibe` binary dispatches to the appropriate compiler crate based on the
subcommand:

```
vibe <command> [flags] [arguments]
```

### Global Flags

| Flag | Short | Description |
|---|---|---|
| `--help` | `-h` | Print help for the command |
| `--version` | `-V` | Print compiler version |
| `--verbose` | `-v` | Increase output verbosity |
| `--quiet` | `-q` | Suppress non-error output |
| `--color` | | Color output: `always`, `never`, `auto` |
| `--threads` | `-j` | Parallel compilation threads |

Every command follows consistent conventions: `--help` for usage, exit code `0`
for success, non-zero for failure.

---

## 15.2 Building and Running

### `vibe check` — Verify Without Building

```bash
vibe check src/main.yb    # Single file
vibe check src/            # All .yb files in directory
vibe check .               # Entire project
```

Runs the full frontend — lexing, parsing, type checking, effect verification,
and contract validation — without generating a binary. This is the fastest
feedback loop.

What `vibe check` verifies:

| Check | Example error |
|---|---|
| Syntax | `E0101: expected ')' after parameter list` |
| Types | `E0102: type mismatch — expected Int, found Str` |
| Effects | `E0401: missing effect declaration — @effect io` |
| Contracts | `E0402: effectful function in contract expression` |
| Exhaustiveness | `E0540: non-exhaustive match expression` |
| Sendability | `E0801: cannot capture mutable binding in go block` |

`vibe check` reports all errors it finds, not just the first one. When run
inside a project directory (containing `vibe.toml`), it automatically discovers
all source files.

### `vibe build` — Compile to Native Binary

```bash
vibe build src/main.yb                    # Debug build
vibe build --release src/main.yb          # Release build
vibe build --profile test src/main.yb     # Test build
vibe build -o myapp src/main.yb           # Custom output name
```

**Compilation flags:**

| Flag | Description |
|---|---|
| `--release` | Release profile (optimized) |
| `--profile <name>` | Select profile: `dev`, `test`, `release` |
| `-o <path>` | Output binary path |
| `--target <triple>` | Cross-compilation target |
| `--emit <stage>` | Emit intermediate representation |

### Build Profiles

| Profile | Optimization | Contracts | Debug Info | Use Case |
|---|---|---|---|---|
| `dev` | None | Runtime checks | Full | Daily development |
| `test` | None | Runtime checks | Full | Running tests |
| `release` | Full | Configurable | Stripped | Production |

**Release contract behavior** is configured in `vibe.toml`:

```toml
[release]
contracts = "checked"    # "checked", "unchecked", or "removed"
```

- `checked` — contracts compiled as runtime checks (safest, default)
- `unchecked` — contracts used for static analysis only, no runtime overhead
- `removed` — contracts stripped entirely (smallest binary)

### `vibe run` — Build and Execute

```bash
vibe run src/main.yb
vibe run src/main.yb -- --flag1 arg1    # Pass arguments to the program
vibe run .                               # Run project entry point
```

Shorthand for `vibe build` followed by executing the binary. Arguments after
`--` are passed to the compiled program. Inside a project directory, uses the
entry point from `vibe.toml`.

---

## 15.3 Testing

### `vibe test` — Run Tests

```bash
vibe test src/main.yb          # Single file
vibe test src/                  # All files in directory
vibe test .                     # Entire project
vibe test . --filter "parse"    # Tests matching "parse"
```

Discovers and runs two kinds of tests:

**1. `@examples` in contracts:**

```vibe
@examples {
    celsius_to_fahrenheit(0.0) == 32.0
    celsius_to_fahrenheit(100.0) == 212.0
}
celsius_to_fahrenheit(celsius: Float) -> Float {
    celsius * 9.0 / 5.0 + 32.0
}
```

```
Testing src/main.yb...
  ✓ celsius_to_fahrenheit: 2 examples passed
```

**2. Test functions** (names starting with `test_`):

```vibe
pub test_edge_cases() -> Int {
    @effect alloc
    assert(celsius_to_fahrenheit(-273.15) >= -459.67, "absolute zero check")
    0
}
```

### Test Output Format

```
Testing <file>...
  ✓ <function>: <N> examples passed
  ✗ <function>: example <N> failed
    Expected: <expression> == <expected>
    Got:      <expression> == <actual>

<summary line>
```

Exit code `0` if all tests pass, `1` if any fail.

### Testing Strategies

**Unit tests via `@examples`:** Best for pure functions. Zero boilerplate,
automatic execution, doubles as documentation.

**Integration tests in `tests/`:** Best for effectful code:

```
myproject/
├── src/
│   └── main.yb
└── tests/
    ├── parser_test.yb
    └── pipeline_test.yb
```

**Concurrent tests:** Use channels to synchronize assertions. Test aggregate
properties (sums, counts) rather than ordering.

---

## 15.4 Code Quality

### `vibe fmt` — Format Code

```bash
vibe fmt src/main.yb       # Format a file
vibe fmt .                  # Format entire project
vibe fmt --check .          # Check without modifying (CI mode)
```

VibeLang has one canonical formatting style with no configuration options. This
eliminates formatting debates and ensures all VibeLang code looks the same.

**Key rules:** 4-space indentation, opening braces on same line, contract
annotations ordered (`@intent`, `@examples`, `@require`, `@ensure`, `@effect`),
100-character soft line limit.

**CI integration:** `vibe fmt --check .` exits with code `1` if any file is not
formatted, printing the diff.

### `vibe lint` — Static Analysis

```bash
vibe lint src/main.yb       # Single file
vibe lint .                  # Entire project
vibe lint --intent .         # Include AI-powered intent analysis
```

Catches patterns that are technically valid but likely incorrect:

| Warning | Description |
|---|---|
| `W0101` | Unused binding |
| `W0102` | Unused import |
| `W0201` | Unreachable code |
| `W0301` | Shadowed binding |
| `W0401` | Over-declared effect |
| `W0402` | Redundant contract |
| `W0601` | Channel created but never used |

### `vibe lint --intent` — AI-Powered Intent Verification

Runs local heuristic checks and, when an API key is available, AI-powered
semantic drift detection using Anthropic Claude:

```
warning[W0801]: possible intent drift in `abs`
 --> src/math.yb:8:1
  |
1 | @intent "Returns the absolute value of a number"
  ...
8 |     if n < 0 { n } else { -n }
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ implementation negates positive numbers
  |
  = help: did you mean `if n < 0 { -n } else { n }`?
```

#### API Key Setup (BYOK)

VibeLang uses a Bring Your Own Key model — there is no centralized proxy.
All LLM traffic goes directly from your machine to the Anthropic API.

Key resolution (highest precedence first):

1. `ANTHROPIC_API_KEY` environment variable
2. `~/.config/vibe/sidecar.toml` (global per-machine config)
3. `vibe.toml` `[sidecar]` section (per-project)

```bash
# Recommended: set via environment variable
export ANTHROPIC_API_KEY="sk-ant-..."
```

Without a key, all local checks (I5001–I5004) still run. AI features are
silently skipped.

#### Project Configuration

Configure the sidecar in your project's `vibe.toml`:

```toml
[sidecar]
enabled = true
mode = "hybrid"                    # local | hybrid | cloud
model = "claude-sonnet-4-20250514"   # any Claude model
endpoint = "https://api.anthropic.com"
max_requests_per_day = 100
max_monthly_budget_usd = 5.0
cache_ttl_hours = 24
redact_strings = true              # strip string literals before sending
```

Do **not** put your API key in `vibe.toml` — use the environment variable or
`~/.config/vibe/sidecar.toml` to avoid committing secrets.

#### Modes

| Mode | Behavior |
|---|---|
| `local` | Heuristic checks only (I5001–I5004). No network calls. |
| `hybrid` | Local checks first, then AI analysis if API key is available. |
| `cloud` | AI analysis for all functions with `@intent`. Requires API key. |

#### Flags

| Flag | Effect |
|---|---|
| `--changed` | Limit analysis to files modified since last commit |
| `--suggest` | Include AI-generated contract and example suggestions |
| `--mode <mode>` | Override the mode from `vibe.toml` |
| `--telemetry-out <path>` | Write telemetry JSON to the given path |

---

## 15.5 Documentation

### `vibe doc` — Generate API Documentation

```bash
vibe doc src/                # Generate docs
vibe doc . --open            # Generate and open in browser
vibe doc . --output docs/    # Custom output directory
```

Extracts function signatures, `@intent` annotations, `@examples` as usage
examples, contracts as documented pre/postconditions, and `@effect` declarations.

**Documentation comments** use `///` for functions or `//!` for modules:

```vibe
//! The math module provides basic mathematical operations.

/// Compute the greatest common divisor using the Euclidean algorithm.
@intent "Greatest common divisor via Euclidean algorithm"
@examples {
    gcd(12, 8) == 4
    gcd(7, 13) == 1
}
@require a > 0
@require b > 0
pub gcd(a: Int, b: Int) -> Int {
    if b == 0 { a } else { gcd(b, a % b) }
}
```

---

## 15.6 Language Server

### `vibe lsp` — Editor Integration

```bash
vibe lsp    # Start the language server (editors call this automatically)
```

Implements the Language Server Protocol (LSP):

- **Diagnostics:** Type errors, effect violations, and contract issues appear
  as you type
- **Completion:** Context-aware suggestions for functions, types, and fields
- **Hover:** Shows signature, contracts, and effects for any symbol
- **Go-to-definition:** Jump to any function, type, or module definition
- **Find references:** Locate all uses of a symbol across the project
- **Rename:** Rename a symbol with automatic updates to all references
- **Format on save:** Runs `vibe fmt` automatically

### Editor Setup

**VS Code / Cursor:** Install the "VibeLang" extension.

**Neovim (nvim-lspconfig):**

```lua
require('lspconfig').vibelang.setup {
    cmd = { "vibe", "lsp" },
    filetypes = { "vibelang" },
}
```

---

## 15.7 Package Management

### `vibe pkg` — Dependency Management

```bash
vibe pkg install <package>       # Add a dependency
vibe pkg update                  # Update all dependencies
vibe pkg audit                   # Check for vulnerabilities
vibe pkg list                    # List installed dependencies
```

Dependencies are declared in `vibe.toml`:

```toml
[dependencies]
http = "1.2.0"
json = "0.8.3"
```

### Dependency Resolution

Resolves dependencies transitively. Conflicts are reported clearly:

```
error: dependency conflict
  myapp requires http >= 1.2.0
  logging 2.1.0 requires http < 1.0.0
```

### Lock Files and Reproducibility

`vibe pkg` writes `vibe.lock` with exact versions and checksums:

```toml
[[package]]
name = "http"
version = "1.2.3"
checksum = "sha256:a1b2c3d4..."
```

Commit `vibe.lock` to version control. This ensures `vibe build` uses identical
dependency versions on every machine.

### Security Auditing

```bash
vibe pkg audit
```

```
advisory[VIBE-2025-001]: json < 0.9.0 — denial of service via deeply nested input
  installed: json 0.8.3
  fix: upgrade to json >= 0.9.0
```

Run in CI to catch vulnerable dependencies before deployment.

---

## 15.8 Debugging Tools

### `vibe ast` — Inspect the Abstract Syntax Tree

```bash
vibe ast src/main.yb              # Print AST
vibe ast --tokens src/main.yb     # Print token stream
vibe ast --json src/main.yb       # Output as JSON
```

Useful for debugging parsing issues or understanding operator precedence.

### `vibe hir` — Inspect High-Level IR

```bash
vibe hir src/main.yb
```

Shows code after desugaring and contract lowering. See how `@require` becomes
runtime checks, how `for ... in` desugars to iterators, and how `?` expands to
match-and-return.

### `vibe mir` — Inspect Mid-Level IR

```bash
vibe mir src/main.yb
```

Shows the optimized control flow graph. Comparing HIR and MIR reveals which
optimizations the compiler applied: dead code elimination, constant folding,
inlining, and contract check elimination.

### `vibe index` — Semantic Indexing

```bash
vibe index src/                   # Build semantic index
vibe index --query "Result" src/  # Search the index
```

Maps symbols to definitions, references, types, and contracts. The language
server uses this index for go-to-definition and find-references.

---

## 15.9 Summary

The `vibe` toolchain provides a complete development environment:

- **`vibe check`** — fastest feedback: types, effects, contracts without
  building
- **`vibe build`** — native binaries with three profiles: `dev`, `test`,
  `release`
- **`vibe run`** — build and execute in one step
- **`vibe test`** — automatic `@examples` and test function discovery
- **`vibe fmt`** — single canonical style, `--check` for CI
- **`vibe lint`** — static analysis; `--intent` for AI-powered drift detection
- **`vibe doc`** — HTML documentation from signatures and contracts
- **`vibe lsp`** — real-time editor integration via LSP
- **`vibe pkg`** — dependency management with lock files and security auditing
- **`vibe ast/hir/mir`** — compiler stage inspection for debugging
- **`vibe index`** — semantic indexing for code navigation

Every command follows consistent conventions: `--help` for usage, `--verbose`
for detail, `--quiet` for CI, and exit code `0` for success.

---

Next: Chapter 16 covers production deployment, release engineering, and team
adoption strategies.
