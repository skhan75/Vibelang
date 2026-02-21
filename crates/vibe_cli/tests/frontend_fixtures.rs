use std::fs;
use std::path::{Path, PathBuf};

use vibe_diagnostics::Diagnostics;
use vibe_indexer::is_supported_source_file;
use vibe_mir::{lower_hir_to_mir, mir_debug_dump};
use vibe_parser::parse_source;
use vibe_types::check_and_lower;

#[test]
fn parse_ok_fixtures() {
    let dir = fixture_dir("parse_ok");
    for file in list_vibe_files(&dir) {
        let src = fs::read_to_string(&file).expect("read parse_ok fixture");
        let parsed = parse_source(&src);
        assert!(
            !parsed.diagnostics.has_errors(),
            "parse_ok fixture failed: {}\n{}",
            file.display(),
            parsed.diagnostics.to_golden()
        );
    }
}

#[test]
fn parse_err_golden() {
    let dir = fixture_dir("parse_err");
    for file in list_vibe_files(&dir) {
        let src = fs::read_to_string(&file).expect("read parse_err fixture");
        let parsed = parse_source(&src);
        assert!(
            parsed.diagnostics.has_errors(),
            "parse_err fixture unexpectedly passed: {}",
            file.display()
        );
        let actual = parsed.diagnostics.to_golden();
        let expected = golden_path(&file, "diag");
        assert_golden(&expected, &actual);
    }
}

#[test]
fn type_ok_fixtures() {
    let dir = fixture_dir("type_ok");
    for file in list_vibe_files(&dir) {
        let src = fs::read_to_string(&file).expect("read type_ok fixture");
        let all = check_output(&src);
        assert!(
            !all.has_errors(),
            "type_ok fixture failed: {}\n{}",
            file.display(),
            all.to_golden()
        );
    }
}

#[test]
fn type_err_golden() {
    let dir = fixture_dir("type_err");
    for file in list_vibe_files(&dir) {
        let src = fs::read_to_string(&file).expect("read type_err fixture");
        let all = check_output(&src);
        assert!(
            all.has_errors(),
            "type_err fixture unexpectedly passed: {}",
            file.display()
        );
        let expected = golden_path(&file, "diag");
        assert_golden(&expected, &all.to_golden());
    }
}

#[test]
fn contract_ok_fixtures() {
    let dir = fixture_dir("contract_ok");
    for file in list_vibe_files(&dir) {
        let src = fs::read_to_string(&file).expect("read contract_ok fixture");
        let all = check_output(&src);
        assert!(
            !all.has_errors(),
            "contract_ok fixture failed: {}\n{}",
            file.display(),
            all.to_golden()
        );
    }
}

#[test]
fn contract_err_golden() {
    let dir = fixture_dir("contract_err");
    for file in list_vibe_files(&dir) {
        let src = fs::read_to_string(&file).expect("read contract_err fixture");
        let all = check_output(&src);
        assert!(
            all.has_errors(),
            "contract_err fixture unexpectedly passed: {}",
            file.display()
        );
        let expected = golden_path(&file, "diag");
        assert_golden(&expected, &all.to_golden());
    }
}

#[test]
fn ownership_err_golden() {
    let dir = fixture_dir("ownership_err");
    for file in list_vibe_files(&dir) {
        let src = fs::read_to_string(&file).expect("read ownership_err fixture");
        let all = check_output(&src);
        assert!(
            all.has_errors(),
            "ownership_err fixture unexpectedly passed: {}",
            file.display()
        );
        let expected = golden_path(&file, "diag");
        assert_golden(&expected, &all.to_golden());
    }
}

#[test]
fn effect_ok_fixtures() {
    let dir = fixture_dir("effect_ok");
    for file in list_vibe_files(&dir) {
        let src = fs::read_to_string(&file).expect("read effect_ok fixture");
        let all = check_output(&src);
        assert!(
            !all.has_errors(),
            "effect_ok fixture failed: {}\n{}",
            file.display(),
            all.to_golden()
        );
    }
}

