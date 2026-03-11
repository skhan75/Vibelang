#!/usr/bin/env bash
set -euo pipefail

PROFILE="${PROFILE:-full}"
VALIDATION_MODE="${VALIDATION_MODE:-strict}"

if ! command -v docker >/dev/null 2>&1; then
  echo "docker CLI missing inside runner container" >&2
  exit 1
fi

if ! timeout 20 docker info >/dev/null 2>&1; then
  echo "docker daemon is unavailable from runner container. Mount /var/run/docker.sock." >&2
  exit 1
fi

if [[ -d ".cache/third_party/plbci/.git" ]]; then
  git config --global --add safe.directory "$(pwd)/.cache/third_party/plbci" || true
fi

export TMPDIR="/workspace/VibeStack/vibelang/.cache/third_party_bench/plbci_tmp"
mkdir -p "$TMPDIR"
echo "TMPDIR set to $TMPDIR for nested Docker path visibility"

echo "Building bench-enabled vibe binary..."
cargo build --release -p vibe_cli --features bench-runtime
cp target/release/vibe /usr/local/bin/vibe
chmod 0755 /usr/local/bin/vibe

collect_args=(--profile "${PROFILE}")
if [[ "${VALIDATION_MODE}" == "strict" ]]; then
  collect_args+=(--publication-mode)
fi

python3 tooling/metrics/collect_third_party_benchmarks.py "${collect_args[@]}"
python3 tooling/metrics/validate_third_party_benchmarks.py --enforcement-mode "${VALIDATION_MODE}"

if [[ -f "reports/benchmarks/third_party/full/results.json" ]]; then
  python3 tooling/metrics/compare_third_party_benchmarks.py \
    --baseline-results "reports/benchmarks/third_party/full/results.json" \
    --candidate-results "reports/benchmarks/third_party/full/results.json"
fi

echo "Third-party benchmark runner completed for profile=${PROFILE}"
