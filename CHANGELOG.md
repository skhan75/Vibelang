# Changelog

All notable changes to this project are documented here. Every released tag
gets an entry before the tag is cut; see [RELEASING.md](RELEASING.md).

The entries for 1.1.0 through 1.6.0 were written after the fact, from the
published GitHub release notes and the commit history. Each item in them was
checked against the tagged tree, and claims that did not hold up were left out
rather than copied over.

## [Unreleased]

### Changed (breaking)

- **Immutability by default is now enforced.** A binding is immutable unless it
  is declared `mut`; `mut` is a reserved keyword for local bindings
  (`mut x := expr`) and parameters (`fn f(mut a: T)`). Reassigning an immutable
  binding is `E2110`, writing a field through one (`x.field = expr`) is
  `E2111`, and calling an in-place container method through one
  (`x.append(v)`, `x.set(k, v)`, `x.remove(k)`) is `E2112`. All three messages
  name the binding and its declaration site.
  `docs/spec/mutability_model.md` records which clauses are implemented, which
  remain targets (`const` and per-field mutability are not implemented), and
  which escape hatches still exist (aliasing is not tracked).

  Migration: add `mut` to every binding your program reassigns, writes a field
  through, or mutates in place. Re-binding with `:=` still shadows and is not
  reassignment. `mut` outside a binding (including `f(mut x)` at a call site,
  which is not a VibeLang form) is `E1213`.

  Also breaking, with no incidence in this repo: `mut` is now a reserved word,
  so an identifier named `mut` — a field (`type Flags { mut: Bool }`), a
  function, a parameter — no longer parses. Rename it.

- `E2101` ("assignment to unknown variable") now fires only when the name is
  genuinely unbound. A name that is bound but whose type is not yet known — a
  `select` receive binding, for instance — no longer reports it, which removes
  a diagnostic pair that contradicted itself about whether the name existed.

## [1.6.0] — 2026-04-02

Released from a single commit on top of 1.2.0. Versions 1.3.0, 1.4.0 and 1.5.0
were never released and never existed in `Cargo.toml`.

### Added

- **Closures and first-class functions**: closure literals in expression
  position, capture-by-value environments, and passing, returning, and calling
  function values.
- **Function-level generics (MVP)**: a single type parameter per function,
  call-site inference, and deterministic monomorphization.
- **Structured error types**: enum variants that carry payloads, with
  destructuring `match` arms.
- **`std.http_router` module**, plus header, retry, and JSON helpers in
  `std.http`.
- **`std.concurrent` module**: timeout and concurrent-map helpers built on
  `go`, `chan`, and `select`.
- **`std.metrics` module**: counters, gauges, and JSON snapshots.
- Runnable examples for closures, generics, concurrent map, HTTP routing, HTTP
  client headers and retries, metrics, and structured errors.

### Changed

- Book chapters updated for closures, generics, structured errors,
  concurrency, production patterns, and the stdlib reference.

## [1.2.0] — 2026-03-27

### Added

- **Modulo and bitwise operators**: `%`, `&`, `|`, `^`, `<<`, `>>` through the
  full pipeline from lexer to Cranelift codegen, with constant folding.
- **String interpolation**: `"Hello {name}"`, with conversion through
  `convert.to_str` and `\{` for a literal brace.
- **`type_of()` builtin**: returns the compile-time type name as a string.
- **`std.crypto` module**: `sha256`, `hmac_sha256`, `uuid_v4`, `random_bytes`,
  `constant_time_eq`.
- **`std.http_server` module**: `parse_request`, `format_response`,
  `cors_headers`.
- **`std.ws` module**: `upgrade`, `read_frame`, `write_frame`, `close_frame`.
- **`std.result` module**: `is_ok`, `is_err`, `unwrap_or`, `wrap_err`.
- **Package manager commands**: `vibe install`, `vibe add`, `vibe remove`,
  `vibe update`, and a `[vibelang]` section in `vibe.toml` that pins a minimum
  compiler version.
- Examples for the new operators, interpolation, `type_of`, HTTP server,
  crypto, WebSocket echo, and error patterns.

### Changed

- The `vibe` binary carries its own stdlib, so `VIBE_STDLIB_PATH` is no longer
  required.
- Installed packages are placed on the import path by the compiler.

### Fixed

- `json.encode(fn_call())` no longer fails codegen; return types are now
  resolved from function signatures.
- `std.json` DOM functions use the `Json` type instead of `Int`.
- `net.write` sends large HTTP responses in full instead of stopping at a
  short write.
- `vibe build main.yb` with a bare filename no longer crashes.
- `json.encode` warns at compile time on structs nested deeper than 16 levels.

## [1.1.1] — 2026-03-25

### Changed

- **Self-hosted stdlib**: 15 standard library modules moved out of hardcoded
  compiler tables into `.yb` modules under `stdlib/std/`.
