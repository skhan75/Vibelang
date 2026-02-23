# Chapter 7: Tooling Workflow

VibeLang’s CLI is designed as an integrated system rather than a loose bundle of
separate tools. This chapter shows how to use the toolchain as a daily
engineering loop.

## 7.1 Core Commands at a Glance

Key commands:

- `vibe check <path>` - parse + type-check + diagnostics
- `vibe build <path>` - native build artifacts
- `vibe run <path>` - build and execute
- `vibe test <path|dir>` - run tests (including generated example checks)
- `vibe fmt [path]` - formatting
- `vibe doc [path]` - API documentation generation
- `vibe lint --intent` - intent-aware linting
- `vibe pkg ...` - package lifecycle operations
- `vibe lsp` - language server
- `vibe index` - semantic indexing

## 7.2 Recommended Local Loop

```bash
vibe check src/main.yb
vibe test src/
vibe lint . --intent --changed
vibe build src/main.yb --profile release
vibe run src/main.yb
```

For teams, this sequence catches most regressions before CI.

## 7.3 Build and Run Flags You Should Know

`build` supports profile and target options:

```bash
vibe build main.yb --profile release --target x86_64-unknown-linux-gnu
```

Useful principles:

- choose target explicitly in cross-platform pipelines,
- keep profile policy explicit in CI and release scripts,
- avoid hidden environment dependence.

## 7.4 Test Strategy with `@examples`

Because `@examples` lower into executable tests, `vibe test` naturally combines:

- hand-authored tests,
- generated example-based behavior checks.

This helps teams keep documentation and test intent in sync.

## 7.5 Formatting and Documentation in CI

Typical quality gates:

```bash
vibe fmt . --check
vibe doc . --out docs/api.md
```

This ensures style and documentation outputs remain reproducible across machines.

## 7.6 Intent Lint in Practice

Use intent lint where it is most valuable:

- changed code paths,
- high-risk refactors,
- AI-assisted generated patches.

Example:

```bash
vibe lint . --intent --changed
```

Design note: intent lint is advisory by default and should not replace the
deterministic compiler/runtime correctness path.

## 7.7 Indexer and LSP

Large codebases benefit from semantic indexing:

```bash
vibe index . --stats
vibe lsp --transport jsonrpc
```

Use index telemetry to watch scaling behavior and keep editor responsiveness
healthy.

## 7.8 Package Lifecycle (`vibe pkg`)

Package tooling supports operations such as:

- resolve,
- lock,
- install,
- publish,
- audit,
- semver checks and upgrade planning.

Treat package lock and dependency audits as part of release reliability, not
optional extras.

## 7.9 CLI Help/Version as Contract

In mature ecosystems, help and version output are part of UX stability.
VibeLang includes dedicated validation for this to prevent accidental CLI drift.

Use:

```bash
vibe --help
vibe help build
vibe --version --json
```

when integrating tooling wrappers or CI plugins.

## 7.10 Troubleshooting by Stage

When commands fail, localize by stage:

1. **check fails** -> syntax/type/contracts/effects issue,
2. **build fails** -> codegen/runtime/link/profile issue,
3. **run fails** -> runtime semantics/input/path/environment,
4. **test fails** -> behavior regression or generated example mismatch,
5. **lint fails** -> intent drift or metadata quality issue.

This stage-based approach shortens diagnosis time significantly.

## 7.11 Golden Path CI Blueprint

A practical CI sequence:

1. formatting/doc checks,
2. check + test suites,
3. intent lint lane,
4. release build lane,
5. reproducibility/quality budget validation lanes.

This mirrors the release-quality engineering philosophy used across the project.

## 7.12 Clarification: CLI Workflow Is Part of the Language Experience

In VibeLang, the CLI is not just a thin wrapper around compilation. It is the
operational surface that keeps syntax, contracts, effects, diagnostics, and
release confidence connected. That is why this chapter emphasizes command
sequence and stage-based debugging in detail.

When teams skip this discipline, they often end up with "works on one machine"
patterns even when the language itself is deterministic. Following the workflow
consistently is what turns language guarantees into reliable engineering
outcomes.

## 7.13 Chapter Checklist

You should now be able to:

- run the complete day-to-day VibeLang CLI loop,
- configure build/test/lint sequences for CI,
- use index/LSP features in larger repos,
- reason about where a failure belongs in the pipeline.

---

Next: Chapter 8 explains intent-driven development and sidecar design in depth.
