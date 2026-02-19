use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn intent_lint_detects_good_match_vs_drift_cases() {
    let project = unique_temp_dir("vibe_phase7_intent_cases");
    fs::create_dir_all(&project).expect("create project dir");
    copy_fixture(
        "phase7/advanced/intent_validation/intent_validation__good_match.yb",
        project.join("good_match.yb"),
    );
    copy_fixture(
        "phase7/advanced/intent_validation/intent_validation__effect_drift.yb",
        project.join("effect_drift.yb"),
    );
    copy_fixture(
        "phase7/advanced/intent_validation/intent_validation__vague_text.yb",
        project.join("vague_text.yb"),
    );

    let out = run_vibe(&["lint", project.to_str().expect("project str"), "--intent"]);
    assert!(
        out.status.success(),
        "vibe lint failed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("I5003"),
        "expected effect drift finding in lint output:\n{}",
        out.stdout
    );
    assert!(
        out.stdout.contains("I5002"),
        "expected vague intent finding in lint output:\n{}",
        out.stdout
    );
    assert!(
        !out.stdout.contains("I5001"),
        "fixtures should not trigger missing intent:\n{}",
        out.stdout
    );
}

#[test]
fn intent_lint_changed_mode_supports_no_git_and_git_present_flows() {
    let no_git_project = unique_temp_dir("vibe_phase7_intent_no_git");
    fs::create_dir_all(&no_git_project).expect("create no-git project");
    fs::write(
        no_git_project.join("no_git_case.yb"),
        r#"
pub noGitCase(x: Int) -> Int {
  @intent "increment input deterministically"
  @examples {
    noGitCase(1) => 2
  }
  x + 1
}
"#
        .trim_start(),
    )
    .expect("write no git case");

    let first = run_vibe(&[
        "lint",
        no_git_project.to_str().expect("project str"),
        "--intent",
        "--changed",
    ]);
    assert!(first.status.success(), "initial changed lint should succeed");

    let second = run_vibe(&[
        "lint",
        no_git_project.to_str().expect("project str"),
        "--intent",
        "--changed",
    ]);
    assert!(
        second.stdout.contains("no changed source files"),
        "expected no changed sources in immediate rerun:\n{}",
        second.stdout
    );

    let source = no_git_project.join("no_git_case.yb");
    let mut changed = fs::read_to_string(&source).expect("read source");
    changed.push('\n');
    fs::write(&source, changed).expect("write changed source");
    let third = run_vibe(&[
        "lint",
        no_git_project.to_str().expect("project str"),
        "--intent",
        "--changed",
    ]);
    assert!(
        !third.stdout.contains("no changed source files"),
        "expected changed detection after source mutation:\n{}",
        third.stdout
    );

    let git_project = unique_temp_dir("vibe_phase7_intent_git");
    fs::create_dir_all(&git_project).expect("create git project");
    fs::write(
        git_project.join("git_case.yb"),
        r#"
pub gitCase(x: Int) -> Int {
  @intent "double an integer"
  @examples {
    gitCase(2) => 4
  }
  x * 2
}
"#
        .trim_start(),
    )
    .expect("write git case");
    run_git(
        &git_project,
        &[
            "init",
            "--initial-branch=main",
        ],
    );
    run_git(&git_project, &["add", "."]);
    run_git(
        &git_project,
        &[
            "-c",
            "user.name=phase7",
            "-c",
            "user.email=phase7@example.com",
            "commit",
            "-m",
            "baseline",
        ],
    );

    let stable = run_vibe(&[
        "lint",
        git_project.to_str().expect("project str"),
        "--intent",
        "--changed",
    ]);
    assert!(
        stable.stdout.contains("no changed source files"),
        "expected git-backed changed mode to report no changes:\n{}",
        stable.stdout
    );

    fs::write(
        git_project.join("git_case.yb"),
        r#"
pub gitCase(x: Int) -> Int {
  @intent "double an integer"
  @examples {
    gitCase(2) => 4
  }
  x * 2 + 1
}
"#
        .trim_start(),
    )
    .expect("mutate git file");

    let changed_git = run_vibe(&[
        "lint",
        git_project.to_str().expect("project str"),
        "--intent",
        "--changed",
    ]);
    assert!(
        !changed_git.stdout.contains("no changed source files"),
        "expected git-backed changed mode to include edited file:\n{}",
        changed_git.stdout
    );
}

#[test]
fn intent_lint_verifier_gate_rejects_invalid_suggestions() {
    let project = unique_temp_dir("vibe_phase7_intent_reject");
    fs::create_dir_all(&project).expect("create project");
    fs::write(
        project.join("reject_case.yb"),
        r#"
pub rejectCase(x: Int) -> Int {
  y = x + 1
  x
}
"#
        .trim_start(),
    )
    .expect("write reject fixture");

    let out = run_vibe(&[
        "lint",
        project.to_str().expect("project str"),
        "--intent",
        "--suggest",
    ]);
    assert!(
        out.status.success(),
        "lint with suggestions should succeed:\nstdout:\n{}\nstderr:\n{}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stdout.contains("I6001"),
        "expected verifier rejection finding:\n{}",
        out.stdout
    );
    assert!(
        !out.stdout.contains("verified=true"),
        "rejected suggestions should never be marked verified:\n{}",
        out.stdout
    );
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

fn run_git(repo: &PathBuf, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("run git command");
    assert!(
        output.status.success(),
        "git command failed:\nargs={:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn copy_fixture(relative: &str, destination: PathBuf) {
    let source = workspace_root()
        .join("compiler/tests/fixtures")
        .join(relative);
    let contents = fs::read_to_string(source).expect("read fixture");
    fs::write(destination, contents).expect("write copied fixture");
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos))
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
