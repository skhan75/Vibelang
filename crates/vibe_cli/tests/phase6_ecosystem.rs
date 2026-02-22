use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn vibe_new_defaults_to_yb_and_scaffolds_app() {
    let root = unique_temp_dir("vibe_phase6_new_default");
    fs::create_dir_all(&root).expect("create temp root");
    let out = run_vibe(&[
        "new",
        "demo_app",
        "--path",
        root.to_str().expect("path str"),
    ]);
    assert!(
        out.status.success(),
        "vibe new failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );

    let project = root.join("demo_app");
    let main_file = project.join("main.yb");
    assert!(main_file.exists(), "expected main.yb scaffold");
    assert!(project.join("vibe.toml").exists(), "expected vibe.toml");

    let check = run_vibe(&["check", main_file.to_str().expect("main path str")]);
    assert!(
        check.status.success(),
        "scaffolded source should typecheck:\nstdout:\n{}\nstderr:\n{}",
        check.stdout,
        check.stderr
    );
    let run = run_vibe(&["run", main_file.to_str().expect("main path str")]);
    assert!(
        run.status.success(),
        "scaffolded source should run:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    let test = run_vibe(&["test", main_file.to_str().expect("main path str")]);
    assert!(
        test.status.success(),
        "scaffolded source should pass `vibe test`:\nstdout:\n{}\nstderr:\n{}",
        test.stdout,
        test.stderr
    );
}

#[test]
fn vibe_new_supports_legacy_extension_opt_in() {
    let root = unique_temp_dir("vibe_phase6_new_legacy");
    fs::create_dir_all(&root).expect("create temp root");
    let out = run_vibe(&[
        "new",
        "legacy_app",
        "--path",
        root.to_str().expect("path str"),
        "--ext",
        "vibe",
    ]);
    assert!(
        out.status.success(),
        "vibe new legacy ext failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(root.join("legacy_app").join("main.vibe").exists());
}

#[test]
fn vibe_new_service_template_scaffolds_multi_module_project() {
    let root = unique_temp_dir("vibe_phase11_new_service");
    fs::create_dir_all(&root).expect("create temp root");
    let out = run_vibe(&[
        "new",
        "demo_service",
        "--path",
        root.to_str().expect("path str"),
        "--service",
    ]);
    assert!(
        out.status.success(),
        "vibe new service failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );

    let project = root.join("demo_service");
    let entry = project.join("demo_service").join("main.yb");
    assert!(entry.exists(), "expected service main entry");
    assert!(
        project.join("demo_service").join("router.yb").exists(),
        "expected service router module"
    );
    let run = run_vibe(&["run", entry.to_str().expect("entry path str")]);
    assert!(
        run.status.success(),
        "service template should run:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "service starting\n");
}

#[test]
fn vibe_new_cli_template_scaffolds_multi_module_project() {
    let root = unique_temp_dir("vibe_phase11_new_cli");
    fs::create_dir_all(&root).expect("create temp root");
    let out = run_vibe(&[
        "new",
        "demo_cli",
        "--path",
        root.to_str().expect("path str"),
        "--cli",
    ]);
    assert!(
        out.status.success(),
        "vibe new cli failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );

    let project = root.join("demo_cli");
    let entry = project.join("demo_cli").join("main.yb");
    assert!(entry.exists(), "expected cli main entry");
    assert!(
        project.join("demo_cli").join("commands.yb").exists(),
        "expected cli commands module"
    );
    let run = run_vibe(&["run", entry.to_str().expect("entry path str")]);
    assert!(
        run.status.success(),
        "cli template should run:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert_eq!(run.stdout, "cli ready\n");
}

#[test]
fn vibe_new_library_template_scaffolds_lib_entry() {
    let root = unique_temp_dir("vibe_phase11_new_library");
    fs::create_dir_all(&root).expect("create temp root");
    let out = run_vibe(&[
        "new",
        "demo_lib",
        "--path",
        root.to_str().expect("path str"),
        "--lib",
    ]);
    assert!(
        out.status.success(),
        "vibe new lib failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    let lib = root.join("demo_lib").join("lib.yb");
    assert!(lib.exists(), "expected lib.yb scaffold");
    let check = run_vibe(&["check", lib.to_str().expect("lib path str")]);
    assert!(
        check.status.success(),
        "library scaffold should typecheck:\nstdout:\n{}\nstderr:\n{}",
        check.stdout,
        check.stderr
    );
}

#[test]
fn vibe_fmt_check_and_write_roundtrip() {
    let root = unique_temp_dir("vibe_phase6_fmt");
    fs::create_dir_all(&root).expect("create temp root");
    let source = root.join("fmt_case.yb");
    fs::write(
        &source,
        "pub main() -> Int {\n\tprintln(\"hi\")    \n\n\n  0\n}\n",
    )
    .expect("write source");

    let check_fail = run_vibe(&["fmt", source.to_str().expect("source str"), "--check"]);
    assert!(
        !check_fail.status.success(),
        "format check should fail before rewrite"
    );

    let write_ok = run_vibe(&["fmt", source.to_str().expect("source str")]);
    assert!(
        write_ok.status.success(),
        "format rewrite failed:\nstdout:\n{}\nstderr:\n{}",
        write_ok.stdout,
        write_ok.stderr
    );

    let check_ok = run_vibe(&["fmt", source.to_str().expect("source str"), "--check"]);
    assert!(
        check_ok.status.success(),
        "format check should pass after rewrite:\nstdout:\n{}\nstderr:\n{}",
        check_ok.stdout,
        check_ok.stderr
    );
}

#[test]
fn vibe_doc_generates_markdown_output() {
    let root = unique_temp_dir("vibe_phase6_doc");
    fs::create_dir_all(&root).expect("create temp root");
    let source = root.join("doc_case.yb");
    let out_path = root.join("api.md");
    fs::write(
        &source,
        r#"@intent "example function"
@effect io
pub main() -> Int {
  println("doc")
  0
}
"#,
    )
    .expect("write source");

    let out = run_vibe(&[
        "doc",
        source.to_str().expect("source str"),
        "--out",
        out_path.to_str().expect("out str"),
    ]);
    assert!(
        out.status.success(),
        "vibe doc failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    let docs = fs::read_to_string(&out_path).expect("read docs");
    assert!(docs.contains("VibeLang Generated Docs"));
    assert!(docs.contains("pub main() -> Int"));
    assert!(docs.contains("Intent: example function"));
}

#[test]
fn vibe_pkg_install_from_offline_mirror() {
    let root = unique_temp_dir("vibe_phase6_pkg");
    let project = root.join("project");
    let mirror = root.join("mirror");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&mirror).expect("create mirror");

    fs::write(
        project.join("vibe.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[dependencies]\nmath = \"^1.0.0\"\n",
    )
    .expect("write root manifest");

    write_mirror_pkg(&mirror, "math", "1.0.0", &[]);
    fs::write(
        mirror.join("math").join("1.0.0").join("lib.yb"),
        "pub inc(x: Int) -> Int { x + 1 }\n",
    )
    .expect("write mirror source");

    let out = run_vibe(&[
        "pkg",
        "install",
        "--path",
        project.to_str().expect("project str"),
        "--mirror",
        mirror.to_str().expect("mirror str"),
    ]);
    assert!(
        out.status.success(),
        "vibe pkg install failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(project.join("vibe.lock").exists(), "expected lockfile");
    assert!(
        project
            .join(".yb")
            .join("pkg")
            .join("store")
            .join("math")
            .join("1.0.0")
            .join("vibe.toml")
            .exists(),
        "expected installed package mirror copy"
    );
}

fn write_mirror_pkg(mirror_root: &Path, name: &str, version: &str, deps: &[(&str, &str)]) {
    let dir = mirror_root.join(name).join(version);
    fs::create_dir_all(&dir).expect("create mirror package dir");
    let mut manifest = String::new();
    manifest.push_str("[package]\n");
    manifest.push_str(&format!("name = \"{name}\"\n"));
    manifest.push_str(&format!("version = \"{version}\"\n"));
    if !deps.is_empty() {
        manifest.push_str("\n[dependencies]\n");
        for (dep, req) in deps {
            manifest.push_str(&format!("{dep} = \"{req}\"\n"));
        }
    }
    fs::write(dir.join("vibe.toml"), manifest).expect("write mirror manifest");
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos))
}

fn run_vibe(args: &[&str]) -> CmdOutput {
    let output = Command::new(vibe_bin())
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

fn vibe_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vibe")
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
