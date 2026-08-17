# Appendix C: Standard Library Reference

This appendix documents every module and function in VibeLang's standard library.
Each module includes its stability status, import syntax, and a complete function
listing with signatures, descriptions, effects, and examples.

## Stability Levels

| Badge        | Meaning                                                        |
|--------------|----------------------------------------------------------------|
| **Stable**   | API is frozen. Breaking changes require a major version bump.  |
| **Preview**  | API is functional but may change. Pin toolchain versions and review changelogs on upgrade. |

---

## C.1 `io` — Input and Output (Stable)

Console output functions. Import: `import std.io`

All functions require `@effect io`.

### `println(message: Str) -> ()`

Prints a string to standard output followed by a newline.

```vibe
import std.io

pub main() -> Int {
  @effect io
  println("Hello, VibeLang!")
  0
}
```

### `print(message: Str) -> ()`

Prints a string to standard output without a trailing newline.

```vibe
print("Loading")
repeat 3 { print(".") }
println("")
```

### `eprintln(message: Str) -> ()`

Prints a string to standard error followed by a newline. Use for diagnostics and
warnings.

```vibe
eprintln("warning: config not found, using defaults")
```

---

## C.2 `core` — Deterministic Utilities (Stable)

Pure utility functions with no side effects. Import: `import std.core`

All functions are pure (no effects). The same inputs always produce the same
outputs. Functions in `core` can be freely memoized, reordered, and inlined by
the optimizer.

```vibe
import std.core

pub double(n: Int) -> Int {
  @ensure . == n * 2
  n * 2
}
```

---

## C.3 `time` — Time and Duration (Preview)

Functions for reading the current time and introducing delays.
Import: `import std.time`

### `now_ms() -> Int`

Returns wall-clock time as milliseconds since the Unix epoch.

**Effects:** `nondet`

```vibe
start := time.now_ms()
do_work()
elapsed := time.now_ms() - start
println("took " + elapsed.to_str() + "ms")
```

### `sleep_ms(duration: Int) -> ()`

Suspends the current task for at least the specified milliseconds.

**Effects:** `nondet`

```vibe
time.sleep_ms(100)
```

### `duration_ms(ms: Int) -> Int`

Creates a duration value in milliseconds. Pure constructor for readability.

**Effects:** None.

```vibe
timeout := time.duration_ms(5000)
```

---

## C.4 `path` — File Path Manipulation (Stable)

Pure functions for manipulating file paths as strings. These do not access the
file system. Import: `import std.path`

All functions are pure (no effects).

### `join(base: Str, segment: Str) -> Str`

Joins two path segments with the platform-appropriate separator.

```vibe
config := path.join("/etc", "app.conf")  // "/etc/app.conf"
```

### `parent(p: Str) -> Str`

Returns the parent directory. Returns empty string if no parent.

```vibe
dir := path.parent("/home/user/file.txt")  // "/home/user"
```

### `basename(p: Str) -> Str`

Returns the final component of a path.

```vibe
name := path.basename("/home/user/file.txt")  // "file.txt"
```

### `is_absolute(p: Str) -> Bool`

Returns `true` if the path is absolute.

```vibe
path.is_absolute("/usr/bin")   // true
path.is_absolute("src/main")   // false
```

---

## C.4a `text` — String utilities (Preview)

Pure string helpers. Import: `import std.text`. Other `text.*` functions are
listed in the module summary and effects tables below.

### `index_of(haystack: Str, needle: Str) -> Int`

Returns the starting byte index of the first occurrence of `needle` in
`haystack`, or `-1` if `needle` does not occur. The search is byte-oriented;
both arguments must be valid UTF-8.

```vibe
text.index_of("hello world", "world")  // 6
text.index_of("abc", "xyz")            // -1
```

### `slice_bytes(s: Str, start: Int, end: Int) -> Str`

Byte-exact substring for strings whose bytes arrived from **outside the
process** — anything reachable from `net.read`, `ws.read_frame`, an HTTP
response, `http_server.parse_request`, or an `encoding.*_decode` fed by those.
None of those sources validate UTF-8, so the bytes they return may not be valid
UTF-8 at all.

Unlike the `s[start:end]` slice sugar, `slice_bytes` is **total**: it never
aborts. Bounds are clamped rather than rejected (`start` below `0` becomes `0`,
past the end becomes the length; `end` past the end becomes the length; an `end`
below `start` yields an empty string), and there is no UTF-8 boundary check. The
result is always a verbatim copy of the requested byte range — no byte is
rewritten, inserted or dropped — so every offset computed on the original string
keeps its meaning and `s[0:i]` plus `s[i:len]` still reassemble into `s`.

Use it wherever a slice offset is derived from peer-supplied data. Keep using
the `s[start:end]` sugar for a program's own strings: it rejects a mid-character
slice by panicking, which catches a real bug in trusted data.

```vibe
text.slice_bytes("abcde", 1, 3)    // "bc"
text.slice_bytes("abcde", 0, 99)   // "abcde"  (end clamped)
text.slice_bytes("abcde", 4, 1)    // ""       (end below start)
```

### `byte_at(s: Str, index: Int) -> Int`

