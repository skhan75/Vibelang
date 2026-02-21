# Packaged Install Troubleshooting

## Checksum Mismatch

Symptoms:

- checksum verification fails against `checksums-<target>.txt`

Actions:

- ensure you downloaded matching package/checksum files from same release
- re-download artifacts
- stop install if mismatch persists

## Signature Verification Failure

Symptoms:

- `cosign verify-blob` fails

Actions:

- confirm file/signature/certificate names match exactly
- confirm OIDC issuer is `https://token.actions.githubusercontent.com`
- verify no proxy/content rewriting is occurring

## Provenance Digest Mismatch

Symptoms:

- provenance subject digest does not match package digest

Actions:

- treat as blocker; do not install
- fetch release artifacts again
- report in release readiness dashboard as integrity incident

## `vibe` Not Found After Extract

Actions:

- confirm extracted path includes `bin/vibe` (or `bin/vibe.exe` on Windows)
- ensure that directory is on your `PATH`
- restart shell/terminal after PATH update

## `vibe run` Fails on Fresh Machine

Actions:

- verify executable permissions (`chmod +x` for Unix systems if needed)
- verify required system linker/build tools are present
- run `vibe --version` to confirm binary starts correctly
