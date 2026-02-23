# VibeLang Cost Model (v1.0)

Status: normative guidance for copies, allocations, and concurrency costs.

## Objective

Provide predictable performance reasoning for user code and release governance.

## Copies

- Primitive scalar assignments are copy-by-value and expected O(1).
- `Str`, `List`, and `Map` copy behavior must be explicit in API docs:
  - full copy semantics where required by operation contract,
  - aliasing/ownership transfer semantics where supported.
- APIs that can trigger full-container copies must document this explicitly.

## Allocations

- Allocation is represented as `@effect alloc`.
- Container growth, string concatenation, and builder expansion may allocate.
- Allocation-heavy APIs should prefer amortized-growth strategies.
- Build/profile outputs must provide allocation visibility artifacts so allocation
  hotspots are observable and auditable.

## Concurrency Operations

- `go`/`thread` spawn has non-zero scheduling overhead.
- Channel operations (`send`, `recv`, `select`) may block and introduce
  scheduling latency.
- Concurrency correctness constraints (sendability/synchronization) take
  priority over micro-optimizations.

## Complexity Baselines

- List append: amortized O(1)
- List index get/set: O(1)
- Map get/insert/remove: expected O(1) average case
- Channel send/recv: expected O(1) per operation excluding blocking wait time

## Profile Expectations

- `dev`: richer diagnostics/debug metadata, higher overhead acceptable.
- `release`: lower overhead, deterministic behavior preserved.
- Contract policy differences across profiles must be explicit and deterministic.

## Release Benchmark Expectations

Each RC/GA cycle must publish benchmark artifacts that include:

- latency metrics (compile/runtime paths),
- memory metrics (index/runtime memory signals),
- CPU metrics (release benchmark lane).

Reports should be linked from `reports/v1/` and validated by CI gates.
