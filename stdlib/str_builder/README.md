# `str_builder` module (preview)

## APIs

- `str_builder.new(initial_cap: Int) -> Int`
- `str_builder.append(handle: Int, text: Str) -> Int`
- `str_builder.append_char(handle: Int, char_code: Int) -> Int`
- `str_builder.finish(handle: Int) -> Str`

## Semantics

- `new` allocates a growable string buffer with the given initial byte capacity and returns an opaque handle (integer token).
- `append` appends the full text string to the buffer and returns the handle for chaining.
- `append_char` appends a single byte (by code point value) to the buffer and returns the handle.
- `finish` finalizes the buffer, returns the built string, and releases the internal buffer. The handle must not be reused after `finish`.

## Error model

- `new` panics on allocation failure.
- `append` / `append_char` grow the buffer automatically; panic only on allocation failure.
- `finish` returns `""` if the buffer is empty.
