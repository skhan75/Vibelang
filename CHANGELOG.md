# Changelog

All notable changes to this project are documented here.

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
