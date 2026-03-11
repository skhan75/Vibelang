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

### `write_text(p: Str, content: Str) -> Result<(), Error>`

Writes a string to a file, creating it if it does not exist, overwriting if it
does. Parent directories must already exist.

```vibe
fs.write_text("output.txt", report)?
```

### `create_dir(p: Str) -> Result<(), Error>`

Creates a directory at the given path.

```vibe
fs.create_dir(path.join(base, "output"))?
```

---

## C.6 `json` — JSON Processing (Preview)

Functions for validating and transforming JSON strings.
Import: `import std.json`

All functions are pure (no effects).

### `is_valid(s: Str) -> Bool`

Returns `true` if the string is syntactically valid JSON.

```vibe
json.is_valid("{\"name\": \"vibe\"}")  // true
json.is_valid("not json")              // false
```

### `parse(s: Str) -> Str`

Returns canonicalized JSON text when input is valid JSON, otherwise returns `""`.

```vibe
json.parse("{ \"a\" : 1 }")  // "{\"a\":1}"
json.parse("nope")           // ""
```

### `stringify(s: Str) -> Str`

If input is already valid JSON, returns canonicalized JSON text. Otherwise returns
a quoted JSON string.

```vibe
json.stringify("{\"a\":1}")  // "{\"a\":1}"
json.stringify("hello")      // "\"hello\""
```

### Generated typed codecs: `encode_<Type>` / `decode_<Type>`

For nominal `type` declarations, the compiler exposes typed JSON codec entrypoints
under `json.*`:

```vibe
type User { id: Int, name: Str, active: Bool }

fallback := User { id: 1, name: "fallback", active: false }
decoded := json.decode_User("{\"id\":7,\"name\":\"sam\",\"active\":true}", fallback)
wire := json.encode_User(decoded)
```

### `parse_i64(s: Str) -> Int`

Parses a JSON string containing a single integer value.

```vibe
val := json.parse_i64("42")  // 42
json.parse_i64("bad")        // 0
```

### `stringify_i64(n: Int) -> Str`

Converts an integer to its JSON string representation.

```vibe
json.stringify_i64(42)   // "42"
json.stringify_i64(-1)   // "-1"
```

### `minify(s: Str) -> Str`

Removes insignificant whitespace from a JSON string.

```vibe
compact := json.minify("{ \"a\" : 1 }")  // "{\"a\":1}"
```

---

## C.7 `http` — HTTP Utilities (Preview)

Sync-first HTTP client helpers plus protocol utilities.
Import: `import std.http`

`status_text/default_port/build_request_line` are pure helpers.
`request/request_status/get/post` require `@effect net`.

### `status_text(code: Int) -> Str`

Returns the standard reason phrase for an HTTP status code. Returns `"Unknown"`
for unrecognized codes.

```vibe
http.status_text(200)   // "OK"
http.status_text(404)   // "Not Found"
http.status_text(500)   // "Internal Server Error"
```

### `default_port(scheme: Str) -> Int`

Returns the default port for a URI scheme. Returns `0` for unrecognized schemes.

```vibe
http.default_port("http")    // 80
http.default_port("https")   // 443
```

### `build_request_line(method: Str, path: Str) -> Str`

Constructs an HTTP/1.1 request line from a method and path.

```vibe
http.build_request_line("GET", "/api/users")
// "GET /api/users HTTP/1.1"
```

### `request(method: Str, url: Str, body: Str, timeout_ms: Int) -> Str`

Performs a sync HTTP request and returns response body text.

```vibe
resp := http.request("POST", "http://127.0.0.1:8080/api", "{\"ok\":true}", 2000)
```

### `request_status(method: Str, url: Str, body: Str, timeout_ms: Int) -> Int`

Returns only the HTTP status code for the request.

```vibe
status := http.request_status("GET", "http://127.0.0.1:8080/health", "", 2000)
```

### `get(url: Str, timeout_ms: Int) -> Str`

Convenience wrapper around `request("GET", ...)`.

### `post(url: Str, body: Str, timeout_ms: Int) -> Str`

Convenience wrapper around `request("POST", ...)`.

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
| `fs`   | **Preview** | `io`             | 4         |
| `net`  | **Preview** | `net`            | 8         |
| `convert` | **Preview** | None          | 10        |
| `text` | **Preview** | None             | 9         |
| `encoding` | **Preview** | None         | 6         |
| `json` | **Preview** | None             | 6         |
| `http` | **Preview** | `net` (client ops) | 7      |
| `log`  | **Preview** | `io`             | 3         |
| `env`  | **Preview** | `nondet`         | 3         |
| `cli`  | **Preview** | `nondet`         | 2         |
| `str_builder` | **Preview** | None      | 4         |
| `regex` | **Preview** | None            | 2         |

---

## C.12 Import Quick Reference

```vibe
import std.io          // println, print, eprintln
import std.core        // deterministic utilities
import std.time        // now_ms, sleep_ms, duration_ms
import std.path        // join, parent, basename, is_absolute
import std.fs          // exists, read_text, write_text, create_dir
import std.net         // listen, listener_port, accept, connect, read, write, close, resolve
import std.convert     // to_int, parse_i64, to_float, parse_f64, to_str, to_str_f64, format_f64, i64_to_f64, f64_to_bits, f64_from_bits
import std.text        // trim, contains, starts_with, ends_with, replace, to_lower, to_upper, byte_len, split_part
import std.encoding    // hex/base64/url encode/decode
import std.json        // is_valid, parse, stringify, parse_i64, stringify_i64, minify
import std.http        // status_text, default_port, build_request_line, request, request_status, get, post
import std.log         // info, warn, error
import std.env         // get, has, get_required
import std.cli         // args_len, arg
import std.str_builder // new, append, append_char, finish
import std.regex       // count, replace_all
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
| `write_text(Str, Str)`         | fs     | `io`     |
| `create_dir(Str)`              | fs     | `io`     |
| `listen(Str, Int)`             | net    | `net`    |
| `listener_port(Int)`           | net    | `net`    |
| `accept(Int)`                  | net    | `net`    |
| `connect(Str, Int)`            | net    | `net`    |
| `read(Int, Int)`               | net    | `net`    |
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
| `hex_encode(Str)`              | encoding | None   |
| `hex_decode(Str)`              | encoding | None   |
| `base64_encode(Str)`           | encoding | None   |
| `base64_decode(Str)`           | encoding | None   |
| `url_encode(Str)`              | encoding | None   |
| `url_decode(Str)`              | encoding | None   |
| `is_valid(Str)`                | json   | None     |
| `parse(Str)`                   | json   | None     |
| `stringify(Str)`               | json   | None     |
| `parse_i64(Str)`               | json   | None     |
| `stringify_i64(Int)`           | json   | None     |
| `minify(Str)`                  | json   | None     |
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
