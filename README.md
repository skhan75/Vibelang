# VibeLang

<!-- markdownlint-disable MD033 -->
<p align="center">
  <img src="assets/branding/vibelang-mascot-hero.png" alt="VibeLang" width="420" />
</p>

<p align="center">
  <strong>The native-first language for intent-driven development.</strong><br />
  <sub>Deterministic AOT compilation &middot; First-class contracts &middot; Structured concurrency</sub>
</p>

<p align="center">
  <a href="https://github.com/skhan75/VibeLang/releases/tag/v1.0.0"><img src="https://img.shields.io/badge/release-v1.0.0-22c55e" alt="release" /></a>
  <a href="#performance"><img src="https://img.shields.io/badge/27x_faster-than_Go-00F5D4" alt="performance" /></a>
  <img src="https://img.shields.io/badge/native-AOT-2563eb" alt="native" />
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache_2.0-blue.svg" alt="license" /></a>
</p>

<p align="center">
  <a href="#quickstart">Quickstart</a> &middot;
  <a href="#performance">Performance</a> &middot;
  <a href="book/">The Book</a> &middot;
  <a href="docs/spec/">Language Spec</a> &middot;
  <a href="CONTRIBUTING.md">Contributing</a>
</p>
<!-- markdownlint-enable MD033 -->

---

VibeLang is a statically typed, natively compiled language where **intent is a
first-class citizen**. You write what a function should do (`@intent`), what it
promises (`@require`, `@ensure`), how to test it (`@examples`), and what side
effects it has (`@effect`) — and the compiler turns all of that into executable
checks and native code.

```
pub clamp_percent(done: Int, total: Int) -> Int {
  @intent "return completion percentage clamped to [0, 100]"
  @examples {
    clamp_percent(0, 10)  => 0
    clamp_percent(5, 10)  => 50
    clamp_percent(10, 10) => 100
  }
  @require total > 0
  @ensure . >= 0
  @ensure . <= 100

  raw := (done * 100) / total
  if raw < 0 { 0 }
  else if raw > 100 { 100 }
  else { raw }
}
```

**Why this matters:** as AI agents generate more production code, the gap between
"compiles" and "correct" widens. VibeLang closes it with language-level guardrails
that work whether code is written by humans, copilots, or autonomous agents.

## Quickstart

### Install

