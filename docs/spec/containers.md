# VibeLang Containers Spec (v1.0 Target)

Status: normative target.

## 7.3.f.1 Implementation Support Freeze (v1 GA Blocker Scope)

This section freezes the runtime/compiler implementation scope for
`docs/development_checklist.md` section `7.3.f.1` so CI/reporting can evaluate
conformance against a stable target.

### Supported in 7.3.f.1 closeout

- `List<Int>`
  - literal construction (`[]`, `[a, b, ...]`)
  - `.append(value: Int)`
  - `.get(index: Int) -> Int`
  - `.set(index: Int, value: Int)`
  - `.len` / `.len() -> Int`
- `Map<Int, Int>`
  - literal construction (`{k: v, ...}`)
  - `.get(key: Int) -> Int`
  - `.set(key: Int, value: Int)`
  - `.contains(key: Int) -> Bool` (implemented as `Int` truthy/falsey at ABI)
  - `.remove(key: Int) -> Bool` (implemented as `Int` truthy/falsey at ABI)
  - `.len` / `.len() -> Int`
- `Map<Str, Int>`
  - same API as `Map<Int, Int>` with `Str` keys
- `Str`
  - literal construction
  - concatenation via `+` (`Str + Str -> Str`)

### Deterministic ordering policy

- `Map` iteration policy for this implementation freeze is **insertion-order**.
- Update of an existing key MUST keep key position stable.
- Remove + reinsert appends at the end.

### Explicitly deferred beyond 7.3.f.1

- Generic `List<T>` and `Map<K,V>` native lowering beyond the supported type
  combinations above.
- String indexing/slicing APIs.
- Container iterators and native `for in` lowering over dynamic containers.
- Container equality/hash APIs beyond what is needed for current v1 smoke
  matrix.

## Scope

This document defines normative behavior for dynamic container families:

- `Str` (text container, see `strings_and_text.md`)
- `List<T>`
- `Map<K,V>`

## Construction

### List

- Literal construction: `[v1, v2, ...]`
- Empty list literal: `[]` (type inferred from context or explicit annotation)

### Map

- Literal construction: `{k1: v1, k2: v2, ...}`
- Empty map literal: `{}` (type inferred from context or explicit annotation)

### String

- Literal and builder construction defined in `strings_and_text.md`.

## Mutation APIs

Baseline container operations expected in v1 target:

- List:
  - append/push
  - pop
  - index read/write
  - iteration
- Map:
  - insert/update
  - get/contains
  - remove
  - key/value iteration

Mutation requires mutable binding per `mutability_model.md`.

## Indexing Semantics

### List Indexing

- Index type: integer (`usize` preferred by API).
- Out-of-bounds access:
  - checked API returns optional/result form, or
  - unchecked form triggers deterministic runtime trap with diagnostic context.

### Map Indexing

- Key lookup semantics:
  - deterministic hash/equality behavior for key type.
  - missing key behavior must be explicit (optional/result/trap by API).

## Iteration and Ordering

- `List` iteration order is insertion/index order.
- `Map` iteration order MUST be deterministic for identical program input and
  toolchain/runtime version.
- If map ordering is not insertion-order, runtime must still provide deterministic
  stable ordering policy and document it.

## Equality and Hashing

- `List<T>` equality is element-wise order-sensitive equality.
- `Map<K,V>` equality is key-set/value equality independent of iteration order.
- Map key types MUST satisfy hash/equality consistency requirements.

## Complexity Contracts (Baseline)

Runtime SHOULD meet:

- List append: amortized O(1)
- List index get/set: O(1)
- Map get/insert/remove: expected O(1) average case

Complexity guarantees are contractual only where explicitly documented for API.

## Determinism Guarantees

- Container operations that expose ordering MUST be deterministic.
- Resizing/reallocation behavior must not introduce nondeterministic observable
  ordering.
- Diagnostics for invalid container operations must be stable.

## Concurrency and Sendability

- Container values crossing task/thread boundaries must satisfy sendability
  rules in `ownership_sendability.md`.
- Concurrent mutation of same container without synchronization is invalid in
  safe mode.

## Memory and Allocation

- Container growth and copy semantics must be explicit and profile-independent.
- Allocation effects from container operations should be reflected via
  `@effect alloc` where relevant.

## Deferred Notes

- Persistent/immutable container variants are deferred unless explicitly added by
  decision log.
