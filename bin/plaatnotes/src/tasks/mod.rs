/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::thread;
use std::time::Duration;

use crate::consts::TASK_RUNNER_INTERVAL_SECONDS;
use crate::context::Context;

pub(crate) mod clear_trashed_notes;

pub(crate) fn start_task_runner(ctx: Context) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(TASK_RUNNER_INTERVAL_SECONDS));
            clear_trashed_notes::run(&ctx);
        }
    });
}
