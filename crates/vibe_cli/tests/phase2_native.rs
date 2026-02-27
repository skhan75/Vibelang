use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn hello_world_build_and_run() {
    let source = temp_fixture_copy("build/hello_world.vibe");
    let build = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        build.status.success(),
        "build failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );

    let binary = artifact_binary_path(&source, "dev", "x86_64-unknown-linux-gnu");
    assert!(
        binary.exists(),
        "binary should be emitted at {}",
        binary.display()
    );

    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "hello from vibelang\n");
}

#[test]
fn hello_world_build_and_run_with_yb_extension() {
    let source = temp_fixture_copy_with_extension("build/hello_world.vibe", "yb");
    let build = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        build.status.success(),
        "build failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );

    let binary = artifact_binary_path(&source, "dev", "x86_64-unknown-linux-gnu");
    assert!(
        binary.exists(),
        "binary should be emitted at {}",
        binary.display()
    );

    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "hello from vibelang\n");
}

#[test]
fn function_call_fixture_runs() {
    let source = temp_fixture_copy("build/function_calls.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "from helper\n");
}

#[test]
fn if_control_flow_fixture_runs() {
    let source = temp_fixture_copy("build/control_flow_if.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "if-branch\n");
}

#[test]
fn concurrency_go_select_fixture_runs() {
    let source = temp_fixture_copy("build/concurrency_go_select.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "go-worker\nselect-recv\n");
}

#[test]
fn select_default_fixture_runs() {
    let source = temp_fixture_copy("build/select_default.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "default\n");
}

#[test]
fn select_closed_fixture_runs() {
    let source = temp_fixture_copy("build/select_closed.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "closed\n");
}

#[test]
fn select_multi_receive_fixture_runs() {
    let source = temp_fixture_copy("build/select_multi_receive.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "from-ch2\n");
}

#[test]
fn select_after_timeout_fixture_runs() {
    let source = temp_fixture_copy("build/select_after_timeout.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "after\n");
}

#[test]
fn builtin_len_and_max_lowering_runs() {
    let source = temp_source_file(
        "vibe_builtin_len_max",
        r#"
pub main() -> Int {
  @effect io
  xs := [2, 5, 1]
  best := max(xs.get(0), xs.get(1))
  if best == 5 {
    if len("abc") == 3 {
      println("builtin-len-max-ok")
    } else {
      println("builtin-len-max-bad-len")
    }
  } else {
    println("builtin-len-max-bad-max")
  }
  0
}
"#,
    );
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "builtin-len-max-ok\n");
}

#[test]
fn float_arithmetic_and_comparison_runs() {
    let source = temp_source_file(
        "vibe_float_stability",
        r#"
pub main() -> Int {
  @effect io
  ratio := 7.5 / 2.5
  if ratio > 2.9 {
    if ratio < 3.1 {
      println("float-stability-ok")
    } else {
      println("float-stability-bad-high")
    }
  } else {
    println("float-stability-bad-low")
  }
  0
}
"#,
    );
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "float-stability-ok\n");
}

#[test]
fn type_point_construction_and_field_access_runs() {
    let source = temp_source_file(
        "type_point_basics",
        r#"
type Point {
  x: Int,
  y: Int
}

pub main() -> Int {
  @effect alloc
  @effect io
  p := Point { x: 3, y: 4 }
  sum := p.x + p.y
  if sum == 7 {
    println("point-sum")
  }
  0
}
"#,
    );
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "point-sum\n");
}

#[test]
fn enum_match_runs() {
    let source = temp_source_file(
        "enum_match_basics",
        r#"
enum Color { Red, Green, Blue }

pub main() -> Int {
  @effect io
  c := Color.Red
  match c {
    case Color.Red => println("red")
    case Color.Green => println("green")
    case Color.Blue => println("blue")
  }
  0
}
"#,
    );
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "red\n");
}

#[test]
fn type_field_assignment_mismatch_reports_diagnostic() {
    let source = temp_source_file(
        "type_field_assignment_mismatch",
        r#"
type Point {
  x: Int,
  y: Int
}

pub main() -> Int {
  @effect alloc
  p := Point { x: 1, y: 2 }
  p.x = "oops"
  0
}
"#,
    );
    let out = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "build unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr
            .contains("field assignment type mismatch `Point.x`"),
        "expected field assignment type mismatch diagnostic:\n{}",
        out.stderr
    );
}

#[test]
fn list_sort_desc_and_take_runs() {
    let source = temp_source_file(
        "vibe_sort_take",
        r#"
pub main() -> Int {
  @effect io
  xs := [5, 2, 9, 1]
  top := xs.sort_desc().take(2)
  if top.get(0) == 9 {
    if top.get(1) == 5 {
      println("sort-take-ok")
    } else {
      println("sort-take-bad-1")
    }
  } else {
    println("sort-take-bad-0")
  }
  0
}
"#,
    );
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "sort-take-ok\n");
}

#[test]
fn vibe_test_supports_append_inside_contract_examples() {
    let source = temp_source_file(
        "vibe_contract_append",
        r#"
build_series(n: Int) -> List<Int> {
  @examples {
    len(build_series(0)) => 0
    len(build_series(3)) => 3
  }
  @effect alloc
  @effect mut_state
  out := []
  i := 0
  while i < n {
    out.append(i)
    i = i + 1
  }
  out
}

pub main() -> Int {
  @effect io
  println("contract-append-main")
  0
}
"#,
    );
    let out = run_vibe(&["test", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "vibe test failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("examples=2 passed=2 failed=0"),
        "unexpected test summary:\n{}",
        out.stdout
    );
}

#[test]
fn run_reports_entrypoint_diagnostic_for_non_main_file() {
    let source = temp_source_file(
        "vibe_module_helper",
        r#"
module demo.math

pub add(a: Int, b: Int) -> Int {
  a + b
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "run unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr
            .contains("requires an entry source file that defines `main`"),
        "missing friendly non-entry diagnostic:\n{}",
        out.stderr
    );
}

#[test]
fn string_len_list_dispatch_and_map_int_int_dispatch_work() {
    let source = temp_source_file(
        "vibe_dispatch_parity",
        r#"
pub main() -> Int {
  @effect io
  @effect alloc
  @effect mut_state
  s := "abcd"
  xs := [10, 20, 30]
  xs.set(1, 99)
  m := {1: 10, 2: 20}
  m.set(3, 30)
  if s.len() == 4 {
    if xs.get(1) == 99 {
      if m.get(3) == 30 {
        println("dispatch-parity-ok")
      } else {
        println("dispatch-parity-bad-map")
      }
    } else {
      println("dispatch-parity-bad-list")
    }
  } else {
    println("dispatch-parity-bad-str")
  }
  0
}
"#,
    );
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "dispatch-parity-ok\n");
}

#[test]
fn deterministic_build_binary_and_metadata() {
    let source = temp_fixture_copy("build/hello_world.vibe");
    let source_str = source.to_str().expect("source path str");

    let first = run_vibe(&["build", source_str]);
    assert!(
        first.status.success(),
        "first build failed:\nstdout:\n{}\nstderr:\n{}",
        first.stdout,
        first.stderr
    );
    let first_bin = fs::read(artifact_binary_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read first binary");
    let first_debug_map = fs::read_to_string(artifact_debug_map_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read first debug map");

    let second = run_vibe(&["build", source_str]);
    assert!(
        second.status.success(),
        "second build failed:\nstdout:\n{}\nstderr:\n{}",
        second.stdout,
        second.stderr
    );
    let second_bin = fs::read(artifact_binary_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read second binary");
    let second_debug_map = fs::read_to_string(artifact_debug_map_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read second debug map");

    assert_eq!(first_bin, second_bin, "binary output must be deterministic");
    assert_eq!(
        first_debug_map, second_debug_map,
        "debug map output must be deterministic"
    );
    assert_eq!(
        first.stdout, second.stdout,
        "build metadata output should be stable"
    );
}

#[test]
fn vibe_test_runs_contract_examples() {
    let source = temp_fixture_copy("contract_ok/topk_contracts.vibe");
    let out = run_vibe(&["test", source.to_str().expect("source path str")]);
    assert!(
        out.status.success(),
        "vibe test failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("examples=2 passed=2 failed=0"),
        "unexpected test summary:\n{}",
        out.stdout
    );
}

#[test]
fn vibe_test_enforces_contract_runtime_checks_by_default() {
    let source = temp_fixture_copy("build/contract_runtime_require.vibe");
    let out = run_vibe(&["test", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "vibe test unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("failed=1"),
        "expected one failing example in summary:\n{}",
        out.stdout
    );
    assert!(
        out.stderr.contains("contract @require failed"),
        "expected require failure details:\n{}",
        out.stderr
    );
}

#[test]
fn vibe_test_can_disable_contract_runtime_checks_with_env_override() {
    let source = temp_fixture_copy("build/contract_runtime_require.vibe");
    let out = run_vibe_with_env(
        &["test", source.to_str().expect("source path str")],
        &[("VIBE_CONTRACT_CHECKS", "off")],
    );
    assert!(
        out.status.success(),
        "vibe test should pass when runtime contract checks are disabled:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("failed=0"),
        "expected no failing examples when checks are disabled:\n{}",
        out.stdout
    );
}

#[test]
fn build_fails_when_contract_examples_fail_in_preflight() {
    let source = temp_fixture_copy("build/contract_runtime_preflight_fail.vibe");
    let out = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "build unexpectedly succeeded despite failing contract examples:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr.contains("contract/example preflight failed"),
        "expected contract preflight failure:\n{}",
        out.stderr
    );
}

#[test]
fn build_can_skip_contract_preflight_with_env_override() {
    let source = temp_fixture_copy("build/contract_runtime_preflight_fail.vibe");
    let out = run_vibe_with_env(
        &["build", source.to_str().expect("source path str")],
        &[("VIBE_CONTRACT_CHECKS", "off")],
    );
    assert!(
        out.status.success(),
        "build should pass when contract preflight is disabled:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
}

#[test]
fn build_accepts_debuginfo_flag_and_writes_metadata() {
    let source = temp_fixture_copy("build/hello_world.vibe");
    let source_str = source.to_str().expect("source path str");
    let out = run_vibe(&["build", source_str, "--debuginfo", "full"]);
    assert!(
        out.status.success(),
        "build failed with debuginfo=full:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    let debug_map = fs::read_to_string(artifact_debug_map_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read debug map");
    assert!(
        debug_map.contains("debuginfo=full"),
        "debug map should record debuginfo level:\n{debug_map}"
    );
}

#[test]
fn debug_map_source_path_is_project_relative_and_stable() {
    let (_project_root, source) = temp_project_source("vibe_debug_map_source_stable");
    let source_str = source.to_str().expect("source path str");
    let out = run_vibe(&["build", source_str]);
    assert!(
        out.status.success(),
        "build failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    let debug_map = fs::read_to_string(artifact_debug_map_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read debug map");
    assert!(
        debug_map.contains("source=project://main.yb"),
        "debug map should emit project-relative source entry:\n{debug_map}"
    );
    assert!(
        !debug_map.contains(source_str),
        "debug map should not embed machine-specific absolute source path:\n{debug_map}"
    );
}

#[test]
fn build_emits_unsafe_audit_and_allocation_profile_artifacts() {
    let source = temp_source_file(
        "vibe_unsafe_audit_ok",
        r#"
pub main() -> Int {
  @effect io
  @effect alloc
  // @unsafe begin: ffi pointer interop for raw syscall envelope
  // @unsafe review: SEC-2026-0007
  // @unsafe end
  payload := []
  println("unsafe-audit-ok")
  payload.len()
}
"#,
    );
    let source_str = source.to_str().expect("source path str");
    let out = run_vibe(&["build", source_str]);
    assert!(
        out.status.success(),
        "build failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );

    let unsafe_audit = fs::read_to_string(artifact_unsafe_audit_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read unsafe audit");
    assert!(
        unsafe_audit.contains("\"format\": \"vibe-unsafe-audit-v1\""),
        "unsafe audit format marker missing:\n{unsafe_audit}"
    );
    assert!(
        unsafe_audit.contains("SEC-2026-0007"),
        "unsafe audit should include review reference:\n{unsafe_audit}"
    );

    let alloc_profile = fs::read_to_string(artifact_alloc_profile_path(
        &source,
        "dev",
        "x86_64-unknown-linux-gnu",
    ))
    .expect("read allocation profile");
    assert!(
        alloc_profile.contains("\"format\": \"vibe-alloc-profile-v1\""),
        "allocation profile format marker missing:\n{alloc_profile}"
    );
    assert!(
        alloc_profile.contains("\"alloc_observed\": true"),
        "allocation profile should report observed alloc effect:\n{alloc_profile}"
    );
}

#[test]
fn build_rejects_unsafe_blocks_without_review_reference() {
    let source = temp_source_file(
        "vibe_unsafe_audit_missing_review",
        r#"
pub main() -> Int {
  @effect io
  // @unsafe begin: low-level fd mutation
  // @unsafe end
  println("unsafe-audit-fail")
  0
}
"#,
    );
    let source_str = source.to_str().expect("source path str");
    let out = run_vibe(&["build", source_str]);
    assert!(
        !out.status.success(),
        "build unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr.contains("unsafe audit found"),
        "expected unsafe audit failure diagnostics:\n{}",
        out.stderr
    );
}

#[test]
fn build_locked_requires_lockfile_when_manifest_exists() {
    let (project_root, source) = temp_project_source("vibe_locked_missing_lock");
    let out = run_vibe(&[
        "build",
        source.to_str().expect("source path str"),
        "--locked",
    ]);
    assert!(
        !out.status.success(),
        "build unexpectedly succeeded without lockfile:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr.contains("`--locked` requires lockfile"),
        "expected missing lockfile diagnostic:\n{}",
        out.stderr
    );
    assert!(
        out.stderr
            .contains(project_root.to_str().expect("project root str")),
        "expected project root path in diagnostic:\n{}",
        out.stderr
    );
}

#[test]
fn build_locked_succeeds_with_lockfile_when_manifest_exists() {
    let (project_root, source) = temp_project_source("vibe_locked_with_lock");
    fs::write(project_root.join("vibe.lock"), "version = 1\n").expect("write lockfile");
    let out = run_vibe(&[
        "build",
        source.to_str().expect("source path str"),
        "--locked",
    ]);
    assert!(
        out.status.success(),
        "build failed with lockfile present:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
}

#[test]
fn run_locked_requires_lockfile_when_manifest_exists() {
    let (_project_root, source) = temp_project_source("vibe_run_locked_missing_lock");
    let out = run_vibe(&["run", source.to_str().expect("source path str"), "--locked"]);
    assert!(
        !out.status.success(),
        "run unexpectedly succeeded without lockfile:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr.contains("`--locked` requires lockfile"),
        "expected missing lockfile diagnostic:\n{}",
        out.stderr
    );
}

#[test]
fn run_locked_succeeds_with_lockfile_when_manifest_exists() {
    let (project_root, source) = temp_project_source("vibe_run_locked_with_lock");
    fs::write(project_root.join("vibe.lock"), "version = 1\n").expect("write lockfile");
    let out = run_vibe(&["run", source.to_str().expect("source path str"), "--locked"]);
    assert!(
        out.status.success(),
        "run failed with lockfile present:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "locked\n");
}

#[test]
fn test_locked_requires_lockfile_when_manifest_exists() {
    let (_project_root, source) = temp_project_source("vibe_test_locked_missing_lock");
    let out = run_vibe(&[
        "test",
        source.to_str().expect("source path str"),
        "--locked",
    ]);
    assert!(
        !out.status.success(),
        "test unexpectedly succeeded without lockfile:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr.contains("`--locked` requires lockfile"),
        "expected missing lockfile diagnostic:\n{}",
        out.stderr
    );
}

#[test]
fn test_locked_succeeds_with_lockfile_when_manifest_exists() {
    let (project_root, source) = temp_project_source("vibe_test_locked_with_lock");
    fs::write(project_root.join("vibe.lock"), "version = 1\n").expect("write lockfile");
    let out = run_vibe(&[
        "test",
        source.to_str().expect("source path str"),
        "--locked",
    ]);
    assert!(
        out.status.success(),
        "test failed with lockfile present:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("main_failures=0"),
        "expected passing main execution summary:\n{}",
        out.stdout
    );
}

#[test]
fn run_enforces_native_require_contracts() {
    let source = temp_source_file(
        "native_require_fail",
        r#"
check_positive(x: Int) -> Int {
  @require x > 0
  @ensure . > 0
  x
}

pub main() -> Int {
  check_positive(0)
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "run unexpectedly succeeded despite @require violation:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr
            .contains("contract @require failed in native execution"),
        "expected native require failure marker:\n{}",
        out.stderr
    );
}

#[test]
fn produced_binary_enforces_native_ensure_contracts() {
    let source = temp_source_file(
        "native_ensure_fail",
        r#"
break_ensure(x: Int) -> Int {
  @ensure . > x
  x
}

pub main() -> Int {
  break_ensure(5)
}
"#,
    );
    let build = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        build.status.success(),
        "build failed:\nstdout:\n{}\nstderr:\n{}",
        build.stdout,
        build.stderr
    );
    let binary = artifact_binary_path(&source, "dev", "x86_64-unknown-linux-gnu");
    let run = Command::new(&binary)
        .current_dir(workspace_root())
        .output()
        .expect("run built binary");
    assert!(
        !run.status.success(),
        "built binary unexpectedly succeeded despite @ensure violation:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("contract @ensure failed in native execution"),
        "expected native ensure failure marker:\n{}",
        stderr
    );
}

#[test]
fn while_loop_fixture_runs() {
    let source = temp_fixture_copy("build/while_loop.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "");
}

#[test]
fn repeat_loop_fixture_runs() {
    let source = temp_fixture_copy("build/repeat_loop.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "");
}

#[test]
fn while_loop_supports_break_and_continue() {
    let source = temp_source_file(
        "while_break_continue",
        r#"
pub main() -> Int {
  @effect io
  i := 0
  sum := 0
  while i < 6 {
    i = i + 1
    if i == 2 {
      continue
    }
    if i == 5 {
      break
    }
    sum = sum + i
  }
  println(json.stringify_i64(sum))
  return 0
}
"#,
    );
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "8\n");
}

#[test]
fn runtime_regex_namespace_supports_count_and_replace_all() {
    let source = temp_source_file(
        "runtime_regex_namespace",
        r#"
pub main() -> Int {
  @effect io
  raw := ">ONE\nagggtaaa\n>TWO\ntttaccct\n"
  seq := regex.replace_all(raw, ">.*\n|\n", "")
  println(seq)
  println(json.stringify_i64(regex.count(seq, "agggtaaa|tttaccct")))
  println(regex.replace_all("aNDWaS", "aND|caN|Ha[DS]|WaS", "<3>"))
  return 0
}
"#,
    );
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "agggtaaatttaccct\n2\n<3><3>\n");
}

#[test]
fn phase11_for_iteration_fixture_is_deterministic() {
    let source = temp_fixture_copy("build/phase11_for_iter_deterministic.vibe");
    let first = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        first.status.success(),
        "first run failed:\nstdout:\n{}\nstderr:\n{}",
        first.stdout,
        first.stderr
    );
    let second = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        second.status.success(),
        "second run failed:\nstdout:\n{}\nstderr:\n{}",
        second.stdout,
        second.stderr
    );
    assert_eq!(
        first.stdout, second.stdout,
        "for-in output must be deterministic across runs"
    );
    assert_eq!(first.stdout, "x\ny\nlist-for-ok\nmap-for-ok\n");
}

#[test]
fn phase11_str_index_slice_unicode_fixture_runs() {
    let source = temp_fixture_copy("build/phase11_str_index_slice_unicode.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "V\nπ\n😊\nZ\nbyte-ok\n");
}

#[test]
fn phase11_str_index_rejects_non_utf8_boundary() {
    let source = temp_source_file(
        "phase11_str_index_non_boundary",
        r#"
pub main() -> Int {
  text := "Vπ😊Z"
  mid := text[2]
  mid
}
"#,
    );
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        !run.status.success(),
        "run unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert!(
        run.stderr.contains("UTF-8 boundary"),
        "expected UTF-8 boundary failure:\n{}",
        run.stderr
    );
}

#[test]
fn phase11_container_equality_fixture_runs() {
    let source = temp_fixture_copy("build/phase11_container_equality.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "list-eq-ok\nmap-eq-ok\nstr-eq-ok\n");
}

#[test]
fn phase11_async_await_and_thread_fixture_runs() {
    let source = temp_fixture_copy("build/phase11_async_thread_basic.vibe");
    let run = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        run.status.success(),
        "run failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "async-await-basic-ok\n");
}

#[test]
fn phase11_async_requires_call_expression() {
    let source = temp_source_file(
        "phase11_async_requires_call",
        r#"
pub main() -> Int {
  @effect concurrency
  token := async 42
  await token
}
"#,
    );
    let out = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "build unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr.contains("`async` expects a call expression"),
        "expected async call-shape diagnostic:\n{}",
        out.stderr
    );
}

#[test]
fn phase11_thread_sendability_blocks_member_capture() {
    let source = temp_source_file(
        "phase11_thread_sendability",
        r#"
consume(x: Int) -> Int {
  x
}

pub main() -> Int {
  @effect concurrency
  values := {"k": 1}
  thread consume(values.get("k"))
  0
}
"#,
    );
    let out = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "build unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr
            .contains("capturing member access in `go` may alias shared mutable state"),
        "expected sendability/member-capture diagnostic:\n{}",
        out.stderr
    );
}

#[test]
fn phase11_async_failure_propagates_through_await() {
    let source = temp_source_file(
        "phase11_async_failure_propagation",
        r#"
must_be_positive(x: Int) -> Int {
  @require x > 0
  x
}

pub main() -> Int {
  @effect concurrency
  task := async must_be_positive(0)
  await task
}
"#,
    );
    let out = run_vibe(&["run", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "run unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr
            .contains("contract @require failed in native execution"),
        "expected native contract failure propagation:\n{}",
        out.stderr
    );
}

#[test]
fn phase11_module_import_resolution_runs_multi_file_project() {
    let project = temp_module_project(
        "phase11_module_import_ok",
        &[
            (
                "demo/main.yb",
                r#"
module demo.main
import demo.math

pub main() -> Int {
  @effect io
  sum := add(1, 2)
  if sum == 3 {
    println("module-ok")
  }
  0
}
"#,
            ),
            (
                "demo/math.yb",
                r#"
module demo.math

pub add(a: Int, b: Int) -> Int {
  a + b
}
"#,
            ),
        ],
    );
    let entry = project.join("demo").join("main.yb");
    let out = run_vibe(&["run", entry.to_str().expect("entry path str")]);
    assert!(
        out.status.success(),
        "multi-module run failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(out.stdout, "module-ok\n");
}

#[test]
fn phase11_module_visibility_diagnostic_blocks_private_import() {
    let project = temp_module_project(
        "phase11_module_private_visibility",
        &[
            (
                "demo/main.yb",
                r#"
module demo.main
import demo.secret

pub main() -> Int {
  hidden(1)
}
"#,
            ),
            (
                "demo/secret.yb",
                r#"
module demo.secret

hidden(x: Int) -> Int {
  x
}
"#,
            ),
        ],
    );
    let entry = project.join("demo").join("main.yb");
    let out = run_vibe(&["check", entry.to_str().expect("entry path str")]);
    assert!(
        !out.status.success(),
        "check unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("is private; mark it `pub` to import"),
        "expected private visibility diagnostic:\n{}",
        out.stdout
    );
}

#[test]
fn phase11_module_cycle_diagnostic_is_reported() {
    let project = temp_module_project(
        "phase11_module_cycle",
        &[
            (
                "demo/main.yb",
                r#"
module demo.main
import demo.a

pub main() -> Int {
  a()
}
"#,
            ),
            (
                "demo/a.yb",
                r#"
module demo.a
import demo.b

pub a() -> Int {
  b()
}
"#,
            ),
            (
                "demo/b.yb",
                r#"
module demo.b
import demo.a

pub b() -> Int {
  1
}
"#,
            ),
        ],
    );
    let entry = project.join("demo").join("main.yb");
    let out = run_vibe(&["check", entry.to_str().expect("entry path str")]);
    assert!(
        !out.status.success(),
        "check unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("import cycle detected"),
        "expected cycle diagnostic:\n{}",
        out.stdout
    );
}

#[test]
fn phase11_module_cross_package_import_is_rejected() {
    let project = temp_module_project(
        "phase11_module_cross_package",
        &[
            (
                "alpha/main.yb",
                r#"
module alpha.main
import beta.util

pub main() -> Int {
  util()
}
"#,
            ),
            (
                "beta/util.yb",
                r#"
module beta.util

pub util() -> Int {
  0
}
"#,
            ),
        ],
    );
    let entry = project.join("alpha").join("main.yb");
    let out = run_vibe(&["check", entry.to_str().expect("entry path str")]);
    assert!(
        !out.status.success(),
        "check unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout
            .contains("cross-package import `beta.util` is not allowed"),
        "expected cross-package boundary diagnostic:\n{}",
        out.stdout
    );
}

#[test]
fn phase11_module_layout_diagnostic_is_reported() {
    let project = temp_module_project(
        "phase11_module_layout",
        &[
            (
                "demo/main.yb",
                r#"
module demo.main
import demo.util

pub main() -> Int {
  util()
}
"#,
            ),
            (
                "demo/wrong.yb",
                r#"
module demo.util

pub util() -> Int {
  0
}
"#,
            ),
        ],
    );
    let entry = project.join("demo").join("main.yb");
    let out = run_vibe(&["check", entry.to_str().expect("entry path str")]);
    assert!(
        !out.status.success(),
        "check unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout
            .contains("does not match file layout `demo.wrong`"),
        "expected package layout diagnostic:\n{}",
        out.stdout
    );
}

#[test]
fn unsupported_member_access_has_stable_codegen_diagnostic() {
    let source = temp_fixture_copy("build_err/member_access_unsupported.vibe");
    let out = run_vibe(&["build", source.to_str().expect("source path str")]);
    assert!(
        !out.status.success(),
        "build unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    let expected = fs::read_to_string(fixture_path("build_err/member_access_unsupported.diag"))
        .expect("read build_err golden");
    assert_eq!(
        expected.trim(),
        out.stderr.trim(),
        "build error output mismatch"
    );
}

#[test]
fn vibe_test_directory_accepts_mixed_extensions_when_stems_differ() {
    let temp_dir = unique_temp_dir("vibe_phase2_mixed_ext_ok");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    fs::write(temp_dir.join("alpha.vibe"), "alpha() -> Int { 0 }\n").expect("write alpha");
    fs::write(temp_dir.join("beta.yb"), "beta() -> Int { 0 }\n").expect("write beta");

    let out = run_vibe(&["test", temp_dir.to_str().expect("temp dir str")]);
    assert!(
        out.status.success(),
        "vibe test should accept mixed source extensions:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
}

#[test]
fn vibe_test_rejects_same_stem_across_extensions() {
    let temp_dir = unique_temp_dir("vibe_phase2_mixed_ext_conflict");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    fs::write(temp_dir.join("same.vibe"), "same() -> Int { 0 }\n").expect("write same vibe");
    fs::write(temp_dir.join("same.yb"), "same() -> Int { 0 }\n").expect("write same yb");

    let out = run_vibe(&["test", temp_dir.to_str().expect("temp dir str")]);
    assert!(
        !out.status.success(),
        "vibe test unexpectedly succeeded with conflicting stems:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr
            .contains("conflicting source files share a stem across extensions"),
        "expected collision guard diagnostic:\n{}",
        out.stderr
    );
}

fn temp_fixture_copy(relative: &str) -> PathBuf {
    let src = fixture_path(relative);
    let contents = fs::read_to_string(&src).expect("read fixture source");
    let file_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .expect("fixture file name");
    let temp_dir = unique_temp_dir("vibe_phase2_native");
    fs::create_dir_all(&temp_dir).expect("create temp dir");
    let dst = temp_dir.join(file_name);
    fs::write(&dst, contents).expect("write temp fixture");
    dst
}

fn temp_project_source(prefix: &str) -> (PathBuf, PathBuf) {
    let project_root = unique_temp_dir(prefix);
    fs::create_dir_all(&project_root).expect("create project root");
    fs::write(
        project_root.join("vibe.toml"),
        "[package]\nname = \"locked-test\"\nversion = \"0.1.0\"\n\n[dependencies]\n",
    )
    .expect("write manifest");
    let source = project_root.join("main.yb");
    fs::write(
        &source,
        "pub main() -> Int {\n  @effect io\n  println(\"locked\")\n  0\n}\n",
    )
    .expect("write source");
    (project_root, source)
}

fn temp_source_file(prefix: &str, source: &str) -> PathBuf {
    let temp_dir = unique_temp_dir(prefix);
    fs::create_dir_all(&temp_dir).expect("create temp source dir");
    let file = temp_dir.join("main.yb");
    fs::write(&file, source.trim_start()).expect("write temp source");
    file
}

fn temp_module_project(prefix: &str, files: &[(&str, &str)]) -> PathBuf {
    let project_root = unique_temp_dir(prefix);
    fs::create_dir_all(&project_root).expect("create module project root");
    fs::write(
        project_root.join("vibe.toml"),
        "[package]\nname = \"module-test\"\nversion = \"0.1.0\"\n\n[dependencies]\n",
    )
    .expect("write module project manifest");
    for (relative, source) in files {
        let path = project_root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create module source parent");
        }
        fs::write(&path, source.trim_start()).expect("write module source");
    }
    project_root
}

fn temp_fixture_copy_with_extension(relative: &str, ext: &str) -> PathBuf {
    let src = temp_fixture_copy(relative);
    let dst = src.with_extension(ext);
    fs::rename(&src, &dst).expect("rename temp fixture extension");
    dst
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos))
}

fn run_vibe(args: &[&str]) -> CmdOutput {
    run_vibe_with_env(args, &[])
}

fn run_vibe_with_env(args: &[&str], envs: &[(&str, &str)]) -> CmdOutput {
    let mut command = Command::new(vibe_bin());
    command.args(args).current_dir(workspace_root());
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output().expect("run vibe command");
    CmdOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn vibe_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vibe")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve workspace root")
}

fn fixture_path(relative: &str) -> PathBuf {
    workspace_root()
        .join("compiler/tests/fixtures")
        .join(relative)
}

fn artifact_binary_path(source: &Path, profile: &str, target: &str) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("source stem");
    source
        .parent()
        .expect("source parent")
        .join(".yb")
        .join("artifacts")
        .join(profile)
        .join(target)
        .join(stem)
}

fn artifact_debug_map_path(source: &Path, profile: &str, target: &str) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("source stem");
    source
        .parent()
        .expect("source parent")
        .join(".yb")
        .join("artifacts")
        .join(profile)
        .join(target)
        .join(format!("{stem}.debug.map"))
}

fn artifact_unsafe_audit_path(source: &Path, profile: &str, target: &str) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("source stem");
    source
        .parent()
        .expect("source parent")
        .join(".yb")
        .join("artifacts")
        .join(profile)
        .join(target)
        .join(format!("{stem}.unsafe.audit.json"))
}

fn artifact_alloc_profile_path(source: &Path, profile: &str, target: &str) -> PathBuf {
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("source stem");
    source
        .parent()
        .expect("source parent")
        .join(".yb")
        .join("artifacts")
        .join(profile)
        .join(target)
        .join(format!("{stem}.alloc.profile.json"))
}

struct CmdOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}