The raw byte at `index`, or `-1` when `index` is out of range. Like
`slice_bytes`, this is the total counterpart of the `s[index]` sugar for
network-supplied bytes: it performs no bounds panic and no UTF-8 boundary check,
so it can read a continuation byte in the middle of a character.

```vibe
text.byte_at("abc", 0)    // 97
text.byte_at("abc", 99)   // -1
```

---

## C.4b `net` — Low-level TCP Networking (Preview)

The full `net` module (`listen`, `accept`, `connect`, `read`, `write`,
`close`, `resolve`, ...) is listed in the Module Summary, Import Quick
Reference, and Effects tables below; this section documents only the one
function added alongside `Bytes` support. Import: `import std.net`

### `read_bytes(socket: Int, max_bytes: Int) -> Bytes`

Reads up to `max_bytes` from a socket, byte-exact — including embedded
`0x00` bytes, which `net.read`'s `Str` result truncates at (`Str` length is
effectively `strlen`, and a truncated read is silent: nothing signals that
bytes were lost). Prefer `read_bytes` over `read` for anything binary —
images, length-prefixed frames, any protocol that isn't plain text.

`max_bytes` is capped at a **4 MiB ceiling** regardless of what value is
passed: a single call never fills more than 4 MiB, the same ceiling
`net.read` already enforces on itself. This protects against a
peer-influenced `max_bytes` (e.g. a length prefix read off the same
connection and forwarded straight into this call) turning into an
unbounded allocation.

```vibe
b := net.read_bytes(conn, 65536)
println(convert.to_str(bytes.len(b)))
println(bytes.to_hex(b))
```

**Effects:** `net`

---

## C.5 `fs` — File System Operations (Preview)

Functions for reading and writing files and directories. All functions perform
I/O and return `Result` types. Import: `import std.fs`

All functions require `@effect io`.

### `exists(p: Str) -> Bool`

Returns `true` if a file or directory exists at the given path.

```vibe
if fs.exists("config.json") {
  println("config found")
}
```

### `read_text(p: Str) -> Result<Str, Error>`

Reads the entire contents of a file as a UTF-8 string.

```vibe
pub load_config(path: Str) -> Result<Str, Error> {
  @effect io
  fs.read_text(path)
}
```

### `read_bytes(p: Str) -> Bytes`

Reads a file's entire contents byte-exact — including embedded `0x00`
bytes, which `read_text`'s `Str` result cannot hold past their first
occurrence. Length comes from the file's actual size, never from a
terminator.

Capped at a **64 MiB ceiling**: a file strictly larger than 64 MiB (a file
of exactly 64 MiB is read in full) is refused, returning an empty `Bytes` —
the same value `read_bytes` returns for a missing file or a genuinely
empty file. All three cases look identical from `read_bytes`'s return value
alone; use `fs.size` first to tell them apart before assuming a `0`-length
result means "empty file."

```vibe
b := fs.read_bytes("image.png")
println(convert.to_str(bytes.len(b)))
```

### `size(p: Str) -> Int`

Returns a file's byte size, or `-1` when it cannot be stat'd (missing or
unreadable). Use this to pre-check a file against `read_bytes`'s 64 MiB
ceiling, and to distinguish "file does not exist" (`-1`) from "file is
genuinely empty" (`0`) from "file is over the ceiling" (`size` greater than
64 MiB, while `read_bytes` on the same path returns a zero-length `Bytes`)
— three cases `read_bytes` alone cannot tell apart. `fs.exists` is not a
substitute: it only checks that the path exists, so it reports `true` for
an oversize file exactly as readily as for a normal one.

```vibe
sz := fs.size("upload.bin")
if sz < 0 {
  println("missing")
} else if sz > 64 * 1024 * 1024 {
  println("too large for fs.read_bytes")
} else {
  b := fs.read_bytes("upload.bin")
}
```

This is a stopgap, not a `Result` — the stdlib does not have a `Result<T,
Error>` type yet (a later plan item). Once it does, this becomes
`fs.size(p: Str) -> Result<Int, Error>` and the `-1` sentinel goes away.

### `write_text(p: Str, content: Str) -> Result<(), Error>`

Writes a string to a file, creating it if it does not exist, overwriting if it
does. Parent directories must already exist.

```vibe
fs.write_text("output.txt", report)?
```

### `write_bytes(p: Str, data: Bytes) -> Bool`

Writes a `Bytes` value byte-exact, creating or overwriting the file. Unlike
`write_text`, the write length comes from `data`'s own length, never from a
terminator — an embedded `0x00` does not truncate the write. Returns `true`
on success, `false` if the file could not be opened for writing (e.g. the
parent directory does not exist) or the write was short.

```vibe
fs.write_bytes("out.bin", bytes.from_hex("89504e470d0a1a0a"))
```

### `create_dir(p: Str) -> Result<(), Error>`

Creates a directory at the given path.

```vibe
fs.create_dir(path.join(base, "output"))?
```

---

## C.5a `bytes` — Binary Data (Preview)

Explicit-length binary data. Use `Bytes` for anything that came off a socket,
a file, or a decoder; use `Str` only for text. A `Str` is NUL-terminated, so
binary data held in one ends at its first `0x00` byte — `Bytes` does not have
that limitation. Import: `import std.bytes`

All functions are pure (no effects) — they operate on an already-allocated
`Bytes` value; the effectful boundary is at the functions that produce one,
`fs.read_bytes` and `net.read_bytes`.

