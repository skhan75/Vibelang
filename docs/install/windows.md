# Install VibeLang on Windows (Packaged, No Cargo)

## Download

From a release page, download:

- `vibe-x86_64-pc-windows-msvc.zip`
- `checksums-x86_64-pc-windows-msvc.txt`
- `vibe-x86_64-pc-windows-msvc.zip.sig`
- `vibe-x86_64-pc-windows-msvc.zip.pem`
- `vibe-x86_64-pc-windows-msvc.zip.provenance.json`
- `vibe-x86_64-pc-windows-msvc.zip.provenance.json.sig`
- `vibe-x86_64-pc-windows-msvc.zip.provenance.json.pem`

## Verify (PowerShell)

```powershell
$pkg = "vibe-x86_64-pc-windows-msvc.zip"
$checksums = "checksums-x86_64-pc-windows-msvc.txt"

$line = Get-Content $checksums | Where-Object { $_ -match [regex]::Escape("  $pkg") }
$expected = ($line -split '\s+')[0].ToLower()
$actual = (Get-FileHash -Algorithm SHA256 $pkg).Hash.ToLower()
if ($expected -ne $actual) { throw "checksum mismatch" }

cosign verify-blob --certificate-identity-regexp ".*" --certificate-oidc-issuer "https://token.actions.githubusercontent.com" --signature "$pkg.sig" --certificate "$pkg.pem" "$pkg"
cosign verify-blob --certificate-identity-regexp ".*" --certificate-oidc-issuer "https://token.actions.githubusercontent.com" --signature "$pkg.provenance.json.sig" --certificate "$pkg.provenance.json.pem" "$pkg.provenance.json"
```

## Install (PowerShell)

```powershell
$installRoot = "$env:USERPROFILE\\vibe"
if (!(Test-Path $installRoot)) { New-Item -ItemType Directory -Path $installRoot | Out-Null }
Expand-Archive -Path "vibe-x86_64-pc-windows-msvc.zip" -DestinationPath $installRoot -Force
$env:Path = "$installRoot\\vibe-x86_64-pc-windows-msvc\\bin;$env:Path"
vibe --version
```

Persist the `Path` update in system/user environment variables if desired.
