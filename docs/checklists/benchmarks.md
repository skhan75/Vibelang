# Benchmarks Checklist (Canonical)

Last updated: 2026-03-01

This is the single canonical checklist for:
- third-party benchmarking (PLB-CI + hyperfine)
- benchmark publication readiness (“apples-to-apples”)
- performance regression hygiene (budgets, deltas, provenance)

If you find any benchmark “checklist” living outside `docs/checklists/`, consolidate it here and replace the other file with a short pointer.

## 1) What we measure (high-level)

- **Runtime**: PLB-CI runtime/memory/concurrency lanes (Docker-first).
- **Compile loop**: hyperfine-based cold + incremental build timings (via PLB-CI build commands).

Sources of truth:
- Publication policy: `benchmarks/third_party/APPLE_TO_APPLE_BENCHMARK_POLICY.md`
- Reproducibility recipe: `benchmarks/third_party/CLOUD_REPRODUCIBILITY.md`
- Runbook: `reports/benchmarks/third_party/analysis/reproducibility_runbook.md`
- Latest full report set: `reports/benchmarks/third_party/full/`

## 2) Strict publication readiness (apples-to-apples)

Public performance claims are allowed only when **all** of these are true:

- [x] **Pinned upstream**: PLB-CI suite is pinned to an immutable commit SHA in the matrix.
- [x] **Same semantics**: VibeLang adapters are canonical (no proxies/stubs/canned outputs).
- [x] **Docker-backed**: run is Docker-enabled (no `--no-docker`) and preflight is `ok`.
- [ ] **Provenance recorded**: report captures:
  - repo revision (git SHA)
  - VibeLang toolchain revision (commit/build metadata)
  - host + toolchain versions
- [ ] **Lane completeness**: all required lanes are `status=ok` for strict publication profile (no “missing subset presented as full conclusion”).
- [ ] **Validation passes in publication mode**:
  - `tooling/metrics/validate_third_party_benchmarks.py --publication-mode`
  - adapter parity validation passes in publication mode
- [ ] **Delta is meaningful**: baseline vs candidate are two distinct strict publication runs (no self-diff).
- [ ] **Repro rerun envelope**: a rerun on the same host class reproduces the same conclusion envelope (directionally consistent).

Current evidence snapshot:
- Latest strict-mode report set: `reports/benchmarks/third_party/full/` (`timestamp_id=20260301_072822Z`)
- Known blockers observed in that report:
  - provenance gaps (`repo_git_revision=unavailable`, `vibe commit=unknown`)
  - missing/unavailable lanes (ex: Swift runtime/compile show `n/a` in summary; Kotlin/Elixir compile `n/a`)
  - delta artifacts currently self-diff (baseline and candidate identical)

## 3) “Strict run” execution checklist

- [x] Preflight is clean:
  - `python3 tooling/metrics/collect_third_party_benchmarks.py --profile full --preflight-only`
- [x] Collect strict evidence:
  - `python3 tooling/metrics/collect_third_party_benchmarks.py --profile full --publication-mode`
- [ ] Validate strict evidence:
  - `python3 tooling/metrics/validate_third_party_benchmarks.py --publication-mode`
- [ ] Produce delta vs last strict baseline:
  - `python3 tooling/metrics/compare_third_party_benchmarks.py --publication-mode ...`
- [x] Archive results under:
  - `reports/benchmarks/third_party/history/`

## 4) Regression hygiene (internal + CI)

- [x] Performance budgets are maintained and intentional:
  - `reports/benchmarks/third_party/analysis/performance_budgets.json`
- [ ] Any “allowlisted unavailable lanes” are treated as temporary and tracked with owners and an exit criterion.
- [ ] If budgets fail, follow rollback protocol:
  - `reports/benchmarks/third_party/analysis/rollback_protocol.md`

### Regression triage checklist (when budgets fail)

- [ ] Confirm result reproducibility by rerunning collector.
- [ ] Verify toolchain version deltas (`dotnet`, `docker`, `vibe`, `hyperfine`).
- [ ] Check if a dependency image/runtime changed unexpectedly.
- [ ] Compare per-problem ratios to isolate where regression starts.
- [ ] Document if a fairness caveat explains the delta (and whether the claim should be held).

Exit criteria:
- [ ] Candidate run no longer violates strict budgets.
- [ ] Delta report shows non-regressing trend for impacted baselines.
- [ ] If rollback is required, link the rollback incident and the baseline pointer update.

## 5) What to say publicly (guidance)

- Prefer publishing **methodology + raw evidence links** and avoid single-number marketing.
- If you include ratios, always state:
  - the exact report path (`results.json` + `summary.md`)
  - the host + toolchain versions
  - that geomeans are computed over the **shared-problem overlap** for each baseline
  - which lanes were unavailable (if any) and why

## 6) Work to do (fill gaps before “public-ready”)

- [ ] **Fix provenance in `results.json`**:
  - ensure `environment.source_revisions.repo_git_revision` is a real git SHA (not `unavailable`)
  - ensure `vibe --version` includes a real commit/build id (not `commit=unknown`)
- [ ] **Get compile lanes healthy** for allowlisted baselines (Swift/Kotlin/Elixir) or remove them from any public-facing ratio tables until they’re healthy and comparable.
- [ ] **Make deltas meaningful**:
  - choose and pin a strict baseline under `reports/benchmarks/third_party/history/`
  - regenerate `latest_delta.*` against a distinct baseline vs candidate
- [ ] **Document overlap semantics clearly** (shared-problem overlap is per-baseline and may differ across languages) in the public report writeup.

