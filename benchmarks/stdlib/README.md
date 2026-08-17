# Stdlib migration benchmarks

Differential benchmark harness for the C-to-VibeLang stdlib migration. Each
case compiles two `.yb` programs that do the same job through two different
paths — one backed by an existing native (C) stdlib call, one written as a
pure VibeLang loop — and reports wall time and peak RSS for both, measured
from outside the process (`/usr/bin/time -l`, macOS only for now).

## Per-tier gate

- **Tier A** (scanning and comparing, no allocation): the VibeLang side must
  land within 5% of the C side's wall time **and** within `RSS_GATE_MULT`
  (default 1.5x, see below) of the C side's peak RSS, or the case fails
  with exit 1. This is the strict gate; it is meant to block a merge.
- **Tier C / D**: both ratios are printed and recorded, but neither fails
  the case. These tiers are not gated until the collector and the rest of
  the type work land, at which point they inherit the tier A gate.
- **Every tier, gated or not**: the two programs' final printed output must
  be identical, or the case fails regardless of tier. A benchmark whose job
  is guarding a migration's correctness *and* speed cannot skip the
  correctness half — a `_vibe.yb` that scans zero bytes or computes the
  wrong count must not be able to pass on an excellent time.

Both wall time and peak RSS are gated (for tier A) or recorded (for other
tiers) because a VibeLang implementation that allocates in a loop can match
C on time while using far more memory — and until a garbage collector
exists, that is the expected failure mode for anything past tier A. See
"Alternative formulations considered" below for a measured example of
exactly that failure mode.

## Usage

```
benchmarks/stdlib/run.sh <case> <tier A|C|D> [iterations]
```

`iterations` (default 200000) is how many times the whole scan-and-count
repeats inside the compiled binary, not the buffer size — a single 64 KB
scan finishes in well under a millisecond, too fast for `/usr/bin/time` to
resolve reliably, so the case repeats the work internally until the total
run is long enough to measure. Requires a release build first:
`cargo build --release -p vibe_cli`.

Each binary is run once (discarded) as a warmup, then `MEASURE_RUNS`
(default 5, override with the env var) times, keeping the minimum wall
time and the peak RSS from that same run. See "What tolerance can this
instrument resolve" below for why, and what that buys.

## `bytes_scan`: what it measures, and why it is (and isn't) like-for-like

The original design was: count occurrences of one byte value across
a 64 KB `Bytes`, the C side via a loop of `text.index_of` calls, the
VibeLang side via a `bytes.get` loop. That design does not survive contact
with the actual API surface, for two independent reasons, both discovered
by trying to write it:

1. **`text.index_of(haystack, needle)` takes no start offset.** Finding a
   second occurrence after the first means either re-searching the whole
   haystack (wrong answer) or reslicing the haystack from the last match to
   the end and searching again. Reslicing allocates a new `Str` on every
   match. For a byte value that occurs, say, a few hundred times in 64 KB,
   that is a few hundred allocations and copies on the "C" side alone —
   which is not a tier A case at all (tier A is defined as no allocation),
   and it inflates the C side's time for a reason that has nothing to do
   with how fast C can scan a buffer.
2. **Bytes -> Str conversion is lossy on embedded `0x00`, downstream of the
   conversion itself.** `bytes.to_str` (`vibe_bytes_to_str` in the C
   runtime) copies every byte verbatim and then appends a NUL terminator —
   it does not stop early. The loss happens one step later, in whatever
   consumes the result as a C string: `strlen`, `println`, `text.*`, all of
   which stop at the first `0x00` because that is what "the length of a
   `Str`" means in this representation. Practically the effect is the same
   (any embedded NUL truncates what a text function can see), but the
   mechanism matters for reasoning about where an allocation happens versus
   where data is lost.

The stdlib does have a native call that counts occurrences in one shot:
`regex.count(text: Str, pattern: Str) -> Int` (`stdlib/std/regex.yb:9-12`,
backed by `vibe_regex_count`). It doesn't fit this case, though:
`regex.count` takes a `Str`, not `Bytes`, so using it on the VibeLang side
would mean converting the 64 KB buffer to a `Str` first — an allocation
this benchmark is built to avoid. It would have been usable on the C side
instead (which already converts the buffer to a `Str` once, see below), and
a faster, more accurate C-side count only widens the gap this benchmark
measures, not narrows it. Among the modules this case actually draws on
(`bytes.*`, `text.*`, `convert.*`, `math`, `fs.read_bytes`/`write_bytes`/
`size`, `net.read_bytes` — a handful of the 23 modules in `stdlib/std/`,
not the whole stdlib), there is no native call that counts multiple
occurrences of a byte in a `Bytes` buffer in one shot without allocating.
So this harness ships a corrected version of the same idea instead of the
literal sketch:

