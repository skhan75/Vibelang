use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::cancellation::CancellationToken;

#[derive(Debug, Clone)]
pub struct BoundedChannel<T> {
    inner: Arc<Inner<T>>,
}

#[derive(Debug)]
struct Inner<T> {
    state: Mutex<State<T>>,
    send_cv: Condvar,
    recv_cv: Condvar,
}

#[derive(Debug)]
struct State<T> {
    queue: VecDeque<T>,
    capacity: usize,
    closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecvStatus<T> {
    Value(T),
    Closed,
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendStatus {
    Sent,
    Closed,
    Cancelled,
    Timeout,
}

impl<T> BoundedChannel<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Inner {
                state: Mutex::new(State {
                    queue: VecDeque::new(),
                    capacity,
                    closed: false,
                }),
                send_cv: Condvar::new(),
                recv_cv: Condvar::new(),
            }),
        }
    }

    pub fn capacity(&self) -> usize {
        self.inner
            .state
            .lock()
            .expect("channel lock poisoned")
            .capacity
    }

    pub fn close(&self) {
        let mut state = self.inner.state.lock().expect("channel lock poisoned");
        state.closed = true;
        self.inner.send_cv.notify_all();
        self.inner.recv_cv.notify_all();
    }

    pub fn is_closed(&self) -> bool {
        self.inner
            .state
            .lock()
            .expect("channel lock poisoned")
            .closed
    }

    pub fn len(&self) -> usize {
        self.inner
            .state
            .lock()
            .expect("channel lock poisoned")
            .queue
            .len()
    }

    pub fn send(&self, value: T) -> SendStatus {
        self.send_with(value, None, None)
    }

    pub fn send_with(
        &self,
        mut value: T,
        cancel: Option<&CancellationToken>,
        timeout: Option<Duration>,
    ) -> SendStatus {
        let start = Instant::now();
        let mut state = self.inner.state.lock().expect("channel lock poisoned");
        loop {
            if state.closed {
                return SendStatus::Closed;
            }
            if cancel.is_some_and(|c| c.is_cancelled()) {
                return SendStatus::Cancelled;
            }
            if state.queue.len() < state.capacity {
                state.queue.push_back(value);
                self.inner.recv_cv.notify_one();
                return SendStatus::Sent;
            }
            if let Some(timeout) = timeout {
                let elapsed = start.elapsed();
                if elapsed >= timeout {
                    return SendStatus::Timeout;
                }
                let remaining = timeout.saturating_sub(elapsed);
                let (new_state, wait_result) = self
                    .inner
                    .send_cv
                    .wait_timeout(state, remaining)
                    .expect("channel condvar poisoned");
                state = new_state;
                if wait_result.timed_out() {
                    return SendStatus::Timeout;
                }
            } else if cancel.is_some() {
                let (new_state, _wait_result) = self
                    .inner
                    .send_cv
                    .wait_timeout(state, Duration::from_millis(5))
                    .expect("channel condvar poisoned");
                state = new_state;
            } else {
                state = self
                    .inner
                    .send_cv
                    .wait(state)
                    .expect("channel condvar poisoned");
            }
            if state.closed {
                return SendStatus::Closed;
            }
            if cancel.is_some_and(|c| c.is_cancelled()) {
                return SendStatus::Cancelled;
            }
            // Value might not have been moved yet because queue was full.
            // The loop retries and eventually pushes `value`.
            let _ = &mut value;
        }
    }

    pub fn recv(&self) -> RecvStatus<T> {
        self.recv_with(None, None)
    }

    pub fn recv_with(
        &self,
        cancel: Option<&CancellationToken>,
        timeout: Option<Duration>,
    ) -> RecvStatus<T> {
        let start = Instant::now();
        let mut state = self.inner.state.lock().expect("channel lock poisoned");
        loop {
            if let Some(value) = state.queue.pop_front() {
                self.inner.send_cv.notify_one();
                return RecvStatus::Value(value);
            }
            if state.closed {
                return RecvStatus::Closed;
            }
            if cancel.is_some_and(|c| c.is_cancelled()) {
                return RecvStatus::Cancelled;
            }
            if let Some(timeout) = timeout {
                let elapsed = start.elapsed();
                if elapsed >= timeout {
                    return RecvStatus::Timeout;
                }
                let remaining = timeout.saturating_sub(elapsed);
                let (new_state, wait_result) = self
                    .inner
                    .recv_cv
                    .wait_timeout(state, remaining)
                    .expect("channel condvar poisoned");
                state = new_state;
                if wait_result.timed_out() {
                    return RecvStatus::Timeout;
                }
            } else if cancel.is_some() {
                let (new_state, _wait_result) = self
                    .inner
                    .recv_cv
                    .wait_timeout(state, Duration::from_millis(5))
                    .expect("channel condvar poisoned");
                state = new_state;
            } else {
                state = self
                    .inner
                    .recv_cv
                    .wait(state)
                    .expect("channel condvar poisoned");
            }
        }
    }

    pub fn try_recv(&self) -> RecvStatus<T> {
        let mut state = self.inner.state.lock().expect("channel lock poisoned");
        if let Some(value) = state.queue.pop_front() {
            self.inner.send_cv.notify_one();
            return RecvStatus::Value(value);
        }
        if state.closed {
            RecvStatus::Closed
        } else {
            RecvStatus::Timeout
        }
    }
}
