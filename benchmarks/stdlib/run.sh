#!/usr/bin/env bash
# Differential benchmark for one stdlib function during the C-to-VibeLang
# migration. Reports wall time and peak RSS for both implementations of the
# same case, checks that both sides computed the SAME answer, and enforces
# the per-tier gate agreed on 2026-08-16:
#   tier A (scan/compare, no allocation) must be within 5% of C on wall
#   time AND within RSS_GATE_MULT of C on peak RSS, or it does not merge.
#   any other tier records its numbers on both axes and never blocks.
# The output check runs for every tier, gated or not: a fast wrong answer
# is not a passing migration, and this harness's only job is guarding both.
set -euo pipefail

case_name="${1:?usage: run.sh <case> <tier A|C|D> [iterations]}"
tier="${2:?usage: run.sh <case> <tier A|C|D> [iterations]}"
iterations="${3:-200000}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
vibe="$root/target/release/vibe"
[ -x "$vibe" ] || { echo "build the release compiler first: cargo build --release -p vibe_cli" >&2; exit 2; }

# Minimum-of-N: a single run's wall time is not precise enough to judge a 5%
# budget. Proven with a byte-identical probe pair (true ratio exactly
# 1.000x) run six times through the single-shot version of this harness:
# 1.299x, 1.078x, 1.037x, 0.650x, 0.987x, 1.092x -- verdict flipped 3 PASS /
# 3 FAIL against the same 5% budget. Repeating each binary and keeping the
# minimum (noise only ever adds time, never removes real work) is the
# standard fix; see README.md for what tolerance this leaves.
MEASURE_RUNS="${MEASURE_RUNS:-5}"

# Tier A peak-RSS budget. Tier A means no allocation in the timed path, so a
# correct candidate's peak RSS should track C's, modulo the fixed footprint
# difference between two statically-linked binaries that both embed the same
# vibe_runtime.o. 1.5x is generous enough to absorb that fixed-footprint
# noise (measured tier A cases so far differ by single-digit percent, never
# close to 1.5x) but tight enough to catch a real allocate-in-the-loop
# defect: with no collector, a loop that allocates blows past 1.5x within a
# handful of repetitions, not asymptotically -- it does not need a large
# multiple to catch it.
RSS_GATE_MULT="${RSS_GATE_MULT:-1.5}"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

# is_number / is_positive_number: guard every parsed timing/RSS value before
# it is used in arithmetic. Without this, an empty or non-numeric field (a
# changed `/usr/bin/time` output format, a killed process) either computes a
# bogus 0.000 ratio that passes the gate, or crashes awk with
# division-by-zero if the "C" side itself rounds to 0.00s at a low
# iteration count -- both silent or confusing failure modes that are worse
# than refusing to score at all. Kept as two checks, not one, because
# "empty/malformed" and "a real, valid zero" need different error messages:
# the first means the parser broke, the second means the run finished too
# fast to measure and needs more iterations, and conflating them produced a
# genuinely misleading "could not parse" message for a value that parsed
# fine.
is_number() {
  [ -n "$1" ] && awk -v v="$1" 'BEGIN { exit !(v ~ /^[0-9]+(\.[0-9]+)?$/) }'
}
is_positive_number() {
  is_number "$1" && awk -v v="$1" 'BEGIN { exit !(v + 0 > 0) }'
}

measure() {                     # $1=binary $2=out-var-prefix
  local bin="$1" prefix="$2"
  local best_secs="" best_rss="" best_out=""

  # Warmup, discarded entirely: macOS validates a freshly written binary's
  # code signature on its first-ever exec. Measured on one binary, same
  # workload, three consecutive runs with no warmup: 0.34s, then 0.00s, then
  # 0.00s. Because build() always uses a fresh mktemp -d, every binary this
  # harness times is on its first exec -- without discarding one exec first,
  # BOTH sides carry that ~0.34s of pure OS overhead every single time,
  # which is additive and drags the ratio toward 1.0 (the gate becomes
  # systematically too lenient), not toward the true per-iteration cost.
  "$bin" "$iterations" >/dev/null 2>&1 || true

  local run_i
  for run_i in $(seq 1 "$MEASURE_RUNS"); do
    local out_file="$tmp_dir/${prefix}.${run_i}.stdout"
    local raw secs rss
    raw="$( { /usr/bin/time -l "$bin" "$iterations" >"$out_file"; } 2>&1 )"
    secs="$(awk '/real/ {print $1}' <<<"$raw")"
    rss="$(awk '/maximum resident set size/ {print int($1/1024)}' <<<"$raw")"
    if ! is_number "$secs" || ! is_number "$rss"; then
      echo "FAIL: could not parse timing/RSS out of /usr/bin/time for $bin (run $run_i)" >&2
      echo "  parsed secs='${secs:-<empty>}' rss='${rss:-<empty>}'" >&2
      echo "  raw /usr/bin/time output:" >&2
      echo "$raw" | sed 's/^/    /' >&2
      exit 2
    fi
    if ! is_positive_number "$secs"; then
      echo "FAIL: $bin ran in 0.00s (run $run_i) -- too fast to measure reliably" >&2
      echo "  rerun with more iterations (current: $iterations)" >&2
      exit 2
    fi
    if [ -z "$best_secs" ] || awk -v a="$secs" -v b="$best_secs" 'BEGIN { exit !(a + 0 < b + 0) }'; then
      best_secs="$secs"
      best_rss="$rss"
      best_out="$(cat "$out_file")"
    fi
  done

  printf -v "${prefix}_secs" '%s' "$best_secs"
  printf -v "${prefix}_rss" '%s' "$best_rss"
  printf -v "${prefix}_out" '%s' "$best_out"
}

