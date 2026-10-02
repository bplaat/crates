/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Stream for blocking backends, a thread reads the device while the stream is playing.

use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use crate::{DataCallback, Error, ErrorCallback, ErrorKind, SampleRate};

/// Sample rate requested from devices that convert any rate.
pub(crate) const DEFAULT_SAMPLE_RATE: SampleRate = 48_000;

/// A blocking capture device, it is owned by the capture thread and closed when dropped.
pub(crate) trait Capture: Send + 'static {
    /// Starts or resumes capturing.
    fn start(&mut self) -> Result<(), Error>;

    /// Stops capturing.
    fn stop(&mut self) -> Result<(), Error>;

    /// Waits a short while for samples and appends them interleaved.
    fn read(&mut self, samples: &mut Vec<f32>) -> Result<(), Error>;
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum State {
    Paused,
    Playing,
    Closed,
}

type Shared = Arc<(Mutex<State>, Condvar)>;

pub(crate) struct Stream {
    shared: Shared,
    thread: Option<JoinHandle<()>>,
}

impl Stream {
    pub(crate) fn spawn(
        capture: impl Capture,
        data: DataCallback,
        error: ErrorCallback,
    ) -> Result<Self, Error> {
        let shared = Arc::new((Mutex::new(State::Paused), Condvar::new()));
        let thread = thread::Builder::new()
            .name("cpal-input".to_string())
            .spawn({
                let shared = shared.clone();
                move || capture_thread(&shared, capture, data, error)
            })
            .map_err(|err| Error::with_message(ErrorKind::BackendError, err.to_string()))?;
        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }

    fn set_state(&self, state: State) {
        let (lock, condvar) = &*self.shared;
        *lock.lock().expect("Failed to lock stream state") = state;
        condvar.notify_all();
    }

    pub(crate) fn play(&self) -> Result<(), Error> {
        self.set_state(State::Playing);
        Ok(())
    }

    pub(crate) fn pause(&self) -> Result<(), Error> {
        self.set_state(State::Paused);
        Ok(())
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        self.set_state(State::Closed);
        if let Some(thread) = self.thread.take() {
            _ = thread.join();
        }
    }
}

fn capture_thread(
    shared: &Shared,
    mut capture: impl Capture,
    mut data: DataCallback,
    mut error: ErrorCallback,
) {
    let (lock, condvar) = &**shared;
    let mut running = false;
    let mut samples = Vec::new();
    loop {
        let state = *lock.lock().expect("Failed to lock stream state");
        let result = match (state, running) {
            (State::Closed, _) => break,
            (State::Playing, false) => capture.start().map(|()| running = true),
            (State::Paused, true) => capture.stop().map(|()| running = false),
            (State::Playing, true) => {
                samples.clear();
                capture.read(&mut samples).map(|()| {
                    if !samples.is_empty() {
                        data(&samples);
                    }
                })
            }
            (State::Paused, false) => {
                drop(
                    condvar
                        .wait_while(lock.lock().expect("Failed to lock stream state"), |state| {
                            *state == State::Paused
                        })
                        .expect("Failed to lock stream state"),
                );
                Ok(())
            }
        };
        if let Err(err) = result {
            error(err);
            break;
        }
    }
}
