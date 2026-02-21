# V1 Distribution Readiness Report

Date: 2026-02-21

## Objective

Track packaged distribution readiness for tier-1 targets, including artifact
integrity controls (checksums, signatures, provenance, SBOM).

## Tier-1 Matrix Status

| Target | Artifact | Packaging Job | Install Smoke Job | Status |
| --- | --- | --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `vibe-x86_64-unknown-linux-gnu.tar.gz` | `package_artifacts` | `install_smoke_linux` | local-install-sim-pass + workflow-wired |
| `x86_64-apple-darwin` | `vibe-x86_64-apple-darwin.tar.gz` | `package_artifacts` | `install_smoke_macos` | workflow-wired |
| `x86_64-pc-windows-msvc` | `vibe-x86_64-pc-windows-msvc.zip` | `package_artifacts` | `install_smoke_windows` | workflow-wired |

## Security Control Status

| Control | Evidence | Status |
| --- | --- | --- |
| SHA256 checksums | workflow `v1-packaged-release.yml` packaging steps + local dry-run artifacts | local-pass + workflow-wired |
| Sigstore/Cosign signatures | workflow job `sign_attest_and_sbom` | workflow-wired |
| Provenance statements | workflow job `sign_attest_and_sbom` | workflow-wired |
| SBOM generation | workflow job `sign_attest_and_sbom` | workflow-wired |
| RC-to-RC reproducibility policy check | workflow job `packaged_reproducibility`, tool `tooling/release/checksum_manifest.py` | local-pass + workflow-wired |

## Governance Alignment

- Distribution policy: `docs/release/distribution_matrix.md`
- Security policy: `docs/release/distribution_security.md`
- Offline policy: `docs/release/offline_install_policy.md`
- Channel policy: `docs/policy/install_channels_v1.md`

## Readiness Status

- Policy/docs: `complete`
- Workflow wiring: `complete`
- Local Linux packaged-install simulation: `pass`
- Local phase8 evidence index published: `complete` (`reports/v1/phase8_ci_evidence.md`)
- First successful tier-1 signed package CI cycle: `pending`

## Open Follow-Up

- Capture first successful CI run URL and release artifact links.
- Update this report from `workflow-wired` to `validated` when CI artifacts are
  attached.
