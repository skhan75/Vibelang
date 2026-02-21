# Install VibeLang on Linux (Packaged, No Cargo)

## Minimum Runtime Requirements

- Architecture: `x86_64`
- Dynamic loader baseline for GNU package: `glibc >= 2.35`
- Recommended validation command:

```bash
ldd --version | head -n 1
```

If your environment is below baseline, use the fallback path in
`docs/install/troubleshooting.md` (`GLIBC_*` mismatch section).

## Download

From a release page, download:

- `vibe-x86_64-unknown-linux-gnu.tar.gz`
- `checksums-x86_64-unknown-linux-gnu.txt`
- `vibe-x86_64-unknown-linux-gnu.tar.gz.sig`
- `vibe-x86_64-unknown-linux-gnu.tar.gz.pem`
- `vibe-x86_64-unknown-linux-gnu.tar.gz.provenance.json`
- `vibe-x86_64-unknown-linux-gnu.tar.gz.provenance.json.sig`
- `vibe-x86_64-unknown-linux-gnu.tar.gz.provenance.json.pem`

## Verify

```bash
pkg="vibe-x86_64-unknown-linux-gnu.tar.gz"
checks="checksums-x86_64-unknown-linux-gnu.txt"
expected="$(awk -v p="$pkg" '$2==p {print $1}' "$checks")"
actual="$(sha256sum "$pkg" | awk '{print $1}')"
[ -n "$expected" ] && [ "$expected" = "$actual" ]

cosign verify-blob \
  --certificate-identity-regexp ".*" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  --signature vibe-x86_64-unknown-linux-gnu.tar.gz.sig \
  --certificate vibe-x86_64-unknown-linux-gnu.tar.gz.pem \
  vibe-x86_64-unknown-linux-gnu.tar.gz

cosign verify-blob \
  --certificate-identity-regexp ".*" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  --signature vibe-x86_64-unknown-linux-gnu.tar.gz.provenance.json.sig \
  --certificate vibe-x86_64-unknown-linux-gnu.tar.gz.provenance.json.pem \
  vibe-x86_64-unknown-linux-gnu.tar.gz.provenance.json
```

## Install

```bash
mkdir -p "$HOME/.local/vibe"
tar -xzf vibe-x86_64-unknown-linux-gnu.tar.gz -C "$HOME/.local/vibe"
export PATH="$HOME/.local/vibe/vibe-x86_64-unknown-linux-gnu/bin:$PATH"
vibe --version
```

Persist `PATH` in your shell profile if desired.

## Fallback Path

If GNU packaged binaries fail to load due runtime ABI mismatch in your Linux
environment, use one of these fallback options:

1. Install from source in the same environment:

```bash
cargo build --release -p vibe_cli
install -m 0755 target/release/vibe "$HOME/.local/bin/vibe"
vibe --version
```

1. Use a static `musl` Linux package when published in release artifacts
(`vibe-x86_64-unknown-linux-musl.tar.gz`).
