// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Shared helpers for `vibe_cli` integration tests.

/// Host target triple used to locate build artifacts.
///
/// Mirrors `default_build_target()` in `crates/vibe_cli/src/main.rs` so tests
/// resolve the same artifact directory the CLI writes to on every host,
/// instead of hardcoding `x86_64-unknown-linux-gnu`.
pub fn host_target_triple() -> &'static str {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        _ => "x86_64-unknown-linux-gnu",
    }
}
