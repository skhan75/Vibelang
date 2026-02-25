#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
IMAGE="${IMAGE:-vibelang-third-party-bench:latest}"
PROFILE="${PROFILE:-full}"
VALIDATION_MODE="${VALIDATION_MODE:-strict}"

docker build \
  --file "${ROOT}/vibelang/benchmarks/third_party/docker/Dockerfile" \
  --tag "${IMAGE}" \
  "${ROOT}"

docker run --rm \
  --volume /var/run/docker.sock:/var/run/docker.sock \
  --volume "${ROOT}:/workspace/VibeStack" \
  --workdir /workspace/VibeStack/vibelang \
  --env PROFILE="${PROFILE}" \
  --env VALIDATION_MODE="${VALIDATION_MODE}" \
  "${IMAGE}"
