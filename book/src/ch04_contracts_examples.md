# Chapter 4: Contracts and Executable Examples

Contracts keep intent and behavior close to implementation.

```vibe
pub nonNegative(x: Int) -> Int {
  @require x >= 0
  @ensure . >= 0
  x
}
```

Use `@examples` for executable behavior narratives where appropriate.
