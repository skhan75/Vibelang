# V1 Distribution Readiness Report

Date: 2026-02-21

## Objective

Track packaged distribution readiness for tier-1 targets, including artifact
integrity controls (checksums, signatures, provenance, SBOM).

## Tier-1 Matrix Status

| Target | Artifact | Packaging Job | Install Smoke Job | Status |
| --- | --- | --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `vibe-x86_64-unknown-linux-gnu.tar.gz` | `package_artifacts` | `install_smoke_linux` | validated |
| `x86_64-apple-darwin` | `vibe-x86_64-apple-darwin.tar.gz` | `package_artifacts` | `install_smoke_macos` | validated |
| `x86_64-pc-windows-msvc` | `vibe-x86_64-pc-windows-msvc.zip` | `package_artifacts` | `install_smoke_windows` | validated |

## Security Control Status

| Control | Evidence | Status |
| --- | --- | --- |
| SHA256 checksums | workflow `.github/workflows/v1-packaged-release.yml` packaging + install verification steps | validated |
| Sigstore/Cosign signatures | workflow `.github/workflows/v1-packaged-release.yml` job `sign_attest_and_sbom` | validated |
| Provenance statements | workflow `.github/workflows/v1-packaged-release.yml` job `sign_attest_and_sbom` | validated |
| SBOM generation | workflow `.github/workflows/v1-packaged-release.yml` job `sign_attest_and_sbom` | validated |
| RC-to-RC reproducibility policy check | workflow `.github/workflows/v1-packaged-release.yml` job `packaged_reproducibility`, tool `tooling/release/checksum_manifest.py` | validated |

## Governance Alignment

- Distribution policy: `docs/release/distribution_matrix.md`
- Security policy: `docs/release/distribution_security.md`
- Offline policy: `docs/release/offline_install_policy.md`
- Channel policy: `docs/policy/install_channels_v1.md`

## Readiness Status

- Policy/docs: `complete`
- Workflow wiring: `complete`
- Tier-1 packaged install and trust cycle: `validated`
- Local phase8 evidence index published: `complete` (`reports/v1/phase8_ci_evidence.md`)
- First successful tier-1 signed package CI cycle: `validated` (jobs `packaged_reproducibility`, `sign_attest_and_sbom`, `install_smoke_linux`, `install_smoke_macos`, `install_smoke_windows`)

## Closure Status

- Distribution trust stack is closure-ready for Phase 8.
- Remaining release-readiness blockers, if any, are outside Phase 8 distribution trust scope.
