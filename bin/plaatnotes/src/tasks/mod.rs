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
pub(crate) mod ip_database;

pub(crate) fn start_task_runner(ctx: Context, mmdb_path: String) {
    // Startup task: download and load the DB-IP database
    let ctx_clone = ctx.clone();
    thread::spawn(move || ip_database::run(mmdb_path, ctx_clone));

    // Background task: clear trashed notes
    thread::spawn(move || {
        loop {
            if let Err(e) = clear_trashed_notes::run(&ctx) {
                log::error!("Failed to clear trashed notes: {e}");
            }
            thread::sleep(Duration::from_secs(TASK_RUNNER_INTERVAL_SECONDS));
        }
    });
}
