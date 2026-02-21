# V1 Distribution Security Policy

Date: 2026-02-21

## Objective

Guarantee release integrity for packaged `vibe` artifacts using a layered trust
model.

## Required Controls (Tier-1, Blocking)

For every tier-1 packaged artifact:

1. **Checksum**
   - SHA256 entry in `checksums-<target>.txt`
2. **Signature**
   - keyless Sigstore/Cosign signature (`.sig`) and certificate (`.pem`)
3. **Provenance**
   - signed provenance statement (`.provenance.json`, plus signature + cert)
4. **SBOM**
   - SPDX JSON SBOM (`.sbom.spdx.json`, plus signature + cert)

## CI Enforcement

Policy is enforced by workflow:

- `.github/workflows/v1-packaged-release.yml`
  - `sign_attest_and_sbom`
  - `install_smoke_linux`
  - `install_smoke_macos`
  - `install_smoke_windows`

## Verification Requirements

Release validation must confirm:

- checksum digest matches artifact bytes,
- signatures verify with OIDC issuer
  `https://token.actions.githubusercontent.com`,
- provenance subject digest matches packaged artifact digest,
- SBOM artifact exists and signature verifies.

## Failure Policy

Any failed/missing security control is a release blocker (`P0`) per
`docs/release/release_blocker_policy.md`.
