# Target Limitations Register

Date: 2026-02-17

| ID | Limitation | Owner | Severity | Target Phase |
| --- | --- | --- | --- | --- |
| TGT-001 | Non-host runtime smoke for `aarch64-unknown-linux-gnu` depends on runner/toolchain availability | Runtime/CI | Medium | Phase 6.5 |
| TGT-002 | Non-host runtime smoke for `aarch64-apple-darwin` requires macOS arm64 runner coverage | Runtime/CI | Medium | Phase 6.5 |
| TGT-003 | Cross-target determinism checks are partial until full multi-host matrix is stable | Codegen/CI | Low | Phase 6.5 |

## Policy

- Every release must review this register.
- Closed items require linked evidence and date.
