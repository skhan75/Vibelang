# Chapter 5: Effects and Performance Reasoning

VibeLang makes effects explicit so performance and correctness remain explainable
at code review time, not only in postmortems.

This chapter focuses on:

- effect declarations,
- cost model reasoning,
- profile-aware expectations,
- practical optimization without losing determinism.

## 5.1 Why Effects Exist

Most languages let effectful behavior hide in call chains. VibeLang takes the
opposite approach: make effect boundaries visible.

The current effect vocabulary is:

- `alloc`
- `mut_state`
- `io`
- `net`
- `concurrency`
- `nondet`

Effect declarations are meant to be meaningful signals, not ceremony.

## 5.2 Example: Allocation and I/O Visibility

```txt
pub emit_report(values: List<i64>) -> Int {
  @intent "aggregate values and print summary"
  @effect alloc
  @effect io

  total := sum(values)          // may allocate in helper pipelines
  println("total=" + to_str(total))
  0
}
```

A reviewer can immediately infer:

- this function touches heap behavior,
- this function performs visible I/O.

## 5.3 Effect Drift and Review Quality

Effects create a compact review checklist:

- does implementation perform undeclared behavior?
- are declared effects still needed?
- do transitive calls imply additional effects?

This gives teams a fast "behavior envelope" before line-by-line deep review.

## 5.4 Cost Model Baselines

From the cost model:

- primitive scalar assignment: expected O(1),
- list append: amortized O(1),
- list index get/set: O(1),
- map get/insert/remove: expected O(1) average,
- channel send/recv: expected O(1) excluding blocking wait.

These baselines help you reason early, before microbenching.

## 5.5 Allocation Reasoning Patterns

Common allocation-heavy patterns:

- repeated string concatenation in loops,
- repeated container expansion without capacity awareness,
- copy-heavy transforms in deep pipelines.

Prefer explicit builders or staged transforms when allocation pressure matters.

## 5.6 Concurrency Cost Awareness

Concurrency is powerful but not free:

- task spawn has overhead,
- channel operations may block and schedule,
- synchronization can impact latency.

The right goal is usually not "max parallelism." The right goal is predictable
throughput under bounded latency and clear failure behavior.

## 5.7 Profile Expectations

VibeLang profiles intentionally differ by emphasis:

- **dev/test**: richer diagnostics and verification,
- **release**: optimized native output with explicit policy behavior.

Contract policy and numeric overflow behavior must be profile-defined and stable,
not surprising.

## 5.8 Integer Overflow Policy Awareness

Numeric model policy may vary by profile (`checked`, `wrapping`, `saturating`).
Choose deliberately.

Guideline:

- default to checked behavior for safety-sensitive domains,
- use wrapping/saturating only with explicit domain rationale.

## 5.9 Practical Optimization Workflow

A practical effect-aware optimization loop:

1. identify high-cost region,
2. inspect effect and allocation envelope,
3. optimize structure (data flow, allocations, task boundaries),
4. preserve contracts and examples,
5. verify with tests + deterministic benchmarks.

This sequence avoids "fast but semantically regressed" outcomes.

## 5.10 Example: Tightening an Allocation-Heavy Function

Initial shape:

```txt
pub join_lines(lines: List<Str>) -> Str {
  @effect alloc
  out := ""
  for line in lines {
    out = out + line + "\n"
  }
  out
}
```

Improved shape (conceptual):

```txt
pub join_lines(lines: List<Str>) -> Str {
  @effect alloc
  builder := StrBuilder.new()
  for line in lines {
    builder.append(line)
    builder.append("\n")
  }
  builder.finish()
}
```

Both are valid. The second better matches predictable growth behavior.

## 5.11 Performance Without Hidden Behavior

VibeLang’s philosophy is not "never optimize." It is:

- optimize with explicit behavior boundaries,
- keep diagnostics stable,
- keep contracts executable,
- keep release evidence reproducible.

This is how teams improve performance while preserving trust.

## 5.12 Clarification: Effects Are Design Contracts, Not Decorations

If a function declares `@effect alloc` or `@effect io`, that is not stylistic
flair. It is a contract with future readers and with tooling. The declaration
means: "this function is permitted to do this class of work, and reviewers
should reason about that boundary when approving changes."

The most common anti-pattern is effect drift: behavior changes but effect lines
do not. Over time, this weakens review quality and can hide performance or
operational regressions. Keep effect declarations synchronized with
implementation and treat mismatches as real maintenance debt.

## 5.13 Chapter Checklist

You should now be able to:

- explain each core effect category,
- identify likely allocation hotspots from code structure,
- reason about concurrency overhead in design,
- apply profile-aware performance decisions,
- optimize while preserving contract confidence.

---

Next: Chapter 6 dives deep into structured concurrency.
