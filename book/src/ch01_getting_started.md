# Chapter 1: Getting Started and Mental Model

VibeLang focuses on deterministic builds, explicit effects, and practical
concurrency primitives.

```vibe
pub main() -> Int {
  @effect io
  println("hello, vibelang")
  0
}
```

Key mindset:

- Prefer explicit effects for observable behavior.
- Keep contracts close to logic.
- Treat deterministic CI as a language feature, not an afterthought.
