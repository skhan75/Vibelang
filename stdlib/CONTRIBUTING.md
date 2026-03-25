# Contributing to VibeLang Standard Libraries

This guide explains how to add, modify, and test standard library modules written in VibeLang.

## Directory Layout

```
vibelang/stdlib/
  std/
    path.yb          # module std.path
    encoding.yb      # module std.encoding (future)
    text.yb          # module std.text (future)
  CONTRIBUTING.md    # this file
  stability_policy.md
```

Each `.yb` file under `stdlib/std/` is a self-contained module. The file `std/path.yb` declares `module std.path` and exports `pub` functions and types.

## Adding a New Module

1. Create `stdlib/std/<name>.yb`.
2. Start the file with a **license header** followed by the `module` declaration:

```vibelang
// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0
//
// std.mymod — short description of what this module provides.

module std.mymod
```

3. Mark every public API with `pub`:

```vibelang
module std.mymod

pub type Config {
  key: Str,
  value: Str
}

pub default_config() -> Config {
  Config { key: "", value: "" }
}
```

4. Add tests (see Testing below).
5. Update `stdlib/stability_policy.md` with the new functions and their stability tier.

## Using `@native` for C-backed Functions

When a function needs to call into the C runtime (e.g. for I/O, networking, or system calls), use the `@native` annotation:

```vibelang
module std.net

pub connect(host: Str, port: Int) -> Int {
  @native("vibe_net_connect")
}
```

The `@native("symbol")` annotation tells the compiler to call the named C symbol instead of compiling the function body. The C function must be defined in `runtime/native/vibe_runtime.c` with matching parameter types.

**FFI-compatible types:** `Int`, `Float`, `Bool`, `Str`, and user-defined struct types (passed as pointers).

## Naming Conventions

- Module names: lowercase, single word (`std.path`, `std.text`, `std.encoding`).
- Function names: `snake_case` (`join`, `trim_start`, `hex_encode`).
- Type names: `PascalCase` (`Config`, `ParseResult`).
- Enum names: `PascalCase` with `PascalCase` variants (`Color.Red`).

## Error Handling

Use `Result<T, E>` for operations that can fail:

```vibelang
pub parse_int(s: Str) -> Result<Int, Str> {
  if valid {
    ok(value)
  } else {
    err("invalid integer")
  }
}
```

Callers use `?` to propagate errors:

```vibelang
n := parse_int(input)?
```

Avoid `panic` in stdlib code. Reserve it for truly unrecoverable situations.

## Testing

Tests live alongside examples in `vibelang/examples/`. For stdlib modules, add integration tests that exercise the public API:

```
examples/
  07_stdlib_io_json_regex_http/
    <nn>_<descriptive_name>.yb
```

Each test file should have a `main()` that prints deterministic output. The `phase12_stdlib` test suite in `crates/vibe_cli/tests/phase12_stdlib.rs` verifies these outputs are stable.

## Style Guide

- Keep functions small and focused.
- Prefer pure functions (no side effects) when possible.
- Declare effects with `@effect` when I/O or allocation is needed.
- Use descriptive parameter names (`host` not `h`, `path` not `p`).
- Every public function should have a clear return type annotation.

### Annotations (use VibeLang's full feature set)

Stdlib modules should showcase idiomatic VibeLang. Use annotations wherever they add value:

```vibelang
pub join(a: Str, b: Str) -> Str {
  @intent "concatenate two path segments with a separator when needed"
  @examples {
    join("/usr", "local")  => "/usr/local"
    join("/usr/", "local") => "/usr/local"
    join("", "file.txt")   => "file.txt"
  }
  // implementation ...
}
```

| Annotation | When to use |
|------------|-------------|
| `@intent "..."` | Every `pub` function — one sentence describing what and why. |
| `@examples { ... }` | Pure functions with deterministic I/O. Serves as documentation *and* executable spec. |
| `@require expr` | Functions with preconditions the caller must satisfy. |
| `@ensure expr` | Functions where the output invariant is non-obvious and worth enforcing. |
| `@effect name` | Any function performing I/O, allocation, network, or concurrency. |

Private helper functions may also use `@intent` and `@examples` when it aids readability.

### Comments

- **License header**: Required at the top of every stdlib file (see Adding a New Module above).
- **Module description**: A short `//` comment after the SPDX line explaining the module's purpose and scope.
- **Inline comments**: Use for non-obvious intent — magic numbers (`// ASCII 47 = '/'`), platform-specific behavior, or algorithmic trade-offs. Do not restate the code.

## Stability Tiers

See `stdlib/stability_policy.md` for the full policy. In summary:

| Tier | Meaning |
|------|---------|
| **stable** | Will not change in breaking ways |
| **preview** | API may change based on feedback |
| **experimental** | May be removed entirely |

New modules start at `preview`. Promotion to `stable` requires at least one release cycle with no API changes.

## How the Compiler Finds stdlib

When any source file in a project contains `import std.*`, the compiler automatically discovers `.yb` files under the stdlib directory. Resolution order:

1. `$VIBE_STDLIB_PATH` environment variable (if set)
2. `../stdlib/std/` relative to the `vibe` binary
3. `../../stdlib/std/` relative to the `vibe` binary

This means contributors can test stdlib changes by building VibeLang from source — the binary finds the stdlib relative to itself.
