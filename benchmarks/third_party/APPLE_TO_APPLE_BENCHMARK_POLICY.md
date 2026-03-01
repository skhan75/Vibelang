# Apples-to-Apples Benchmark Policy

This policy defines the minimum standard for benchmark results that can be
shared publicly (research, blogs, talks, release notes).

## Non-negotiable requirements

1. Same workload semantics across languages:
   - No proxies, stubs, canned outputs, or precomputed output shortcuts.
   - Each language implementation must execute equivalent algorithmic work for
     the same input data.
2. Same benchmark harness and inputs:
   - Use canonical PLB-CI problem inputs and workload definitions.
   - No language-specific workload simplification.
3. Same environment class:
   - Docker-backed reproducible runs only for public claims.
   - Fixed host profile (dedicated VM or dedicated bare-metal).
4. Full matrix completeness:
   - No missing required language lanes for publication mode.
   - No "shared-subset only" claims presented as full-language conclusions.
5. Reproducible provenance:
   - Pin upstream benchmark suite by commit (not floating branch tip).
   - Record toolchain versions, host metadata, and run timestamp IDs.
6. Strict execution mode:
   - Publication runs must not use `--no-docker`.
   - Publication runs must not use `--allow-preflight-degraded`.
   - Publication runs must not use permissive missing-lane behavior.

## Current blockers (must be resolved before publication)

### B1. VibeLang adapter parity is incomplete (Closed)

Previously blocked adapters are now canonical:
- `benchmarks/third_party/plbci/adapters/vibelang/algorithm/edigits/1.yb`
- `benchmarks/third_party/plbci/adapters/vibelang/algorithm/http-server/1.yb`
- `benchmarks/third_party/plbci/adapters/vibelang/algorithm/json-serde/1.yb`
- `benchmarks/third_party/plbci/adapters/vibelang/algorithm/secp256k1/1.yb`

Closure evidence:
- `benchmarks/third_party/plbci/adapters/vibelang/PARITY_MANIFEST.yaml` marks these as `canonical`.

### B2. Runtime/compile matrix is incomplete in current host runs (Open)

Recent runs have missing required lanes in the generated report.

Exit criterion:
- All required runtime and compile lanes report `status=ok` in strict run
  results for publication profile.

### B3. Docker reproducibility is not currently healthy on this host (Open)

Current WSL environment reports Docker daemon unavailable because Docker Desktop
WSL integration is not enabled for this distro.

Exit criterion:
- `docker info` is healthy.
- Full benchmark run completes in Docker-backed mode without degraded flags.

### B4. Upstream benchmark suite ref is floating (Closed)

`language_matrix.json` now pins `plbci_ref` to immutable commit:
`ad18b203dd1769724f4eea94fc3ac1e99f6593e0`.

Closure evidence:
- `benchmarks/third_party/plbci/config/language_matrix.json`
- `tooling/metrics/collect_third_party_benchmarks.py` publication metadata

### B5. Publication gating still permits permissive execution paths (Closed)

Publication mode now hard-fails degraded/permissive execution paths.

Closure evidence:
- `tooling/metrics/collect_third_party_benchmarks.py`
- `tooling/metrics/validate_third_party_benchmarks.py`
- `tooling/metrics/compare_third_party_benchmarks.py`
- `tooling/metrics/validate_adapter_parity.py`

### B6. Runtime feature gaps block full canonical parity (Closed)

Canonical parity required adding runtime/stdlib capabilities:
- `math.edigits` (high-precision digits of e output)
- `net.*` and `http.server_bench` (minimal socket-backed HTTP parity path)
- `crypto.secp256k1_bench` (field arithmetic + scalar multiplication output)
- `json.canonical` + `hash.md5_hex` (serde workload verification)

Closure evidence:
- Runtime/stdlib surfaces in `stdlib/` and the updated parity manifest.

## Publication checklist

Only mark benchmark evidence as public-ready when all of the following are true:

- [ ] B1 through B5 are closed.
- [ ] Report generated in strict publication mode.
- [ ] All required lanes are present and validated.
- [ ] Delta report compares two strict publication runs.
- [ ] Cloud reproducibility rerun reproduces the same conclusion envelope.

## Current status

- Publication status: `blocked`
- Shareability status: `internal-only until blockers are closed`
