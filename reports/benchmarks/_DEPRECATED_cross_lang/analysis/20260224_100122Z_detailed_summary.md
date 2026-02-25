# VibeLang Cross-Language Optimization Detailed Summary

- generated_at_utc: `2026-02-24T10:01:22Z`
- baseline_reference_utc: `2026-02-24T03:31:23Z`
- current_full_summary: `reports/benchmarks/cross_lang/full/summary.md`
- current_quick_summary: `reports/benchmarks/cross_lang/quick/summary.md`
- latest_delta: `reports/benchmarks/cross_lang/analysis/deltas/latest_delta.md`

## Executive Outcome

The optimization program has been implemented across Phases 0-6 with measurable benchmark gains.

### Geomean Ratios (Baseline -> Current Full)

| baseline | baseline_ratio | current_ratio | delta_abs | delta_pct |
| --- | ---: | ---: | ---: | ---: |
| c | 2.623 | 1.831 | -0.792 | -30.19% |
| rust | 1.909 | 1.370 | -0.539 | -28.23% |
| go | 2.752 | 1.826 | -0.926 | -33.65% |
| python | 0.154 | 0.118 | -0.036 | -23.38% |
| typescript | 0.174 | 0.138 | -0.036 | -20.69% |

## Case-Level Impact Highlights

### Major Wins

- `hashmap_frequency` vs C: `146.571x` -> `11.323x` (`-92.28%`).
- `hashmap_frequency_int_key` was added and now tracks dedicated int-key map path (`3.467x` vs C).
- `string_concat_checksum` vs C: `3.828x` -> `2.611x` (`-31.79%`).
- `json_roundtrip` vs C: `2.265x` -> `1.871x` (`-17.39%`).

### Neutral / Remaining Hotspots

- `channel_pingpong` vs Go: `182.018x` -> `181.697x` (effectively flat in full profile; still top remaining gap).
- `worker_pool_reduce` remains noisy in quick profile and is covered by rerun warning policy.

## Implemented Phase Evidence

- Phase 0: metadata hardening, p99/MAD/RSD, trend artifacts, fairness notes, reproducibility runbook.
- Phase 1: hash-backed map runtime backend with collision/resize/probe counters.
- Phase 2: map key-path coverage expanded with explicit int-key/str-key benchmark variants.
- Phase 3: channel fast-path counters and contention/wait instrumentation.
- Phase 4: parse/stringify/minify/validate conversion-path optimizations with counters.
- Phase 5: compiler phase timing report emission and runtime object compile caching.
- Phase 6: budget gating, rerun policy warnings, CI integration, triage + rollback templates.

## Current Guardrails

- budget file: `reports/benchmarks/cross_lang/analysis/performance_budgets.json`
- triage template: `reports/benchmarks/cross_lang/analysis/regression_triage_template.md`
- rollback protocol: `reports/benchmarks/cross_lang/analysis/rollback_protocol.md`

## Residual Risk Notes

- Channel performance relative to Go remains the highest-priority unresolved hotspot.
- Quick-profile variance for some short cases can trigger rerun recommendations; this is expected and now policy-managed.
- Compile throughput is now measurable per phase; future work should ratchet budget targets after several trend windows.
