// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! Arity and argument type checking for stdlib namespace calls
//! (audit P0-02): `text.trim(42)` previously built with exit 0 and then
//! SIGSEGVed at runtime because only the return type of the mapped stdlib
//! signature was consulted. Namespace calls must now emit the same
//! E2264/E2265 codes as direct calls, while correct usages, Int→Float
//! coercion, handle-as-Int APIs, and Unknown-typed arguments stay accepted.

use std::collections::BTreeMap;

use vibe_diagnostics::{Diagnostic, Severity};
use vibe_parser::parse_source;
use vibe_types::check_and_lower_with_ns;

/// Mangled stdlib declarations mirroring what the CLI module resolver
/// injects for native-only stdlib functions (see
/// `crates/vibe_cli/src/module_resolver.rs::load_stdlib_namespace_functions`).
/// Bodies are stand-ins; only the signatures matter for the checker.
const STDLIB_DECLS: &str = r#"
__stdlib_text__trim(s: Str) -> Str { s }
__stdlib_math__sqrt(x: Float) -> Float { x }
__stdlib_str_builder__new(capacity: Int) -> Int { capacity }
__stdlib_str_builder__finish(handle: Int) -> Str { "" }
__stdlib_path__join(a: Str, b: Str) -> Str { a }
__stdlib_opaque__handle(x: Int) { x }
__stdlib_convert__to_str(n: Int) -> Str { "" }
"#;

fn namespace_map() -> BTreeMap<(String, String), String> {
    let mut map = BTreeMap::new();
    for (ns, field) in [
        ("text", "trim"),
        ("math", "sqrt"),
        ("str_builder", "new"),
        ("str_builder", "finish"),
        ("path", "join"),
        ("opaque", "handle"),
        ("convert", "to_str"),
    ] {
        map.insert(
            (ns.to_string(), field.to_string()),
            format!("__stdlib_{ns}__{field}"),
        );
    }
    map
}

fn check(src: &str) -> Vec<Diagnostic> {
    let full = format!("{src}\n{STDLIB_DECLS}");
    let parsed = parse_source(&full);
    assert!(
        !parsed.diagnostics.has_errors(),
        "unexpected parse errors:\n{}",
        parsed.diagnostics.to_golden()
    );
    check_and_lower_with_ns(&parsed.ast, &namespace_map())
        .diagnostics
        .sorted()
}

fn codes(diags: &[Diagnostic]) -> Vec<&str> {
    diags.iter().map(|d| d.code.as_str()).collect()
}

#[test]
fn text_trim_int_arg_is_rejected() {
    // Audit repro: previously built with exit 0, then SIGSEGVed (42
    // dereferenced as char*).
    let diags = check(
        r#"pub main() -> Int {
  s := text.trim(42)
  len(s)
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2265"], "diagnostics: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(
        d.message,
        "argument 1 type mismatch in call to `text.trim`: expected `Str`, got `Int`"
    );
    assert_eq!(d.span.line_start, 2, "span should point at the argument");
}

#[test]
fn math_sqrt_str_arg_is_rejected() {
    // Audit repro: previously produced no type error, then died as a
    // spanless Cranelift verifier error.
    let diags = check(
        r#"pub main() -> Int {
  x := math.sqrt("hello")
  if x > 1.0 {
    return 1
  }
  0
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2265"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "argument 1 type mismatch in call to `math.sqrt`: expected `Float`, got `Str`"
    );
    assert_eq!(diags[0].span.line_start, 2);
}

#[test]
fn str_builder_new_arity_is_rejected() {
    // Audit repro: `str_builder.new()` (declared arity 1) previously
    // passed the checker.
    let diags = check(
        r#"pub main() -> Int {
  sb := str_builder.new()
  sb
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2264"], "diagnostics: {diags:?}");
    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(
        d.message,
        "function `str_builder.new` expects 1 argument(s), got 0"
    );
    assert_eq!(d.span.line_start, 2, "span should point at the call");
}

#[test]
fn path_join_int_arg_is_rejected() {
    // Audit repro: previously built, then crashed at runtime.
    let diags = check(
        r#"pub main() -> Int {
  p := path.join("/tmp", 99)
  len(p)
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2265"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "argument 2 type mismatch in call to `path.join`: expected `Str`, got `Int`"
    );
}

#[test]
fn too_many_args_is_rejected() {
    let diags = check(
        r#"pub main() -> Int {
  s := text.trim("a", "b")
  len(s)
}
"#,
    );
    assert_eq!(codes(&diags), vec!["E2264"], "diagnostics: {diags:?}");
    assert_eq!(
        diags[0].message,
        "function `text.trim` expects 1 argument(s), got 2"
    );
}

#[test]
fn multiple_bad_args_report_each_position() {
    let diags = check(
        r#"pub main() -> Int {
  p := path.join(1, 2)
  len(p)
}
"#,
    );
    assert_eq!(
        codes(&diags),
        vec!["E2265", "E2265"],
        "diagnostics: {diags:?}"
    );
    assert_eq!(
        diags[0].message,
        "argument 1 type mismatch in call to `path.join`: expected `Str`, got `Int`"
    );
    assert_eq!(
        diags[1].message,
        "argument 2 type mismatch in call to `path.join`: expected `Str`, got `Int`"
    );
}

