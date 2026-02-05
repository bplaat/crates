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
        return Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report));
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
            "SELECT {} FROM notes WHERE body LIKE :search_query ORDER BY updated_at DESC LIMIT :limit OFFSET :offset",
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
struct NoteCreateBody {
    #[validate(ascii, length(min = 1))]
    body: String,
}

impl From<api::NoteCreateBody> for NoteCreateBody {
    fn from(body: api::NoteCreateBody) -> Self {
        Self { body: body.body }
    }
}

pub(crate) fn notes_create(req: &Request, ctx: &Context) -> Response {
    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::NoteCreateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => Into::<NoteCreateBody>::into(body),
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report));
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
        .expect("note_id param should be present")
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

#[derive(Validate)]
struct NoteUpdateBody {
    #[validate(ascii, length(min = 1))]
    body: String,
}

impl From<api::NoteUpdateBody> for NoteUpdateBody {
    fn from(body: api::NoteUpdateBody) -> Self {
        Self { body: body.body }
    }
}

pub(crate) fn notes_update(req: &Request, ctx: &Context) -> Response {
    // Get note
    let mut note = match get_note(req, ctx) {
        Some(note) => note,
        None => return not_found(req, ctx),
    };

    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::NoteUpdateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => Into::<NoteUpdateBody>::into(body),
        Err(_) => return Response::with_status(Status::BadRequest),
    };
    if let Err(report) = body.validate() {
        return Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report));
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

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::router;

    #[test]
    fn test_notes_index() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Fetch /notes check if empty
        let res = router.handle(&Request::get("http://localhost/api/notes"));
        assert_eq!(res.status, Status::Ok);
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert!(notes.is_empty());

        // Create note
        let note = Note {
            body: "This is my first note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Fetch /notes check if note is there
        let res = router.handle(&Request::get("http://localhost/api/notes"));
        assert_eq!(res.status, Status::Ok);
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].body, "This is my first note");
    }

    #[test]
    fn test_notes_index_search() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create multiple notes
        ctx.database.insert_note(Note {
            body: "Meeting notes from today".to_string(),
            ..Default::default()
        });
        ctx.database.insert_note(Note {
            body: "Shopping list for tomorrow".to_string(),
            ..Default::default()
        });

        // Search for "meeting"
        let res = router.handle(&Request::get("http://localhost/api/notes?q=meeting"));
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].body, "Meeting notes from today");
    }

    #[test]
    fn test_notes_index_pagination() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create multiple notes
        for i in 1..=30 {
            ctx.database.insert_note(Note {
                body: format!("Note number {i}"),
                ..Default::default()
            });
        }

        // Fetch /notes with limit 10 and page 1
        let res = router.handle(&Request::get("http://localhost/api/notes?limit=10&page=1"));
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 10);
        assert_eq!(response.pagination.page, 1);
        assert_eq!(response.pagination.limit, 10);
        assert_eq!(response.pagination.total, 30);

        // Fetch /notes with limit 5 and page 2
        let res = router.handle(&Request::get("http://localhost/api/notes?limit=5&page=2"));
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 5);
        assert_eq!(response.pagination.page, 2);
        assert_eq!(response.pagination.limit, 5);
        assert_eq!(response.pagination.total, 30);
    }

    #[test]
    fn test_notes_create() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create note
        let res = router
            .handle(&Request::post("http://localhost/api/notes").body("body=This+is+a+new+note"));
        assert_eq!(res.status, Status::Ok);
        let note = serde_json::from_slice::<api::Note>(&res.body).unwrap();
        assert_eq!(note.body, "This is a new note");
    }

    #[test]
    fn test_notes_show() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create note
        let note = Note {
            body: "My important note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Fetch /notes/:note_id check if note is there
        let res = router.handle(&Request::get(format!(
            "http://localhost/api/notes/{}",
            note.id
        )));
        assert_eq!(res.status, Status::Ok);
        let note = serde_json::from_slice::<api::Note>(&res.body).unwrap();
        assert_eq!(note.body, "My important note");

        // Fetch other note by random id should be 404 Not Found
        let res = router.handle(&Request::get(format!(
            "http://localhost/api/notes/{}",
            Uuid::now_v7()
        )));
        assert_eq!(res.status, Status::NotFound);
    }

    #[test]
    fn test_notes_update() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create note
        let note = Note {
            body: "Original note content".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Update note
        let res = router.handle(
            &Request::put(format!("http://localhost/api/notes/{}", note.id))
                .body("body=Updated+note+content"),
        );
        assert_eq!(res.status, Status::Ok);
        let note = serde_json::from_slice::<api::Note>(&res.body).unwrap();
        assert_eq!(note.body, "Updated note content");

        // Update note with validation errors (empty body)
        let res = router
            .handle(&Request::put(format!("http://localhost/api/notes/{}", note.id)).body("body="));
        assert_eq!(res.status, Status::BadRequest);
    }

    #[test]
    fn test_notes_delete() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create note
        let note = Note {
            body: "Note to be deleted".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Delete note
        let res = router.handle(&Request::delete(format!(
            "http://localhost/api/notes/{}",
            note.id
        )));
        assert_eq!(res.status, Status::Ok);

        // Fetch /notes check if empty
        let res = router.handle(&Request::get("http://localhost/api/notes"));
        assert_eq!(res.status, Status::Ok);
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert!(notes.is_empty());
    }
}
