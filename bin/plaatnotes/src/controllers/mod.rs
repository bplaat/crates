/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use small_http::{Request, Response, Status};

pub(crate) use self::auth::{auth_login, auth_logout, auth_validate};
pub(crate) use self::imports::imports_google_keep;
pub(crate) use self::notes::{
    notes_archived, notes_archived_reorder, notes_create, notes_delete, notes_index, notes_pinned,
    notes_pinned_reorder, notes_reorder, notes_show, notes_trashed, notes_trashed_clear,
    notes_update,
};
pub(crate) use self::sessions::{
    sessions_active, sessions_delete, sessions_index, sessions_show, users_sessions,
    users_sessions_active,
};
pub(crate) use self::users::{
    users_change_password, users_create, users_delete, users_index, users_notes,
    users_notes_archived, users_notes_pinned, users_notes_trashed, users_show, users_update,
};
use crate::Context;

mod auth;
mod imports;
mod notes;
mod sessions;
mod users;

// MARK: Handlers
pub(crate) fn home(_: &Request, _: &Context) -> Result<Response> {
    Ok(Response::with_body(concat!(
        "PlaatNotes API v",
        env!("CARGO_PKG_VERSION")
    )))
}

pub(crate) fn not_found(_: &Request, _: &Context) -> Result<Response> {
    Ok(Response::with_status(Status::NotFound))
}

// MARK: Macros

/// Unwrap the authenticated user from context. All callers must be behind `auth_required_pre_layer`.
macro_rules! require_auth {
    ($ctx:expr) => {
        match $ctx.auth_user.as_ref() {
            Some(u) => u,
            None => return Ok(Response::with_status(Status::Unauthorized)),
        }
    };
}

/// Parse and validate the index query parameters from the request URL.
macro_rules! parse_index_query {
    ($req:expr) => {{
        use validate::Validate as _;
        let query = match $req.url.query() {
            Some(q) => match serde_urlencoded::from_str::<crate::models::IndexQuery>(q) {
                Ok(q) => q,
                Err(_) => {
                    return Ok(small_http::Response::with_status(
                        small_http::Status::BadRequest,
                    ));
                }
            },
            None => crate::models::IndexQuery::default(),
        };
        if let Err(report) = query.validate() {
            return Ok(
                small_http::Response::with_status(small_http::Status::BadRequest)
                    .json(crate::api::Report::from(report)),
            );
        }
        query
    }};
}

/// Parse and validate a URL-encoded form body. `$api_type` is the API struct, `$internal_type` the validated internal struct.
macro_rules! parse_body {
    ($req:expr, $api_type:ty, $internal_type:ty) => {{
        use validate::Validate as _;
        let body =
            match serde_urlencoded::from_bytes::<$api_type>($req.body.as_deref().unwrap_or(&[])) {
                Ok(b) => Into::<$internal_type>::into(b),
                Err(_) => {
                    return Ok(small_http::Response::with_status(
                        small_http::Status::BadRequest,
                    ));
                }
            };
        if let Err(report) = body.validate() {
            return Ok(
                small_http::Response::with_status(small_http::Status::BadRequest)
                    .json(crate::api::Report::from(report)),
            );
        }
        body
    }};
}

/// Parse and validate a URL-encoded form body where the validator needs the request context.
macro_rules! parse_body_ctx {
    ($req:expr, $api_type:ty, $internal_type:ty, $ctx:expr) => {{
        use validate::Validate as _;
        let body =
            match serde_urlencoded::from_bytes::<$api_type>($req.body.as_deref().unwrap_or(&[])) {
                Ok(b) => Into::<$internal_type>::into(b),
                Err(_) => {
                    return Ok(small_http::Response::with_status(
                        small_http::Status::BadRequest,
                    ));
                }
            };
        if let Err(report) = body.validate_with($ctx) {
            return Ok(
                small_http::Response::with_status(small_http::Status::BadRequest)
                    .json(crate::api::Report::from(report)),
            );
        }
        body
    }};
}

pub(crate) use parse_body;
pub(crate) use parse_body_ctx;
pub(crate) use parse_index_query;
pub(crate) use require_auth;