- Both `bytes_scan_c.yb` and `bytes_scan_vibe.yb` build the **identical**
  deterministic 64 KB buffer: 65,535 bytes of `0x41` ("A") followed by a
  single `0x42` ("B") marker as the very last byte (65,536 total).
  Construction (`14` doublings of a 1-byte seed via `bytes.concat`, plus
  one `bytes.slice` and one more `concat` to land on exactly 65,536)
  happens once, outside the timed repeat loop, so it is a small, roughly
  equal, one-time cost on both sides rather than something that dilutes
  the per-repetition ratio. Real, measured one-time cost: well under 5 ms
  (see "process startup" correction below) — negligible next to a
  multi-second timed loop.
- `bytes_scan_c.yb` converts the buffer to a `Str` once (safe here — no
  embedded NUL, so nothing is lost) and, per repetition, makes **one**
  native call to `text.index_of`, backed by `vibe_text_index_of`
  (`strstr` under the hood, called with a *runtime* needle — see below for
  why that distinction matters) in the C runtime. No allocation, one
  native crossing per repetition, and the whole traversal happens inside
  C.
- `bytes_scan_vibe.yb` does **not** call into any bulk native operation.
  Per repetition it runs a VibeLang `while` loop over all 65,536 positions,
  calling `bytes.get` once per byte (the only way VibeLang source can read
  a byte — it has no raw memory access, and `Bytes` cannot be indexed with
  `b[i]` at all: that is `E2233`, "indexing is only supported for
  `List<T>`, `Map<K,V>`, and `Str`" —
  `crates/vibe_types/src/lib.rs:2699-2708`), comparing to the target and
  counting. No allocation, but 65,536 native crossings per repetition
  instead of one.
- **The marker sits at the very last byte on purpose.** `text.index_of`
  short-circuits on the first match. If the marker were anywhere else, the
  C side could stop early while the VibeLang side (which has to count, not
  just detect, so it never exits early) kept going — that would measure
  "did the C call get lucky" rather than "how fast is a full scan," which
  is the actual thing tier A migrations need to know. Putting the marker
  last forces both sides to examine (almost) every byte.
- The one place this still isn't perfectly symmetric: `text.index_of`
  reports presence/absence of a single match rather than a running count of
  several. With the marker constrained to one occurrence (for the fairness
  reason above), "count" and "found" coincide (0 or 1), so this does not
  change what's being measured — a full linear pass with a comparison per
  byte — but it means the case no longer literally exercises "count
  occurrences" in the multiple-hits sense originally intended.

**What this case actually measures:** the cost of doing a full-buffer
linear scan as ONE native (C) call versus the same logical scan expressed
as 65,536 separate native accessor calls driven by a VibeLang loop. That is
exactly the shape a real tier A migration will hit: today's C
implementation of `text.index_of` does its entire scan inside one C call;
a VibeLang reimplementation can only touch memory through the existing
per-element native accessors, so the loop and the native-call frequency
both move from 1 to N. That is a like-for-like comparison of "the
operation," even though the *mechanism* on each side is now deliberately
different (bulk native call vs. per-element native call in a VibeLang
loop) — which is the actual question a migration decision needs answered.

## What tolerance can this instrument resolve

A single, unwarmed run of each binary is not precise enough to judge a 5%
budget. Proven with a byte-identical probe pair (same `.yb` file on both
sides — the true ratio is exactly 1.000x): six single-shot runs of the
pre-fix harness (one exec per binary, no warmup) measured 1.299x, 1.078x,
1.037x, 0.650x, 0.987x, 1.092x against that same 5% budget — verdict
flipped 3 PASS / 3 FAIL on a case with zero real difference. The dominant
cause: macOS validates a freshly written binary's code signature on its
first-ever exec, and `run.sh`'s `build()` always compiles into a fresh
`mktemp -d`, so every measured binary WAS on its first exec. Proven in
isolation, one binary, three consecutive runs of the identical workload:
0.34s, then 0.00s, then 0.00s.

