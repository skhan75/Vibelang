# Installation Verification

Use this checklist after packaged install on any platform.

## 1) Binary Availability

```bash
vibe --version
```

Expected: version string includes `version`, `commit`, `target`, and `profile`.

## 2) Hello-World Run

Create `hello.yb`:

```txt
pub main() -> Int {
  @effect io
  println("hello from installed vibe")
  0
}
```

Run:

```bash
vibe run hello.yb
```

Expected output:

```txt
hello from installed vibe
```

## 3) Optional Format/Check Smoke

```bash
vibe check hello.yb
vibe fmt . --check
```

## 4) Uninstall Sanity

Remove extracted install directory and ensure `vibe` no longer resolves on PATH.
