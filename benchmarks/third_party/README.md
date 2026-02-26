# Third-Party Benchmark Suite (Canonical)

This directory defines the canonical benchmark stack for VibeLang performance
tracking.

## Standard tools

- Runtime/memory/concurrency: **Programming Language Benchmarks CI (PLB-CI)**
- Compile-loop timing: **hyperfine**

## Goals

- Compare VibeLang against common language baselines on known workloads.
- Publish timestamped machine-readable benchmark evidence.
- Generate developer-friendly summaries with wins, gaps, and recommendations.

## Layout

- `plbci/config/` - PLB-CI run configuration and language matrix
- `plbci/adapters/vibelang/` - VibeLang benchmark adapters used by PLB-CI
- `docker/` - Docker runner image + execution wrapper scripts
- `CLOUD_REPRODUCIBILITY.md` - Dedicated VM + Docker reproducibility recipe
- `APPLE_TO_APPLE_BENCHMARK_POLICY.md` - strict publication standard and blocker checklist
- `plbci/adapters/vibelang/PARITY_MANIFEST.yaml` - canonical/proxy parity ledger per problem

## Canonical report root

- `reports/benchmarks/third_party/`

## Publication note

Do not publish performance claims externally unless
`APPLE_TO_APPLE_BENCHMARK_POLICY.md` publication checklist is fully satisfied.
Current strict publication attempt report:
`reports/benchmarks/third_party/analysis/apples_to_apples_public_report.md`.
