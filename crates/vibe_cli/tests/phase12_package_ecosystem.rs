use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn phase12_pkg_publish_writes_registry_index() {
    let root = unique_temp_dir("vibe_phase12_pkg_publish");
    let project = root.join("project");
    let registry = root.join("registry");
    fs::create_dir_all(&project).expect("create project");
    fs::write(
        project.join("vibe.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[dependencies]\n",
    )
    .expect("write project manifest");
    fs::write(project.join("lib.yb"), "pub ping() -> Int { 1 }\n").expect("write source");

    let out = run_vibe(&[
        "pkg",
        "publish",
        "--path",
        project.to_str().expect("project str"),
        "--registry",
        registry.to_str().expect("registry str"),
    ]);
    assert!(
        out.status.success(),
        "publish failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        registry.join("index.toml").exists(),
        "expected registry index file"
    );
    let index = fs::read_to_string(registry.join("index.toml")).expect("read index");
    assert!(index.contains("name = \"demo\""));
    assert!(index.contains("version = \"0.1.0\""));
}

#[test]
fn phase12_pkg_audit_reports_policy_violations() {
    let root = unique_temp_dir("vibe_phase12_pkg_audit");
    let project = root.join("project");
    let mirror = root.join("mirror");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&mirror).expect("create mirror");
    fs::write(
        project.join("vibe.toml"),
        "[package]\nname = \"audit-demo\"\nversion = \"0.1.0\"\n\n[dependencies]\nmath = \"^1.0.0\"\n",
    )
    .expect("write project manifest");
    write_mirror_pkg(&mirror, "math", "1.0.0", "GPL-3.0");
    let policy = project.join("audit_policy.toml");
    fs::write(&policy, "[licenses]\ndeny = [\"GPL-3.0\"]\n").expect("write policy");
    let advisories = project.join("advisories.toml");
    fs::write(
        &advisories,
        "[[advisory]]\nid = \"VIBESEC-2026-0002\"\npackage = \"math\"\naffected = \"<2.0.0\"\nseverity = \"high\"\n",
    )
    .expect("write advisories");

    let out = run_vibe(&[
        "pkg",
        "audit",
        "--path",
        project.to_str().expect("project str"),
        "--mirror",
        mirror.to_str().expect("mirror str"),
        "--policy",
        policy.to_str().expect("policy str"),
        "--advisory-db",
        advisories.to_str().expect("advisories str"),
    ]);
    assert!(
        !out.status.success(),
        "audit unexpectedly succeeded:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(out.stdout.contains("findings=2"));
    assert!(out.stdout.contains("license"));
    assert!(out.stdout.contains("vulnerability"));
}

#[test]
fn phase12_pkg_upgrade_plan_reports_manifest_change_flag() {
    let root = unique_temp_dir("vibe_phase12_pkg_upgrade");
    let project = root.join("project");
    let mirror = root.join("mirror");
    fs::create_dir_all(&project).expect("create project");
    fs::create_dir_all(&mirror).expect("create mirror");
    fs::write(
        project.join("vibe.toml"),
        "[package]\nname = \"upgrade-demo\"\nversion = \"0.1.0\"\n\n[dependencies]\nutil = \"^1.0.0\"\n",
    )
    .expect("write project manifest");
    write_mirror_pkg(&mirror, "util", "1.2.0", "MIT");
    write_mirror_pkg(&mirror, "util", "2.0.0", "MIT");

    let out = run_vibe(&[
        "pkg",
        "upgrade-plan",
        "--path",
        project.to_str().expect("project str"),
        "--mirror",
        mirror.to_str().expect("mirror str"),
    ]);
    assert!(
        out.status.success(),
        "upgrade-plan failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(out.stdout.contains("manifest_change=yes"));
}

#[test]
fn phase12_pkg_semver_check_classifies_major() {
    let out = run_vibe(&[
        "pkg",
        "semver-check",
        "--current",
        "1.2.3",
        "--next",
        "2.0.0",
    ]);
    assert!(
        out.status.success(),
        "semver-check failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(out.stdout.contains("(major)"));
}

fn write_mirror_pkg(mirror_root: &Path, name: &str, version: &str, license: &str) {
    let dir = mirror_root.join(name).join(version);
    fs::create_dir_all(&dir).expect("create mirror package dir");
    let manifest = format!(
        "[package]\nname = \"{name}\"\nversion = \"{version}\"\nlicense = \"{license}\"\n"
    );
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