### `new(len: Int) -> Bytes`

Allocates `len` zero bytes. A negative `len` clamps to `0`.

```vibe
b := bytes.new(4)   // 4 zero bytes
```

### `len(b: Bytes) -> Int`

The number of bytes, counted explicitly — never the position of a
terminator the way `Str` length effectively is.

### `get(b: Bytes, index: Int) -> Int`

The byte at `index` (`0`-`255`), or `-1` when `index` is out of range
(negative, or at/past the length). Total: never aborts.

### `slice(b: Bytes, start: Int, end: Int) -> Bytes`

Byte-exact subrange with clamped bounds: `start < 0` clamps to `0`; `start`
or `end` past the length clamps to the length; `end < start` yields an
empty result. Total, matching `text.slice_bytes`'s clamping style.

### `concat(a: Bytes, b: Bytes) -> Bytes`

Joins two byte sequences. The result's length is the sum of the two inputs'.

### `from_str(s: Str) -> Bytes`

The bytes of a string, up to its first `0x00` — since a `Str` is already
NUL-terminated, this cannot recover bytes a `Str` never had in the first
place. To mint a `Bytes` value containing a `0x00`, use `bytes.from_hex`,
not a `.yb` string literal (the lexer has no `\xNN` escape).

### `to_str(b: Bytes) -> Str`

Reinterprets bytes as text. **Lossy by design**: the result ends at the
first `0x00` byte, exactly like any other NUL-terminated `Str`. This means
`bytes.to_str` is the wrong tool for verifying a round trip through
`Bytes` — a truncated `to_str` result can look identical for two `Bytes`
values that differ after their first `0x00`. Use `bytes.to_hex` for a
lossless, byte-exact view instead.

```vibe
b := bytes.from_hex("68656c6c6f00776f726c64")  // "hello\0world"
bytes.to_str(b)   // "hello" -- "world" is silently gone
bytes.to_hex(b)   // "68656c6c6f00776f726c64" -- nothing lost
```

### `from_hex(hex: Str) -> Bytes`

Decodes a hexadecimal string into bytes. This is the only way to construct
a `Bytes` value containing an arbitrary byte (including `0x00`) directly
from a `.yb` source literal.

**Fails closed and indistinguishably**: an odd-length input, or any
character outside `[0-9a-fA-F]`, both produce the same zero-length `Bytes`
as decoding `""` does. There is no separate error signal — a caller cannot
tell "you passed empty hex" apart from "you passed malformed hex" from the
return value alone.

### `to_hex(b: Bytes) -> Str`

Lossless lowercase hexadecimal encoding of the whole buffer — the byte-exact
counterpart to `to_str`, and the right choice whenever a result needs to be
compared or logged without losing data at an embedded `0x00`.

---

## C.6 `json` — JSON Processing (Preview)

Structured JSON values (`Json`), text codecs, a **canonical** streaming builder for
dynamic output, and **compatibility** helpers. Import: `import std.json`

All functions are pure (no effects).

### Value type: `Json`

`json.parse` produces a `Json` value; `json.stringify` / `json.stringify_pretty`
consume one. Scalar constructors wrap native values:

| Function | Signature | Role |
|----------|-----------|------|
| `json.null` | `() -> Json` | JSON `null` |
| `json.bool` | `(Bool) -> Json` | JSON boolean |
| `json.i64` | `(Int) -> Json` | JSON number (integer) |
| `json.f64` | `(Float) -> Json` | JSON number (float) |
| `json.str` | `(Str) -> Json` | JSON string |

### `parse(raw: Str) -> Json`

Parses UTF-8 JSON text into a `Json` value. Invalid input is handled by the
runtime (this is **not** a `Result`), so `json.parse` is for text you control.
For text that arrived from a socket use **`json.try_parse`**, or validate first
with `json.is_valid`.

```vibe
doc := json.parse("{\"a\":1}")
println(json.stringify(doc))              // compact wire text
println(json.stringify_pretty(doc))       // indented, for debugging
println(json.stringify(json.str("vibe"))) // "\"vibe\""
```

Nesting is capped at **256 levels** for `json.parse`, `json.try_parse` and
`json.is_valid` alike. Past the cap `json.parse` reports
`json.parse exceeds maximum nesting depth of 256` and `json.try_parse` returns
an `Err`. The cap exists because the parser is recursive: without it, deeply
nested text exhausts the thread stack and kills the process outright.

### `try_parse(raw: Str) -> Result<Json, Str>`

Parses JSON **without aborting on bad input**. Use this for anything read off a
network: a request body, a response body, a WebSocket frame. No byte sequence,
however malformed or however deeply nested, can end the process through this
entry point.

`std.result.unwrap_or` only supports `Result<Int, _>`, so a parsed document is
read back with `json.result_value` (the document, or a JSON `null` node when the
parse failed) and `json.result_error` (the message, or `""` when it succeeded).

```vibe
handle_body(conn: Int, body: Str) -> Int {
  @effect net
  parsed := json.try_parse(body)
  if result.is_err(parsed) {
    net.write(conn, http_server.format_response(400, "", json.result_error(parsed)))
    return 0
  }
  doc := json.result_value(parsed)
  net.write(conn, http_server.format_response(200, "", json.stringify(doc)))
  0
}
```