The fix (warmup exec discarded, then minimum-of-5) was run against the same
identical-pair probe, twice, at two very different workload sizes:

```
iterations=30000 (~3.4s per binary): 1.000x, 1.000x, 1.000x, 1.003x, 1.000x, 1.000x
iterations=200   (~0.03s per binary): 1.000x, 1.000x, 1.000x, 1.000x
```

Ten runs, ten verdicts of PASS, range 1.000x-1.003x. That is the tolerance
this instrument can actually resolve on this machine today: comfortably
under 1%, an order of magnitude tighter than the 5% budget it is enforcing.
This does not mean every case on every machine will be this clean — it
means the specific failure mode that produced ±30% noise (cold-exec code
signing, paid once and shared unevenly across a single-shot measurement)
is fixed, not that all timing noise is eliminated.

## Root cause of the gap: not vectorization, and mostly not FFI crossing

An earlier draft of this document attributed the gap to `text.index_of`
resolving to a vectorized libc routine (`strstr`/`memchr`) against a scalar
VibeLang loop. That attribution does not survive an actual measurement of
`strstr`, and it flattered the result — it made the gap sound like an
unfair contest (hand-tuned SIMD vs. a plain loop) rather than what it
actually is.

**The trap, found and reproduced on this machine:** `clang -O2` rewrites
`strstr(haystack, "B")` — a *literal* needle — straight into `strchr`:

```
$ clang -O2 -S -o - strstr_check.c
_lit:                    ; strstr(h, "B")
	mov	w1, #66
	b	_strchr          ; tail-called to strchr, NOT a strstr call at all
_rt:                     ; strstr(h, n) -- n is a runtime value
	b	_strstr          ; genuinely calls strstr
```

A casual check that passes a literal needle is measuring `strchr`, not
`strstr` — and `strchr`, like `memchr`, is vectorized. `vibe_text_index_of`
calls `strstr(h, n)` where `n` is a function parameter (a runtime value),
so it takes the un-rewritten path and never gets that optimization.
Independently measured on this machine, all with a `volatile`-forced
memory barrier every iteration to stop the *loop* itself from being
constant-folded (a second trap: a naive microbenchmark loop over the same
buffer and needle is loop-invariant and gets hoisted out entirely,
producing a nonsense near-zero time):

| routine | ns/byte | note |
|---|---|---|
| `memchr` | 0.016 | genuinely vectorized (NEON) |
| `strstr`, runtime needle | 0.25-0.27 | statistically the same as scalar C below |
| scalar C loop, `-fno-vectorize` | 0.249 | true, unvectorized baseline |
| scalar C loop, plain `-O2` | 0.12-0.15 | **auto-vectorized by clang** (confirmed via `-S`: `cmeq.16b`/`dup.2d` NEON opcodes appear) — a second instance of the same trap, this time in the "scalar" control itself |

`strstr` with a runtime needle is not vectorized and is statistically
indistinguishable from a genuine scalar C loop. `bytes_scan_c.yb` is
therefore not winning by SIMD; it's winning by doing the whole scan inside
one C function call versus 65,536 VibeLang-driven native calls.

**Where the real cost is**, decomposed with a control case (identical loop
shape, no native call at all — just `i == target` against a local `Int`,
65,536 comparisons per repetition):

