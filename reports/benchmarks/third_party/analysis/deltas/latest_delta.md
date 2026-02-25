# Third-Party Benchmark Delta Report

- generated_at_utc: `2026-02-25T18:04:12Z`
- baseline_results: `reports/benchmarks/third_party/history/20260225_091020Z_full_results.json`
- candidate_results: `reports/benchmarks/third_party/latest/results.json`

## Runtime Geomean Delta (VibeLang/Baseline)

| baseline | before | after | delta_abs | delta_pct |
| --- | ---: | ---: | ---: | ---: |
| c | 0.794 | 0.090 | -0.704 | -88.71% |
| cpp | 0.683 | 0.100 | -0.583 | -85.36% |
| elixir | 0.003 | 0.003 | -0.000 | -0.79% |
| go | 0.426 | 0.047 | -0.378 | -88.91% |
| kotlin | 0.000 | 2.696 | 2.696 | 0.00% |
| python | 0.022 | 0.006 | -0.017 | -74.95% |

Interpretation: negative delta is improvement (ratio got smaller).

## Compile Cold Delta (VibeLang/Baseline)

| baseline | before | after | delta_abs | delta_pct |
| --- | ---: | ---: | ---: | ---: |
| elixir | 0.313 | 0.355 | 0.042 | 13.26% |
| go | 1.006 | 1.043 | 0.037 | 3.72% |
| kotlin | 0.305 | 0.312 | 0.008 | 2.47% |

Interpretation: negative delta is improvement (ratio got smaller).

