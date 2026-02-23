# Chapter 10: Advanced Internals Overview

The pipeline is parser -> type checker -> MIR -> codegen -> runtime/link.

```vibe
pub square(x: Int) -> Int {
  x * x
}
```

For deep dives:

- `compiler/frontend/README.md`
- `compiler/ir/overview.md`
- `compiler/codegen/README.md`
- `runtime/concurrency/design.md`
