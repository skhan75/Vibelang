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
      // In preview, many JSON helpers operate on JSON text.
      // Use the runnable examples below for the current semantics.
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

In the current preview surface, `std.json` provides helpers for:

- validation (`json.is_valid`)
- canonicalization / quoting (`json.parse`, `json.stringify`)
- typed codec entrypoints for nominal `type` values (`json.decode_<Type>`, `json.encode_<Type>`)

Runnables:

- `examples/07_stdlib_io_json_regex_http/47_json_parse_stringify_and_codecs.yb`

---

## 17.3 Making HTTP requests (sync-first, explicit effects)

Network calls are never “pure” in VibeLang—request APIs require `@effect net`.
The sync-first client returns response body text and/or status.

Runnables:

- `examples/07_stdlib_io_json_regex_http/48_http_sync_client_unreachable_smoke.yb`

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

