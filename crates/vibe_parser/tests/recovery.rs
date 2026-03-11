// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use vibe_diagnostics::Severity;
use vibe_parser::parse_source;

#[test]
fn reports_multiple_errors_in_single_pass() {
    let src = r#"
broken( {
  @examples {
    broken([1]) [1]
  }
  x :=
}
"#;
    let out = parse_source(src);
    let errors = out
        .diagnostics
        .as_slice()
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    assert!(errors >= 2, "expected multiple parse errors, got {errors}");
}
