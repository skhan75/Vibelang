# `bytes` module (preview)

Binary data with an explicit length. Use `Bytes` for anything that came off a
socket, a file, or a decoder, and `Str` only for text. A `Str` is
NUL-terminated (its length is `strlen`), so binary data held in one ends at
its first zero byte.

## APIs

- `bytes.new(len: Int) -> Bytes`
- `bytes.len(b: Bytes) -> Int`
- `bytes.get(b: Bytes, index: Int) -> Int`
- `bytes.slice(b: Bytes, start: Int, end: Int) -> Bytes`
- `bytes.concat(a: Bytes, b: Bytes) -> Bytes`
- `bytes.from_str(s: Str) -> Bytes`
- `bytes.to_str(b: Bytes) -> Str`
- `bytes.from_hex(hex: Str) -> Bytes`
- `bytes.to_hex(b: Bytes) -> Str`

## Semantics

- `new` allocates `len` zero bytes. A negative `len` is treated as `0`.
- `len` returns the explicit byte count. It is never a scan for a terminator,
  so embedded `0x00` bytes are counted like any other byte.
- `get` returns the byte at `index` as an `Int` in `0..=255`, or `-1` when
  `index` is out of range.
- `slice` is byte-exact and total: bounds are clamped, never rejected.
  `start < 0` clamps to `0`; `start > len` clamps to `len`; `end > len`
  clamps to `len`; `end < start` collapses to an empty result. It never
  panics on bad bounds (the only failure path left is allocation failure,
  see "Error model" below).
- `concat` joins two byte sequences and is total.
- `from_str` copies the bytes of `s` up to (not including) its terminating
  zero byte, since that is all a `Str` can hold.
- `to_str` reinterprets bytes as text and is **deliberately lossy**: the
  result ends at the first `0x00` byte in `b`, exactly like any NUL-scanning
  C string consumer. Binary data with an embedded NUL has no faithful `Str`
  representation. Use `to_hex` when every byte must round-trip.
- `to_hex` encodes the whole buffer as lowercase hexadecimal and is
  lossless: every byte, including an embedded NUL, round-trips through
  `from_hex`.
- `from_hex` decodes hexadecimal into bytes. Both uppercase and lowercase
  hex digits are accepted.

## Error model

- All nine functions are total and never panic on malformed or
  out-of-range input. The one exception is allocation failure (OOM), which
  is not peer-controlled: every function that allocates (`new`, and
  `slice`/`concat`/`from_str`/`from_hex`/`to_str`/`to_hex`, which all
  allocate their result) calls `vibe_panic` if the underlying `calloc`
  fails. `len` and `get` never allocate and cannot hit this path.
- `from_hex` fails closed: an odd-length input, any character outside
  `[0-9a-fA-F]`, or a NULL input all return a **zero-length** `Bytes`. There
  is no distinct error signal, so a decode failure is not distinguishable
  from decoding `""` — check `bytes.len(result) == 0` if the distinction
  matters to the caller, not any exception or sentinel value.
