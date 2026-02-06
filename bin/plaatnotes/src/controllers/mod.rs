/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use small_http::{Request, Response, Status};

pub(crate) use self::auth::{auth_login, auth_logout, auth_validate};
pub(crate) use self::notes::{notes_create, notes_delete, notes_index, notes_show, notes_update};
pub(crate) use self::sessions::{sessions_delete, sessions_index, sessions_show};
pub(crate) use self::users::{
    users_change_password, users_create, users_delete, users_index, users_notes, users_show,
    users_update,
};
use crate::Context;

mod auth;
mod notes;
mod sessions;
mod users;

pub(crate) fn home(_: &Request, _: &Context) -> Response {
    Response::with_body(concat!("PlaatNotes API v", env!("CARGO_PKG_VERSION")))
}

pub(crate) fn not_found(_: &Request, _: &Context) -> Response {
    Response::with_status(Status::NotFound).body("404 Not found")
}
