// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};
use std::process::Command;

pub mod cancellation;
pub mod channel;
pub mod scheduler;
pub mod select;
pub mod task;

pub use cancellation::CancellationToken;
pub use channel::{BoundedChannel, RecvStatus, SendStatus};
pub use scheduler::{Scheduler, SchedulerHandle, SchedulerMetrics};
pub use select::{select_recv, SelectRecvStatus};
pub use task::{spawn_task, TaskHandle, TaskId};

pub const RUNTIME_C_SOURCE: &str = include_str!("../../../runtime/native/vibe_runtime.c");
#[cfg(feature = "bench-runtime")]
pub const BENCH_RUNTIME_C_SOURCE: &str =
    include_str!("../../../runtime/native/vibe_runtime_bench.c");
pub const SUPPORTED_TARGETS: &[&str] = &[
    "x86_64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
];

#[derive(Debug, Clone)]
pub struct RuntimeBuildOptions {
    pub target: String,
    pub profile: String,
    pub debuginfo: String,
}

impl Default for RuntimeBuildOptions {
    fn default() -> Self {
        Self {
            target: "x86_64-unknown-linux-gnu".to_string(),
            profile: "dev".to_string(),
            debuginfo: "line".to_string(),
        }
    }
}

pub fn runtime_source_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../runtime/native/vibe_runtime.c")
}

#[cfg(feature = "bench-runtime")]
pub fn bench_runtime_source_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../runtime/native/vibe_runtime_bench.c")
}