### `result_value(parsed: Result<Json, Str>) -> Json`

The document from `json.try_parse`, or a JSON `null` node when it returned
`Err`. Never aborts.

### `result_error(parsed: Result<Json, Str>) -> Str`

The rejection message from `json.try_parse`, or `""` when it returned `Ok`.
Never aborts.

### `stringify(value: Json) -> Str`

Serializes a `Json` value to compact JSON text (UTF-8 `Str`).

### `stringify_pretty(value: Json) -> Str`

Same as `stringify`, with insignificant whitespace added for readability.

### `json.builder` — canonical dynamic construction

For objects and arrays whose shape is computed at runtime, **`json.builder` is
the recommended path**: you write keys and typed values (`value_bool`,
`value_i64`, `value_f64`, `value_str`, `value_null`, or nested `value_json`), not
hand-escaped string literals. The builder returns a `Str` from `finish` (or you
can treat that string as wire text for HTTP, files, and logs).

```vibe
jb := json.builder.new(128)
jb = json.builder.begin_object(jb)
jb = json.builder.key(jb, "ok")
jb = json.builder.value_bool(jb, true)
jb = json.builder.key(jb, "count")
jb = json.builder.value_i64(jb, 2)
jb = json.builder.key(jb, "items")
jb = json.builder.begin_array(jb)
jb = json.builder.value_str(jb, "a")
jb = json.builder.value_str(jb, "b")
jb = json.builder.end_array(jb)
jb = json.builder.end_object(jb)
body := json.builder.finish(jb)
```

Typical call sequence: `new` → `begin_object` or `begin_array` → (`key` +
value)* in objects → matching `end_*` → `finish`. Use `value_json` to embed an
already-built subtree when needed.

Runnable examples: `examples/07_stdlib_io_json_regex_http/59_json_builder_object_basics.yb`
through `62_json_builder_http_post_body.yb`, and `47_json_parse_stringify_and_codecs.yb`.

### Typed codecs: `json.encode` / `json.decode`

For nominal `type` declarations, the compiler generates typed codecs that
are invoked through `json.encode` and `json.decode`. The compiler infers the
struct type from the argument — no type suffix needed. **This is the preferred
approach for all structured data** — API payloads, config objects, domain models:

```vibe
type Address { city: Str, zip: Int }
type User { id: Int, name: Str, active: Bool, address: Address }

user := User { id: 7, name: "sam", active: true, address: Address { city: "NYC", zip: 10001 } }
wire := json.encode(user)
// {"id":7,"name":"sam","active":true,"address":{"city":"NYC","zip":10001}}

fallback := User { id: 0, name: "fb", active: false, address: Address { city: "", zip: 0 } }
decoded := json.decode(wire, fallback)
```

Nested struct fields are recursively encoded to JSON objects and recursively
decoded back. Missing fields in the JSON fall back to the corresponding field
in the `fallback` value.

**`json.encode` vs `json.stringify` — when to use which:**

- **`json.encode(value)`** takes a **typed struct** and serializes it to a
  JSON `Str`. The compiler knows the fields at compile time.
- **`json.stringify(json_val)`** takes a **runtime `Json` value** (from
  `json.parse`, `json.i64`, `json.bool`, etc.) and converts it to a `Str`.

Use `json.encode` when your data has a known shape (which is most of the time).
Use `json.stringify` when working with dynamic/untyped `Json` values parsed from
unknown sources.

### `is_valid(s: Str) -> Bool`

Returns `true` if the string is syntactically valid JSON. This runs the real
parser and shares its grammar, its trailing-content rule and its 256-level depth
cap, so the guarantee is exact: **if `json.is_valid(s)` is `true`, then
`json.parse(s)` returns rather than aborting.** It walks the whole document but
does not build one, so it stays cheap in memory; when you are going to parse
anyway, `json.try_parse` does it in a single pass.

```vibe
json.is_valid("{\"name\": \"vibe\"}")  // true
json.is_valid("not json")            // false
json.is_valid("[x]")                 // false
json.is_valid("[1,2,]")              // false
```

### `parse_i64(s: Str) -> Int` / `stringify_i64(n: Int) -> Str`

Parse or emit a JSON text fragment that is a single integer (compatibility /
scalar helpers).

```vibe
val := json.parse_i64("42")   // 42
json.stringify_i64(42)        // "42"
```

### `minify(s: Str) -> Str`

Removes insignificant whitespace from a JSON **text** string.

```vibe
compact := json.minify("{ \"a\" : 1 }")  // "{\"a\":1}"
```

### `from_map(map: Map<Str, Str>) -> Str` — convenience / legacy

Serializes a `Map<Str, Str>` to JSON object text using **string values** plus
heuristic coercion (numeric- and boolean-looking strings become JSON numbers and
booleans). Handy when you already have stringly-typed maps; it is **not** the
canonical way to build JSON—prefer `json.builder` (or `Json` values +
`stringify`) for structured intent.

```vibe
preview := {"title": "VibeLang", "score": "95", "active": "true"}
json.from_map(preview)  // {"title":"VibeLang","score":95,"active":true}
```

---

## C.7 `http` — HTTP Client and Server (Preview)

