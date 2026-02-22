# Package Security and Provenance Policy (Phase 12)

## Scope

This policy governs local package lifecycle checks in `vibe pkg`:

- vulnerability policy checks (`vibe pkg audit`)
- license policy checks (`vibe pkg audit`)
- deterministic publish index updates (`vibe pkg publish`)

## Policy files

### Audit policy

```toml
[licenses]
deny = ["GPL-3.0"]
```

### Advisory database

```toml
[[advisory]]
id = "VIBESEC-2026-0001"
package = "demo"
affected = "<2.0.0"
severity = "high"
```

## Enforcement

- `vibe pkg audit` exits non-zero when findings exist.
- CI should treat non-zero audit status as a release-blocking gate.

## Provenance baseline

- Publish artifacts are filesystem-local and deterministic by path/version.
- Registry index includes package identity, version, and source class (`local`).
- Full cryptographic signing and remote trust bundles are tracked for later phases.
