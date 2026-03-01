/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use log::info;

use crate::consts::TRASHED_NOTE_EXPIRY_DAYS;
use crate::context::Context;

pub(crate) fn run(ctx: &Context) {
    let expiry_secs = TRASHED_NOTE_EXPIRY_DAYS * 24 * 60 * 60;
    let count = ctx
        .database
        .query_some::<i64>(
            "SELECT COUNT(id) FROM notes WHERE is_trashed = 1 AND updated_at <= unixepoch() - ?",
            expiry_secs,
        )
        .expect("Database error");
    if count > 0 {
        ctx.database
            .execute(
                "DELETE FROM notes WHERE is_trashed = 1 AND updated_at <= unixepoch() - ?",
                expiry_secs,
            )
            .expect("Database error");
        info!("Cleared {count} trashed note(s) older than {TRASHED_NOTE_EXPIRY_DAYS} days");
    }
}
