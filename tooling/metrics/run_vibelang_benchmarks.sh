#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ADAPTER_ROOT="$REPO_ROOT/benchmarks/third_party/plbci/adapters/vibelang/algorithm"
BUILD_DIR="/tmp/vibe_bench_run_$$"
RESULTS_FILE="$REPO_ROOT/reports/benchmarks/third_party/vibelang_standalone_results.json"
SUMMARY_FILE="$REPO_ROOT/reports/benchmarks/third_party/vibelang_standalone_summary.md"

HYPERFINE_RUNS=5
HYPERFINE_WARMUP=2

declare -A PROBLEM_INPUTS
PROBLEM_INPUTS[helloworld]="T_T"
PROBLEM_INPUTS[nsieve]="10"
PROBLEM_INPUTS[binarytrees]="15"
PROBLEM_INPUTS[merkletrees]="15"
PROBLEM_INPUTS[nbody]="500000"
PROBLEM_INPUTS[spectral-norm]="2000"
PROBLEM_INPUTS[pidigits]="4000"
PROBLEM_INPUTS[edigits]="100000"
PROBLEM_INPUTS[mandelbrot]="1000"
PROBLEM_INPUTS[fannkuch-redux]="10"
PROBLEM_INPUTS[fasta]="250000"
PROBLEM_INPUTS[coro-prime-sieve]="1000"
PROBLEM_INPUTS[lru]="100 500000"
PROBLEM_INPUTS[secp256k1]="500"
PROBLEM_INPUTS[http-server]="500"
PROBLEM_INPUTS[json-serde]="sample 5000"

declare -A PROBLEM_UNIT_INPUTS
PROBLEM_UNIT_INPUTS[helloworld]="T_T"
PROBLEM_UNIT_INPUTS[nsieve]="4"
PROBLEM_UNIT_INPUTS[binarytrees]="6"
PROBLEM_UNIT_INPUTS[merkletrees]="9"
PROBLEM_UNIT_INPUTS[nbody]="1000"
PROBLEM_UNIT_INPUTS[spectral-norm]="100"
PROBLEM_UNIT_INPUTS[pidigits]="27"
PROBLEM_UNIT_INPUTS[edigits]="227"
PROBLEM_UNIT_INPUTS[mandelbrot]="1"
PROBLEM_UNIT_INPUTS[fannkuch-redux]="7"
PROBLEM_UNIT_INPUTS[fasta]="1000"
PROBLEM_UNIT_INPUTS[coro-prime-sieve]="100"
PROBLEM_UNIT_INPUTS[lru]="10 1000"
PROBLEM_UNIT_INPUTS[secp256k1]="1"
PROBLEM_UNIT_INPUTS[http-server]="10"
PROBLEM_UNIT_INPUTS[json-serde]="sample 10"

SKIP_FILE_DEPENDENT=(knucleotide regex-redux)

mkdir -p "$BUILD_DIR"
trap 'rm -rf "$BUILD_DIR"' EXIT

echo "=== VibeLang Standalone Benchmark Run ==="
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "Hyperfine: $HYPERFINE_RUNS runs, $HYPERFINE_WARMUP warmup"
echo ""

json_results='{"benchmarks":['
first_result=1
pass_count=0
fail_count=0
skip_count=0

