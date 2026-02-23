# Chapter 5: Effects and Performance Reasoning

Effects make side effects explicit and easier to reason about.

```vibe
pub grow() -> Int {
  @effect alloc
  values := []
  values.append(1)
  values.len()
}
```

Use the cost model in `docs/spec/cost_model.md` to reason about copies,
allocations, and concurrency overhead.