#[test]
fn correct_usages_are_accepted() {
    // Handle-as-Int APIs (str_builder) take Int handles by declared
    // signature; Int arguments must keep passing there.
    let diags = check(
        r#"pub main() -> Int {
  s := text.trim("  x  ")
  sb := str_builder.new(16)
  out := str_builder.finish(sb)
  p := path.join("/tmp", "logs")
  len(s) + len(out) + len(p)
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn int_to_float_coercion_is_accepted() {
    let diags = check(
        r#"pub main() -> Int {
  x := math.sqrt(4)
  if x > 1.0 {
    return 1
  }
  0
}
"#,
    );
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn interpolated_str_value_is_accepted() {
    // Regression guard: the parser desugars `"hello {s}"` into
    // `"hello " + convert.to_str(s)`, and HIR lowering rewrites
    // `convert.to_str(<Str>)` into a passthrough of the argument (see
    // `is_convert_to_str_call` in vibe_types), so a `Str` argument is
    // valid even though the stdlib declares `to_str(n: Int)`. The stdlib
    // arg check must not reject it (it briefly did, breaking
    // `examples/01_basics/75_string_interpolation.yb`).
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  s := "world"
  println("hello {s}")
  0
}
"#,
    );
    assert!(
        !codes(&diags).contains(&"E2264") && !codes(&diags).contains(&"E2265"),
        "Str interpolation must pass the stdlib arg check: {diags:?}"
    );
}

#[test]
fn interpolated_int_value_is_accepted() {
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  n := 42
  println("num {n}")
  0
}
"#,
    );
    assert!(
        !codes(&diags).contains(&"E2264") && !codes(&diags).contains(&"E2265"),
        "Int interpolation must pass the stdlib arg check: {diags:?}"
    );
}

#[test]
fn interpolated_bool_value_is_rejected_with_spanned_e2265() {
    // Ground truth (pre-Theme-A binary, empirically verified): Bool
    // interpolation never worked — it died as a spanless Cranelift
    // verifier error at codegen. The spanned check-time E2265 is a
    // deliberate improvement, not a regression; do not add Bool to the
    // accepted set without codegen support.
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  b := true
  println("flag {b}")
  0
}
"#,
    );
    assert!(
        codes(&diags).contains(&"E2265"),
        "Bool interpolation must be rejected at check time: {diags:?}"
    );
    let d = diags.iter().find(|d| d.code == "E2265").expect("E2265");
    assert_eq!(
        d.message,
        "argument 1 type mismatch in call to `convert.to_str`: expected `Int`, got `Bool`"
    );
    assert_eq!(d.severity, Severity::Error);
}

#[test]
fn interpolated_float_value_is_not_rejected_at_check_time() {
    // Ground truth (pre-Theme-A binary, empirically verified): Float
    // interpolation fails in codegen (Cranelift verifier error, the
    // known P1-12 float bug), same as today. The checker accepts it via
    // the Int→Float coercion rule, so the failure stays a codegen
    // problem — this test only pins that the checker does not change
    // that boundary.
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  f := 1.5
  println("val {f}")
  0
}
"#,
    );
    assert!(
        !codes(&diags).contains(&"E2264") && !codes(&diags).contains(&"E2265"),
        "Float interpolation is a codegen (P1-12) issue, not a check-time one: {diags:?}"
    );
}

#[test]
fn direct_convert_to_str_str_arg_is_accepted() {
    // Direct user-written calls take the same HIR passthrough as the
    // interpolation desugaring, so `Str` arguments must stay accepted.
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  s := "abc"
  println(convert.to_str(s))
  println(convert.to_str("lit"))
  0
}
"#,
    );
    assert!(
        !codes(&diags).contains(&"E2264") && !codes(&diags).contains(&"E2265"),
        "direct convert.to_str(Str) must pass: {diags:?}"
    );
}

#[test]
fn direct_convert_to_str_int_arg_is_accepted() {
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  println(convert.to_str(42))
  0
}
"#,
    );
    assert!(
        !codes(&diags).contains(&"E2264") && !codes(&diags).contains(&"E2265"),
        "direct convert.to_str(Int) must pass: {diags:?}"
    );
}

#[test]
fn direct_convert_to_str_bool_arg_is_rejected() {
    let diags = check(
        r#"pub main() -> Int {
  @effect io
  b := false
  println(convert.to_str(b))
  0
}
"#,
    );
    assert!(
        codes(&diags).contains(&"E2265"),
        "direct convert.to_str(Bool) must be rejected: {diags:?}"
    );
    let d = diags.iter().find(|d| d.code == "E2265").expect("E2265");
    assert_eq!(
        d.message,
        "argument 1 type mismatch in call to `convert.to_str`: expected `Int`, got `Bool`"
    );
}

#[test]
fn unknown_typed_arg_is_accepted() {
    // `opaque.handle` has no declared return type, so its call site
    // infers `Unknown` — the permissive `type_compatible` rule must let
    // an Unknown-typed argument through.
    let diags = check(
        r#"pub main() -> Int {
  v := opaque.handle(1)
  s := text.trim(v)
  len(s)
}
"#,
    );
    assert!(
        !codes(&diags).contains(&"E2264") && !codes(&diags).contains(&"E2265"),
        "Unknown-typed argument must pass: {diags:?}"
    );
}