build() {                       # $1 = source, echoes the built binary path
  local dir; dir="$(mktemp -d)"
  cp "$1" "$dir/case.yb"
  # `vibe build` reads its source path as a positional argument BEFORE any
  # flags: `vibe build --profile release case.yb` fails with "unknown
  # argument `release`" because the parser reads args[0] unconditionally as
  # the source path.
  ( cd "$dir" && "$vibe" build case.yb --profile release >/dev/null )
  # Find the artifact rather than composing its path: the directory is the
  # Rust target triple (aarch64-apple-darwin) while `uname -m` says arm64,
  # so composing it by hand produces a path that does not exist.
  find "$dir/.yb/artifacts" -type f -name case -perm -u+x | head -1
}

c_bin="$(build "$root/benchmarks/stdlib/cases/${case_name}_c.yb")"
vb_bin="$(build "$root/benchmarks/stdlib/cases/${case_name}_vibe.yb")"

measure "$c_bin" c
measure "$vb_bin" v

# Differential check, before any timing talk, for every tier: a "fast wrong
# answer" must never pass. A _vibe.yb that scans zero bytes or computes the
# wrong count would otherwise sail through on an excellent time.
if [ "$c_out" != "$v_out" ]; then
  echo "FAIL: C and VibeLang paths printed different results" >&2
  echo "  C        printed: $c_out" >&2
  echo "  VibeLang printed: $v_out" >&2
  exit 1
fi

# Belt-and-braces: measure() already refuses to return a zero c_secs/c_rss,
# but the ratio math divides by these two values specifically, so re-check
# right before dividing rather than trusting that invariant to hold forever
# across future edits.
if ! is_positive_number "$c_secs" || ! is_positive_number "$c_rss"; then
  echo "FAIL: C-side measurement is not usable as a divisor (secs='$c_secs' rss='$c_rss')" >&2
  exit 2
fi

ratio="$(awk -v a="$v_secs" -v b="$c_secs" 'BEGIN { printf "%.3f", a / b }')"
rss_ratio="$(awk -v a="$v_rss" -v b="$c_rss" 'BEGIN { printf "%.3f", a / b }')"

printf 'case=%s tier=%s iterations=%s runs=%s(min)\n' "$case_name" "$tier" "$iterations" "$MEASURE_RUNS"
printf '  C       : %ss  peak %sKB\n' "$c_secs" "$c_rss"
printf '  VibeLang: %ss  peak %sKB\n' "$v_secs" "$v_rss"
printf '  ratio   : %sx wall time (lower is better)\n' "$ratio"
printf '  rss     : %sx peak RSS  (lower is better)\n' "$rss_ratio"
printf '  output  : matches (%s)\n' "$c_out"

if [ "$tier" = "A" ]; then
  time_ok=1
  awk -v r="$ratio" 'BEGIN { exit (r > 1.05) ? 1 : 0 }' || time_ok=0
  rss_ok=1
  awk -v r="$rss_ratio" -v m="$RSS_GATE_MULT" 'BEGIN { exit (r > m) ? 1 : 0 }' || rss_ok=0

  if [ "$time_ok" -eq 1 ] && [ "$rss_ok" -eq 1 ]; then
    echo "PASS: within the tier A wall-time and peak-RSS budgets"
  else
    [ "$time_ok" -eq 0 ] && echo "FAIL: tier A requires the VibeLang implementation within 5% of C on wall time" >&2
    [ "$rss_ok" -eq 0 ] && echo "FAIL: tier A requires the VibeLang implementation within ${RSS_GATE_MULT}x of C on peak RSS" >&2
    exit 1
  fi
else
  echo "recorded, not gated: tier $tier does not block until its blocker ships"
fi
