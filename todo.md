# VibeLang Compiler Backlog

Scoped follow-ups parked during active work. Newest at top; finished items move to `git log`.

- types: opaque nominal handle types for str_builder/net/ws (P0-13 layer 2, parked): the Json/JsonBuilder pattern is a hardcoded variant across `vibe_types::TypeKind` + `vibe_mir::MirType` + codegen mappings (new machinery per family), and net handles can't change type without breaking corpus code that annotates them as Int (e.g. `66_websocket_echo.yb` `ws_handle(conn: Int)`). Runtime live-handle registry shipped instead; revisit if a general opaque-handle TypeKind lands. Note: user code can bind `@native(...)` directly, which bypasses any nominal typing — consider restricting @native to stdlib modules.
- mir: the inliner substitutes raw argument expressions into the callee body, so a side-effecting argument is evaluated the wrong number of times, including zero. `x := maybe(false, bump())` where `maybe` only uses the parameter in an untaken branch prints `bump` then `0` in dev and just `0` in release (verified 2026-08-15). Fix needs argument pre-binding to temps before substitution.
- mir: `?` in a call argument position to an inlinable callee fails the release build. `t := twice(parse_it(s)?)` reports "codegen failed: unknown local t in function run" (verified 2026-08-15); the callee itself is `?`-free, so the tail-position and `?` candidate filters do not cover this shape.
- mir: `substitute_stmts` gives `Select` and `Match` statements no parameter substitution, so inlining a callee that matches on a parameter fails the release build with "unknown local c in function main" (verified 2026-08-15). Either substitute into both, or exclude such callees.
- codegen: assignment inside a `match` arm is silently a no-op. `match c { case Color.Blue => out = "blue" ... }` leaves `out` at its initial value in dev while arm dispatch itself works (verified 2026-08-15 with no inlining involved); either lower the assignment or reject it with a diagnostic.
- types: `convert.to_str` of a user-function call returning Str prints a pointer — HIR passthrough keys on the type hint while inference disagrees (reproduced 2026-08-13)
- types: closure literals bypass the return-conflict (E2201) and unknown-type (E2005) checks; call-site E2265 only partially compensates
- types: if/else as tail expression infers Void, causing a spurious E2201 on functions with a declared return type
- types: `@require` diagnostics are emitted twice (contract expression walked by both validation and lowering)
- diagnostics: E2005 spans anchor on the declaration rather than the offending type annotation — TypeRef nodes carry no span
- diagnostics: monomorphized return-conflict messages leak mangled instance names (e.g. `pick__Int`)
- codegen: float-to-string paths fail with spanless Cranelift verifier errors (interpolation, println, and the unused `to_str_f64` route)
- tooling: 13 example `.yb` files are absent from `examples_manifest.json` and escape the suite (how the Str-interpolation regression stayed invisible); add a full-corpus build sweep to CI
- tests: `compiler/tests/fixtures/stdlib/fs/basic.yb` writes `phase12_fs_fixture.txt` into the current working directory instead of a temp dir, leaving an untracked artifact after any local `vibe test` run (same class as the committed `examples_fs_demo.txt` scratch file)
- perf: `metrics_threshold_smoke` fails the v1 quality budget — `index_memory_ratio=12.58` exceeds `max_index_memory_ratio=10.0`. Pre-existing (the standalone phase6-metrics lane failed the same way before this work). Needs either indexer memory work or a re-baselined budget with justification.
- ci: `compiler/tests/fixtures/stdlib/**` is only exercised by the `phase12_stdlib_gate` CI job, not by workspace tests or the corpus sweep — which is why an ill-typed fixture (`println` of an `HttpResponse`) stayed invisible locally. Fold these fixtures into a Rust-side test or the sweep.
