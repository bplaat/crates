/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::context::Context;

pub(crate) mod ip_database;

pub(crate) fn start_task_runner(ctx: Context, mmdb_path: String) {
    std::thread::Builder::new()
        .name("ip-database".to_string())
        .spawn(move || ip_database::run(mmdb_path, ctx))
        .expect("Failed to spawn IP database thread");
}