**Packaged binary** (no Cargo required) — download from
[Releases](https://github.com/skhan75/VibeLang/releases/tag/v1.0.0):

```bash
tar xzf vibe-x86_64-unknown-linux-gnu.tar.gz
sudo mv vibe /usr/local/bin/
vibe --version
```

Platform guides: [Linux](docs/install/linux.md) · [macOS](docs/install/macos.md) · [Windows](docs/install/windows.md)

**From source** — requires [Rust](https://rustup.rs/) (stable) and a C linker:

```bash
git clone https://github.com/skhan75/VibeLang.git
cd VibeLang
cargo build --release -p vibe_cli
export PATH="$PWD/target/release:$PATH"
```

### Hello World

```bash
cat > hello.yb <<'EOF'
pub main() -> Int {
  @effect io
  println("hello from vibelang")
  0
}
EOF

vibe run hello.yb
```

### Developer loop

```bash
vibe new myproject && cd myproject
vibe run main.yb            # build + execute
vibe test main.yb           # run tests including @examples
vibe fmt . --check          # check formatting
vibe lint . --intent        # AI-powered drift detection (optional)
```

## Performance

18 benchmarks from the [PLB-CI](benchmarks/third_party/plbci/) suite, measured
with Hyperfine on AMD Ryzen 9 5900X (24 cores, 32 GB RAM).

| vs | Speedup (geomean) | Benchmarks |
|---|---|---|
| Python | **100x** | 16 |
| TypeScript | **71x** | 12 |
| Go | **27x** | 18 |
| C | **10.7x** | 5 |
| C++ | **10.5x** | 5 |

| | VibeLang | Go | Python |
|---|---|---|---|
| binarytrees | **1.53 ms** | 199 ms | 372 ms |
| spectral-norm | **1.40 ms** | 102 ms | 2,078 ms |
| http-server | **2.29 ms** | 89 ms | 1,700 ms |
| coro-prime-sieve | **1.64 ms** | 12.3 ms | 317 ms |

Average memory footprint: **4.3 MB** (vs Go 9.5 MB, Python 27 MB, TypeScript 74.5 MB).

Full results and methodology: [`reports/benchmarks/`](reports/benchmarks/third_party/full/summary.md)

## Key Features

### Contracts and intent

Every function can declare its purpose, preconditions, postconditions, test cases,
and side effects — all checked by the compiler or runtime.

| Annotation | What it does |
|---|---|
| `@intent "..."` | Natural-language purpose; checked by AI sidecar for drift |
| `@require expr` | Precondition — verified at function entry |
| `@ensure expr` | Postcondition — `.` is the return value, `old()` snapshots pre-state |
| `@examples { f(x) => y }` | Executable test cases via `vibe test` |
| `@effect tag` | Declares side effects (`io`, `alloc`, `mut_state`, `concurrency`, `nondet`) — tracked transitively |

### Structured concurrency

```
done := chan(4)
go worker(3, done)
go worker(7, done)
result := done.recv()

select {
  case msg := inbox.recv() => handle(msg)
  case after 5 => timeout()
}
```

`go` spawns tasks, `chan` creates typed channels, `select` multiplexes with
timeouts. The compiler tracks `@effect concurrency` transitively and checks
sendability at compile time.

### Deterministic builds

Same source + same flags = same binary. Diagnostics are deterministically ordered.
Release artifacts include checksums, signatures, provenance, and SBOM.

## Documentation

| Resource | What it covers |
|---|---|
| [The Book](book/) | 20-chapter learning path from basics to production patterns |
| [Language Spec](docs/spec/) | Normative grammar, type system, semantics, memory model |
| [CLI Manual](docs/cli/help_manual.md) | All commands, flags, and exit codes |
| [Stdlib Reference](docs/stdlib/reference_index.md) | Module index with stability tiers |
| [Examples](examples/) | 70 programs across 10 categories |
| [Architecture](docs/charter.md) | Mission, principles, and design guardrails |

## Toolchain

One binary, nine commands:

```
vibe check   — type-check and validate contracts
vibe build   — compile to native binary
vibe run     — build and execute
vibe test    — run tests including @examples
vibe fmt     — format source code
vibe doc     — generate API documentation
vibe lint    — intent drift detection (optional AI sidecar)
vibe pkg     — dependency management
vibe lsp     — language server for editors
```

VS Code extension with syntax highlighting and LSP: [`editor-support/vscode/`](editor-support/vscode/)

## Project Status

| Area | Status |
|---|---|
| Compiler (lex → parse → type-check → HIR → MIR → Cranelift → native) | Shipped |
| Contracts (`@intent`, `@require`, `@ensure`, `@examples`, `@effect`) | Shipped |
| Concurrency (`go`, `chan`, `select`, cancellation) | Shipped |
| Stdlib (io, core, path, time, fs, json, http) | Shipped (stable + preview) |
| Toolchain (check, build, run, test, fmt, doc, lint, pkg, lsp) | Shipped |
| Packaged releases (checksums, signatures, provenance, SBOM) | Shipped |
| Deeper AI autocomplete | In progress |
| Self-hosting compiler | In progress |
| ARM64 and WASM targets | Planned |

Detailed tracker: [`docs/development_checklist.md`](docs/development_checklist.md) ·
Feature gaps: [`docs/checklists/features_and_optimizations.md`](docs/checklists/features_and_optimizations.md)

## Contributing

See **[CONTRIBUTING.md](CONTRIBUTING.md)** for the full guide. The short version:

```bash
git clone https://github.com/<you>/VibeLang.git && cd VibeLang
cargo build --release -p vibe_cli
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p vibe_cli
```

**Principles:** determinism first · tests required · no AI in the compile path · clear code over clever code

**Where to start:** issues labeled `good first issue` · the [feature gaps checklist](docs/checklists/features_and_optimizations.md) · the [development checklist](docs/development_checklist.md)

## License

[Apache License 2.0](LICENSE) — Copyright 2025–2026 VibeLang Contributors