fn compile_runtime_c_object(
    out_obj: &Path,
    source_for_compile: &Path,
    stamp_path: &Path,
    stamp_tag: &str,
    options: &RuntimeBuildOptions,
) -> Result<(), String> {
    let current_stamp = format!(
        "tag={stamp_tag}\ntarget={}\nprofile={}\ndebuginfo={}\n",
        options.target, options.profile, options.debuginfo
    );
    if out_obj.exists() && stamp_path.exists() {
        let stamp_matches = std::fs::read_to_string(stamp_path)
            .map(|contents| contents == current_stamp)
            .unwrap_or(false);
        if stamp_matches {
            let src_meta = std::fs::metadata(source_for_compile).ok();
            let out_meta = std::fs::metadata(out_obj).ok();
            if let (Some(src_meta), Some(out_meta)) = (src_meta, out_meta) {
                if let (Ok(src_modified), Ok(out_modified)) =
                    (src_meta.modified(), out_meta.modified())
                {
                    if out_modified >= src_modified {
                        return Ok(());
                    }
                }
            }
        }
    }

    let mut cmd = Command::new("cc");
    cmd.arg("-c")
        .arg(source_for_compile)
        .arg("-o")
        .arg(out_obj)
        .arg("-fno-ident")
        .arg("-ffunction-sections")
        .arg("-fdata-sections")
        .arg("-std=c11");
    if !is_windows_target(&options.target) {
        cmd.arg("-pthread");
    }

    if options.profile == "release" {
        cmd.arg("-O2");
    } else {
        cmd.arg("-O0");
    }
    if options.debuginfo != "none" {
        cmd.arg("-g");
    } else {
        cmd.arg("-g0");
    }

    if let Some(flag) = cc_target_flag(&options.target) {
        cmd.arg(flag);
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
    let _ = std::fs::write(stamp_path, current_stamp);
    Ok(())
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
    let source_for_compile = if src.exists() {
        src
    } else {
        let embedded_src = output_dir.join("vibe_runtime_embedded.c");
        std::fs::write(&embedded_src, RUNTIME_C_SOURCE).map_err(|e| {
            format!(
                "failed to write embedded runtime source `{}`: {e}",
                embedded_src.display()
            )
        })?;
        embedded_src
    };
    let stamp_path = output_dir.join("vibe_runtime.build.stamp");
    compile_runtime_c_object(
        &out_obj,
        &source_for_compile,
        &stamp_path,
        "core",
        options,
    )?;

    #[cfg(feature = "bench-runtime")]
    {
        let bench_obj = output_dir.join("vibe_runtime_bench.o");
        let src = bench_runtime_source_path();
        let source_for_compile = if src.exists() {
            src
        } else {
            let embedded_src = output_dir.join("vibe_runtime_bench_embedded.c");
            std::fs::write(&embedded_src, BENCH_RUNTIME_C_SOURCE).map_err(|e| {
                format!(
                    "failed to write embedded bench runtime source `{}`: {e}",
                    embedded_src.display()
                )
            })?;
            embedded_src
        };
        let stamp_path = output_dir.join("vibe_runtime_bench.build.stamp");
        compile_runtime_c_object(
            &bench_obj,
            &source_for_compile,
            &stamp_path,
            "bench",
            options,
        )?;
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
        .args({
            #[cfg(feature = "bench-runtime")]
            {
                let Some(dir) = runtime_object.parent() else {
                    return Err(format!(
                        "runtime object has no parent directory: `{}`",
                        runtime_object.display()
                    ));
                };
                let bench_obj = dir.join("vibe_runtime_bench.o");
                if !bench_obj.exists() {
                    return Err(format!(
                        "bench runtime object does not exist: `{}`",
                        bench_obj.display()
                    ));
                }
                vec![bench_obj]
            }
            #[cfg(not(feature = "bench-runtime"))]
            {
                Vec::<std::path::PathBuf>::new()
            }
        })
        .arg("-o")
        .arg(output_binary);
    if !is_windows_target(&options.target) {
        cmd.arg("-pthread");
        cmd.arg("-lm");
    }
    if is_linux_gnu_target(&options.target) {
        cmd.arg("-Wl,--build-id=none").arg("-Wl,--gc-sections");
    } else if is_apple_target(&options.target) {
        cmd.arg("-Wl,-dead_strip");
    }
    if options.debuginfo != "none" {
        cmd.arg("-g");
    }
    if let Some(flag) = cc_target_flag(&options.target) {
        cmd.arg(flag);
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
    if SUPPORTED_TARGETS.contains(&target) {
        return Ok(());
    }
    Err(format!(
        "unsupported target `{target}` (supported: {})",
        SUPPORTED_TARGETS.join(", ")
    ))
}

fn cc_target_flag(target: &str) -> Option<&'static str> {
    match target {
        "x86_64-unknown-linux-gnu" => Some("-m64"),
        "aarch64-unknown-linux-gnu" => Some("--target=aarch64-linux-gnu"),
        "x86_64-apple-darwin" => Some("--target=x86_64-apple-darwin"),
        "aarch64-apple-darwin" => Some("--target=arm64-apple-darwin"),
        _ => None,
    }
}

fn is_linux_gnu_target(target: &str) -> bool {
    target.ends_with("-unknown-linux-gnu")
}

fn is_apple_target(target: &str) -> bool {
    target.ends_with("-apple-darwin")
}

fn is_windows_target(target: &str) -> bool {
    target.ends_with("-pc-windows-msvc")
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::{
        ensure_supported_target, select_recv, spawn_task, BoundedChannel, CancellationToken,
        RecvStatus, Scheduler, SelectRecvStatus, SendStatus,
    };

    #[test]
    fn channel_close_send_recv_semantics() {
        let ch = BoundedChannel::new(2);
        assert_eq!(ch.send(7), SendStatus::Sent);
        assert_eq!(ch.recv(), RecvStatus::Value(7));
        ch.close();
        assert_eq!(ch.send(9), SendStatus::Closed);
        assert_eq!(ch.recv(), RecvStatus::Closed);
    }

    #[test]
    fn select_fairness_smoke_non_starvation() {
        let ch_a = BoundedChannel::new(512);
        let ch_b = BoundedChannel::new(512);
        for _ in 0..200 {
            assert_eq!(ch_a.send(1), SendStatus::Sent);
            assert_eq!(ch_b.send(2), SendStatus::Sent);
        }

        let mut count_a = 0usize;
        let mut count_b = 0usize;
        for _ in 0..200 {
            match select_recv(&[ch_a.clone(), ch_b.clone()], false, None, None) {
                SelectRecvStatus::Received { index, .. } => {
                    if index == 0 {
                        count_a += 1;
                    } else {
                        count_b += 1;
                    }
                }
                other => panic!("unexpected select status: {other:?}"),
            }
        }

        assert!(count_a > 0, "channel A should not starve");
        assert!(count_b > 0, "channel B should not starve");
    }

    #[test]
    fn cancellation_unblocks_receive_promptly() {
        let ch: BoundedChannel<i64> = BoundedChannel::new(1);
        let token = CancellationToken::new();
        let cancel_clone = token.clone();
        let start = Instant::now();
        let waiter = thread::spawn(move || ch.recv_with(Some(&cancel_clone), None));
        thread::sleep(Duration::from_millis(10));
        token.cancel();
        let result = waiter.join().expect("join waiter");
        assert_eq!(result, RecvStatus::Cancelled);
        assert!(
            start.elapsed() < Duration::from_secs(1),
            "cancelled receive should return promptly"
        );
    }

    #[test]
    fn scheduler_runs_many_jobs_under_contention() {
        let scheduler = Scheduler::new(4);
        let handle = scheduler.handle();
        let counter = Arc::new(AtomicUsize::new(0));
        for _ in 0..2000 {
            let c = Arc::clone(&counter);
            handle.spawn(move || {
                c.fetch_add(1, Ordering::SeqCst);
            });
        }

        let deadline = Instant::now() + Duration::from_secs(5);
        while counter.load(Ordering::SeqCst) < 2000 && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(counter.load(Ordering::SeqCst), 2000);
    }

    #[test]
    fn scheduler_bounded_soak_stability() {
        let scheduler = Scheduler::new(4);
        let handle = scheduler.handle();
        let counter = Arc::new(AtomicUsize::new(0));
        for _batch in 0..20 {
            for _ in 0..500 {
                let c = Arc::clone(&counter);
                handle.spawn(move || {
                    c.fetch_add(1, Ordering::SeqCst);
                });
            }
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while counter.load(Ordering::SeqCst) < 10_000 && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(counter.load(Ordering::SeqCst), 10_000);
    }

    #[test]
    fn channel_contention_close_stress() {
        let ch = BoundedChannel::new(32);
        let sent = Arc::new(AtomicUsize::new(0));
        let received = Arc::new(AtomicUsize::new(0));
        let producers_done = Arc::new(AtomicUsize::new(0));

        let mut producer_handles = Vec::new();
        for producer_id in 0..4 {
            let local_ch = ch.clone();
            let local_sent = Arc::clone(&sent);
            let local_done = Arc::clone(&producers_done);
            producer_handles.push(thread::spawn(move || {
                for i in 0..500 {
                    let value = (producer_id * 1000 + i) as i64;
                    if local_ch.send(value) != SendStatus::Sent {
                        break;
                    }
                    local_sent.fetch_add(1, Ordering::SeqCst);
                }
                local_done.fetch_add(1, Ordering::SeqCst);
            }));
        }

        let mut consumer_handles = Vec::new();
        for _ in 0..4 {
            let local_ch = ch.clone();
            let local_received = Arc::clone(&received);
            let local_done = Arc::clone(&producers_done);
            consumer_handles.push(thread::spawn(move || loop {
                match local_ch.recv_with(None, Some(Duration::from_millis(10))) {
                    RecvStatus::Value(_) => {
                        local_received.fetch_add(1, Ordering::SeqCst);
                    }
                    RecvStatus::Closed => break,
                    RecvStatus::Timeout => {
                        if local_done.load(Ordering::SeqCst) == 4 && local_ch.is_empty() {
                            break;
                        }
                    }
                    RecvStatus::Cancelled => {}
                }
            }));
        }

        for h in producer_handles {
            h.join().expect("producer join");
        }
        ch.close();
        for h in consumer_handles {
            h.join().expect("consumer join");
        }

        assert_eq!(sent.load(Ordering::SeqCst), 2000);
        assert_eq!(received.load(Ordering::SeqCst), 2000);
    }

    #[test]
    fn task_failure_propagates_to_join_error() {
        let handle = spawn_task(CancellationToken::new(), |_token| -> i64 {
            panic!("intentional panic for propagation test");
        });
        let err = handle
            .join()
            .expect_err("panic should propagate as join error");
        assert!(
            err.contains("join failed"),
            "unexpected join error format: {err}"
        );
    }

    #[test]
    fn ensure_supported_target_accepts_release_targets() {
        assert!(ensure_supported_target("x86_64-unknown-linux-gnu").is_ok());
        assert!(ensure_supported_target("x86_64-apple-darwin").is_ok());
        assert!(ensure_supported_target("x86_64-pc-windows-msvc").is_ok());
        assert!(ensure_supported_target("aarch64-unknown-linux-gnu").is_ok());
        assert!(ensure_supported_target("aarch64-apple-darwin").is_ok());
    }

    #[test]
    fn ensure_supported_target_rejects_unknown_target() {
        let err =
            ensure_supported_target("wasm32-unknown-unknown").expect_err("target should fail");
        assert!(err.contains("unsupported target"));
    }
}
