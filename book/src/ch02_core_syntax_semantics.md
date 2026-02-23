# Chapter 2: Core Syntax and Semantics

VibeLang uses expression-oriented function bodies and explicit control flow.

```vibe
pub choose(a: Int, b: Int, useA: Bool) -> Int {
  if useA {
    return a
  } else {
    return b
  }
}
```

Normative semantics live in `docs/spec/syntax.md` and `docs/spec/semantics.md`.
