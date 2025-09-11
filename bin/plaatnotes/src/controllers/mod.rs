/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use small_http::{Request, Response, Status};

pub(crate) use self::notes::{notes_create, notes_delete, notes_index, notes_show, notes_update};
use crate::Context;

mod notes;

pub(crate) fn home(_: &Request, _: &Context) -> Response {
    Response::with_body(concat!("PlaatNotes API v", env!("CARGO_PKG_VERSION")))
}

pub(crate) fn not_found(_: &Request, _: &Context) -> Response {
    Response::with_status(Status::NotFound).body("404 Not found")
}
