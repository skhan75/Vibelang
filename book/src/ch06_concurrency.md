# Chapter 6: Concurrency (`go`, `chan`, `select`, cancellation)

VibeLang concurrency is explicit and deterministic where synchronized.

```vibe
consume(ch) -> Int {
  @effect concurrency
  ch.recv()
}

pub main() -> Int {
  @effect concurrency
  @effect alloc
  jobs := chan(1)
  jobs.send(1)
  jobs.close()
  consume(jobs)
}
```

See `docs/spec/concurrency_and_scheduling.md` for normative behavior.
