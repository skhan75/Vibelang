# Incremental Compilation Strategy (v0.1)

## Objectives

- Rebuild only what changed
- Keep cache invalidation correct and conservative
- Preserve deterministic outputs

## Cache Units

Granularity:

- File-level parse cache
- Declaration-level type cache
- Function-level IR cache
- Module-level codegen object cache

## Cache Keys

Each unit key includes:

- Content hash of source fragment
- Toolchain version hash
- Dependency signature hash
- Relevant compiler flags/profile hash
- Contract hash (for functions with annotations)

## Invalidation Rules

Invalidate unit when:

- Source hash changes
- Exported signature dependency changes
- Contract/effect metadata changes
- Compiler version or flags change

Do not invalidate unrelated modules/functions.

## Dependency Tracking

Track both:

- **Syntactic deps**: imports and symbol references
- **Semantic deps**: inferred type relationships and effect summaries

This avoids stale artifacts when type/effect assumptions change.

## Pipeline with Incrementality

```mermaid
flowchart LR
  edit[FileEdit] --> hash[HashDiff]
  hash --> parseCache[ParseCacheLookup]
  parseCache --> typeCache[TypeCacheLookup]
  typeCache --> irCache[IRCacheLookup]
  irCache --> objCache[ObjectCacheLookup]
  objCache --> link[IncrementalLink]
```

## Contract-Aware Incrementality

Contracts impact invalidation:

- `@examples` changes invalidate generated test node
- `@ensure/@require` changes invalidate function HIR/MIR
- `@effect` changes invalidate effect summaries and dependent diagnostics

## Storage

- Local disk cache in `.vibe/cache`
- Content-addressed blobs for IR and object artifacts
- LRU eviction with size cap

## Failure Safety

- Corrupted cache entries are discarded and rebuilt.
- Cache misses never fail build; they only impact speed.

## Measurement and Telemetry

Record:

- cache hit rate by stage
- invalidation fan-out size
- incremental build latency distribution

## Targets (v0.1)

- Parse cache hit rate: over 90% in small edit loops
- Single-function edit invalidation fan-out: ideally under 5 dependent functions
- Incremental check latency: under 500 ms median for one-file edit
