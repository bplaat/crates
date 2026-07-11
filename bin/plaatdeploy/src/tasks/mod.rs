/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::thread;

use crate::context::Context;

pub(crate) mod ip_database;

pub(crate) fn start_task_runner(ctx: Context, mmdb_path: String) {
    thread::spawn(move || ip_database::run(mmdb_path, ctx));
}
