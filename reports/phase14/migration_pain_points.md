# Pilot Migration Pain Points Backlog

Date: 2026-02-22

| Pain Point | Observed In | Owner | Priority | Planned Mitigation |
| --- | --- | --- | --- | --- |
| Effect declarations (`@effect`) missed during first draft implementations | service + CLI pilots | DX + Compiler | High | Add lint quick-fixes and docs cookbook examples |
| Sendability diagnostics on channel values with ambiguous inferred types | service pilot | Compiler + Runtime | High | Improve diagnostic suggestions and typed channel examples |
| Manual release evidence collation is error-prone | both pilots | Release + Tooling | Medium | Keep automation-first evidence scripts and gate checks |

## Mapping to Follow-Up Work

- `DX-201`: effect annotation quick-fix coverage.
- `COMP-317`: sendability diagnostic hint improvements.
- `REL-112`: evidence collation automation hardening.
