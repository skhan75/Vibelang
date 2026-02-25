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

python3 tooling/metrics/collect_third_party_benchmarks.py --profile "${PROFILE}"
python3 tooling/metrics/validate_third_party_benchmarks.py --enforcement-mode "${VALIDATION_MODE}"

if [[ -f "reports/benchmarks/third_party/full/results.json" ]]; then
  python3 tooling/metrics/compare_third_party_benchmarks.py \
    --baseline-results "reports/benchmarks/third_party/full/results.json" \
    --candidate-results "reports/benchmarks/third_party/latest/results.json"
fi

echo "Third-party benchmark runner completed for profile=${PROFILE}"
