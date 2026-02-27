# Benchmark Gaps, Optimizations, and Blockers Checklist

Updated at: `2026-02-27T13:45:17Z`  
Scope: third-party benchmark stack (`PLB-CI + Hyperfine`)

## Resolved this cycle

- [x] Added runtime-level regex capability path and canonicalized `regex-redux`
- [x] Cleared old generated benchmark reports and regenerated fresh artifacts
- [x] Installed local toolchains: `zig`, `deno`, `kotlin`, `pypy3`, `pyston3`
- [x] Enabled `clang`/`clang++` availability via Zig Clang frontend wrappers
- [x] Fixed VibeLang PLB-CI wrapper creation (runner script now copied from adapter include path)
- [x] Restored Rust lane availability in no-docker runs (removed `sudo` after-build, set `CC/CXX` to `gcc/g++`)

## Open blockers (must-fix for strict public report)

- [ ] **B1: Remaining noncanonical adapters (hard gate)**
  - `edigits`
  - `http-server`
  - `json-serde`
  - `secp256k1`
  - Evidence: parity validator fails in publication mode

- [ ] **B2: Strict lane completeness still missing**
  - Runtime lane still unavailable: `zig`
  - Compile lane still unavailable: `zig`
  - Runtime/compile lanes for `rust` and `swift` are now available in local no-docker sweeps
  - Evidence: `reports/benchmarks/third_party/full/summary.md` budget violations

- [ ] **B3: Zig local compatibility mismatch**
  - Local Zig (`0.15.2`) is incompatible with portions of upstream PLB-CI Zig source set in no-docker mode
  - Current symptom: Zig build failures before result directory generation in compile/runtime sweep

- [ ] **B4: Docker strict run instability**
  - This WSL distro cannot currently reach Docker daemon (`docker info` reports Docker Desktop WSL integration not enabled)
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

- [ ] Enable Docker Desktop WSL integration for this distro and confirm `docker info` is healthy
- [ ] Resolve Zig local compatibility (pin compatible Zig for no-docker or rely on strict Docker lane)
- [ ] Complete canonical implementations for the remaining 4 adapters
- [ ] Re-run strict collection:
  - `python3 tooling/metrics/collect_third_party_benchmarks.py --profile full --publication-mode`
- [ ] Re-run strict validation:
  - `python3 tooling/metrics/validate_third_party_benchmarks.py --publication-mode`
- [ ] Publish only when all strict checks pass (no noncanonical adapters, no missing required lanes)