Structured HTTP client with `HttpRequest`/`HttpResponse` types, plus protocol
helpers. These types are defined in `stdlib/std/http.yb` and loaded automatically.

### Built-in types

```vibe
type HttpRequest {
  method: Str,
  url: Str,
  headers: Str,
  body: Str,
  timeout_ms: Int
}

type HttpResponse {
  status: Int,
  headers: Str,
  body: Str
}
```

`headers` uses raw HTTP header format: `"Content-Type: application/json\r\nAuthorization: Bearer tok"`.

### `send(req: HttpRequest) -> HttpResponse`

Full-control HTTP client. Requires `@effect net`.

```vibe
req := HttpRequest {
  method: "POST",
  url: "https://api.example.com/data",
  headers: "Content-Type: application/json",
  body: json.encode(MyPayload { name: "test" }),
  timeout_ms: 5000
}
resp := http.send(req)
if resp.status == 200 {
  result := json.decode(resp.body, fallback)
}
```

### `get(url: Str, timeout_ms: Int) -> HttpResponse`

Convenience GET request. Returns structured `HttpResponse`.

```vibe
resp := http.get("https://example.com/api/health", 3000)
println(resp.body)
println(convert.to_str(resp.status))
```

### `post(url: Str, body: Str, timeout_ms: Int) -> HttpResponse`

Convenience POST request. Returns structured `HttpResponse`.

```vibe
type LoginReq { email: Str, password: Str }

resp := http.post("https://api.example.com/login", json.encode(LoginReq { email: "a@b.com", password: "secret" }), 5000)
if resp.status == 200 {
  println(resp.body)
}
```

### `ok(resp: HttpResponse) -> Bool`

True when `resp.status` is between 200 and 299 (inclusive).

### `get_with_headers(url: Str, headers: Str, timeout_ms: Int) -> HttpResponse`

GET with a raw header block (same `\r\n`-separated format as `HttpRequest.headers`).

### `post_with_headers(url: Str, headers: Str, body: Str, timeout_ms: Int) -> HttpResponse`

POST with explicit headers and body.

### `post_json(url: Str, body_json: Str, timeout_ms: Int) -> HttpResponse`

POST with `Content-Type: application/json` and the given JSON **string** body.

### `get_retry(url: Str, timeout_ms: Int, retries: Int, retry_delay_ms: Int) -> HttpResponse`

Calls `http.get` in a loop. After the first attempt, retries up to `retries`
times when `status == 0` (no HTTP response parsed — typical of connection
failures), waiting `retry_delay_ms` between tries (`time.sleep_ms`). Requires
`@effect net` and `@effect io` at the call site.

### `response(resp: HttpResponse) -> Str`

Formats an `HttpResponse` into an HTTP/1.1 wire string for sending over a
socket (server use).

```vibe
wire := http.response(HttpResponse { status: 200, headers: "", body: json.encode(data) })
net.write(conn, wire)
```

### `build_response(status: Int, body: Str) -> Str`

Convenience server helper — builds a wire-format HTTP response with JSON
content type and CORS headers.

### `status_text(code: Int) -> Str`

Returns the standard reason phrase for an HTTP status code.

```vibe
http.status_text(200)   // "OK"
http.status_text(404)   // "Not Found"
```

### `default_port(scheme: Str) -> Int`

Returns the default port for a URI scheme.

### `build_request_line(method: Str, path: Str) -> Str`

Constructs an HTTP/1.1 request line from a method and path.

### `request(method: Str, url: Str, body: Str, timeout_ms: Int) -> Str`

Legacy unstructured request — returns body text only. Prefer `http.send`.

### `request_status(method: Str, url: Str, body: Str, timeout_ms: Int) -> Int`

Returns only the HTTP status code for the request.

---

## C.8 `convert` — Additional Conversion Functions (Preview)

The core `convert` functions (`to_int`, `parse_i64`, `to_float`, `parse_f64`,
`to_str`, `to_str_f64`) are listed above. The following functions were added
to support Float codegen and bit-level operations.

### `format_f64(value: Float, precision: Int) -> Str`

Formats a float with a fixed number of decimal places.

```vibe
convert.format_f64(3.14159, 2)   // "3.14"
convert.format_f64(1.0, 6)       // "1.000000"
```

### `i64_to_f64(n: Int) -> Float`

Converts an integer to a float.

```vibe
f := convert.i64_to_f64(42)   // 42.0
```

### `f64_to_bits(f: Float) -> Int`

Returns the IEEE 754 bit representation of a float as an integer. Useful for
bit-level manipulation (e.g. hash functions, serialization).

```vibe
bits := convert.f64_to_bits(1.0)   // 4607182418800017408
```

### `f64_from_bits(bits: Int) -> Float`

Reconstructs a float from its IEEE 754 bit representation.

```vibe
f := convert.f64_from_bits(4607182418800017408)   // 1.0
```

---

## C.9 `str_builder` — String Builder (Preview)

Efficient mutable string construction. Use when building strings incrementally
in a loop to avoid O(n²) concatenation. Import: `import std.str_builder`

All functions are pure (no effects).

### `new(capacity: Int) -> Int`

Creates a new string builder with the given initial capacity. Returns a handle.

```vibe
sb := str_builder.new(1024)
```

### `append(handle: Int, s: Str) -> Int`

