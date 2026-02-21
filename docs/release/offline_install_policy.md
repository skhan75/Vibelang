# Offline Install Policy (Packaged Binaries)

Date: 2026-02-21

## Objective

Support air-gapped and restricted-network environments using pre-fetched release
bundles.

## Offline Bundle Requirements

An offline install bundle must include, per target:

- packaged archive (`.tar.gz` or `.zip`)
- `checksums-<target>.txt`
- signature/certificate files for package, provenance, and SBOM
- provenance statement (`.provenance.json`)
- SBOM (`.sbom.spdx.json`)

## Mirror Strategy

- Store release bundles in an internal artifact mirror (filesystem/object store).
- Mirror path should preserve exact filenames from upstream release.
- Validation scripts must run against mirrored files before promotion to internal
  users.

## Verification in Offline Mode

When internet transparency lookups are unavailable:

- verify checksums locally,
- verify signatures using bundled certificates,
- verify provenance digest locally against package digest.

## Operational Rules

- Offline bundle publication must follow same promotion criteria as online
  release artifacts.
- Any missing required file blocks offline promotion for that target.
