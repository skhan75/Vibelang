# Contributing to VibeLang

Thank you for your interest in contributing to VibeLang. This guide covers
everything you need to get started.

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- A C linker: `build-essential` (Debian/Ubuntu), `clang`, or Xcode Command Line Tools (macOS)
- Git

## Getting Started

```bash
# 1. Fork the repository on GitHub, then clone your fork
git clone https://github.com/<your-username>/VibeLang.git
cd VibeLang

# 2. Build the compiler
cargo build --release -p vibe_cli

# 3. Add the binary to your PATH
export PATH="$PWD/target/release:$PATH"

# 4. Verify it works
vibe --version
vibe run examples/01_basics/01_hello_world.yb
```

## Development Workflow

### Before you start

1. Check [existing issues](https://github.com/skhan75/VibeLang/issues) to avoid duplicate work
2. For non-trivial changes, open an issue first to discuss the approach
3. Issues labeled [`good first issue`](https://github.com/skhan75/VibeLang/labels/good%20first%20issue) are a great starting point

### Making changes

1. Create a feature branch from `main`:
   ```bash
   git checkout -b feature/your-change
   ```

2. Make your changes. Follow the project conventions:
   - Rust code: follow `rustfmt` defaults
   - VibeLang examples: follow the style in `examples/`
   - Documentation: keep it concise and actionable

3. Run the validation suite before submitting:
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test -p vibe_cli
   ```

4. If you modified the compiler, verify examples still work:
   ```bash
   vibe run examples/01_basics/01_hello_world.yb
   vibe check examples/10_contracts_intent/63_all_annotations_combo.yb
   ```

### Submitting a pull request

1. Push your branch and open a PR against `main`
2. Fill in the PR template with:
   - What changed and why
   - How to test the change
   - Any breaking changes or migration notes
3. Ensure CI passes — all checks must be green before merge

## Project Structure

```
vibelang/
├── crates/              # Rust compiler crates (17 crates)
│   ├── vibe_lexer/      # Tokenization
│   ├── vibe_parser/     # Parsing
│   ├── vibe_ast/        # AST definitions
│   ├── vibe_types/      # Type checking + effect inference
│   ├── vibe_hir/        # High-level IR
│   ├── vibe_mir/        # Mid-level IR
│   ├── vibe_codegen/    # Cranelift code generation
│   ├── vibe_runtime/    # C runtime library
│   ├── vibe_cli/        # CLI entry point
│   └── ...              # fmt, doc, pkg, lsp, indexer, sidecar, diagnostics
├── compiler/            # Compiler documentation and test fixtures
├── runtime/             # Runtime system (native, concurrency, GC)
├── stdlib/              # Standard library modules
├── examples/            # 70 example programs across 10 categories
├── docs/                # Documentation
├── book/                # The VibeLang Book (mdBook)
├── editor-support/      # VS Code extension
├── benchmarks/          # Performance benchmark suite
└── reports/             # Generated benchmark and quality reports
```

## What to Work On

### Good first contributions

- Fix a bug from the [issue tracker](https://github.com/skhan75/VibeLang/issues)
- Add a new example program to `examples/`
- Improve documentation or fix typos
- Add a missing test case for an existing feature

### Intermediate contributions

- Implement a feature from the [development checklist](docs/development_checklist.md)
- Address a gap in the [feature gaps checklist](examples/FEATURE_GAPS_CHECKLIST.md)
- Improve error messages in `vibe_diagnostics`
- Add a new stdlib module or extend an existing one

### Advanced contributions

- Compiler optimizations (MIR passes, inlining, DCE)
- New codegen targets
- LSP feature improvements
- Self-hosting compiler components

## Coding Principles

1. **Determinism first** — same source + same flags = same binary. Never introduce
   non-determinism into the compile path.

2. **Tests required** — every change must include or update tests. The CI gate
   enforces this.

3. **No AI in the compile path** — the AI sidecar is optional. Compilation must
   always work offline without any AI service.

4. **Keep it readable** — clear code over clever code. If a comment is needed to
   explain what the code does, the code should be rewritten.

5. **Deterministic diagnostics** — error messages must be stable and sorted. Golden
   test fixtures enforce this.

## Secrets and credentials

- **Never commit secrets**: `.env`, API keys, access tokens, private keys, credentials, or any
  files that contain them.
- **Use local env files**: if you need local configuration, use `.env` in your working copy and
  keep it untracked. The repository ignores `.env` by default.
- **If a secret is committed**: treat it as compromised, revoke/rotate it immediately, then
  remove it from the repository and add prevention (CI secret scanning, tighter ignores).

## Testing

### Run all tests

```bash
cargo test -p vibe_cli
```

### Run a specific example

```bash
vibe run examples/04_algorithms/25_bubble_sort.yb
```

### Run contract/example tests

```bash
vibe test examples/10_contracts_intent/63_all_annotations_combo.yb
```

### Check formatting and lints

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Commit Messages

Use clear, descriptive commit messages:

```
<area>: <short description>

<optional longer explanation>
```

Examples:
- `lexer: handle escaped unicode in string literals`
- `cli: add --profile flag to vibe build`
- `examples: add graph BFS example`
- `docs: update installation guide for macOS ARM64`

## License

By contributing to VibeLang, you agree that your contributions will be licensed
under the [Apache License 2.0](LICENSE).
