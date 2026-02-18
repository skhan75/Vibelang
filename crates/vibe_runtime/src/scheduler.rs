use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

#[derive(Debug, Clone, Default)]
pub struct SchedulerMetrics {
    pub runnable_tasks: Arc<AtomicUsize>,
    pub queue_depth: Arc<AtomicUsize>,
    pub steal_attempts: Arc<AtomicUsize>,
    pub blocking_waits: Arc<AtomicUsize>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Clone)]
pub struct SchedulerHandle {
    shared: Arc<SharedState>,
    metrics: SchedulerMetrics,
}

impl SchedulerHandle {
    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut queue = self.shared.queue.lock().expect("scheduler queue poisoned");
        queue.push_back(Box::new(f));
        self.metrics.runnable_tasks.fetch_add(1, Ordering::SeqCst);
        self.metrics
            .queue_depth
            .store(queue.len(), Ordering::SeqCst);
        self.shared.cv.notify_one();
    }

    pub fn metrics(&self) -> SchedulerMetrics {
        self.metrics.clone()
    }
}

pub struct Scheduler {
    shared: Arc<SharedState>,
    workers: Vec<JoinHandle<()>>,
    metrics: SchedulerMetrics,
}

struct SharedState {
    queue: Mutex<VecDeque<Job>>,
    cv: Condvar,
    shutdown: AtomicBool,
}

impl Scheduler {
    pub fn new(worker_count: usize) -> Self {
        let worker_count = worker_count.max(1);
        let shared = Arc::new(SharedState {
            queue: Mutex::new(VecDeque::new()),
            cv: Condvar::new(),
            shutdown: AtomicBool::new(false),
        });
        let metrics = SchedulerMetrics::default();
        let mut workers = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            let shared_state = Arc::clone(&shared);
            let m = metrics.clone();
            workers.push(thread::spawn(move || worker_loop(shared_state, m)));
        }
        Self {
            shared,
            workers,
            metrics,
        }
    }

    pub fn handle(&self) -> SchedulerHandle {
        SchedulerHandle {
            shared: Arc::clone(&self.shared),
            metrics: self.metrics.clone(),
        }
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.shared.shutdown.store(true, Ordering::SeqCst);
        self.shared.cv.notify_all();
        while let Some(worker) = self.workers.pop() {
            let _ = worker.join();
        }
    }
}

fn worker_loop(shared: Arc<SharedState>, metrics: SchedulerMetrics) {
    loop {
        let job = {
            let mut queue = shared.queue.lock().expect("scheduler queue poisoned");
            loop {
                if shared.shutdown.load(Ordering::SeqCst) {
                    return;
                }
                if let Some(job) = queue.pop_front() {
                    metrics
                        .queue_depth
                        .store(queue.len(), Ordering::SeqCst);
                    break job;
                }
                metrics.blocking_waits.fetch_add(1, Ordering::SeqCst);
                metrics.steal_attempts.fetch_add(1, Ordering::SeqCst);
                queue = shared.cv.wait(queue).expect("scheduler condvar poisoned");
            }
        };
        metrics.runnable_tasks.fetch_sub(1, Ordering::SeqCst);
        job();
    }
}
