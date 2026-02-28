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

### `parse_i64(s: Str) -> Result<Int, Error>`

Parses a JSON string containing a single integer value.

```vibe
val := json.parse_i64("42")?  // 42
```

### `stringify_i64(n: Int) -> Str`

Converts an integer to its JSON string representation.

```vibe
json.stringify_i64(42)   // "42"
json.stringify_i64(-1)   // "-1"
```

### `minify(s: Str) -> Result<Str, Error>`

Removes insignificant whitespace from a JSON string.

```vibe
compact := json.minify("{ \"a\" : 1 }")?  // "{\"a\":1}"
```

---

## C.7 `http` — HTTP Utilities (Preview)

Helper functions for working with the HTTP protocol. These are protocol-level
utilities; actual network I/O is handled by runtime HTTP client/server APIs.
Import: `import std.http`

All functions are pure (no effects).

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

---

## C.8 Module Summary

| Module | Stability   | Effects Required | Functions |
|--------|-------------|------------------|:---------:|
| `io`   | **Stable**  | `io`             | 3         |
| `core` | **Stable**  | None             | —         |
| `time` | **Preview** | `nondet`         | 3         |
| `path` | **Stable**  | None             | 4         |
| `fs`   | **Preview** | `io`             | 4         |
| `json` | **Preview** | None             | 4         |
| `http` | **Preview** | None             | 3         |

---

## C.9 Import Quick Reference

```vibe
import std.io      // println, print, eprintln
import std.core    // deterministic utilities
import std.time    // now_ms, sleep_ms, duration_ms
import std.path    // join, parent, basename, is_absolute
import std.fs      // exists, read_text, write_text, create_dir
import std.json    // is_valid, parse_i64, stringify_i64, minify
import std.http    // status_text, default_port, build_request_line
```

---

## C.10 Effects by Function

| Function                       | Module | Effects  |
|--------------------------------|--------|----------|
| `println(Str)`                 | io     | `io`     |
| `print(Str)`                   | io     | `io`     |
| `eprintln(Str)`                | io     | `io`     |
| `now_ms()`                     | time   | `nondet` |
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
| `is_valid(Str)`                | json   | None     |
| `parse_i64(Str)`               | json   | None     |
| `stringify_i64(Int)`           | json   | None     |
| `minify(Str)`                  | json   | None     |
| `status_text(Int)`             | http   | None     |
| `default_port(Str)`            | http   | None     |
| `build_request_line(Str, Str)` | http   | None     |