- Cranelift-compiled loop alone (no native call): ~1.05 ns/iteration —
  already several times a true scalar C loop's 0.25 ns/byte. This is
  ordinary codegen-quality overhead (no auto-vectorization, plus whatever
  bounds/loop bookkeeping VibeLang's `while` emits), not something this
  benchmark can separate further without profiling the generated code.
- Adding the `bytes.get` native call back in: ~2.0 ns/iteration — roughly
  double the loop-only cost. **This doubling has a specific, verified
  mechanism, not just "FFI overhead" in the abstract.** `bytes.get` is a
  `pub` stdlib function (`stdlib/std/bytes.yb:23-26`) whose body is a
  single `@native("vibe_bytes_get")` call — a trampoline. VibeLang's
  small-function inliner (`inline_small_functions`,
  `crates/vibe_mir/src/optimize.rs:596-608`) requires `!f.is_public` to
  even consider a function for inlining. Checked directly: all 121
  `@native`-annotated function bodies across `stdlib/std/*.yb` sit inside a
  `pub` signature (`awk` count, zero exceptions). So the trampoline is
  never inlined, and every `bytes.get` call in a VibeLang loop is actually
  **two** calls — one to the VibeLang trampoline function, one from the
  trampoline into the C accessor — where a hand-written equivalent would
  need only one.

**Corrected conclusion:** VibeLang did not lose to SIMD here. It lost to
(a) ordinary scalar-loop codegen overhead versus a real scalar C loop, and
(b) a structural double-call tax on every native stdlib accessor, because
every `@native` declaration in the stdlib is `pub` and the inliner
categorically skips public functions. Neither of those is "the compiler is
slow at loops in general" — they're two specific, separately fixable
things.

**One more correction, to the same document that made the vectorization
claim**: it also attributed roughly 0.2-0.3s per side to "process startup
and setup," inferred by curve-fitting several ratio measurements. That
number was an artifact of the same cold-exec problem described above, not
real cost. Measured directly: warmed up once, then five runs of the
`bytes_scan_c.yb` binary at `iterations=1` (isolates process startup plus
the one-time 64 KB buffer construction and `Str` conversion, with no real
scan-loop cost) — all five rounded to `0.00 real`, i.e. under
`/usr/bin/time`'s ~10ms resolution. Real startup-plus-setup cost is a few
milliseconds at most, not hundreds. The earlier curve fit was measuring the
code-signing tax bleeding unevenly across the fitted data points, not
genuine fixed overhead.

## Result: `bytes_scan`, tier A

Five independent runs, warmup + minimum-of-5, `iterations=30000`, taken
across this fix round (not cherry-picked — this is every run of the real
case at this iteration count):

```
run 1:  C 0.49s  VibeLang 3.42s  ratio 6.980x  rss 0.954x  FAIL
run 2:  C 0.49s  VibeLang 3.42s  ratio 6.980x  rss 0.954x  FAIL
run 3:  C 0.49s  VibeLang 3.44s  ratio 7.020x  rss 0.954x  FAIL
run 4:  C 0.53s  VibeLang 3.42s  ratio 6.453x  rss 0.954x  FAIL
run 5:  C 0.52s  VibeLang 3.48s  ratio 6.692x  rss 0.954x  FAIL
```

**Measured ratio: 6.45x-7.02x depending on background system load during
the run, median ~6.98x.** This supersedes an earlier, noisier reading of
~4.7x-5.0x taken before the warmup/min-of-5 fix — that reading understated
the gap because the shared cold-exec cost diluted it. The verdict (FAIL)
was identical in all five runs, and the RSS ratio was identical to three
decimal places in all five (0.954x) — the run-to-run variance is entirely
in the wall-time ratio, and even at its narrowest (6.45x) the gate misses
by a factor of well over 6, not a few percent. Worth being explicit about
what this range means for the "what tolerance can this instrument
resolve" claim above: an *identical* pair resolves to under 1% noise, but
two genuinely different code paths with different absolute costs (C here
runs in ~0.5s, about seven times faster than VibeLang's ~3.4s) can still show
several-percent run-to-run drift from ambient system load even with
warmup and minimum-of-5 — irrelevant at a 6-7x gap, but worth knowing
before trusting this harness to resolve a case that lands close to the 5%
line.

Peak RSS: VibeLang (1664 KB) was slightly *lower* than C's (1744 KB), ratio
0.954x, comfortably inside the 1.5x tier A RSS budget, and stable across
every run. This case doesn't hit the "matches on time but leaks on memory"
failure mode described above — see the next section for a formulation
that does.

**This is not being reported as a pass.** A byte-scan loop is the most
favorable case this migration will ever have — no allocation, one
comparison per byte, no branching complexity — and it still misses the
tier A budget by a factor of ~7. That means either (a) tier A migrations
as currently scoped (a VibeLang loop calling a per-element native
accessor) cannot meet a 5% budget against C's one-call implementations
with the compiler and stdlib calling convention as they stand today, or
(b) the 5% number itself needs revisiting. Both remain open. The data does
implicate two concrete, separately actionable things (see "Named
unblockers" below) rather than pointing at "make the compiler
auto-vectorize," which the measurements above rule out as the fix.

## Alternative formulations considered

Two other ways to write the VibeLang side were built and measured, to
check whether the per-byte-native-call shape is the only option (it isn't)
and what the alternatives actually cost (both still miss tier A, for
different reasons). Not committed as gated cases — built, measured, and
deleted the same way the gate-fail probe is; numbers recorded here.

**`List<Int>` + `xs[i]`, instead of `bytes.get`.** Convert the `Bytes`
buffer to a `List<Int>` once (an O(n) `bytes.get` pass, outside the timed
loop), then scan with bracket indexing. `xs[i]` on a `List<Int>` is the one
Bytes-adjacent read that does NOT go through a native call: it lowers
straight to a bounds-checked Cranelift load
(`crates/vibe_codegen/src/lib.rs:3110-3157` — load length at offset 8,
bounds-check, load the element at offset 24 plus `index*8`), the same
shape as any other in-language array access. Measured, `iterations=30000`,
same buffer, same warmup+min-of-5 methodology:

```
case=bytes_scan_list tier=A iterations=30000 runs=5(min)
  C       : 0.49s  peak 1744KB
  VibeLang: 1.99s  peak 2704KB
  ratio   : 4.061x wall time (lower is better)
  rss     : 1.550x peak RSS  (lower is better)
FAIL: tier A requires the VibeLang implementation within 5% of C on wall time
FAIL: tier A requires the VibeLang implementation within 1.5x of C on peak RSS
```

Faster than the `bytes.get` loop (6.980x -> 4.061x, i.e. the direct-load
path is about 1.7x faster than the double-call trampoline path — the same
direction and a similar magnitude to the codegen argument above) but still
4x over the tier A time budget, and the one-time list-building pass pushes
peak RSS to 1.55x of C's — just over this harness's own RSS gate. Unlike
the next formulation, this is a *fixed*, one-time cost (building the list
once), not a per-repetition leak — worth distinguishing, because the two
failure shapes need different fixes.

**Chunked `bytes.slice` + content comparison, instead of per-byte
comparison.** Scan the buffer in 1024-byte chunks, and per chunk ask "does
this chunk differ from a known all-`0x41` reference chunk?" One wrinkle
found while building this: **`Bytes == Bytes` is pointer/handle equality,
not content equality** — verified directly:

```
a := bytes.from_hex("aabb")
b := bytes.from_hex("aabb")   // same content, different allocation
a == b                         // -> false
d := a
a == d                         // -> true (same handle)
```

So content comparison has to go through `bytes.to_hex(chunk) ==
bytes.to_hex(reference)` (a `Str` comparison), which is itself an extra
allocation per chunk on top of the `bytes.slice` copy. Measured,
`iterations=3000` (a smaller repeat count than the other cases — see why
below), 1024-byte chunks, same warmup+min-of-5 methodology:

```
case=bytes_scan_chunk tier=A iterations=3000 runs=5(min)
  C       : 0.05s  peak 1744KB
  VibeLang: 0.14s  peak 778720KB
  ratio   : 2.800x wall time (lower is better)
  rss     : 446.514x peak RSS  (lower is better)
FAIL: tier A requires the VibeLang implementation within 5% of C on wall time
FAIL: tier A requires the VibeLang implementation within 1.5x of C on peak RSS
```

Wall time is the closest of the three to C (2.8x — fewer, bulkier native
calls per repetition genuinely help), but peak RSS is 778 MB against C's
1.7 MB: a **446x** blowup, because every one of the 64 chunks allocates a
1024-byte slice and a ~2 KB hex string, every repetition, and nothing ever
frees them (no collector exists). This is exactly the harness's own
feared failure mode, live: closer to parity on time, catastrophic on
memory. `iterations=3000` (not 30000) is already enough to demonstrate it
cleanly without exhausting machine memory during measurement; at the same
30,000 iterations used elsewhere this would be on the order of several
gigabytes.

**Net read on batching:** moving from one native call per byte to fewer,
bulkier native calls (List indexing, chunked slicing) does reduce the wall
time gap — but every version tried that gets closer to C parity does so by
allocating, which tier A forbids outright and which the missing collector
turns from "against policy" into "unboundedly leaks memory in production."
Nothing tried reached the 5% tier A time budget without leaning on
allocation.

**A construct that must never be reached for in a migration, found while
building the above:** `text.byte_at(s, i)` and `s[i]` (`Str` indexing) both
call `strlen` on *every single access*
(`runtime/native/vibe_runtime.c:1739-1749` for `vibe_str_get_byte`, backing
`s[i]`; `:1829-1836` for `vibe_str_byte_at`, backing `text.byte_at`). A
scan loop built on either is not O(n), it's O(n²). Measured directly (own
micro-probe, not the gated harness — the point is the trend, not a
gate-comparable number):

| buffer | `bytes.get` loop | `text.byte_at` loop | ratio |
|---|---|---|---|
| 4,096 B (2000 full-buffer scans) | 0.02s | 0.69s | ~34.5x |
| 16,384 B (2000 full-buffer scans) | 0.05s | 9.13s | ~182.6x |

The ratio roughly quadrupled when the buffer size quadrupled, exactly the
signature of comparing an O(n) loop to an O(n²) one — at the 64 KB scale
this harness's other cases use, extrapolating that trend puts the gap in
the same order of magnitude as several hundred times, not measured
directly here because a single full run at that scale would itself take
minutes. A migration reaching for `text.byte_at` or `s[i]` in a per-byte
scan loop would not just miss tier A — it would regress catastrophically
on any input larger than the ones it happened to get tested on.

## Named unblockers

The data above implicates two specific, separately shippable changes, not
"make the compiler faster" in general and not auto-vectorization (ruled
out above — the C side this benchmark competes against isn't vectorized
either):

1. **Collapse the `@native` trampoline.** Every `@native`-declared stdlib
   function is `pub`, and `inline_small_functions` categorically skips
   `pub` functions, so every stdlib accessor call is two calls (VibeLang
   trampoline, then the native function) instead of one. Making native
   trampolines inlinable (or otherwise collapsing the indirection) would
   remove roughly half of the per-call overhead measured above without
   touching a single migrated function's logic.
2. **Add zero-copy bulk `Bytes` primitives** (`bytes.index_of`,
   `bytes.eq`, a view/subslice type that doesn't copy) so a migrated
   scan/compare function can do its work in one native crossing over the
   real buffer, the way `text.index_of` does today, instead of forcing a
   choice between "one call per byte" (slow, tier A compliant) or
   "allocate a copy to get fewer calls" (faster, but not tier A compliant
   until the collector exists).

Neither of these is optional plumbing — until at least one lands, no
scan/compare function migrated the way `bytes_scan_vibe.yb` demonstrates
can plausibly clear the tier A gate as agreed.

## Proving the gate fails when it should

A gate that has never failed is not known to work. During development, a
throwaway `slow_probe` case (VibeLang side: the same trivial sum as the C
side, plus a deliberate loop that redoes 50x more work, discarded so both
sides still print the same total) was built, run through the current
(warmup + min-of-5 + output-check + RSS-gate) harness, and confirmed to
fail with exit 1, the `FAIL: tier A requires ... on wall time` message, and
a correct output match:

```
case=slow_probe tier=A iterations=20000 runs=5(min)
  C       : 0.01s  peak 1376KB
  VibeLang: 0.51s  peak 1376KB
  ratio   : 51.000x wall time (lower is better)
  rss     : 1.000x peak RSS  (lower is better)
  output  : matches (9990000000)
FAIL: tier A requires the VibeLang implementation within 5% of C on wall time
```

The RSS gate was proven separately by the `bytes_scan_chunk` alternative
above (778 MB vs. 1.7 MB, correctly triggers `FAIL: ... within 1.5x of C on
peak RSS`). All probe/alternative case files were deleted after use.

## Adding a new case

1. Write `<name>_c.yb` and `<name>_vibe.yb` under `cases/`. Both must read
   the repeat count from `cli.arg(0)` (via `convert.to_int`) and loop that
   many times, printing a result at the end so the loop can't be trivially
   optimized away — **and both must print the same result**, or the
   harness fails the case regardless of tier.
2. Decide the tier: A if the operation only scans/compares without
   allocating anywhere in the timed path (on *either* side, not just the
   VibeLang one); otherwise C or D per the migration plan.
3. Run `benchmarks/stdlib/run.sh <name> <tier> [iterations]`, tuning
   `iterations` so the total run is long enough (at least a few hundred ms
   per side, ideally low seconds) that process-startup and one-time setup
   cost don't dominate the measurement. `MEASURE_RUNS` and `RSS_GATE_MULT`
   are overridable via environment variables if a case needs a different
   noise/RSS tolerance, but changing either for tier A should be treated
   like changing the gate itself — a decision to record, not a default to
   quietly tune per case.
