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
        .arg("-pthread")
        .arg("-fno-ident")
        .arg("-ffunction-sections")
        .arg("-fdata-sections")
        .arg("-std=c11");

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
        .arg("-pthread")
        .arg("-Wl,--build-id=none")
        .arg("-Wl,--gc-sections");
    if options.debuginfo != "none" {
        cmd.arg("-g");
    }
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
        "unsupported target `{target}` in phase 3 baseline (supported: x86_64-unknown-linux-gnu)"
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::{
        select_recv, spawn_task, BoundedChannel, CancellationToken, RecvStatus, Scheduler,
        SelectRecvStatus, SendStatus,
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
                        if local_done.load(Ordering::SeqCst) == 4 && local_ch.len() == 0 {
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
}
