# Chapter 8: Intent-Driven Development and Sidecar Model

Intent annotations can guide linting and review workflows.

```vibe
pub greet(name: Str) -> Int {
  @intent "print a deterministic greeting"
  @effect io
  println(name)
  0
}
```

The sidecar model remains non-blocking for core compile paths.
