/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};

use bwindow::EventLoopProxy;

use crate::AppEvent;
use crate::engine::{Move, Othello, Player};

#[derive(Clone, Copy)]
pub(crate) struct AiJob {
    pub(crate) revision: u64,
    pub(crate) board: Othello,
}

pub(crate) struct AiResult {
    pub(crate) job: AiJob,
    pub(crate) movement: Option<Move>,
}

pub(crate) struct AiWorker {
    sender: Option<mpsc::Sender<AiJob>>,
    revision: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl AiWorker {
    pub(crate) fn new(proxy: EventLoopProxy<AppEvent>) -> Self {
        let (sender, receiver) = mpsc::channel::<AiJob>();
        let revision = Arc::new(AtomicU64::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let current = revision.clone();
        let stopping = stop.clone();
        let thread = thread::spawn(move || {
            while let Ok(mut job) = receiver.recv() {
                // Only search the newest queued position after a reset.
                for newer in receiver.try_iter() {
                    job = newer;
                }
                let cancelled = || {
                    stopping.load(Ordering::Relaxed)
                        || current.load(Ordering::Relaxed) != job.revision
                };
                if stopping.load(Ordering::Relaxed) {
                    break;
                }
                if cancelled() {
                    continue;
                }
                let movement =
                    job.board
                        .compute_move_cancellable(Player::White, 64, 500_000, &cancelled);
                if !cancelled()
                    && proxy
                        .send_user_event(AppEvent::Ai(AiResult { job, movement }))
                        .is_err()
                {
                    break;
                }
            }
        });
        Self {
            sender: Some(sender),
            revision,
            stop,
            thread: Some(thread),
        }
    }

    pub(crate) fn submit(&self, job: AiJob) {
        self.cancel(job.revision);
        self.sender
            .as_ref()
            .expect("AI worker running")
            .send(job)
            .expect("AI worker stopped");
    }

    pub(crate) fn cancel(&self, revision: u64) {
        self.revision.store(revision, Ordering::Relaxed);
    }
}

impl Drop for AiWorker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.sender.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