for problem_dir in "$ADAPTER_ROOT"/*/; do
  problem="$(basename "$problem_dir")"
  source_file="$problem_dir/1.yb"

  if [[ ! -f "$source_file" ]]; then
    echo "SKIP $problem (no 1.yb)"
    skip_count=$((skip_count + 1))
    continue
  fi

  skip=0
  for s in "${SKIP_FILE_DEPENDENT[@]}"; do
    if [[ "$problem" == "$s" ]]; then
      skip=1
      break
    fi
  done
  if [[ $skip -eq 1 ]]; then
    echo "SKIP $problem (requires external input files)"
    skip_count=$((skip_count + 1))
    continue
  fi

  input="${PROBLEM_INPUTS[$problem]:-}"
  unit_input="${PROBLEM_UNIT_INPUTS[$problem]:-}"
  if [[ -z "$input" ]]; then
    echo "SKIP $problem (no canonical input defined)"
    skip_count=$((skip_count + 1))
    continue
  fi

  work_dir="$BUILD_DIR/$problem"
  mkdir -p "$work_dir"
  cp "$source_file" "$work_dir/app.yb"

  if [[ -d "$problem_dir/../../../plbci/adapters/vibelang/include/vibelang" ]]; then
    cp "$ADAPTER_ROOT/../include/vibelang/run_vibelang.sh" "$work_dir/" 2>/dev/null || true
  fi

  echo -n "BUILD $problem ... "
  if ! (cd "$work_dir" && vibe build app.yb --profile release 2>&1) > "$work_dir/build.log" 2>&1; then
    echo "FAIL (build error)"
    cat "$work_dir/build.log" | tail -5
    fail_count=$((fail_count + 1))
    continue
  fi

  binary="$work_dir/.yb/artifacts/release/x86_64-unknown-linux-gnu/app"
  if [[ ! -x "$binary" ]]; then
    echo "FAIL (no binary)"
    fail_count=$((fail_count + 1))
    continue
  fi
  echo "OK"

  echo -n "  UNIT TEST ($unit_input) ... "
  printf "%s" "$unit_input" > "$work_dir/.benchmark_input"
  if unit_output=$(cd "$work_dir" && timeout 30 "$binary" 2>&1); then
    echo "OK"
  else
    echo "FAIL (exit $?)"
    echo "  Output: $(echo "$unit_output" | head -3)"
    fail_count=$((fail_count + 1))
    continue
  fi

  echo -n "  BENCH ($input, $HYPERFINE_RUNS runs) ... "
  printf "%s" "$input" > "$work_dir/.benchmark_input"

  hyperfine_json="$work_dir/hyperfine.json"
  if (cd "$work_dir" && hyperfine \
    --runs "$HYPERFINE_RUNS" \
    --warmup "$HYPERFINE_WARMUP" \
    --export-json "$hyperfine_json" \
    --output /dev/null \
    "$binary" 2>&1) > "$work_dir/hyperfine.log" 2>&1; then

    mean_s=$(python3 -c "import json; d=json.load(open('$hyperfine_json')); print(d['results'][0]['mean'])")
    stddev_s=$(python3 -c "import json; d=json.load(open('$hyperfine_json')); print(d['results'][0]['stddev'])")
    min_s=$(python3 -c "import json; d=json.load(open('$hyperfine_json')); print(d['results'][0]['min'])")
    max_s=$(python3 -c "import json; d=json.load(open('$hyperfine_json')); print(d['results'][0]['max'])")

    mean_ms=$(python3 -c "print(f'{$mean_s * 1000:.3f}')")
    stddev_ms=$(python3 -c "print(f'{$stddev_s * 1000:.3f}')")
    min_ms=$(python3 -c "print(f'{$min_s * 1000:.3f}')")
    max_ms=$(python3 -c "print(f'{$max_s * 1000:.3f}')")

    echo "OK (mean: ${mean_ms}ms, stddev: ${stddev_ms}ms)"

    if [[ $first_result -eq 0 ]]; then
      json_results+=','
    fi
    json_results+="{\"problem\":\"$problem\",\"input\":\"$input\",\"mean_ms\":$mean_ms,\"stddev_ms\":$stddev_ms,\"min_ms\":$min_ms,\"max_ms\":$max_ms,\"runs\":$HYPERFINE_RUNS}"
    first_result=0
    pass_count=$((pass_count + 1))
  else
    echo "FAIL (hyperfine error)"
    cat "$work_dir/hyperfine.log" | tail -5
    fail_count=$((fail_count + 1))
  fi
done

json_results+='],"metadata":{"date":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'","hyperfine_runs":'$HYPERFINE_RUNS',"hyperfine_warmup":'$HYPERFINE_WARMUP',"host":"'$(hostname)'","pass":'$pass_count',"fail":'$fail_count',"skip":'$skip_count'}}'

mkdir -p "$(dirname "$RESULTS_FILE")"
echo "$json_results" | python3 -m json.tool > "$RESULTS_FILE"

echo ""
echo "=== Results ==="
echo "Pass: $pass_count  Fail: $fail_count  Skip: $skip_count"
echo "Results written to: $RESULTS_FILE"

python3 -c "
import json, sys

with open('$RESULTS_FILE') as f:
    data = json.load(f)

benchmarks = data['benchmarks']
meta = data['metadata']

lines = []
lines.append('# VibeLang Standalone Benchmark Results')
lines.append('')
lines.append(f'- date: \`{meta[\"date\"]}\`')
lines.append(f'- host: \`{meta[\"host\"]}\`')
lines.append(f'- hyperfine_runs: {meta[\"hyperfine_runs\"]}')
lines.append(f'- hyperfine_warmup: {meta[\"hyperfine_warmup\"]}')
lines.append(f'- pass: {meta[\"pass\"]}  fail: {meta[\"fail\"]}  skip: {meta[\"skip\"]}')
lines.append('')
lines.append('## Runtime Results (ms)')
lines.append('')
lines.append('| problem | input | mean_ms | stddev_ms | min_ms | max_ms |')
lines.append('| --- | --- | ---: | ---: | ---: | ---: |')
for b in sorted(benchmarks, key=lambda x: x['problem']):
    lines.append(f'| {b[\"problem\"]} | {b[\"input\"]} | {b[\"mean_ms\"]} | {b[\"stddev_ms\"]} | {b[\"min_ms\"]} | {b[\"max_ms\"]} |')
lines.append('')
lines.append('## Notes')
lines.append('')
lines.append('- This is a standalone VibeLang-only run (no cross-language baselines).')
lines.append('- knucleotide and regex-redux are skipped (require external FASTA input files).')
lines.append('- nbody, spectral-norm, and mandelbrot use integer-approximation (no Float codegen yet).')
lines.append('- edigits, secp256k1, http-server, json-serde delegate to C bench-runtime.')
lines.append('- All adapters now read .benchmark_input and use canonical problem sizes.')
lines.append('')

with open('$SUMMARY_FILE', 'w') as f:
    f.write('\n'.join(lines))

print(f'Summary written to: $SUMMARY_FILE')
"