- The compiler resolves `namespace.function()` calls to compiled module
  functions through a namespace-to-module map in the compilation pipeline.
- C-backed stdlib functions are declared with `@native("symbol")` instead of
  being wired in by the compiler.
- `HttpRequest` and `HttpResponse` are defined in `stdlib/std/http.yb` rather
  than injected by the compiler.
- `json.encode` / `json.decode`, `json.builder.*`, and `simd.*` stayed as
  compiler special cases.

### Fixed

- Updated `rustls-webpki` to 0.103.10 for RUSTSEC-2026-0049.
- Corrected the `cargo-deny` license allowlist and the secret-scan workflow.

## [1.1.0] — 2026-03-14

### Added

- **AI sidecar intent drift detection**: `vibe lint --intent --mode hybrid`
  compares each function's `@intent` against its implementation and reports
  drift as `W0801` with a confidence score and an explanation.
- **Bring your own key**: the sidecar reads `ANTHROPIC_API_KEY` from the
  environment or `~/.config/vibe/sidecar.toml`, and calls the Anthropic API
  directly from the developer's machine. VibeLang hosts no proxy.
- **`--suggest`**: drafts `@require` / `@ensure`, `@examples`, and `@intent`
  annotations, revalidated by the compiler before being shown.
- **`[sidecar]` section in `vibe.toml`** for the model, the endpoint, the cache
  TTL, and string redaction.

### Changed

- String literals are redacted before any request leaves the machine when
  `redact_strings` is set.
- Cloud sidecar support sits behind the `cloud` feature of `vibe_sidecar`,
  which is on by default.
- Local heuristic intent checks continue to run with no API key configured.

## [1.0.2] — 2026-03-11

### Added

- **Float codegen**: Native `Float` (f64) type with full arithmetic, `math.sqrt`,
  and `convert.format_f64` / `i64_to_f64` / `f64_to_bits` / `f64_from_bits`.
- **MIR optimization passes**: Constant folding (Int, Float, Bool, Str), dead code
  elimination, function inlining (≤12 stmts), loop-invariant code motion.
- **`str_builder` module**: Efficient O(N) amortized string construction via
  `new`, `append`, `append_char`, `finish`.
- **`regex` module**: POSIX regex support with `count` and `replace_all`.
- **Benchmark suite**: 18-program PLB-CI suite with Docker-reproducible runs
  against C, C++, Rust, Zig, Python, TypeScript, PHP, and Elixir.
- **Benchmark policy**: Apples-to-apples publication policy with strict
  reproducibility requirements.
- **Landing page**: `/benchmarks` page with full results, methodology, and
  known limitations; `/benchmarks/policy` page.
- **Documentation**: Chapter 17 (Building Real Apps), expanded stdlib reference
  (Appendix C) with `str_builder`, `regex`, and additional `convert` functions.
- Phase 6 ecosystem baseline:
  - `vibe new`, `vibe fmt`, `vibe doc`, and `vibe pkg` command flows.
  - Package manager foundation (`vibe.toml`, deterministic resolver, lockfile,
    offline mirror install flow).
  - Self-host seed component and conformance harness.
  - Policy docs, migration guides, release process, target governance docs.
  - Metrics collection scripts and CI workflow gates.

### Changed

- Source extension policy now treats `.yb` as canonical and `.vibe` as legacy in
  v1.x migration window.
- Default metadata and scaffold conventions favor `.yb`.
- String `+` operator now uses `str_builder` internally (O(N) amortized, was
  O(N²) repeated concat).
- LRU benchmark adapter uses amortized O(1) FIFO eviction queue (was O(N)
  linear scan).
- `List<Float>` access uses zero-cost bitcast (was bit-packing workaround).
- Compiler internals documentation corrected: removed CSE from MIR passes list
  (not implemented), fixed stale chapter references.

### Fixed

- Added explicit guard for same-stem mixed extension collisions.
- Float codegen type mismatch errors in Cranelift resolved.
- Benchmark adapters rewritten to read `.benchmark_input` and use canonical
  problem sizes (was hardcoded/fake).
- Docker benchmark Deno install made resilient to network failures.

### Performance

- **vs C**: Geomean 0.89x (VibeLang 1.1x faster, 3 shared benchmarks).
- **vs Rust**: Geomean 0.93x (VibeLang 1.1x faster, wins 9 of 16 shared
  benchmarks).
- **vs Zig**: Geomean 0.82x (VibeLang 1.2x faster, 12 shared benchmarks).
- **vs Python**: ~20x faster. **vs TypeScript**: ~8.6x faster.
- Compiles 1.5x faster than Rust; 328 KB hello-world binary.

### Migration Notes

- See `docs/migrations/v1_0_source_extension_transition.md`.
