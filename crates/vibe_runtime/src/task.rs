use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crate::cancellation::CancellationToken;

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub u64);

#[derive(Debug)]
pub struct TaskHandle<T: Send + 'static> {
    pub id: TaskId,
    token: CancellationToken,
    recv: Receiver<T>,
}

impl<T: Send + 'static> TaskHandle<T> {
    pub fn join(self) -> Result<T, String> {
        self.recv
            .recv()
            .map_err(|e| format!("task {} join failed: {e}", self.id.0))
    }

    pub fn cancel(&self) {
        self.token.cancel();
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        self.token.clone()
    }
}

pub fn spawn_task<F, T>(token: CancellationToken, job: F) -> TaskHandle<T>
where
    F: FnOnce(CancellationToken) -> T + Send + 'static,
    T: Send + 'static,
{
    let id = TaskId(NEXT_TASK_ID.fetch_add(1, Ordering::SeqCst));
    let (tx, rx) = mpsc::channel();
    let cloned = token.clone();
    thread::spawn(move || {
        let out = job(cloned);
        let _ = tx.send(out);
    });
    TaskHandle {
        id,
        token,
        recv: rx,
    }
}
