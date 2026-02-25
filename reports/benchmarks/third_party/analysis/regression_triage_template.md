# Third-Party Benchmark Regression Triage Template

Use this template for triaging regressions detected by
`tooling/metrics/validate_third_party_benchmarks.py`.

## Incident metadata

- date_utc:
- owner:
- candidate_results:
- baseline_results:
- workflow_run_url:

## Detection summary

- budget_mode: `warn|strict`
- violated_rules:
  - runtime ratio:
  - compile ratio:
  - required language/problem availability:

## Impact

- user-facing impact:
- release impact:
- affected benchmark classes:
  - runtime:
  - memory:
  - concurrency:
  - compile:

## Investigation checklist

- [ ] Confirm result reproducibility by rerunning collector.
- [ ] Verify toolchain version deltas (`dotnet`, `docker`, `vibe`, `hyperfine`).
- [ ] Check if a dependency image/runtime changed unexpectedly.
- [ ] Compare per-problem ratios to isolate where regression starts.
- [ ] Document if fairness caveat explains the delta.

## Resolution plan

- immediate mitigation:
- code/config changes:
- owner + ETA:

## Exit criteria

- [ ] Candidate run no longer violates strict budgets.
- [ ] Delta report shows non-regressing trend for impacted baselines.
- [ ] Rollback required? If yes, link rollback incident.
