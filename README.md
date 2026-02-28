<p align="center">
  <img src="assets/branding/vibelang-mascot-hero.png" alt="VibeLang" width="360" />
</p>

<h1 align="center">VibeLang</h1>

<p align="center">
  <strong>Code with Intent.</strong><br />
  <sub>Native AOT compilation · First-class contracts · Structured concurrency · AI-era guardrails</sub>
</p>

<p align="center">
  <a href="https://github.com/skhan75/VibeLang/releases/tag/v1.0.0"><img src="https://img.shields.io/badge/release-v1.0.0-22c55e" alt="release" /></a>
  <a href="#performance"><img src="https://img.shields.io/badge/27×_faster-than_Go-00f0ff" alt="performance" /></a>
  <a href="#performance"><img src="https://img.shields.io/badge/100×_faster-than_Python-b026ff" alt="performance" /></a>
  <a href="#performance"><img src="https://img.shields.io/badge/4.3_MB-avg_memory-ff2d95" alt="memory" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache_2.0-blue.svg" alt="license" /></a>
</p>

<p align="center">
  <a href="https://www.thevibelang.org">Website</a> ·
  <a href="https://www.thevibelang.org/documentation">Documentation</a> ·
  <a href="#quickstart">Quickstart</a> ·
  <a href="#performance">Performance</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

---

VibeLang is a statically typed, natively compiled language where **intent is a first-class citizen**. You declare what a function should do, what it promises, how to test it, and what side effects it has — the compiler turns all of that into executable checks and native code.

As AI agents generate more production code, the gap between "compiles" and "correct" widens. VibeLang closes it with language-level guardrails that work whether code is written by humans, copilots, or autonomous agents.

```vibelang
pub transfer(from: Account, to: Account, amount: Int) -> Result<Receipt, BankError> {
  @intent "move funds between accounts preserving total balance"
  @require amount > 0
  @require from.balance >= amount
  @ensure to.balance == old(to.balance) + amount
  @ensure from.balance == old(from.balance) - amount
  @effect io

  from.balance = from.balance - amount
  to.balance   = to.balance + amount
  ok(Receipt { from: from.id, to: to.id, amount })
}
```

## Quickstart

**Download a binary** — no Cargo required:

```bash
# From GitHub Releases
tar xzf vibe-x86_64-unknown-linux-gnu.tar.gz
sudo mv vibe /usr/local/bin/
vibe --version
```

**Or build from source** (requires [Rust stable](https://rustup.rs/) + a C linker):

```bash
git clone https://github.com/skhan75/VibeLang.git && cd VibeLang
cargo build --release -p vibe_cli
export PATH="$PWD/target/release:$PATH"
```

**Create and run a project:**

```bash
vibe new myproject && cd myproject
vibe run main.yb          # compile + execute
vibe test main.yb         # run tests including @examples
vibe fmt . --check        # check formatting
vibe lint . --intent      # AI-powered drift detection (optional)
```

Platform guides: [Linux](docs/install/linux.md) · [macOS](docs/install/macos.md) · [Windows](docs/install/windows.md)

## Performance

18 benchmarks from the [PLB-CI](benchmarks/third_party/plbci/) suite, measured with Hyperfine on AMD Ryzen 9 5900X.

| vs Language | Speedup (geomean) |
|---|---|
| Python | **100×** |
| TypeScript | **71×** |
| Go | **27×** |
| C | **10.7×** |
| C++ | **10.5×** |

| Benchmark | VibeLang | Go | Python |
|---|---|---|---|
| binarytrees | **1.53 ms** | 199 ms | 372 ms |
| spectral-norm | **1.40 ms** | 102 ms | 2,078 ms |
| http-server | **2.29 ms** | 89 ms | 1,700 ms |
| coro-prime-sieve | **1.64 ms** | 12.3 ms | 317 ms |

Average memory: **4.3 MB** (Go 9.5 MB · Python 27 MB · TypeScript 74.5 MB)

Full methodology: [`reports/benchmarks/`](reports/benchmarks/third_party/full/summary.md)

## Contracts & Intent

| Annotation | Purpose |
|---|---|
| `@intent "..."` | Natural-language purpose; checked by AI sidecar for drift |
| `@require expr` | Precondition — verified at function entry |
| `@ensure expr` | Postcondition — `.` is the return value, `old()` snapshots pre-state |
| `@examples { f(x) => y }` | Executable test cases via `vibe test` |
| `@effect tag` | Side effects (`io`, `alloc`, `mut_state`, `concurrency`, `nondet`) — tracked transitively |

## Concurrency

```vibelang
results := chan(4)
go worker(3, results)
go worker(7, results)
first := results.recv()

select {
  case msg := inbox.recv() => handle(msg)
  case after 5 => timeout()
}
```

`go` spawns tasks, `chan` creates typed channels, `select` multiplexes with timeouts. The compiler tracks `@effect concurrency` transitively and checks sendability at compile time.

## Toolchain

One binary, nine commands:

| Command | What it does |
|---|---|
| `vibe check` | Type-check and validate contracts |
| `vibe build` | Compile to native binary |
| `vibe run` | Build and execute |
| `vibe test` | Run tests including `@examples` |
| `vibe fmt` | Format source code |
| `vibe doc` | Generate API documentation |
| `vibe lint` | Intent drift detection (optional AI sidecar) |
| `vibe pkg` | Dependency management |
| `vibe lsp` | Language server for editors |

VS Code extension: [`editor-support/vscode/`](editor-support/vscode/)

## Documentation

| Resource | Description |
|---|---|
| [The Book](https://www.thevibelang.org/documentation) | Learning path from basics to production patterns |
| [Language Spec](docs/spec/) | Grammar, type system, semantics, memory model |
| [CLI Manual](docs/cli/help_manual.md) | Commands, flags, and exit codes |
| [Stdlib Reference](docs/stdlib/reference_index.md) | Module index with stability tiers |
| [Examples](examples/) | 70+ programs across 10 categories |

## Contributing

See **[CONTRIBUTING.md](CONTRIBUTING.md)** for the full guide.

```bash
git clone https://github.com/<you>/VibeLang.git && cd VibeLang
cargo build --release -p vibe_cli
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
cargo test -p vibe_cli
```

**Where to start:** issues labeled `good first issue` · [feature gaps](docs/checklists/features_and_optimizations.md) · [development checklist](docs/development_checklist.md)

## License

[Apache License 2.0](LICENSE) — Copyright 2025–2026 VibeLang Contributors
