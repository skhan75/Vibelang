# Third-Party Benchmark Rollback Protocol

This protocol is used when strict benchmark budgets fail and the candidate cannot
be fixed in the current release window.

## Trigger conditions

- Strict validation fails in CI with budget violations.
- Regression is reproducible on at least one rerun.
- Fix ETA is incompatible with release deadline.

## Rollback steps

1. Pin benchmark evidence to last known good results:
   - `reports/benchmarks/third_party/analysis/baseline_pointer.json`
2. Re-run validator in strict mode against rollback candidate.
3. Publish a delta report documenting rollback rationale.
4. Open a follow-up optimization issue with owner + SLA.

## Command reference

```bash
python tooling/metrics/compare_third_party_benchmarks.py \
  --baseline-results reports/benchmarks/third_party/history/<last_good>.json \
  --candidate-results reports/benchmarks/third_party/full/results.json
```

## Evidence required

- rollback decision record
- delta report URL/path
- follow-up remediation plan and owner assignment
