# V1 Distribution Matrix

Date: 2026-02-21

## Purpose

Define packaged artifact formats and install validation coverage for tier-1
independent installation (no Cargo dependency required on end-user machines).

## Tier-1 Distribution Targets

| Platform | Target Triple | Artifact | Install Mode | CI Build Job | CI Install Smoke Job |
| --- | --- | --- | --- | --- | --- |
| Linux | `x86_64-unknown-linux-gnu` | `vibe-x86_64-unknown-linux-gnu.tar.gz` | archive extract + `bin/vibe` | workflow `.github/workflows/v1-packaged-release.yml` job `package_artifacts` | workflow `.github/workflows/v1-packaged-release.yml` job `install_smoke_linux` |
| macOS | `x86_64-apple-darwin` | `vibe-x86_64-apple-darwin.tar.gz` | archive extract + `bin/vibe` | workflow `.github/workflows/v1-packaged-release.yml` job `package_artifacts` | workflow `.github/workflows/v1-packaged-release.yml` job `install_smoke_macos` |
| Windows | `x86_64-pc-windows-msvc` | `vibe-x86_64-pc-windows-msvc.zip` | archive extract + `bin/vibe.exe` | workflow `.github/workflows/v1-packaged-release.yml` job `package_artifacts` | workflow `.github/workflows/v1-packaged-release.yml` job `install_smoke_windows` |

## Required Sidecar Artifacts

Each packaged artifact is accompanied by:

- checksum manifest: `checksums-<target>.txt`
- signature + certificate pairs (`.sig`, `.pem`) for package, provenance, and SBOM
- provenance statement: `<artifact>.provenance.json`
- SBOM: `<artifact>.sbom.spdx.json`

## Promotion Rule

A release candidate is not promotable if any tier-1 package:

- is missing from the release bundle,
- fails signature/provenance/SBOM verification,
- fails clean-machine install smoke.
