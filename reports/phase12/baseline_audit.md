# Phase 12 Baseline Audit

Date: 2026-02-17

## Scope mapping

### 12.1 Core stdlib surface

- Baseline before implementation:
  - only minimal `io.print/println` + deterministic helpers in docs
  - no first-class `time/path/fs/json/http` module docs
  - no dedicated phase test suite for stdlib behavior
- Delivery in this execution:
  - runtime/type/codegen surfaces added for `time/path/fs/json/http`
  - module docs published under `stdlib/*/README.md`
  - deterministic/error-model integration tests added in
    `crates/vibe_cli/tests/phase12_stdlib.rs`
  - fixture corpus added under `compiler/tests/fixtures/stdlib/`

### 12.2 Package lifecycle and registry readiness

- Baseline before implementation:
  - `vibe pkg` supported `resolve|lock|install`
  - local mirror layout documented, no publish/index/security policy docs
- Planned in this phase:
  - add `publish` flow and deterministic registry index artifact
  - add policy + tooling for vulnerability/license checks and semver guidance

### 12.3 Testing and QA ecosystem

- Baseline before implementation:
  - `vibe test` supported whole-tree execution + summary only
  - no built-in filter/shard/report JSON output
  - no explicit phase tooling for coverage or golden update automation
- Planned in this phase:
  - `vibe test` ergonomics: filter/shard/report
  - add tooling under `tooling/coverage` and `tooling/golden`
  - publish phase readiness reports and checklist evidence links
