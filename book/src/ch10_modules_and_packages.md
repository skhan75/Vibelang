# Chapter 10: Modules and Packages

As programs grow beyond a single file, you need a way to split code into
manageable pieces, control what's visible to the outside world, and pull in
libraries written by others. VibeLang's module and package system is designed
for clarity and determinism: every name resolves the same way every time, every
dependency is explicitly declared, and visibility is private by default.

This chapter covers the full journey from a single-file program to a multi-module
project with external dependencies.

## 10.1 Organizing Code with Modules

### 10.1.1 The `module` Declaration

Every VibeLang source file can declare which module it belongs to using the
`module` keyword at the top of the file:

```vibe
module app.math.stats

pub mean(values: List<Int>) -> Int {
    @require values.len() > 0, "cannot compute mean of empty list"

    mut sum := 0
    for v in values {
        sum = sum + v
    }
    sum / values.len()
}

pub median(values: List<Int>) -> Int {
    @require values.len() > 0, "cannot compute median of empty list"
    @effect alloc

    mut sorted := values
    sorted.sort_desc()
    sorted.get(sorted.len() / 2)
}
```

The `module` declaration must be the first non-comment statement in the file.
It establishes the file's identity within the project.

### 10.1.2 File-to-Module Mapping

VibeLang follows a convention where the module path mirrors the directory
structure:

```
my_project/
├── vibe.toml
└── src/
    └── app/
        ├── main.yb          → module app
        └── math/
            ├── stats.yb      → module app.math.stats
            └── linalg.yb     → module app.math.linalg
```

This convention is not strictly enforced by the compiler — you *can* declare
`module foo.bar` in a file located at `src/baz/qux.yb`. But doing so creates
confusion for anyone reading the project. The strong recommendation is: **keep
module declarations aligned with directory paths.**

### 10.1.3 Qualified Names

Module names use dot-separated segments that form a qualified name. These
segments create a namespace hierarchy:

```
app                    → top-level application module
app.math               → math sub-module
app.math.stats         → statistics sub-module within math
app.math.linalg        → linear algebra sub-module within math
```

Qualified names serve two purposes: they prevent name collisions between
unrelated modules, and they communicate the organizational structure of your
code to both humans and tools.

## 10.2 Imports

### 10.2.1 Import Syntax

To use symbols from another module, import it:

```vibe
module app

import app.math.stats
import app.math.linalg

pub main() -> Int {
    @effect alloc

    data := [10, 20, 30, 40, 50]
    avg := stats.mean(data)
    mid := stats.median(data)

    0
}
```

The `import` statement brings the module's public symbols into scope. After
importing `app.math.stats`, you refer to its public functions using the last
segment of the module name as a prefix: `stats.mean(...)`.

### 10.2.2 What Gets Imported

An import makes all `pub`-marked symbols from the target module available. It
does *not* import private symbols — those remain invisible regardless of import
statements.

```vibe
module app.math.stats

helper_sum(values: List<Int>) -> Int {
    mut total := 0
    for v in values {
        total = total + v
    }
    total
}

pub mean(values: List<Int>) -> Int {
    @require values.len() > 0
    helper_sum(values) / values.len()
}
```

Here `helper_sum` has no `pub` keyword, so it is private to the
`app.math.stats` module. Code in other modules cannot call `stats.helper_sum()`
even after importing the module:

```vibe
module app

import app.math.stats

pub main() -> Int {
    data := [10, 20, 30]
    total := stats.helper_sum(data)
    0
}
```

```
error[E0401]: function `helper_sum` is private to module `app.math.stats`
 --> main.yb:6:14
  |
6 |     total := stats.helper_sum(data)
  |              ^^^^^^^^^^^^^^^^^^^^^ private function
  |
  = note: only `pub` functions can be accessed from other modules
```

### 10.2.3 Name Resolution Rules

When the compiler encounters an unqualified name, it resolves it using this
precedence order:

1. **Local scope** — bindings in the current block or function.
2. **Module scope** — symbols defined in the current module's file.
3. **Imported modules** — public symbols from imported modules, accessed via
   the module's short name.

If two imported modules export the same name, you must use the full qualified
prefix to disambiguate:

```vibe
import app.graphics.color
import app.terminal.color

pub main() -> Int {
    gc := graphics.color.red()
    tc := terminal.color.red()
    0
}
```

Shadowing a module-level name with a local binding is permitted but discouraged
in public-facing code. The compiler may emit a warning if a local binding
shadows an imported name.

## 10.3 Visibility with `pub`

### 10.3.1 Private by Default

Every function, type, and constant in VibeLang is **private by default**. This
is a deliberate design choice: it forces you to make an explicit decision about
what constitutes your module's public API.

```vibe
module app.auth

validate_token_format(token: Str) -> Bool {
    token.len() > 0 && token.len() <= 256
}

hash_token(token: Str) -> Str {
    @effect alloc
    token + ".hashed"
}

pub authenticate(token: Str) -> Result<Bool, Str> {
    if !validate_token_format(token) {
        Err("invalid token format")
    } else {
        hashed := hash_token(token)
        Ok(true)
    }
}
```

