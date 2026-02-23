# VibeLang Debugging and Profiling Workflow

Date: 2026-02-22

## Goal

Provide a deterministic workflow for symbols, stack context, and performance
diagnostics in Vibe programs.

## Build With Rich Debug Metadata

```bash
vibe build app/main.yb --debuginfo full
```

Build output includes:

- binary path
- object/runtime object paths
- debug map (`*.debug.map`)
- unsafe audit (`*.unsafe.audit.json`)
- allocation profile (`*.alloc.profile.json`)

## Runtime Failure Triage Path

1. Reproduce with deterministic inputs (`--locked` where applicable).
2. Capture stderr/stdout and build artifact paths.
3. Inspect debug map and allocation profile to correlate failing functions and
   allocation-heavy paths.
4. Attach crash repro artifact package (see `docs/support/crash_repro_format.md`).

## Stack Context

Current v1 workflow uses:

- deterministic debug map function signatures and source references,
- runtime error diagnostics (`Result`/contract/panic channels),
- optional native debugger attachment (`gdb`/`lldb`) when host tooling is
  available.

## Performance Diagnostics

Use release benchmark and metrics tooling:

```bash
python3 tooling/metrics/collect_phase6_metrics.py
python3 tooling/metrics/collect_release_benchmarks.py
```

Performance artifacts:

- `reports/phase6/metrics/phase6_metrics.json`
- `reports/v1/release_benchmarks.json`
- `reports/v1/allocation_visibility_smoke.json`

## Determinism Expectations

- Identical source + toolchain should produce stable debug artifacts.
- Diagnostic ordering and message templates remain deterministic.
