# Benchmark Gaps, Optimizations, and Blockers Checklist

Updated at: `2026-02-26T07:21:50Z`  
Scope: third-party benchmark stack (`PLB-CI + Hyperfine`)

## Resolved this cycle

- [x] Added runtime-level regex capability path and canonicalized `regex-redux`
- [x] Cleared old generated benchmark reports and regenerated fresh artifacts
- [x] Installed local toolchains: `zig`, `deno`, `kotlin`, `pypy3`, `pyston3`
- [x] Enabled `clang`/`clang++` availability via Zig Clang frontend wrappers

## Open blockers (must-fix for strict public report)

- [ ] **B1: Remaining noncanonical adapters (hard gate)**
  - `edigits`
  - `http-server`
  - `json-serde`
  - `secp256k1`
  - Evidence: parity validator fails in publication mode

- [ ] **B2: Strict lane completeness still missing**
  - Runtime lanes unavailable: `rust`, `zig`, `swift`
  - Compile lanes unavailable: `rust`, `zig`, `swift`
  - Evidence: `reports/benchmarks/third_party/latest/summary.md` budget violations

- [ ] **B3: Swift toolchain not usable locally**
  - `swift`/`swiftc` still missing from preflight in local mode
  - Swiftly initialization works, but toolchain fetch did not complete in this environment

- [ ] **B4: Docker strict run instability**
  - Docker was intermittently unreachable during this session
  - Strict collection requires stable `docker info` and Docker-enabled run metadata

## Optimization opportunities (post-blocker)

- [ ] **Runtime hotspot reduction vs Kotlin**
  - Prioritize: `json-serde`, `secp256k1`, `http-server`
  - Goal: runtime geomean ratio vs Kotlin < `1.0`

- [ ] **Compile cold-start optimization vs C/C++/Go**
  - Current ratios: C `1.112`, C++ `1.123`, Go `1.041`
  - Focus: frontend startup and codegen/link path latency

- [ ] **Noise and reproducibility hardening**
  - Stabilize host load and pin execution environment
  - Keep strict Docker path as the default publish path

## Next execution checklist

- [ ] Confirm Docker daemon is stable (`docker info` succeeds consistently)
- [ ] Complete canonical implementations for the remaining 4 adapters
- [ ] Re-run strict collection:
  - `python3 tooling/metrics/collect_third_party_benchmarks.py --profile full --publication-mode`
- [ ] Re-run strict validation:
  - `python3 tooling/metrics/validate_third_party_benchmarks.py --publication-mode`
- [ ] Publish only when all strict checks pass (no noncanonical adapters, no missing required lanes)
