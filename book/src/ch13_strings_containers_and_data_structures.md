# Chapter 13: Strings, Containers, and Data Structures

This chapter covers VibeLang’s data-structure baseline: text (`Str`), dynamic
sequences (`List<T>`), and maps (`Map<K,V>`), with emphasis on semantics and
deterministic behavior.

## 13.1 Data-Structure Philosophy

VibeLang favors practical built-ins with explicit behavior contracts:

- deterministic ordering policies where order is observable,
- clear mutation requirements (`mut`),
- explicit allocation/effect reasoning,
- safe boundary movement via sendability checks.

## 13.2 Strings (`Str`) Basics

From the text semantics model:

- `Str` stores UTF-8 encoded text,
- APIs should distinguish byte length vs higher-level text units,
- default `len(str)` is byte length unless documented otherwise,
- equality defaults to byte-level equality.

Example:

```txt
pub greeting(name: Str) -> Str {
  "hello, " + name
}
```

## 13.3 Escapes and Unicode

String escape support includes:

- `\\`, `\"`
- `\n`, `\r`, `\t`
- `\u{...}`

Invalid escapes are parse errors, which keeps text correctness issues visible
early.

## 13.4 String Indexing and Slicing Caution

String indexing is sensitive because UTF-8 boundaries matter. The spec emphasizes
deterministic error behavior for invalid boundary access.

Practical rule:

- use higher-level helpers where possible,
- avoid assuming one byte equals one visible character.

## 13.5 Lists

List construction:

```txt
values := [1, 2, 3]
empty := []
```

Common operations:

- append/push,
- index get/set,
- iteration.

Mutation requires mutable binding:

```txt
mut xs := [1, 2, 3]
xs[0] = 10
```

## 13.6 Maps

Map construction:

```txt
scores := {"alice": 10, "bob": 12}
```

Core map operations include:

- get/contains,
- insert/update,
- remove,
- iteration.

Map key behavior requires deterministic hash/equality behavior for key type.

## 13.7 Deterministic Ordering Guarantees

Data-structure determinism is a first-class concern:

- list iteration is insertion/index order,
- map iteration must be deterministic for same inputs/toolchain/runtime.

For implementation-freeze contexts, insertion-order policies are explicitly
documented in container conformance reports.

## 13.8 Complexity Baselines

Expected baseline costs:

- list append: amortized O(1),
- list index read/write: O(1),
- map get/insert/remove: expected O(1) average.

Do not treat these as excuses for unbounded memory growth; complexity and
allocation behavior must be considered together.

## 13.9 Allocation Awareness in Data Structures

Container operations often imply `@effect alloc`. Write effects deliberately and
use builders/structured transforms to avoid accidental churn.

Example:

```txt
pub collect_non_negative(xs: List<i64>) -> List<i64> {
  @effect alloc
  mut out := []
  for x in xs {
    if x >= 0 {
      out.append(x)
    }
  }
  out
}
```

## 13.10 Mutation Rules Recap

From mutability model:

- immutable bindings cannot be reassigned,
- field/index mutation requires mutable context,
- contracts are pure-expression contexts and should not mutate data.

This matters for data structures because mutation-heavy code is often the source
of subtle regressions.

## 13.11 Equality and Comparison

Data-structure equality principles:

- list equality is order-sensitive element-wise equality,
- map equality is key/value equality independent of iteration order.

For strings, `==` compares byte representation unless explicit
normalization/collation APIs are used.

## 13.12 Boundary Safety with Containers

When containers cross concurrency/thread boundaries:

- payload members must be sendable,
- concurrent unsynchronized mutation is invalid in safe mode.

In practice, transfer ownership through channels or keep mutation centralized.

## 13.13 Practical Patterns

### Pattern A: Build then publish

Accumulate mutable local container, publish immutable result at boundary.

### Pattern B: Stream through channels

Use channels for staged processing instead of shared mutable container mutation.

### Pattern C: Contract data shape

Use `@ensure` to validate output shape:

```txt
@ensure len(.) <= len(xs)
```

## 13.14 Common Mistakes

1. using string indexing as if all text were fixed-width characters,
2. mutating containers through aliases across concurrent boundaries,
3. relying on undocumented ordering assumptions,
4. hiding allocation-heavy transforms without `@effect alloc`,
5. treating map missing-key behavior as implicit instead of explicit.

## 13.15 Extended Example

```txt
pub non_negative(xs: List<i64>) -> List<i64> {
  @intent "return all non-negative values preserving input order"
  @require len(xs) >= 0
  @ensure len(.) <= len(xs)
  @effect alloc

  mut out := []
  for x in xs {
    if x >= 0 {
      out.append(x)
    }
  }
  out
}
```

Even in conceptual form, note how intent/contracts/effects and container ops
work together.

## 13.16 Clarification: Data Structures Are Semantics, Not Just Storage

A common mistake is to treat `Str`, `List`, and `Map` as neutral containers with
only API ergonomics implications. In VibeLang, they are semantic surfaces:
ordering guarantees, mutation legality, boundary sendability, and allocation
behavior all carry correctness implications.

That is why this chapter links data structures to contracts, effects, and
concurrency boundaries. If those links remain explicit, data-heavy code stays
auditable and predictable as the system grows.

## 13.17 Chapter Checklist

You should now be able to:

- reason about `Str`, `List<T>`, and `Map<K,V>` semantics,
- design deterministic container workflows,
- account for mutation and allocation behavior explicitly,
- avoid common Unicode and ordering pitfalls.

---

Next: Chapter 14 covers modules, imports, and visibility rules.
