// Copyright 2025-2026 VibeLang Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use crate::cancellation::CancellationToken;
use crate::channel::{BoundedChannel, RecvStatus};

static SELECT_CURSOR: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectRecvStatus<T> {
    Received { index: usize, value: T },
    Closed { index: usize },
    Cancelled,
    Timeout,
    WouldBlock,
}

pub fn select_recv<T: Clone>(
    channels: &[BoundedChannel<T>],
    default_non_blocking: bool,
    after: Option<Duration>,
    cancel: Option<&CancellationToken>,
) -> SelectRecvStatus<T> {
    if channels.is_empty() {
        return SelectRecvStatus::WouldBlock;
    }
    let start = Instant::now();
    loop {
        if cancel.is_some_and(|c| c.is_cancelled()) {
            return SelectRecvStatus::Cancelled;
        }
        let start_idx = SELECT_CURSOR.fetch_add(1, Ordering::SeqCst) % channels.len();
        for offset in 0..channels.len() {
            let idx = (start_idx + offset) % channels.len();
            match channels[idx].try_recv() {
                RecvStatus::Value(v) => {
                    return SelectRecvStatus::Received {
                        index: idx,
                        value: v,
                    }
                }
                RecvStatus::Closed => return SelectRecvStatus::Closed { index: idx },
                RecvStatus::Cancelled | RecvStatus::Timeout => {}
            }
        }

        if default_non_blocking {
            return SelectRecvStatus::WouldBlock;
        }
        if let Some(after) = after {
            if start.elapsed() >= after {
                return SelectRecvStatus::Timeout;
            }
        }
        thread::sleep(Duration::from_millis(1));
    }
}
