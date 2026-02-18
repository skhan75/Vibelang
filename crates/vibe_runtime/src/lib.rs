use std::path::{Path, PathBuf};
use std::process::Command;

pub const RUNTIME_C_SOURCE: &str = include_str!("../../../runtime/native/vibe_runtime.c");

#[derive(Debug, Clone)]
pub struct RuntimeBuildOptions {
    pub target: String,
    pub profile: String,
}

impl Default for RuntimeBuildOptions {
    fn default() -> Self {
        Self {
            target: "x86_64-unknown-linux-gnu".to_string(),
            profile: "dev".to_string(),
        }
    }
}

pub fn runtime_source_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../runtime/native/vibe_runtime.c")
}

pub fn compile_runtime_object(
    output_dir: &Path,
    options: &RuntimeBuildOptions,
) -> Result<PathBuf, String> {
    ensure_supported_target(&options.target)?;
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("failed to create runtime output directory: {e}"))?;
    let out_obj = output_dir.join("vibe_runtime.o");
    let src = runtime_source_path();
    if !src.exists() {
        return Err(format!(
            "runtime source file not found at `{}`",
            src.display()
        ));
    }

    let mut cmd = Command::new("cc");
    cmd.arg("-c")
        .arg(&src)
        .arg("-o")
        .arg(&out_obj)
        .arg("-fno-ident")
        .arg("-ffunction-sections")
        .arg("-fdata-sections")
        .arg("-std=c11");

    if options.profile == "release" {
        cmd.arg("-O2");
    } else {
        cmd.arg("-O0");
    }

    if options.target == "x86_64-unknown-linux-gnu" {
        cmd.arg("-m64");
    }

    let output = cmd
        .output()
        .map_err(|e| format!("failed to execute C compiler: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "runtime compilation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(out_obj)
}

pub fn link_executable(
    object_file: &Path,
    runtime_object: &Path,
    output_binary: &Path,
    options: &RuntimeBuildOptions,
) -> Result<(), String> {
    ensure_supported_target(&options.target)?;
    if !object_file.exists() {
        return Err(format!(
            "object file does not exist: `{}`",
            object_file.display()
        ));
    }
    if !runtime_object.exists() {
        return Err(format!(
            "runtime object does not exist: `{}`",
            runtime_object.display()
        ));
    }
    if let Some(parent) = output_binary.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create output binary directory: {e}"))?;
    }

    let mut cmd = Command::new("cc");
    cmd.arg(object_file)
        .arg(runtime_object)
        .arg("-o")
        .arg(output_binary)
        .arg("-Wl,--build-id=none")
        .arg("-Wl,--gc-sections");
    if options.target == "x86_64-unknown-linux-gnu" {
        cmd.arg("-m64");
    }

    let output = cmd
        .output()
        .map_err(|e| format!("failed to execute linker: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "link failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub fn ensure_supported_target(target: &str) -> Result<(), String> {
    if target == "x86_64-unknown-linux-gnu" {
        return Ok(());
    }
    Err(format!(
        "unsupported target `{target}` in phase 2 (supported: x86_64-unknown-linux-gnu)"
    ))
}
