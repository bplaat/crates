/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::{execute_args, query_args};
use chrono::Utc;
use const_format::formatcp;
use small_http::{Request, Response, Status};
use uuid::Uuid;
use validate::Validate;

use crate::api;
use crate::context::{Context, DatabaseHelpers};
use crate::controllers::not_found;
use crate::models::{IndexQuery, Note};

pub(crate) fn notes_index(req: &Request, ctx: &Context) -> Response {
    // Parse request query
    let query = match req.url.query() {
        Some(query) => match serde_urlencoded::from_str::<IndexQuery>(query) {
            Ok(query) => query,
            Err(_) => return Response::with_status(Status::BadRequest),
        },
        None => IndexQuery::default(),
    };
    if let Err(report) = query.validate() {
        return Response::with_status(Status::BadRequest).json(report);
    }

    // Get notes
    let search_query = format!("%{}%", query.query.replace("%", "\\%"));
    let total = ctx.database.query_some::<i64>(
        "SELECT COUNT(id) FROM notes WHERE body LIKE ?",
        search_query.clone(),
    );
    let notes = query_args!(
        Note,
        ctx.database,
        formatcp!(
            "SELECT {} FROM notes WHERE body LIKE :search_query LIMIT :limit OFFSET :offset",
            Note::columns()
        ),
        Args {
            search_query: search_query,
            limit: query.limit,
            offset: (query.page - 1) * query.limit
        }
    )
    .map(Into::<api::Note>::into)
    .collect::<Vec<_>>();

    // Return notes
    Response::with_json(api::NoteIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: notes,
    })
}

#[derive(Validate)]
struct NoteCreateUpdateBody {
    #[validate(ascii, length(min = 1))]
    body: String,
}

impl From<api::NoteCreateUpdateBody> for NoteCreateUpdateBody {
    fn from(body: api::NoteCreateUpdateBody) -> Self {
        Self { body: body.body }
    }
}

pub(crate) fn notes_create(req: &Request, ctx: &Context) -> Response {
    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::NoteCreateUpdateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => Into::<NoteCreateUpdateBody>::into(body),
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(report);
    }

    // Create note
    let note = Note {
        body: body.body,
        ..Default::default()
    };
    ctx.database.insert_note(note.clone());

    // Return created note
    Response::with_json(Into::<api::Note>::into(note))
}

pub(crate) fn get_note(req: &Request, ctx: &Context) -> Option<Note> {
    // Parse note id from url
    let note_id = match req
        .params
        .get("note_id")
        .expect("Should be some")
        .parse::<Uuid>()
    {
        Ok(id) => id,
        Err(_) => return None,
    };

    // Get note
    ctx.database
        .query::<Note>(
            formatcp!("SELECT {} FROM notes WHERE id = ? LIMIT 1", Note::columns()),
            note_id,
        )
        .next()
}

pub(crate) fn notes_show(req: &Request, ctx: &Context) -> Response {
    // Get note
    let note = match get_note(req, ctx) {
        Some(note) => note,
        None => return not_found(req, ctx),
    };

    // Return note
    Response::with_json(Into::<api::Note>::into(note))
}

pub(crate) fn notes_update(req: &Request, ctx: &Context) -> Response {
    // Get note
    let mut note = match get_note(req, ctx) {
        Some(note) => note,
        None => return not_found(req, ctx),
    };

    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::NoteCreateUpdateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => Into::<NoteCreateUpdateBody>::into(body),
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(report);
    }

    // Update note
    note.body = body.body;
    note.updated_at = Utc::now();
    execute_args!(
        ctx.database,
        "UPDATE notes SET body = :body, updated_at = :updated_at WHERE id = :id",
        Args {
            body: note.body.clone(),
            updated_at: note.updated_at,
            id: note.id
        }
    );

    // Return updated note
    Response::with_json(Into::<api::Note>::into(note))
}

pub(crate) fn notes_delete(req: &Request, ctx: &Context) -> Response {
    // Get note
    let note = match get_note(req, ctx) {
        Some(note) => note,
        None => return not_found(req, ctx),
    };

    // Delete note
    ctx.database
        .execute("DELETE FROM notes WHERE id = ?", note.id);

    // Success response
    Response::new()
}
