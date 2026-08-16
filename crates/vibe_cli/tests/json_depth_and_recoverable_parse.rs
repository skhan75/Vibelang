// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

//! VL-HTTP-03 and VL-HTTP-04: JSON parsing must not be able to end the process
//! on behalf of whoever sent the bytes.
//!
//! Three properties are pinned here.
//!
//! 1. DEPTH IS CAPPED. `vibe_json_parse_value_internal` and its array/object
//!    siblings are mutually recursive; before the cap, a POST body of repeated
//!    `[` walked the handler thread's stack off its end and the process died by
//!    signal with no diagnostic at all. The cap is 256 and is shared by
//!    `json.parse`, `json.try_parse` and `json.is_valid`.
//!
//! 2. THERE IS A RECOVERABLE PARSE. `json.try_parse` returns `Result<Json, Str>`
//!    and never aborts, whatever the input.
//!
//! 3. `json.is_valid` IS GENUINE. It used to compare only the first and last
//!    non-space characters, so it answered `true` for `[x]` and `[1,2,]` and the
//!    guard-then-parse idiom the book recommends still aborted on the following
//!    `json.parse`. It now runs the real parser, so `if json.is_valid(s) {
//!    json.parse(s) }` cannot abort.
//!
//! CERTIFICATION BIAS IS THE THING TO AVOID HERE. The offending token is placed
//! FIRST, in the MIDDLE and LAST in the malformed cases, and the over-deep
//! nesting is exercised both at offset 0 and behind a well-formed prefix, so a
//! fix that only handles "damage at the end of the document" cannot pass.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Matches VIBE_JSON_MAX_DEPTH in runtime/native/vibe_runtime.c.
const MAX_DEPTH: usize = 256;

#[test]
fn json_parse_turns_stack_exhausting_nesting_into_a_deterministic_panic() {
    // Pre-fix this input did not panic: the process died by SIGBUS/SIGSEGV with
    // an empty stderr, which is why a server could not even log what hit it.
    let dir = unique_temp_dir("json_depth_parse_panic");
    fs::create_dir_all(&dir).expect("create temp dir");
    let deep = dir.join("deep.json");
    fs::write(&deep, "[".repeat(200_000)).expect("write deep json");

    let source = temp_source_in(
        &dir,
        &format!(
            r#"
pub main() -> Int {{
  @effect io
  doc := json.parse(fs.read_text("{}"))
  println(json.stringify(doc))
  0
}}
"#,
            deep.display()
        ),
    );

    let out = run_vibe(&["run", source.to_str().expect("source path")]);
    assert!(
        !out.status.success(),
        "over-deep json.parse should fail:\nstdout:\n{}",
        out.stdout
    );
    assert!(
        out.stderr
            .contains("panic: json.parse exceeds maximum nesting depth"),
        "over-deep json.parse must report the depth cap rather than dying by signal:\nstderr:\n{}",
        out.stderr
    );
}

