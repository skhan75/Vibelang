#!/usr/bin/env bash
# Differential benchmark for one stdlib function during the C-to-VibeLang
# migration. Reports wall time and peak RSS for both implementations of the
# same case, and enforces the per-tier gate agreed on 2026-08-16:
#   tier A (scan/compare, no allocation) must be within 5% of C or fail
#   any other tier records its number and never blocks
set -euo pipefail

case_name="${1:?usage: run.sh <case> <tier A|C|D> [iterations]}"
tier="${2:?usage: run.sh <case> <tier A|C|D> [iterations]}"
iterations="${3:-200000}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
vibe="$root/target/release/vibe"
[ -x "$vibe" ] || { echo "build the release compiler first: cargo build --release -p vibe_cli" >&2; exit 2; }

measure() {                     # $1 = binary, echoes "seconds peak_rss_kb"
  local out
  out="$( { /usr/bin/time -l "$1" "$iterations" >/dev/null; } 2>&1 )"
  local secs peak
  secs="$(awk '/real/ {print $1}' <<<"$out")"
  peak="$(awk '/maximum resident set size/ {print int($1/1024)}' <<<"$out")"
  echo "$secs $peak"
}

build() {                       # $1 = source, echoes the built binary path
  local dir; dir="$(mktemp -d)"
  cp "$1" "$dir/case.yb"
  # `vibe build` takes its source path as a positional argument BEFORE any
  # flags, not after: `vibe build --profile release case.yb` fails with
  # "unknown argument `release`" because the parser reads args[0]
  # unconditionally as the source path. See task-7-report.md.
  ( cd "$dir" && "$vibe" build case.yb --profile release >/dev/null )
  # Find the artifact rather than composing its path: the directory is the Rust
  # target triple (aarch64-apple-darwin) while `uname -m` says arm64, so
  # composing it by hand produces a path that does not exist.
  find "$dir/.yb/artifacts" -type f -name case -perm -u+x | head -1
}

c_bin="$(build "$root/benchmarks/stdlib/cases/${case_name}_c.yb")"
vb_bin="$(build "$root/benchmarks/stdlib/cases/${case_name}_vibe.yb")"

read -r c_secs c_rss <<<"$(measure "$c_bin")"
read -r v_secs v_rss <<<"$(measure "$vb_bin")"

ratio="$(awk -v a="$v_secs" -v b="$c_secs" 'BEGIN { printf "%.3f", a/b }')"
printf 'case=%s tier=%s iterations=%s\n' "$case_name" "$tier" "$iterations"
printf '  C       : %ss  peak %sKB\n' "$c_secs" "$c_rss"
printf '  VibeLang: %ss  peak %sKB\n' "$v_secs" "$v_rss"
printf '  ratio   : %sx (lower is better)\n' "$ratio"

if [ "$tier" = "A" ]; then
  awk -v r="$ratio" 'BEGIN { exit (r > 1.05) ? 1 : 0 }' || {
    echo "FAIL: tier A requires the VibeLang implementation within 5% of C" >&2
    exit 1
  }
  echo "PASS: within the tier A budget"
else
  echo "recorded, not gated: tier $tier does not block until its blocker ships"
fi
