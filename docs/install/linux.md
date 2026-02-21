# Install VibeLang on Linux (Packaged, No Cargo)

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
grep "  vibe-x86_64-unknown-linux-gnu.tar.gz$" checksums-x86_64-unknown-linux-gnu.txt | sha256sum -c -

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