#[test]
fn effect_err_golden() {
    let dir = fixture_dir("effect_err");
    for file in list_vibe_files(&dir) {
        let src = fs::read_to_string(&file).expect("read effect_err fixture");
        let all = check_output(&src);
        let expected = golden_path(&file, "diag");
        assert_golden(&expected, &all.to_golden());
    }
}

#[test]
fn concurrency_ok_fixtures() {
    let dir = fixture_dir("concurrency_ok");
    for file in list_vibe_files(&dir) {
        let src = fs::read_to_string(&file).expect("read concurrency_ok fixture");
        let all = check_output(&src);
        assert!(
            !all.has_errors(),
            "concurrency_ok fixture failed: {}\n{}",
            file.display(),
            all.to_golden()
        );
    }
}

#[test]
fn concurrency_err_golden() {
    let dir = fixture_dir("concurrency_err");
    for file in list_vibe_files(&dir) {
        let src = fs::read_to_string(&file).expect("read concurrency_err fixture");
        let all = check_output(&src);
        let expected = golden_path(&file, "diag");
        assert_golden(&expected, &all.to_golden());
    }
}

#[test]
fn snapshots_ast_hir_and_diag() {
    let sample = fixture_dir("snapshots").join("pipeline_sample.vibe");
    let src = fs::read_to_string(&sample).expect("read snapshot sample");
    let parsed = parse_source(&src);
    let checked = check_and_lower(&parsed.ast);

    let mut all = Diagnostics::default();
    all.extend(parsed.diagnostics.into_sorted());
    all.extend(checked.diagnostics.into_sorted());

    let ast_snapshot = format!("{:#?}\n", parsed.ast);
    let hir_snapshot = format!("{:#?}\n", checked.hir);
    let mir_snapshot = match lower_hir_to_mir(&checked.hir) {
        Ok(mir) => mir_debug_dump(&mir),
        Err(err) => format!("MIR lowering failed: {err}\n"),
    };
    let diag_snapshot = all.to_golden();

    let root = workspace_root();
    assert_golden(
        &root.join("compiler/tests/snapshots/pipeline_sample.ast"),
        &ast_snapshot,
    );
    assert_golden(
        &root.join("compiler/tests/snapshots/pipeline_sample.hir"),
        &hir_snapshot,
    );
    assert_golden(
        &root.join("compiler/tests/snapshots/pipeline_sample.mir"),
        &mir_snapshot,
    );
    assert_golden(
        &root.join("compiler/tests/snapshots/pipeline_sample.diag"),
        &diag_snapshot,
    );
}

#[test]
fn deterministic_repeat_runs_for_check_hir_and_mir() {
    let sample = fixture_dir("snapshots").join("pipeline_sample.vibe");
    let src = fs::read_to_string(&sample).expect("read snapshot sample");

    let first = run_and_capture(&src);
    let second = run_and_capture(&src);
    assert_eq!(
        first.0, second.0,
        "diagnostics output must be deterministic"
    );
    assert_eq!(first.1, second.1, "HIR debug output must be deterministic");
    assert_eq!(first.2, second.2, "MIR debug output must be deterministic");
}

#[test]
fn snapshots_container_ops_mir_is_deterministic() {
    let sample = fixture_dir("snapshots").join("container_ops_sample.yb");
    let src = fs::read_to_string(&sample).expect("read container snapshot sample");
    let first = run_and_capture(&src);
    let second = run_and_capture(&src);
    assert_eq!(
        first.0, second.0,
        "container diagnostics output must be deterministic"
    );
    assert_eq!(first.1, second.1, "container HIR output must be deterministic");
    assert_eq!(first.2, second.2, "container MIR output must be deterministic");
    assert!(
        first.2.contains("Map([") && first.2.contains("List(["),
        "container MIR sample should include explicit list/map forms:\n{}",
        first.2
    );
}

