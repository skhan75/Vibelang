# Building Real Apps (CLI + Services)

This chapter shows the minimal “production loop” for real programs:

- accept inputs (CLI + env)
- load configuration (file + defaults)
- talk to the outside world (HTTP + networking)
- observe behavior (logging + timing)

Throughout the book, the **canonical user-facing string/text surface is `Str` methods**
(for example `raw.trim().to_lower()`), even when low-level primitives exist in `std.text`.

---

## 17.1 A practical config-loading pattern

A common, boring problem in real apps is deciding where configuration comes from.
A simple, predictable precedence order:

1. CLI flags / args
2. environment variables
3. config file
4. hard-coded defaults

VibeLang’s standard library includes the core building blocks for this.

```vibe
import std.cli
import std.env
import std.fs
import std.json
import std.log

pub load_port(default_port: Int) -> Int {
  @effect io
  @effect nondet

  // 1) CLI (example: argv[1] = port)
  if cli.args_len() > 1 {
    raw := cli.arg(1).trim()
    p := convert.to_int(raw)
    if p > 0 { return p }
  }

  // 2) Env (example: PORT=8080)
  port_env := env.get("PORT")
  if port_env != "" {
    p := convert.to_int(port_env.trim())
    if p > 0 { return p }
  }

  // 3) File (example: config.json)
  cfg := fs.read_text("config.json")
  if cfg.is_ok() {
    raw := cfg.unwrap()
    if json.is_valid(raw) {
      // Parse to Json (or use json.decode for a fixed schema); see section 17.2.
      log.info("loaded config.json")
    }
  }

  // 4) Default
  default_port
}
```

Runnables you can copy from:

- `examples/07_stdlib_io_json_regex_http/55_env_cli_surface_smoke.yb`
- `examples/07_stdlib_io_json_regex_http/43_fs_read_write_exists.yb`
- `examples/07_stdlib_io_json_regex_http/45_json_basics.yb`

---

## 17.2 JSON at boundaries (preview)

For many apps, JSON is the boundary format: config files, HTTP APIs, logs.

**Typed structs + `json.encode` / `json.decode` — the preferred path**

When your data has a known shape, define a `type` and use `json.encode` /
`json.decode`. The compiler knows the fields at compile time, handles nested
structs recursively, and produces clean JSON with zero manual escaping:

```vibe
type Address { city: Str, zip: Int }
type User { id: Int, name: Str, address: Address }

user := User { id: 7, name: "sam", address: Address { city: "NYC", zip: 10001 } }

wire := json.encode(user)
// {"id":7,"name":"sam","address":{"city":"NYC","zip":10001}}

fallback := User { id: 0, name: "", address: Address { city: "", zip: 0 } }
parsed := json.decode(wire, fallback)
// missing fields fall back to the fallback value
```

**Dynamic / untyped JSON — `json.parse` + `json.stringify`**

When the shape isn't known at compile time (e.g. arbitrary config files, third-party
API responses you haven't modeled), use the runtime `Json` value type:

```vibe
doc := json.parse("{\"a\":1}")
println(json.stringify(doc))           // compact text
println(json.stringify_pretty(doc))    // indented, for debugging
println(json.stringify(json.str("v"))) // "\"v\""
```

`json.parse` returns a `Json` value; `json.stringify` turns it back into a `Str`.
Use `json.null`, `json.bool`, `json.i64`, `json.f64`, `json.str` to construct
scalar `Json` values for `stringify`.

**When to use which**

| Scenario | Use | Why |
|----------|-----|-----|
| Sending/receiving API payloads with known fields | `json.encode` / `json.decode` | Type-safe, no manual escaping |
| Parsing unknown or polymorphic JSON | `json.parse` / `json.stringify` | Runtime `Json` value is flexible |
| Building JSON with dynamic keys or arrays | `json.builder` | Streaming builder, no hand-typed `{` |
| Stringly-typed maps to JSON | `json.from_map` | Legacy convenience |

**At the wire**

- Validate unknown text with **`json.is_valid`** before **`json.parse`** if you
  need a guard; HTTP bodies and file contents are still **`Str`** until parsed.

Runnables:

- `examples/07_stdlib_io_json_regex_http/47_json_parse_stringify_and_codecs.yb`
- `examples/07_stdlib_io_json_regex_http/59_json_builder_object_basics.yb`
- `examples/07_stdlib_io_json_regex_http/62_json_builder_http_post_body.yb`

---

## 17.3 Making HTTP requests (sync-first, explicit effects)

Network calls are never “pure” in VibeLang—request APIs require `@effect net`.
The HTTP client uses structured `HttpRequest` / `HttpResponse` types
(defined in `std.http` and loaded automatically — you never need to define them yourself).

**Quick GET**

```vibe
resp := http.get("https://api.example.com/health", 3000)
if resp.status == 200 {
  println(resp.body)
}
```

`http.get` and `http.post` return `HttpResponse` with `.status`, `.headers`,
and `.body` fields.

**Full-control request with `http.send`**

For custom methods, headers, or structured payloads, build an `HttpRequest`:

```vibe
type CreateUser { name: Str, role: Str }

req := HttpRequest {
  method: "POST",
  url: "https://api.example.com/users",
  headers: "Content-Type: application/json\r\nAuthorization: Bearer tok123",
  body: json.encode(CreateUser { name: "sam", role: "admin" }),
  timeout_ms: 5000
}
resp := http.send(req)

type ApiResult { id: Int, ok: Bool }
fallback := ApiResult { id: 0, ok: false }

if resp.status == 201 {
  result := json.decode(resp.body, fallback)
  println(convert.to_str(result.id))
}
```

Use `json.encode` to serialize the request body — never hand-escape JSON strings.
Use `json.decode` to parse the response body back into a typed struct.

**Server-side responses**

When writing a server handler, use `http.response` to turn a structured
`HttpResponse` into a wire-format string, or `http.build_response` as a shortcut:

```vibe
type StatusBody { ok: Bool, message: Str }

wire := http.response(HttpResponse {
  status: 200,
  headers: "",
  body: json.encode(StatusBody { ok: true, message: "created" })
})
net.write(conn, wire)

// Convenience — adds JSON content type and CORS headers automatically
wire2 := http.build_response(200, json.encode(StatusBody { ok: true, message: "done" }))
```

Runnables:

- `examples/07_stdlib_io_json_regex_http/48_http_sync_client_unreachable_smoke.yb`
- `examples/07_stdlib_io_json_regex_http/63_http_send_structured_request.yb`

---

## 17.4 Logging + timing (preview)

Logging is a real-world requirement (and an effect). When measuring durations for timeouts
and metrics, prefer monotonic time to avoid wall-clock jumps.

Runnables:

- `examples/07_stdlib_io_json_regex_http/54_log_primitives_smoke.yb`
- `examples/07_stdlib_io_json_regex_http/53_time_monotonic_smoke.yb`

---

## 17.5 Networking foundation (preview)

For non-HTTP services (or for deterministic integration tests), the `std.net` module gives
you TCP primitives and DNS resolution under `@effect net`.

Runnables:

- `examples/07_stdlib_io_json_regex_http/49_net_listen_and_resolve_smoke.yb`