Only `authenticate` is visible outside this module. The helper functions
`validate_token_format` and `hash_token` are implementation details that callers
never see and cannot depend on.

### 10.3.2 Making Functions and Types Public

Add `pub` before a function or type definition to export it:

```vibe
module app.models

pub type User {
    name: Str,
    email: Str,
    age: Int
}

pub create_user(name: Str, email: Str, age: Int) -> User {
    @require age >= 0, "age must be non-negative"
    @require name.len() > 0, "name must not be empty"

    User { name: name, email: email, age: age }
}

pub display_name(user: User) -> Str {
    user.name
}
```

### 10.3.3 API Design Principles

Good module APIs in VibeLang follow these principles:

1. **Minimize the public surface.** Export only what consumers need. Every
   public symbol is a commitment — changing it later may break downstream code.

2. **Use contracts on public functions.** `@require` and `@ensure` on public
   functions document the API contract in a machine-checkable way.

3. **Return `Result` for fallible operations.** Public functions that can fail
   should return `Result<T, E>` rather than panicking.

4. **Name for clarity at the call site.** A function named `parse_port` is
   clearer than `parse` when called as `config.parse_port(raw)` vs
   `config.parse(raw)`.

### 10.3.4 Stable Public API Discipline

Once a module is published and other code depends on it, its public API becomes
a contract. VibeLang encourages treating public APIs with the same rigor as
function contracts:

- **Adding** a new `pub` function is always safe.
- **Removing** a `pub` function is a breaking change.
- **Changing** a `pub` function's signature is a breaking change.
- **Weakening** a `@require` (accepting more inputs) is safe.
- **Strengthening** a `@require` (accepting fewer inputs) is breaking.
- **Strengthening** an `@ensure` (guaranteeing more) is safe.
- **Weakening** an `@ensure` (guaranteeing less) is breaking.

This mirrors the Liskov Substitution Principle applied to module boundaries.

## 10.4 Packages

### 10.4.1 The `vibe.toml` Manifest

A package is a collection of modules with a manifest file called `vibe.toml`:

```toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2026"

[dependencies]
json_parser = "1.2.0"
http_client = "0.8.3"
```

The manifest declares the package's identity, version, and its dependencies on
other packages.

### 10.4.2 Package Structure

A typical VibeLang package follows this layout:

```
my_app/
├── vibe.toml
├── vibe.lock
├── src/
│   ├── main.yb
│   ├── config.yb
│   ├── handlers/
│   │   ├── auth.yb
│   │   └── api.yb
│   └── models/
│       ├── user.yb
│       └── session.yb
└── tests/
    ├── config_test.yb
    └── handlers/
        ├── auth_test.yb
        └── api_test.yb
```

- `vibe.toml` — the package manifest.
- `vibe.lock` — the lockfile, generated by the toolchain, ensuring
  deterministic dependency resolution.
- `src/` — source code, organized into modules.
- `tests/` — test files, mirroring the source structure.

### 10.4.3 SemVer Dependencies

VibeLang uses Semantic Versioning (SemVer) for all package dependencies:

- **Major** version (1.x.x → 2.x.x): breaking changes.
- **Minor** version (1.2.x → 1.3.x): new features, backward compatible.
- **Patch** version (1.2.3 → 1.2.4): bug fixes, backward compatible.

In `vibe.toml`, you specify the minimum compatible version:

```toml
[dependencies]
json_parser = "1.2.0"
```

This means "any version >= 1.2.0 and < 2.0.0". The resolver picks the newest
compatible version and records the exact version in `vibe.lock`.

## 10.5 Package Management with `vibe pkg`

### 10.5.1 Installing Dependencies

After adding a dependency to `vibe.toml`, install it:

```bash
vibe pkg install
```

This command:

1. Reads `vibe.toml` to find declared dependencies.
2. Resolves compatible versions using the SemVer rules.
3. Downloads packages from the registry.
4. Writes exact resolved versions to `vibe.lock`.
5. Places package source in the local cache.

The lockfile should be committed to version control. It ensures that every
developer and CI system uses exactly the same dependency versions.

### 10.5.2 Updating Packages

To update dependencies to their newest compatible versions:

```bash
vibe pkg update
```

This re-resolves dependencies within the SemVer constraints in `vibe.toml` and
updates `vibe.lock`. It will not cross major version boundaries unless you
change the constraint in `vibe.toml`.

To update a specific package:

```bash
vibe pkg update json_parser
```

### 10.5.3 Auditing for Issues

VibeLang's package manager includes a security and quality audit command:

```bash
vibe pkg audit
```

This checks your dependency tree against known vulnerability databases and
reports any issues:

```
Auditing dependencies for my_app v0.1.0...

  WARN  json_parser v1.2.0 has known issue CVE-2026-1234
        → fixed in v1.2.1
        → run `vibe pkg update json_parser` to resolve

  OK    http_client v0.8.3 — no known issues

Audit complete: 1 warning, 0 errors.
```

