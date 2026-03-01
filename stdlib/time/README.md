# `time` module (preview)

## APIs

- `time.now_ms() -> Int`
  - returns wall-clock milliseconds since Unix epoch
  - nondeterministic by design
- `time.monotonic_now_ms() -> Int`
  - returns monotonic milliseconds suitable for elapsed-duration measurement
- `time.sleep_ms(ms: Int) -> Void`
  - sleeps current thread for `ms` milliseconds (`ms <= 0` is a no-op)
- `time.duration_ms(seconds: Int) -> Int`
  - converts whole seconds to milliseconds
  - saturates at `Int` max on overflow

## Error model

- No panics for normal invalid ranges (`sleep_ms` negative/zero is ignored).
- `duration_ms` returns `0` for non-positive input.

## Effects

- `time.now_ms`: `nondet`
- `time.monotonic_now_ms`: `nondet`
- `time.sleep_ms`: `io`
- `time.duration_ms`: pure
