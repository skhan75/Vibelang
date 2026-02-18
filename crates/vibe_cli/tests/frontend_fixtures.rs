use std::fs;
use std::path::{Path, PathBuf};

use vibe_diagnostics::Diagnostics;
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
        &root.join("compiler/tests/snapshots/pipeline_sample.diag"),
        &diag_snapshot,
    );
}

#[test]
fn deterministic_repeat_runs_for_check_and_hir() {
    let sample = fixture_dir("snapshots").join("pipeline_sample.vibe");
    let src = fs::read_to_string(&sample).expect("read snapshot sample");

    let first = run_and_capture(&src);
    let second = run_and_capture(&src);
    assert_eq!(
        first.0, second.0,
        "diagnostics output must be deterministic"
    );
    assert_eq!(first.1, second.1, "HIR debug output must be deterministic");
}

fn run_and_capture(src: &str) -> (String, String) {
    let parsed = parse_source(src);
    let checked = check_and_lower(&parsed.ast);
    let mut all = Diagnostics::default();
    all.extend(parsed.diagnostics.into_sorted());
    all.extend(checked.diagnostics.into_sorted());
    (all.to_golden(), format!("{:#?}", checked.hir))
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
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("vibe"))
        .collect::<Vec<_>>();
    files.sort();
    files
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