Appends a string to the builder. Returns the handle.

```vibe
str_builder.append(sb, "Hello, ")
str_builder.append(sb, "world!")
```

### `append_char(handle: Int, ch: Int) -> Int`

Appends a single byte (as an ASCII code point) to the builder. Returns the handle.

```vibe
str_builder.append_char(sb, 10)   // newline
```

### `finish(handle: Int) -> Str`

Finalizes the builder and returns the built string. The handle is invalidated.

```vibe
result := str_builder.finish(sb)
println(result)   // "Hello, world!\n"
```

---

## C.10 `regex` — Regular Expressions (Preview)

POSIX extended regular expression matching. Import: `import std.regex`

All functions are pure (no effects).

### `count(text: Str, pattern: Str) -> Int`

Returns the number of non-overlapping matches of `pattern` in `text`.

```vibe
regex.count("abcabc", "abc")   // 2
regex.count("hello", "x")      // 0
```

### `replace_all(text: Str, pattern: Str, replacement: Str) -> Str`

Replaces all non-overlapping matches of `pattern` in `text` with `replacement`.

```vibe
regex.replace_all("foo bar foo", "foo", "baz")   // "baz bar baz"
```

---

## C.11 Module Summary

| Module | Stability   | Effects Required | Functions |
|--------|-------------|------------------|:---------:|
| `io`   | **Stable**  | `io`             | 3         |
| `core` | **Stable**  | None             | —         |
| `time` | **Preview** | `nondet`         | 4         |
| `path` | **Stable**  | None             | 4         |
| `fs`   | **Preview** | `io`             | 7         |
| `bytes` | **Preview** | None            | 9         |
| `net`  | **Preview** | `net`            | 9         |
| `convert` | **Preview** | None          | 10        |
| `text` | **Preview** | None             | 10        |
| `encoding` | **Preview** | None         | 6         |
| `json` | **Preview** | None             | 13+       |
| `http` | **Preview** | `net` (client ops) | 7      |
| `log`  | **Preview** | `io`             | 3         |
| `metrics` | **Preview** | `alloc` / `nondet` | 5      |
| `env`  | **Preview** | `nondet`         | 3         |
| `cli`  | **Preview** | `nondet`         | 2         |
| `str_builder` | **Preview** | None      | 4         |
| `regex` | **Preview** | None            | 2         |
| `crypto` | **Preview** | `nondet` (random) | 5     |
| `http_server` | **Preview** | `net`    | 3         |
| `http_router` | **Preview** | None (uses `http_server`) | 5 |
| `ws`    | **Preview** | `net`            | 4         |
| `result` | **Preview** | None           | 4         |

---

## C.11a `crypto` — Cryptographic Primitives (Preview)

Hashing, HMAC, random generation, and constant-time comparison.

### `sha256(data: Str) -> Str`

Returns the SHA-256 hash of `data` as a lowercase hex string.

```vibe
hash := crypto.sha256("hello world")
// "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
```

### `hmac_sha256(key: Str, data: Str) -> Str`

Computes HMAC-SHA256 for webhook signature verification and message authentication.

### `uuid_v4() -> Str`

Generates a random UUID v4 string. Requires `@effect nondet`.

### `random_bytes(n: Int) -> Str`

Generates `n` cryptographically secure random bytes as a hex string. Requires `@effect nondet`.

### `constant_time_eq(a: Str, b: Str) -> Bool`

Compares two strings in constant time to prevent timing attacks.

---

## C.11b `http_server` — HTTP Server Helpers (Preview)

Structured HTTP request parsing and response building for servers.

### `parse_request(raw: Str) -> HttpServerRequest`

Parses a raw HTTP request into a structured `HttpServerRequest` with `method`, `path`, `query`, `headers`, and `body` fields.

### `format_response(status: Int, headers: Str, body: Str) -> Str`

Builds a complete HTTP/1.1 response with correct Content-Length and custom headers.

### `cors_headers() -> Str`

Returns standard CORS headers for API responses.

---

## C.11c `http_router` — Tiny server routing helpers (Preview)

Pure VibeLang helpers on top of `http_server.parse_request` / `format_response`: lookup header and query values, build JSON or plain-text responses with CORS + `Content-Type`, and dispatch with a handler closure.

### `header_get(headers: Str, name: Str) -> Str`

Returns the trimmed value for a header name, or `""` if missing. Header names are matched case-insensitively.

### `query_get(query: Str, name: Str) -> Str`

Returns the value for an exact query key (from the `query` field of `HttpServerRequest`), or `""` if absent.

### `json_response(status: Int, body_json: Str) -> Str` / `text_response(status: Int, body: Str) -> Str`

Build full HTTP/1.1 responses with CORS, `Content-Type`, and correct `Content-Length` via `http_server.format_response`.

### `route(req: HttpServerRequest, method: Str, path: Str, handler: fn(HttpServerRequest) -> Str, fallback: Str) -> Str`

When `req.method` and `req.path` match, returns `handler(req)`; otherwise returns `fallback`. Use this to wire a small set of routes without a callback table.

---

## C.11d `ws` — WebSocket (Preview)

WebSocket handshake and frame handling for real-time communication.

### `upgrade(conn: Int, raw_request: Str) -> Bool`

Performs the WebSocket handshake on an accepted TCP connection.