Running `vibe pkg audit` in CI pipelines catches vulnerable dependencies before
they reach production.

### 10.5.4 Deterministic Resolution

VibeLang's dependency resolver is fully deterministic: given the same
`vibe.toml` and `vibe.lock`, it always produces the same dependency tree. This
is not a nice-to-have — it is a core design requirement.

Non-deterministic resolution leads to "works on my machine" bugs where different
developers get different dependency versions. VibeLang eliminates this by
design.

## 10.6 Multi-File Projects

### 10.6.1 Project Layout Conventions

For a project with multiple modules, follow this convention:

```
pipeline_app/
├── vibe.toml
├── src/
│   ├── main.yb              → module pipeline_app
│   ├── reader.yb            → module pipeline_app.reader
│   ├── processor.yb         → module pipeline_app.processor
│   └── writer.yb            → module pipeline_app.writer
└── tests/
    ├── reader_test.yb
    ├── processor_test.yb
    └── writer_test.yb
```

Each `.yb` file declares its module and exports the symbols other modules need.

### 10.6.2 Building Multi-Module Programs

The VibeLang compiler resolves modules automatically based on the project
structure and `vibe.toml`. To build:

```bash
vibe build
```

The compiler:

1. Reads `vibe.toml` to find the project root and entry point.
2. Discovers all `.yb` files under `src/`.
3. Parses `module` declarations and `import` statements.
4. Checks for import cycles (rejected with a clear error).
5. Type-checks all modules, verifying that imported symbols are `pub`.
6. Compiles and links into a single binary.

### 10.6.3 Example: A Data Pipeline Project

Let's build a complete multi-module project that reads data, processes it, and
writes results.

**`vibe.toml`:**

```toml
[package]
name = "pipeline_app"
version = "0.1.0"
edition = "2026"
```

**`src/reader.yb`** — reads raw data:

```vibe
module pipeline_app.reader

pub read_scores() -> List<Int> {
    @effect alloc
    @ensure .len() > 0

    [85, 92, 78, 95, 88, 67, 73, 91, 82, 76]
}
```

**`src/processor.yb`** — transforms data:

```vibe
module pipeline_app.processor

pub passing_only(scores: List<Int>, threshold: Int) -> List<Int> {
    @effect alloc
    @require threshold >= 0
    @ensure .len() <= scores.len()

    mut result : List<Int> := []
    for score in scores {
        if score >= threshold {
            result.append(score)
        }
    }
    result
}

pub compute_stats(scores: List<Int>) -> Map<Str, Int> {
    @effect alloc
    @require scores.len() > 0, "need at least one score"

    mut total := 0
    mut best := scores.get(0)
    mut worst := scores.get(0)

    for s in scores {
        total = total + s
        if s > best { best = s }
        if s < worst { worst = s }
    }

    mut stats : Map<Str, Int> := {}
    stats.set("count", scores.len())
    stats.set("total", total)
    stats.set("mean", total / scores.len())
    stats.set("best", best)
    stats.set("worst", worst)
    stats
}
```

**`src/writer.yb`** — formats and outputs results:

```vibe
module pipeline_app.writer

pub print_stats(stats: Map<Str, Int>) -> Int {
    print("=== Score Statistics ===")
    for (key, value) in stats {
        print(key + ": " + value.to_str())
    }
    0
}
```

**`src/main.yb`** — the entry point that ties everything together:

```vibe
module pipeline_app

import pipeline_app.reader
import pipeline_app.processor
import pipeline_app.writer

pub main() -> Int {
    @effect alloc

    scores := reader.read_scores()
    passing := processor.passing_only(scores, 75)
    stats := processor.compute_stats(passing)
    writer.print_stats(stats)
}
```

Build and run:

```bash
vibe build
./pipeline_app
```

Output:

```
=== Score Statistics ===
count: 7
total: 611
mean: 87
best: 95
worst: 76
```

This example demonstrates the key principles: each module has a focused
responsibility, public APIs are marked with `pub` and documented with contracts,
and the main module orchestrates the pipeline by importing and composing the
other modules.

## 10.7 Summary

VibeLang's module and package system is built on a few clear principles:

- **Modules** organize code into named units. The `module` declaration at the
  top of each file establishes its identity, and the directory structure should
  mirror the module hierarchy.

- **Imports** bring other modules' public symbols into scope. Name resolution
  follows a deterministic precedence: local scope, then module scope, then
  imports.

- **Visibility** is private by default. The `pub` keyword explicitly marks
  symbols as part of a module's public API. This forces intentional API design
  and prevents accidental coupling to implementation details.

- **Packages** are collections of modules with a `vibe.toml` manifest. SemVer
  dependencies ensure compatibility, and the lockfile guarantees deterministic
  resolution.

- **`vibe pkg`** commands handle installation, updates, and security auditing.
  Deterministic resolution means every developer gets the same dependency tree.

With modules and packages in hand, you can build large, well-organized VibeLang
projects. In the next chapter, we'll explore VibeLang's concurrency model —
where modules become especially important for isolating concurrent components
and reasoning about which effects cross module boundaries.