#[test]
fn json_depth_cap_boundary_is_exact_for_arrays_and_objects() {
    let dir = unique_temp_dir("json_depth_boundary");
    fs::create_dir_all(&dir).expect("create temp dir");

    // Arrays and objects share the cap. The "prefixed" case buries the deep
    // subtree behind well-formed content so the check cannot be passing merely
    // because the document starts with a bracket run.
    let cases: [(&str, String, bool); 5] = [
        ("arr_at_cap", nested_array(MAX_DEPTH), true),
        ("arr_over_cap", nested_array(MAX_DEPTH + 1), false),
        ("obj_at_cap", nested_object(MAX_DEPTH), true),
        ("obj_over_cap", nested_object(MAX_DEPTH + 1), false),
        (
            "prefixed_over_cap",
            format!(
                "{{\"ok\":1,\"list\":[1,2,3],\"deep\":{}}}",
                nested_array(MAX_DEPTH)
            ),
            // The wrapper object occupies level 1, so a MAX_DEPTH subtree nested
            // inside it needs MAX_DEPTH + 1 levels: depth is counted from the
            // document root, not from where the bracket run happens to start.
            false,
        ),
    ];

    let mut checks = String::new();
    for (name, body, _) in &cases {
        let path = dir.join(format!("{name}.json"));
        fs::write(&path, body).expect("write case");
        checks.push_str(&format!(
            "  report(\"{name}\", fs.read_text(\"{}\"))\n",
            path.display()
        ));
    }

    let source = temp_source_in(
        &dir,
        &format!(
            r#"
pub main() -> Int {{
  @effect io
{checks}  0
}}

report(name: Str, raw: Str) -> Int {{
  @effect io
  parsed := json.try_parse(raw)
  if result.is_ok(parsed) {{
    if json.is_valid(raw) {{
      println(name + " accepted")
    }} else {{
      println(name + " DISAGREE-ok-but-invalid")
    }}
  }} else {{
    if json.is_valid(raw) {{
      println(name + " DISAGREE-err-but-valid")
    }} else {{
      println(name + " rejected " + json.result_error(parsed))
    }}
  }}
  0
}}
"#
        ),
    );

    let out = run_vibe(&["run", source.to_str().expect("source path")]);
    assert!(
        out.status.success(),
        "boundary probe must not abort:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    let lines: Vec<&str> = out.stdout.lines().collect();
    assert_eq!(lines.len(), cases.len(), "stdout:\n{}", out.stdout);
    for (line, (name, _, expected_ok)) in lines.iter().zip(cases.iter()) {
        assert!(
            !line.contains("DISAGREE"),
            "json.try_parse and json.is_valid must agree on `{name}`: {line}"
        );
        if *expected_ok {
            assert_eq!(*line, format!("{name} accepted"), "stdout:\n{}", out.stdout);
        } else {
            assert!(
                line.starts_with(&format!("{name} rejected"))
                    && line.contains("nesting exceeds the maximum depth"),
                "`{name}` must be rejected for depth specifically: {line}"
            );
        }
    }
}

#[test]
fn json_try_parse_survives_every_malformed_shape_and_still_parses_valid_input() {
    let dir = unique_temp_dir("json_try_parse_total");
    fs::create_dir_all(&dir).expect("create temp dir");
    let deep = dir.join("deep.json");
    fs::write(&deep, "[".repeat(200_000)).expect("write deep json");

    // The offending token sits FIRST, in the MIDDLE and LAST across these
    // cases; a fix that only copes with trailing damage fails here.
    let source = temp_source_in(
        &dir,
        &format!(
            r#"
pub main() -> Int {{
  @effect io
  show("{{\"a\":1,\"b\":[1,2,3]}}")
  show("[]")
  show("x[1,2]")
  show("[1,x,2]")
  show("[1,2,x]")
  show("not-json")
  show("[1,2,]")
  show("{{\"a\":1}} trailing")
  show("")
  show(fs.read_text("{}"))
  println("survived")
  0
}}

show(raw: Str) -> Int {{
  @effect io
  parsed := json.try_parse(raw)
  if result.is_ok(parsed) {{
    println("ok " + json.stringify(json.result_value(parsed)))
  }} else {{
    println("err " + json.result_error(parsed))
  }}
  0
}}
"#,
            deep.display()
        ),
    );

    let out = run_vibe(&["run", source.to_str().expect("source path")]);
    assert!(
        out.status.success(),
        "json.try_parse must never abort:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(
        out.stdout,
        concat!(
            "ok {\"a\":1,\"b\":[1,2,3]}\n",
            "ok []\n",
            "err invalid JSON\n",
            "err invalid JSON\n",
            "err invalid JSON\n",
            "err invalid JSON\n",
            "err invalid JSON\n",
            "err invalid trailing content\n",
            "err invalid JSON\n",
            "err JSON nesting exceeds the maximum depth of 256\n",
            "survived\n",
        ),
        "stderr:\n{}",
        out.stderr
    );
}

#[test]
fn json_is_valid_is_a_real_parse_so_the_guard_then_parse_idiom_cannot_abort() {
    // Pre-fix, is_valid("[x]") and is_valid("[1,2,]") were both true because it
    // only looked at the first and last non-space characters. This program
    // therefore reached json.parse and aborted. It must now run to completion.
    let dir = unique_temp_dir("json_is_valid_guard");
    fs::create_dir_all(&dir).expect("create temp dir");
    let deep = dir.join("deep_balanced.json");
    fs::write(&deep, format!("{}{}", "[".repeat(4000), "]".repeat(4000)))
        .expect("write balanced deep json");

    let source = temp_source_in(
        &dir,
        &format!(
            r#"
pub main() -> Int {{
  @effect io
  guard("[x]")
  guard("[1,2,]")
  guard("{{\"a\":1,}}")
  guard("{{\"a\" 1}}")
  guard("[1,2,3]")
  guard("{{\"a\":\"say \\\"hi\\\"\"}}")
  guard(fs.read_text("{}"))
  println("survived")
  0
}}

guard(raw: Str) -> Int {{
  @effect io
  if json.is_valid(raw) {{
    println("valid " + json.stringify(json.parse(raw)))
  }} else {{
    println("invalid")
  }}
  0
}}
"#,
            deep.display()
        ),
    );

    let out = run_vibe(&["run", source.to_str().expect("source path")]);
    assert!(
        out.status.success(),
        "guard-then-parse must not abort:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(
        out.stdout,
        concat!(
            "invalid\n",
            "invalid\n",
            "invalid\n",
            "invalid\n",
            "valid [1,2,3]\n",
            "valid {\"a\":\"say \\\"hi\\\"\"}\n",
            "invalid\n",
            "survived\n",
        ),
        "stderr:\n{}",
        out.stderr
    );
}

#[test]
fn json_number_grammar_survives_the_rewrite_and_now_reaches_inside_containers() {
    // Two jobs in one program, because they share a fixture.
    //
    // REGRESSION GUARD (must stay green on any build): the earlier
    // number-grammar and string-escape fixes must not move, now that
    // json.parse and json.is_valid share one depth-capped core.
    //
    // NEW BEHAVIOUR (fails on the pre-fix runtime): the number grammar now
    // applies to numbers nested INSIDE a container. The old json.is_valid
    // looked at the first and last non-space character only, so every one of
    // the `[...]` and `{...}` cases below -- malformed numbers wrapped in
    // brackets -- was answered `true`. It is the same malformed-number
    // vocabulary as the guard above, moved one level in, which is exactly where
    // the old implementation stopped looking.
    let dir = unique_temp_dir("json_regression_unchanged");
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = temp_source_in(
        &dir,
        r#"
pub main() -> Int {
  @effect io
  println(json.stringify(json.parse("1e-5")))
  println(json.stringify(json.parse("-2.5e-10")))
  println(json.stringify(json.parse("1e+5")))
  println(json.stringify(json.parse("1E-3")))
  println(json.stringify(json.parse("[0.1, 1e300, -2.5e-10]")))
  println(json.stringify(json.parse("{\"q\":\"say \\\"hi\\\"\",\"nl\":\"a\\nb\",\"tab\":\"a\\tb\",\"esc\":\"a\\\\b\"}")))
  println(json.stringify(json.parse(json.stringify(json.parse("{\"q\":\"say \\\"hi\\\"\"}")))))
  flag(json.is_valid("1e-5"))
  flag(json.is_valid("-2.5e-10"))
  flag(json.is_valid("1e300"))
  flag(json.is_valid("1e-"))
  flag(json.is_valid("1e+"))
  flag(json.is_valid("+1"))
  flag(json.is_valid("1.-5"))
  flag(json.is_valid("[1e-5,1e300]"))
  flag(json.is_valid("[1e-]"))
  flag(json.is_valid("[1e+]"))
  flag(json.is_valid("[+1]"))
  flag(json.is_valid("[1.-5]"))
  flag(json.is_valid("{\"a\":1e-}"))
  flag(json.is_valid("{\"a\":1.2.3}"))
  0
}

flag(v: Bool) -> Int {
  @effect io
  if v {
    println("valid")
  } else {
    println("invalid")
  }
  0
}
"#,
    );

    let out = run_vibe(&["run", source.to_str().expect("source path")]);
    assert!(
        out.status.success(),
        "regression probe failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(
        out.stdout,
        concat!(
            "1e-05\n",
            "-2.5e-10\n",
            "1e+05\n",
            "0.001\n",
            "[0.1,1e+300,-2.5e-10]\n",
            "{\"q\":\"say \\\"hi\\\"\",\"nl\":\"a\\nb\",\"tab\":\"a\\tb\",\"esc\":\"a\\\\b\"}\n",
            "{\"q\":\"say \\\"hi\\\"\"}\n",
            // Top-level answers: the regression guard, unchanged by this work.
            "valid\nvalid\nvalid\ninvalid\ninvalid\ninvalid\ninvalid\n",
            // The same vocabulary one level in: `valid` then six `invalid`.
            // The pre-fix runtime answered `valid` to all seven.
            "valid\ninvalid\ninvalid\ninvalid\ninvalid\ninvalid\ninvalid\n",
        ),
        "stderr:\n{}",
        out.stderr
    );
}

#[test]
fn json_is_valid_answers_without_materializing_the_document() {
    // The property here cannot be read off stdout: a validator that builds the
    // whole document and throws it away gives exactly the same answers, it just
    // allocates the body first. Measured on the attempt this replaces, a 4 MB
    // body cost 121 MB of peak RSS to validate against 5.7 MB to read -- ~28x
    // the body in transient heap, on a code path a server can reach without
    // ever parsing. Peak-RSS assertions are flaky and platform-specific, so the
    // guard is structural instead: json.is_valid must go through the shared
    // parse core in validate-only mode, which is the thing that skips every
    // per-node allocation. If someone "simplifies" it back to building a tree,
    // this fails.
    let runtime = fs::read_to_string(workspace_root().join("runtime/native/vibe_runtime.c"))
        .expect("read runtime source");
    let signature = "int64_t vibe_json_is_valid(const char *raw) {";
    let start = runtime
        .find(signature)
        .unwrap_or_else(|| panic!("vibe_json_is_valid not found in the runtime"));
    let body_start = start + signature.len();
    let end = runtime[body_start..]
        .find("\n}\n")
        .expect("end of vibe_json_is_valid");
    let body = &runtime[body_start..body_start + end];

    assert!(
        body.contains("vibe_json_parse_document(raw, NULL, &state)"),
        "json.is_valid must call the shared parse core in validate-only mode \
         (a NULL out pointer), so that it allocates nothing per node:\n{body}"
    );
    assert!(
        !body.contains("vibe_json_free_value"),
        "json.is_valid should have no tree to free; freeing one means it built \
         one, which is the memory-amplification shape this guard exists to \
         prevent:\n{body}"
    );
    // And the core really does honour it: no node constructor may run
    // unconditionally in the recursive descent.
    for constructor in [
        "vibe_json_value *json = vibe_json_new_value(VIBE_JSON_ARRAY);",
        "vibe_json_value *json = vibe_json_new_value(VIBE_JSON_OBJECT);",
    ] {
        assert!(
            !runtime.contains(constructor),
            "`{constructor}` is unconditional, so validate-only mode would \
             still allocate one node per container"
        );
    }
}

fn nested_array(depth: usize) -> String {
    format!("{}{}", "[".repeat(depth), "]".repeat(depth))
}

fn nested_object(depth: usize) -> String {
    format!("{}1{}", "{\"a\":".repeat(depth), "}".repeat(depth))
}

fn temp_source_in(dir: &Path, source: &str) -> PathBuf {
    let file = dir.join("main.yb");
    fs::write(&file, source.trim_start()).expect("write temp source file");
    file
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos))
}

fn run_vibe(args: &[&str]) -> CmdOutput {
    let output = Command::new(env!("CARGO_BIN_EXE_vibe"))
        .args(args)
        .current_dir(workspace_root())
        .output()
        .expect("run vibe command");
    CmdOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve workspace root")
}

struct CmdOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}