### `read_frame(conn: Int) -> Str`

Reads and decodes the next WebSocket text frame.

### `write_frame(conn: Int, data: Str)`

Sends a WebSocket text frame.

### `close_frame(conn: Int)`

Sends a WebSocket close frame.

---

## C.11e `metrics` — In-process counters and gauges (Preview)

Minimal observability for long-running programs: named counters (monotonic sums),
gauges (last-written values), and a JSON snapshot for scraping or logging.
Storage is **per process** and **in memory** (not shared across workers).

Import: metrics are available as the `metrics` namespace (e.g. `metrics.counter_inc`).

### `counter_inc(name: Str, delta: Int) -> ()`

Adds `delta` to the named counter, creating it at zero if needed.

**Effects:** `alloc`

### `counter_get(name: Str) -> Int`

Returns the counter value, or `0` if the name was never incremented.

**Effects:** `nondet`

### `gauge_set(name: Str, value: Int) -> ()`

Sets the named gauge to `value`.

**Effects:** `alloc`

### `gauge_get(name: Str) -> Int`

Returns the gauge value, or `0` if unset.

**Effects:** `nondet`

### `snapshot_json() -> Str`

Returns UTF-8 JSON of the form `{"counters":{...},"gauges":{...}}` with string
keys and integer values.

**Effects:** `alloc`

```vibe
metrics.counter_inc("requests", 1)
metrics.gauge_set("workers", 4)
println(metrics.snapshot_json())
```

---

## C.11f `result` — Result Helpers (Preview)

Utilities for working with `Result` values.

### `is_ok(result) -> Bool` / `is_err(result) -> Bool`

Check the status of a Result value.

### `unwrap_or(result, default: Int) -> Int`

Extract the Ok value or return a default.

### `wrap_err(result, context: Str) -> Result`

Add context to an error message.

---

## C.11g `concurrent` — spawn, timeout, parallel map (Preview)

Small helpers on top of `go`, `chan`, and `time.sleep_ms`. No async/await.

### `spawn(task: fn() -> Int)`

Runs `go task()` (fire-and-forget).

### `with_timeout(task: fn() -> Int, timeout_ms: Int, fallback: Int) -> Int`

Races `task()` against a timer; first value sent on an internal channel is
returned (wall-clock semantics; requires `@effect io` for sleep).

### `map_int(items, worker: fn(Int) -> Int, max_workers: Int) -> List<Int>`

### `map_str(items, worker: fn(Str) -> Str, max_workers: Int) -> List<Str>`

Bounded parallelism with a token channel; results are written back by index so
output order matches input order.

**Effects:** `@effect concurrency`; maps declare `@effect alloc` and
`@effect mut_state`; `with_timeout` also uses `@effect io` and `@effect alloc`.

---

## C.12 Import Quick Reference

```vibe
import std.io          // println, print, eprintln
import std.core        // deterministic utilities
import std.time        // now_ms, sleep_ms, duration_ms
import std.path        // join, parent, basename, is_absolute
import std.fs          // exists, read_text, read_bytes, size, write_text, write_bytes, create_dir
import std.bytes       // new, len, get, slice, concat, from_str, to_str, from_hex, to_hex
import std.net         // listen, listener_port, accept, connect, read, read_bytes, write, close, resolve
import std.convert     // to_int, parse_i64, to_float, parse_f64, to_str, to_str_f64, format_f64, i64_to_f64, f64_to_bits, f64_from_bits
import std.text        // trim, contains, starts_with, ends_with, replace, to_lower, to_upper, byte_len, split_part, index_of, slice_bytes, byte_at
import std.encoding    // hex/base64/url encode/decode
import std.json        // Json: parse, stringify, stringify_pretty, null/bool/i64/f64/str; builder.*; is_valid, minify; parse_i64, stringify_i64; from_map; encode_<T>, decode_<T>
import std.http        // status_text, default_port, build_request_line, request, request_status, get, post, send, response, ok, get_with_headers, post_with_headers, post_json, get_retry
import std.log         // info, warn, error
import std.metrics     // counter_inc, counter_get, gauge_set, gauge_get, snapshot_json
import std.env         // get, has, get_required
import std.cli         // args_len, arg
import std.str_builder // new, append, append_char, finish
import std.regex       // count, replace_all
import std.crypto      // sha256, hmac_sha256, uuid_v4, random_bytes, constant_time_eq
import std.http_server // parse_request, format_response, cors_headers
import std.http_router // header_get, query_get, json_response, text_response, route
import std.ws          // upgrade, read_frame, write_frame, close_frame
import std.result      // is_ok, is_err, unwrap_or, wrap_err
import std.concurrent  // spawn, with_timeout, map_int, map_str
```

---

## C.13 Effects by Function

