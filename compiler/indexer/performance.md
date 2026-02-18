# VibeLang Indexer Performance Baseline (Phase 4)

## Goals

Phase 4 performance gates focus on three measurements emitted by `vibe index --stats`:

- `cold_ms`: end-to-end index build latency for target path
- `incremental_ms`: single-file incremental update latency
- `memory_bytes`: estimated in-memory index footprint

## Local Benchmark Command

Run against the fixture corpus:

```bash
cargo run -q -p vibe_cli -- index compiler/tests/fixtures --rebuild --stats
```

Example output:

```txt
index stats: files=20 symbols=97 references=58 function_meta=28 diagnostics=21 cold_ms=7 incremental_ms=0 memory_bytes=24355 memory_ratio=8.7044 root=compiler/tests/fixtures/.vibe/index
```

## CI Smoke Thresholds (Phase 4 Baseline)

Current CI smoke thresholds in `.github/workflows/phase4-indexer-lsp.yml`:

- `cold_ms <= 15000`
- `incremental_ms <= 5000`
- `memory_bytes <= 50000000`

These are intentionally permissive for early stabilization and should be tightened as:

- corpus size increases
- index data structures are optimized
- reference hardware profile is formalized

## Determinism Requirement

Index snapshots must be deterministic for identical source and toolchain:

```bash
cargo run -q -p vibe_cli -- index compiler/tests/fixtures/snapshots --rebuild
cargo run -q -p vibe_cli -- index compiler/tests/fixtures/snapshots --rebuild
diff -u compiler/tests/fixtures/snapshots/.vibe/index/index_v1.json compiler/tests/fixtures/snapshots/.vibe/index/index_v1.json
```

The deterministic check is also enforced in `phase4-indexer-lsp.yml`.