#[test]
fn docs_syntax_samples_compile_without_errors() {
    let sample = workspace_root().join("docs/spec/syntax_samples.yb");
    let src = fs::read_to_string(&sample).expect("read syntax samples");
    let all = check_output(&src);
    assert!(
        !all.has_errors(),
        "docs syntax samples should compile without errors:\n{}",
        all.to_golden()
    );
}

#[test]
fn phase7_basic_and_intermediate_matrix() {
    for level in ["basic", "intermediate"] {
        let dir = fixture_dir("phase7").join(level);
        for file in list_vibe_files_recursive(&dir) {
            let src = fs::read_to_string(&file).expect("read phase7 fixture");
            let all = check_output(&src);
            let expected = golden_path(&file, "diag");
            if expected.exists() {
                assert_golden(&expected, &all.to_golden());
            } else {
                assert!(
                    !all.has_errors(),
                    "phase7 fixture should compile without errors: {}\n{}",
                    file.display(),
                    all.to_golden()
                );
            }
        }
    }
}

#[test]
fn phase7_frontend_outputs_are_deterministic() {
    let first = collect_phase7_outputs();
    let second = collect_phase7_outputs();
    assert_eq!(first, second, "phase7 frontend output should be stable");
}

fn run_and_capture(src: &str) -> (String, String, String) {
    let parsed = parse_source(src);
    let checked = check_and_lower(&parsed.ast);
    let mut all = Diagnostics::default();
    all.extend(parsed.diagnostics.into_sorted());
    all.extend(checked.diagnostics.into_sorted());
    let mir = match lower_hir_to_mir(&checked.hir) {
        Ok(mir) => mir_debug_dump(&mir),
        Err(err) => format!("MIR lowering failed: {err}"),
    };
    (all.to_golden(), format!("{:#?}", checked.hir), mir)
}

fn check_output(src: &str) -> Diagnostics {
    let parsed = parse_source(src);
    let checked = check_and_lower(&parsed.ast);
    let mut all = Diagnostics::default();
    all.extend(parsed.diagnostics.into_sorted());
    all.extend(checked.diagnostics.into_sorted());
    all
}

fn list_vibe_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(dir)
        .expect("read fixtures directory")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| is_supported_source_file(p))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn list_vibe_files_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_vibe_files_recursive(dir, &mut out);
    out.sort();
    out
}

fn collect_vibe_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir)
        .expect("read fixtures directory recursively")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect::<Vec<_>>();
    for path in entries {
        if path.is_dir() {
            collect_vibe_files_recursive(&path, out);
        } else if is_supported_source_file(&path) {
            out.push(path);
        }
    }
}

fn collect_phase7_outputs() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for level in ["basic", "intermediate"] {
        let dir = fixture_dir("phase7").join(level);
        for file in list_vibe_files_recursive(&dir) {
            let src = fs::read_to_string(&file).expect("read phase7 fixture");
            let diags = check_output(&src).to_golden();
            out.push((file.display().to_string(), diags));
        }
    }
    out.sort();
    out
}

fn golden_path(src_path: &Path, ext: &str) -> PathBuf {
    src_path.with_extension(ext)
}

fn assert_golden(path: &Path, actual: &str) {
    if std::env::var("UPDATE_GOLDEN").as_deref() == Ok("1") {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create golden parent");
        }
        fs::write(path, actual).expect("write golden output");
        return;
    }
    let expected = fs::read_to_string(path).unwrap_or_else(|_| {
        panic!(
            "missing golden file: {} (run tests with UPDATE_GOLDEN=1 to generate)",
            path.display()
        )
    });
    assert_eq!(expected, actual, "golden mismatch at {}", path.display());
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("resolve workspace root")
}

fn fixture_dir(group: &str) -> PathBuf {
    workspace_root().join("compiler/tests/fixtures").join(group)
}