| Function                       | Module | Effects  |
|--------------------------------|--------|----------|
| `println(Str)`                 | io     | `io`     |
| `print(Str)`                   | io     | `io`     |
| `eprintln(Str)`                | io     | `io`     |
| `now_ms()`                     | time   | `nondet` |
| `monotonic_now_ms()`           | time   | `nondet` |
| `sleep_ms(Int)`                | time   | `nondet` |
| `duration_ms(Int)`             | time   | None     |
| `join(Str, Str)`               | path   | None     |
| `parent(Str)`                  | path   | None     |
| `basename(Str)`                | path   | None     |
| `is_absolute(Str)`             | path   | None     |
| `exists(Str)`                  | fs     | `io`     |
| `read_text(Str)`               | fs     | `io`     |
| `read_bytes(Str)`              | fs     | `io`     |
| `size(Str)`                    | fs     | `io`     |
| `write_text(Str, Str)`         | fs     | `io`     |
| `write_bytes(Str, Bytes)`      | fs     | `io`     |
| `create_dir(Str)`              | fs     | `io`     |
| `new(Int)`                     | bytes  | None     |
| `len(Bytes)`                   | bytes  | None     |
| `get(Bytes, Int)`              | bytes  | None     |
| `slice(Bytes, Int, Int)`       | bytes  | None     |
| `concat(Bytes, Bytes)`         | bytes  | None     |
| `from_str(Str)`                | bytes  | None     |
| `to_str(Bytes)`                | bytes  | None     |
| `from_hex(Str)`                | bytes  | None     |
| `to_hex(Bytes)`                | bytes  | None     |
| `listen(Str, Int)`             | net    | `net`    |
| `listener_port(Int)`           | net    | `net`    |
| `accept(Int)`                  | net    | `net`    |
| `connect(Str, Int)`            | net    | `net`    |
| `read(Int, Int)`               | net    | `net`    |
| `read_bytes(Int, Int)`         | net    | `net`    |
| `write(Int, Str)`              | net    | `net`    |
| `close(Int)`                   | net    | `net`    |
| `resolve(Str)`                 | net    | `net`    |
| `to_int(Str)`                  | convert | None    |
| `parse_i64(Str)`               | convert | None    |
| `to_float(Str)`                | convert | None    |
| `parse_f64(Str)`               | convert | None    |
| `to_str(Int)`                  | convert | None    |
| `to_str_f64(Float)`            | convert | None    |
| `format_f64(Float, Int)`       | convert | None    |
| `i64_to_f64(Int)`              | convert | None    |
| `f64_to_bits(Float)`           | convert | None    |
| `f64_from_bits(Int)`           | convert | None    |
| `trim(Str)`                    | text   | None     |
| `contains(Str, Str)`           | text   | None     |
| `starts_with(Str, Str)`        | text   | None     |
| `ends_with(Str, Str)`          | text   | None     |
| `replace(Str, Str, Str)`       | text   | None     |
| `to_lower(Str)`                | text   | None     |
| `to_upper(Str)`                | text   | None     |
| `byte_len(Str)`                | text   | None     |
| `split_part(Str, Str, Int)`    | text   | None     |
| `index_of(Str, Str)`           | text   | None     |
| `slice_bytes(Str, Int, Int)`   | text   | None     |
| `byte_at(Str, Int)`            | text   | None     |
| `hex_encode(Str)`              | encoding | None   |
| `hex_decode(Str)`              | encoding | None   |
| `base64_encode(Str)`           | encoding | None   |
| `base64_decode(Str)`           | encoding | None   |
| `url_encode(Str)`              | encoding | None   |
| `url_decode(Str)`              | encoding | None   |
| `is_valid(Str)`                | json   | None     |
| `parse(Str)`                   | json   | None     |
| `try_parse(Str)`               | json   | None     |
| `result_value(Result<Json, Str>)` | json | None    |
| `result_error(Result<Json, Str>)` | json | None    |
| `stringify(Json)`              | json   | None     |
| `stringify_pretty(Json)`       | json   | None     |
| `null()` … `str(Str)`          | json   | None     |
| `parse_i64(Str)`               | json   | None     |
| `stringify_i64(Int)`           | json   | None     |
| `minify(Str)`                  | json   | None     |
| `from_map(Map<Str, Str>)`      | json   | None     |
| `status_text(Int)`             | http   | None     |
| `default_port(Str)`            | http   | None     |
| `build_request_line(Str, Str)` | http   | None     |
| `request(Str, Str, Str, Int)`  | http   | `net`    |
| `request_status(Str, Str, Str, Int)` | http | `net` |
| `get(Str, Int)`                | http   | `net`    |
| `post(Str, Str, Int)`          | http   | `net`    |
| `info(Str)`                    | log    | `io`     |
| `warn(Str)`                    | log    | `io`     |
| `error(Str)`                   | log    | `io`     |
| `counter_inc(Str, Int)`        | metrics | `alloc` |
| `counter_get(Str)`             | metrics | `nondet` |
| `gauge_set(Str, Int)`          | metrics | `alloc` |
| `gauge_get(Str)`               | metrics | `nondet` |
| `snapshot_json()`              | metrics | `alloc` |
| `get(Str)`                     | env    | `nondet` |
| `has(Str)`                     | env    | `nondet` |
| `get_required(Str)`            | env    | `nondet` |
| `args_len()`                   | cli    | `nondet` |
| `arg(Int)`                     | cli    | `nondet` |
| `new(Int)`                     | str_builder | None |
| `append(Int, Str)`             | str_builder | None |
| `append_char(Int, Int)`        | str_builder | None |
| `finish(Int)`                  | str_builder | None |
| `count(Str, Str)`              | regex  | None     |
| `replace_all(Str, Str, Str)`   | regex  | None     |
