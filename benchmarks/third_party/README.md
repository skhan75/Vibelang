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

## Canonical report root

- `reports/benchmarks/third_party/`
