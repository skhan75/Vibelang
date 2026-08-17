# Stdlib migration benchmarks

Differential benchmark harness for the C-to-VibeLang stdlib migration. Each
case compiles two `.yb` programs that do the same job through two different
paths — one backed by an existing native (C) stdlib call, one written as a
pure VibeLang loop — and reports wall time and peak RSS for both, measured
from outside the process (`/usr/bin/time -l`, macOS only for now).

## Per-tier gate (decision Q6, approved 2026-08-16)

- **Tier A** (scanning and comparing, no allocation): the VibeLang side must
  land within 5% of the C side's wall time or the case fails with exit 1.
  This is the strict gate; it is meant to block a merge.
- **Tier C / D**: the ratio is printed and recorded, but nothing fails.
  These tiers are not gated until the collector and the rest of the type
  work land (see the plan's phase 2/3 notes), at which point they inherit
  the tier A gate.

Both wall time and peak RSS are reported for every case, gated or not,
because a VibeLang implementation that allocates in a loop can match C on
time while using far more memory — and until a garbage collector exists,
that is the expected failure mode for anything past tier A.

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

## `bytes_scan`: what it measures, and why it is (and isn't) like-for-like

The brief's original sketch was: count occurrences of one byte value across
a 64 KB `Bytes`, the C side via a loop of `text.index_of` calls, the
VibeLang side via a `bytes.get` loop. That sketch does not survive contact
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
2. **Bytes -> Str conversion is lossy on embedded `0x00`.** `bytes.to_str`
   stops at the first zero byte (see `stdlib/bytes/README.md`). If the
   scanned buffer contains any NUL anywhere before the region of interest,
   the "C" side would silently be scanning a truncated prefix, not the
   full 64 KB the case claims to cover.

Given the currently available stdlib (`bytes.*`, `text.*`, `convert.*`,
`math`, `fs.read_bytes`/`write_bytes`/`size`, `net.read_bytes`), there is no
native call that counts multiple occurrences of a byte in one shot without
allocating. So this harness ships a corrected version of the same idea
instead of the literal sketch:

- Both `bytes_scan_c.yb` and `bytes_scan_vibe.yb` build the **identical**
  deterministic 64 KB buffer: 65,535 bytes of `0x41` ("A") followed by a
  single `0x42` ("B") marker as the very last byte. Construction (`14`
  doublings of a 1-byte seed via `bytes.concat`, plus one `bytes.slice` and
  one more `concat` to land on exactly 65,536) happens once, outside the
  timed repeat loop, so it is a small, roughly equal, one-time cost on both
  sides rather than something that dilutes the per-repetition ratio.
- `bytes_scan_c.yb` converts the buffer to a `Str` once (safe here — no
  embedded NUL, so nothing is lost) and, per repetition, makes **one**
  native call to `text.index_of`, backed by `vibe_text_index_of`
  (`strstr` under the hood) in the C runtime. No allocation, one native
  crossing per repetition, and the whole traversal happens inside C.
- `bytes_scan_vibe.yb` does **not** call into any bulk native operation.
  Per repetition it runs a VibeLang `while` loop over all 65,536 positions,
  calling `bytes.get` once per byte (the only way VibeLang source can read
  a byte — it has no raw memory access), comparing to the target and
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
  occurrences" in the multiple-hits sense the brief's wording suggested.

**What this case actually measures:** the cost of doing a full-buffer
linear scan as ONE native (C) call versus the same logical scan expressed
as 65,536 separate native accessor calls driven by a VibeLang loop. That is
exactly the shape a real tier A migration will hit: today's C
implementations of things like `text.index_of` do their entire scan inside
one C call (often backed by a vectorized libc routine — see the result
below); a VibeLang reimplementation can only touch memory through the
existing per-element native accessors, so the loop and the native-call
frequency both move from 1 to N. That is a like-for-like comparison of
"the operation," even though the *mechanism* on each side is now
deliberately different (bulk native call vs. per-element native call in a
VibeLang loop) — which is the actual question a migration decision needs
answered, not an artifact of a rigged benchmark.

## Result: `bytes_scan`, tier A

Measured on this machine (Apple Silicon, `aarch64-apple-darwin`) after
`cargo build --release -p vibe_cli`, two independent runs at
`iterations=30000`:

```
$ ./benchmarks/stdlib/run.sh bytes_scan A 30000
case=bytes_scan tier=A iterations=30000
  C       : 0.81s  peak 1776KB
  VibeLang: 3.79s  peak 1696KB
  ratio   : 4.679x (lower is better)
FAIL: tier A requires the VibeLang implementation within 5% of C

$ ./benchmarks/stdlib/run.sh bytes_scan A 30000
case=bytes_scan tier=A iterations=30000
  C       : 0.76s  peak 1776KB
  VibeLang: 3.77s  peak 1696KB
  ratio   : 4.961x (lower is better)
FAIL: tier A requires the VibeLang implementation within 5% of C
```

**The gate fails, reproducibly, at roughly 4.7x-5.0x — not within a few
percent of the 5% budget, off by a factor of ~5.** The verdict (FAIL, exit
1) was reproduced at every iteration count tried during development (500;
10,000; 20,000; 30,000 x2; 40,000; 50,000 x2), and the measured ratio
*increases* with iteration count (3.46x at 10,000 up to ~5.7x at 50,000)
because a fixed ~0.2-0.3s process-startup-and-setup cost is shared by both
sides and dilutes more at low repeat counts. A rough linear fit across
those runs puts the steady-state per-repetition ratio (with the shared
setup cost fully removed) closer to **6-7x**, meaning the 30,000-iteration
numbers above, if anything, understate the gap. In no case measured did the
ratio move toward 1.0x as iterations grew.

Peak RSS: VibeLang (1696-1696KB) was slightly *lower* than C (1776KB)
across every run — the C side's one-time `Str` conversion and the
`text.index_of` call don't allocate per repetition either, so this case
doesn't exercise the "matches on time but leaks on memory" failure mode the
plan is worried about. That failure mode is expected to show up in tier C
functions (`concat`, `slice`, `to_hex`), not here.

**Root cause, isolated:** a control case with the identical loop shape but
no native call at all (just `i == target` on a local `Int`, no `bytes.get`)
ran at ~1.05ns per iteration, already about 2.4x slower than
`text.index_of`'s ~0.44ns/byte. Adding the `bytes.get` native call back in
roughly doubles that to ~2.0ns/iteration. So the gap is not purely FFI
overhead: roughly half of it is VibeLang's scalar loop codegen versus
whatever vectorized routine `strstr`/`memchr` resolves to in the platform
libc for a single-byte needle, and the other half is the cost of crossing
into native code once per element instead of once per call. Closing the
first half is a compiler question (auto-vectorization or scan intrinsics),
not something a migration can fix by writing a better loop.

**This is not being reported as a pass.** Per the standing instruction: a
byte-scan loop is the most favorable case this migration will ever have —
no allocation, one comparison per byte, no branching complexity — and it
still misses the tier A budget by roughly 5x. That means either (a) tier A
migrations as currently scoped (VibeLang loop calling a per-element native
accessor) cannot meet a 5% budget against C's vectorized library routines
with the compiler as it stands today, or (b) the 5% number itself needs
revisiting. Both are decisions for the plan owner, not something this
harness should quietly work around by widening the budget or picking an
easier case. See `task-7-report.md` for the full writeup.

## Proving the gate fails when it should

A gate that has never failed is not known to work. During development, a
throwaway `slow_probe` case (VibeLang side: the same trivial sum as the C
side, plus a deliberate loop that redoes 50x more work) was built, run, and
confirmed to fail with exit 1 and the `FAIL: tier A requires ...` message,
at two different iteration counts (200000: ratio 15.6x; 20000: ratio
3.32x). The probe case was deleted afterward — the C-path and VibeLang-path
files are not part of the committed benchmark set. See `task-7-report.md`
for the exact commands and output.

## Adding a new case

1. Write `<name>_c.yb` and `<name>_vibe.yb` under `cases/`. Both must read
   the repeat count from `cli.arg(0)` (via `convert.to_int`) and loop that
   many times, printing a result at the end so the loop can't be trivially
   optimized away.
2. Decide the tier: A if the operation only scans/compares without
   allocating anywhere in the timed path (on *either* side, not just the
   VibeLang one); otherwise C or D per the migration plan.
3. Run `benchmarks/stdlib/run.sh <name> <tier> [iterations]`, tuning
   `iterations` so the total run is long enough (at least a few hundred ms
   per side) that process-startup and one-time setup cost don't dominate
   the measurement.
