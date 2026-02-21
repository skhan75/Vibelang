# Install VibeLang on macOS (Packaged, No Cargo)

## Download

From a release page, download:

- `vibe-x86_64-apple-darwin.tar.gz`
- `checksums-x86_64-apple-darwin.txt`
- `vibe-x86_64-apple-darwin.tar.gz.sig`
- `vibe-x86_64-apple-darwin.tar.gz.pem`
- `vibe-x86_64-apple-darwin.tar.gz.provenance.json`
- `vibe-x86_64-apple-darwin.tar.gz.provenance.json.sig`
- `vibe-x86_64-apple-darwin.tar.gz.provenance.json.pem`

## Verify

```bash
grep "  vibe-x86_64-apple-darwin.tar.gz$" checksums-x86_64-apple-darwin.txt | shasum -a 256 -c -

cosign verify-blob \
  --certificate-identity-regexp ".*" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  --signature vibe-x86_64-apple-darwin.tar.gz.sig \
  --certificate vibe-x86_64-apple-darwin.tar.gz.pem \
  vibe-x86_64-apple-darwin.tar.gz

cosign verify-blob \
  --certificate-identity-regexp ".*" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  --signature vibe-x86_64-apple-darwin.tar.gz.provenance.json.sig \
  --certificate vibe-x86_64-apple-darwin.tar.gz.provenance.json.pem \
  vibe-x86_64-apple-darwin.tar.gz.provenance.json
```

## Install

```bash
mkdir -p "$HOME/.local/vibe"
tar -xzf vibe-x86_64-apple-darwin.tar.gz -C "$HOME/.local/vibe"
export PATH="$HOME/.local/vibe/vibe-x86_64-apple-darwin/bin:$PATH"
vibe --version
```

Persist `PATH` in your shell profile if desired.
