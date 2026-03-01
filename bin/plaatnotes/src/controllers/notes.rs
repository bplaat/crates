/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::{execute_args, query_args};
use chrono::Utc;
use const_format::formatcp;
use from_derive::FromStruct;
use small_http::{Request, Response, Status};
use uuid::Uuid;
use validate::Validate;

use crate::api;
use crate::context::{Context, DatabaseHelpers};
use crate::controllers::not_found;
use crate::models::note::policies;
use crate::models::{IndexQuery, Note, User, UserRole};

pub(crate) fn notes_index(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Check authorization
    if !policies::can_index(user) {
        return Response::with_status(Status::Forbidden);
    }

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

    // Get notes for authenticated user or all notes if admin
    let search_query = format!("%{}%", query.query.replace("%", "\\%"));
    let (total, notes) = match user.role {
        UserRole::Admin => {
            // Admin sees all notes
            let total = query_args!(
                i64,
                ctx.database,
                "SELECT COUNT(id) FROM notes WHERE is_trashed = 0 AND body LIKE :search_query",
                Args {
                    search_query: search_query.clone()
                }
            )
            .expect("Database error")
            .next()
            .map(|r| r.expect("Database error"))
            .unwrap_or(0);
            let notes = query_args!(
                Note,
                ctx.database,
                formatcp!(
                    "SELECT {} FROM notes WHERE is_trashed = 0 AND body LIKE :search_query ORDER BY updated_at DESC LIMIT :limit OFFSET :offset",
                    Note::columns()
                ),
                Args {
                    search_query: search_query,
                    limit: query.limit,
                    offset: (query.page - 1) * query.limit
                }
            )
            .expect("Database error")
            .map(|r| Into::<api::Note>::into(r.expect("Database error")))
            .collect::<Vec<_>>();
            (total, notes)
        }
        UserRole::Normal => {
            // Normal user sees only their own notes
            let total = query_args!(
                i64,
                ctx.database,
                "SELECT COUNT(id) FROM notes WHERE is_trashed = 0 AND user_id = :user_id AND body LIKE :search_query",
                Args {
                    user_id: user.id,
                    search_query: search_query.clone()
                }
            )
            .expect("Database error")
            .next()
            .map(|r| r.expect("Database error"))
            .unwrap_or(0);
            let notes = query_args!(
                Note,
                ctx.database,
                formatcp!(
                    "SELECT {} FROM notes WHERE is_trashed = 0 AND user_id = :user_id AND body LIKE :search_query ORDER BY updated_at DESC LIMIT :limit OFFSET :offset",
                    Note::columns()
                ),
                Args {
                    user_id: user.id,
                    search_query: search_query,
                    limit: query.limit,
                    offset: (query.page - 1) * query.limit
                }
            )
            .expect("Database error")
            .map(|r| Into::<api::Note>::into(r.expect("Database error")))
            .collect::<Vec<_>>();
            (total, notes)
        }
    };

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

#[derive(Validate, FromStruct)]
#[from_struct(api::NoteCreateBody)]
struct NoteCreateBody {
    #[validate(ascii, length(min = 1))]
    title: Option<String>,
    #[validate(ascii, length(min = 1))]
    body: String,
    is_pinned: Option<bool>,
}

pub(crate) fn notes_create(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Check authorization
    if !policies::can_create(user) {
        return Response::with_status(Status::Forbidden);
    }

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

    // Create note with authenticated user's ID
    let note = Note {
        user_id: user.id,
        title: body.title,
        body: body.body,
        is_pinned: body.is_pinned.unwrap_or(false),
        is_archived: false,
        is_trashed: false,
        ..Default::default()
    };
    ctx.database.insert_note(note.clone());

    // Return created note
    Response::with_json(Into::<api::Note>::into(note))
}

pub(crate) fn notes_show(_req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Get note (admins can access any note, normal users only their own)
    let note = match fetch_note_for_user(_req, ctx, user) {
        Some(note) => note,
        None => return not_found(_req, ctx),
    };

    // Check authorization
    if !policies::can_show(user, &note) {
        return Response::with_status(Status::Forbidden);
    }

    // Return note
    Response::with_json(Into::<api::Note>::into(note))
}

#[derive(Validate, FromStruct)]
#[from_struct(api::NoteUpdateBody)]
struct NoteUpdateBody {
    #[validate(ascii, length(min = 1))]
    title: Option<String>,
    #[validate(ascii, length(min = 1))]
    body: String,
    is_pinned: bool,
    is_archived: bool,
    is_trashed: bool,
}

pub(crate) fn notes_update(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Get note (admins can access any note, normal users only their own)
    let mut note = match fetch_note_for_user(req, ctx, user) {
        Some(note) => note,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_update(user, &note) {
        return Response::with_status(Status::Forbidden);
    }

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
    note.title = body.title;
    note.body = body.body;
    note.is_pinned = body.is_pinned;
    note.is_archived = body.is_archived;
    note.is_trashed = body.is_trashed;
    note.updated_at = Utc::now();
    execute_args!(
        ctx.database,
        "UPDATE notes SET title = :title, body = :body, is_pinned = :is_pinned, is_archived = :is_archived, is_trashed = :is_trashed, updated_at = :updated_at WHERE id = :id",
        Args {
            title: note.title.clone(),
            body: note.body.clone(),
            is_pinned: note.is_pinned,
            is_archived: note.is_archived,
            is_trashed: note.is_trashed,
            updated_at: note.updated_at,
            id: note.id
        }
    ).expect("Database error");

    // Return updated note
    Response::with_json(Into::<api::Note>::into(note))
}

pub(crate) fn notes_delete(_req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Get note (admins can access any note, normal users only their own)
    let note = match fetch_note_for_user(_req, ctx, user) {
        Some(note) => note,
        None => return not_found(_req, ctx),
    };

    // Check authorization
    if !policies::can_delete(user, &note) {
        return Response::with_status(Status::Forbidden);
    }

    // Delete note
    ctx.database
        .execute("DELETE FROM notes WHERE id = ?", note.id)
        .expect("Database error");

    // Success response
    Response::new()
}

pub(crate) fn notes_pinned(req: &Request, ctx: &Context) -> Response {
    notes_filtered(req, ctx, "is_trashed = 0 AND is_pinned")
}

pub(crate) fn notes_archived(req: &Request, ctx: &Context) -> Response {
    notes_filtered(req, ctx, "is_trashed = 0 AND is_archived")
}

pub(crate) fn notes_trashed(req: &Request, ctx: &Context) -> Response {
    notes_filtered(req, ctx, "is_trashed")
}

// MARK: Utils
fn notes_filtered(req: &Request, ctx: &Context, filter: &str) -> Response {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Check authorization
    if !policies::can_index(user) {
        return Response::with_status(Status::Forbidden);
    }

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

    // Get filtered notes
    let search_query = format!("%{}%", query.query.replace("%", "\\%"));
    let (total, notes) = match user.role {
        UserRole::Admin => {
            // Admin sees all filtered notes
            let total = query_args!(
                i64,
                ctx.database,
                &format!(
                    "SELECT COUNT(id) FROM notes WHERE {filter} = 1 AND body LIKE :search_query"
                ),
                Args {
                    search_query: search_query.clone()
                }
            )
            .expect("Database error")
            .next()
            .map(|r| r.expect("Database error"))
            .unwrap_or(0);
            let notes = query_args!(
                Note,
                ctx.database,
                format!(
                    "SELECT {} FROM notes WHERE {} = 1 AND body LIKE :search_query ORDER BY updated_at DESC LIMIT :limit OFFSET :offset",
                    Note::columns(),
                    filter
                ),
                Args {
                    search_query: search_query,
                    limit: query.limit,
                    offset: (query.page - 1) * query.limit
                }
            )
            .expect("Database error")
            .map(|r| Into::<api::Note>::into(r.expect("Database error")))
            .collect::<Vec<_>>();
            (total, notes)
        }
        UserRole::Normal => {
            // Normal user sees only their own filtered notes
            let total = query_args!(
                i64,
                ctx.database,
                &format!("SELECT COUNT(id) FROM notes WHERE {filter} = 1 AND user_id = :user_id AND body LIKE :search_query"),
                Args {
                    user_id: user.id,
                    search_query: search_query.clone()
                }
            )
            .expect("Database error")
            .next()
            .map(|r| r.expect("Database error"))
            .unwrap_or(0);
            let notes = query_args!(
                Note,
                ctx.database,
                format!(
                    "SELECT {} FROM notes WHERE {} = 1 AND user_id = :user_id AND body LIKE :search_query ORDER BY updated_at DESC LIMIT :limit OFFSET :offset",
                    Note::columns(),
                    filter
                ),
                Args {
                    user_id: user.id,
                    search_query: search_query,
                    limit: query.limit,
                    offset: (query.page - 1) * query.limit
                }
            )
            .expect("Database error")
            .map(|r| Into::<api::Note>::into(r.expect("Database error")))
            .collect::<Vec<_>>();
            (total, notes)
        }
    };

    // Return filtered notes
    Response::with_json(api::NoteIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: notes,
    })
}

fn get_note_by_id(req: &Request, _ctx: &Context) -> Option<Uuid> {
    req.params
        .get("note_id")
        .expect("note_id param should be present")
        .parse::<Uuid>()
        .ok()
}

fn fetch_note_for_user(req: &Request, ctx: &Context, user: &User) -> Option<Note> {
    let note_id = get_note_by_id(req, ctx)?;

    match user.role {
        UserRole::Admin => {
            // Admin can fetch any note
            query_args!(
                Note,
                ctx.database,
                formatcp!(
                    "SELECT {} FROM notes WHERE id = :note_id LIMIT 1",
                    Note::columns()
                ),
                Args { note_id: note_id }
            )
            .expect("Database error")
            .next()
            .map(|r| r.expect("Database error"))
        }
        UserRole::Normal => {
            // Normal user can only fetch their own notes
            query_args!(
                Note,
                ctx.database,
                formatcp!(
                    "SELECT {} FROM notes WHERE id = :note_id AND user_id = :user_id LIMIT 1",
                    Note::columns()
                ),
                Args {
                    note_id: note_id,
                    user_id: user.id
                }
            )
            .expect("Database error")
            .next()
            .map(|r| r.expect("Database error"))
        }
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;
    use crate::models::Session;
    use crate::router;
    use crate::test_utils::{
        create_test_user_with_session, create_test_user_with_session_and_role,
    };

    #[test]
    fn test_notes_index() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Fetch /notes check if empty
        let res = router.handle(
            &Request::get("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert!(notes.is_empty());

        // Create note for authenticated user
        let note = Note {
            user_id: user.id,
            title: Some("My First Note".to_string()),
            body: "This is my first note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Fetch /notes check if note is there
        let res = router.handle(
            &Request::get("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {token}")),
        );
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
        let (user, token) = create_test_user_with_session(&ctx);

        // Create multiple notes
        ctx.database.insert_note(Note {
            user_id: user.id,
            title: Some("Meeting Notes".to_string()),
            body: "Meeting notes from today".to_string(),
            ..Default::default()
        });
        ctx.database.insert_note(Note {
            user_id: user.id,
            title: Some("Shopping List".to_string()),
            body: "Shopping list for tomorrow".to_string(),
            ..Default::default()
        });

        // Search for "meeting"
        let res = router.handle(
            &Request::get("http://localhost/api/notes?q=meeting")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].body, "Meeting notes from today");
    }

    #[test]
    fn test_notes_index_pagination() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create multiple notes
        for i in 1..=30 {
            ctx.database.insert_note(Note {
                user_id: user.id,
                title: Some(format!("Note {i}")),
                body: format!("Note number {i}"),
                ..Default::default()
            });
        }

        // Fetch /notes with limit 10 and page 1
        let res = router.handle(
            &Request::get("http://localhost/api/notes?limit=10&page=1")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 10);
        assert_eq!(response.pagination.page, 1);
        assert_eq!(response.pagination.limit, 10);
        assert_eq!(response.pagination.total, 30);

        // Fetch /notes with limit 5 and page 2
        let res = router.handle(
            &Request::get("http://localhost/api/notes?limit=5&page=2")
                .header("Authorization", format!("Bearer {token}")),
        );
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
        let (user, token) = create_test_user_with_session(&ctx);

        // Create note
        let res = router.handle(
            &Request::post("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {token}"))
                .body("title=Test+Note&body=This+is+a+new+note&isPinned=false"),
        );
        assert_eq!(res.status, Status::Ok);
        let note = serde_json::from_slice::<api::Note>(&res.body).unwrap();
        assert_eq!(note.title, Some("Test Note".to_string()));
        assert_eq!(note.body, "This is a new note");
        assert_eq!(note.user_id, user.id);
    }

    #[test]
    fn test_notes_show() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create note
        let note = Note {
            user_id: user.id,
            title: Some("Important".to_string()),
            body: "My important note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Fetch /notes/:note_id check if note is there
        let res = router.handle(
            &Request::get(format!("http://localhost/api/notes/{}", note.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let note = serde_json::from_slice::<api::Note>(&res.body).unwrap();
        assert_eq!(note.body, "My important note");
    }

    #[test]
    fn test_notes_show_not_found() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (_, token) = create_test_user_with_session(&ctx);

        // Fetch note by random id should be 404 Not Found
        let res = router.handle(
            &Request::get(format!("http://localhost/api/notes/{}", Uuid::now_v7()))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::NotFound);
    }

    #[test]
    fn test_notes_update() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create note
        let note = Note {
            user_id: user.id,
            title: Some("Original Title".to_string()),
            body: "Original note content".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Update note
        let res = router.handle(
            &Request::put(format!("http://localhost/api/notes/{}", note.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("title=Updated+Title&body=Updated+note+content&isPinned=false&isArchived=false&isTrashed=false"),
        );
        assert_eq!(res.status, Status::Ok);
        let note = serde_json::from_slice::<api::Note>(&res.body).unwrap();
        assert_eq!(note.title, Some("Updated Title".to_string()));
        assert_eq!(note.body, "Updated note content");
    }

    #[test]
    fn test_notes_update_validation_error() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create note
        let note = Note {
            user_id: user.id,
            title: Some("Original Title".to_string()),
            body: "Original note content".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Update note with validation errors (empty body)
        let res = router.handle(
            &Request::put(format!("http://localhost/api/notes/{}", note.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("title=Test&body=&isPinned=false&isArchived=false&isTrashed=false"),
        );
        assert_eq!(res.status, Status::BadRequest);
    }

    #[test]
    fn test_notes_delete() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create note
        let note = Note {
            user_id: user.id,
            title: Some("To Delete".to_string()),
            body: "Note to be deleted".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Delete note
        let res = router.handle(
            &Request::delete(format!("http://localhost/api/notes/{}", note.id))
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);

        // Fetch /notes check if empty
        let res = router.handle(
            &Request::get("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert!(notes.is_empty());
    }

    #[test]
    fn test_notes_index_admin_can_see_all_notes() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create admin user
        let (_admin, admin_token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create first normal user and their note
        let (user1, _) = create_test_user_with_session(&ctx);
        let user1_note = Note {
            user_id: user1.id,
            title: Some("User 1 Note".to_string()),
            body: "User 1's note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(user1_note);

        // Create second normal user and their note
        let (user2, _) = create_test_user_with_session(&ctx);
        let user2_note = Note {
            user_id: user2.id,
            title: Some("User 2 Note".to_string()),
            body: "User 2's note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(user2_note);

        // Admin should see all notes
        let res = router.handle(
            &Request::get("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {admin_token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(notes.len(), 2);
    }

    #[test]
    fn test_notes_show_admin_can_access_any_note() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create admin user
        let (_admin, admin_token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create normal user and their note
        let (user, _) = create_test_user_with_session(&ctx);
        let note = Note {
            user_id: user.id,
            title: Some("Private Note".to_string()),
            body: "User's private note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Admin should be able to access the user's note
        let res = router.handle(
            &Request::get(format!("http://localhost/api/notes/{}", note.id))
                .header("Authorization", format!("Bearer {admin_token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let fetched_note = serde_json::from_slice::<api::Note>(&res.body).unwrap();
        assert_eq!(fetched_note.id, note.id);
        assert_eq!(fetched_note.body, "User's private note");
    }

    #[test]
    fn test_notes_update_admin_can_update_any_note() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create admin user
        let (_admin, admin_token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create normal user and their note
        let (user, _) = create_test_user_with_session(&ctx);
        let note = Note {
            user_id: user.id,
            title: Some("Original Title".to_string()),
            body: "Original content".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Admin should be able to update the user's note
        let res = router.handle(
            &Request::put(format!("http://localhost/api/notes/{}", note.id))
                .header("Authorization", format!("Bearer {admin_token}"))
                .body("title=Admin+Title&body=Admin+updated+this&isPinned=false&isArchived=false&isTrashed=false"),
        );
        assert_eq!(res.status, Status::Ok);
        let updated_note = serde_json::from_slice::<api::Note>(&res.body).unwrap();
        assert_eq!(updated_note.body, "Admin updated this");
    }

    #[test]
    fn test_notes_delete_admin_can_delete_any_note() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create admin user
        let (_admin, admin_token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create normal user and their note
        let (user, _) = create_test_user_with_session(&ctx);
        let note = Note {
            user_id: user.id,
            title: Some("To Delete".to_string()),
            body: "Note to delete".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Admin should be able to delete the user's note
        let res = router.handle(
            &Request::delete(format!("http://localhost/api/notes/{}", note.id))
                .header("Authorization", format!("Bearer {admin_token}")),
        );
        assert_eq!(res.status, Status::Ok);

        // Verify note is deleted
        let res = router.handle(
            &Request::get(format!("http://localhost/api/notes/{}", note.id))
                .header("Authorization", format!("Bearer {admin_token}")),
        );
        assert_eq!(res.status, Status::NotFound);
    }

    #[test]
    fn test_notes_index_user_isolation() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create first user and their note
        let (user1, token1) = create_test_user_with_session(&ctx);
        let note1 = Note {
            user_id: user1.id,
            title: Some("User 1 Note".to_string()),
            body: "User 1's private note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note1.clone());

        // Create second user and their note
        let (user2, token2) = create_test_user_with_session(&ctx);

        let note2 = Note {
            user_id: user2.id,
            title: Some("User 2 Note".to_string()),
            body: "User 2's private note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note2.clone());

        // User 1 should only see their own note
        let res = router.handle(
            &Request::get("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {token1}")),
        );
        assert_eq!(res.status, Status::Ok);
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].body, "User 1's private note");
        assert_eq!(notes[0].user_id, user1.id);

        // User 2 should only see their own note
        let res = router.handle(
            &Request::get("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {token2}")),
        );
        assert_eq!(res.status, Status::Ok);
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].body, "User 2's private note");
        assert_eq!(notes[0].user_id, user2.id);
    }

    #[test]
    fn test_notes_show_user_cannot_access_other_user_note() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create first user and their note
        let (user1, token1) = create_test_user_with_session(&ctx);
        let note1 = Note {
            user_id: user1.id,
            title: Some("Private Note".to_string()),
            body: "User 1's private note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note1.clone());

        // Create second user
        let user2 = User {
            first_name: "User2".to_string(),
            last_name: "Test".to_string(),
            email: "user2@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            ..Default::default()
        };
        ctx.database.insert_user(user2.clone());
        let token2 = format!("test-token-{}", user2.id);
        let session2 = Session {
            user_id: user2.id,
            token: token2.clone(),
            expires_at: Utc::now() + Duration::from_secs(3600),
            ..Default::default()
        };
        ctx.database.insert_session(session2);

        // User 2 should not be able to access User 1's note
        let res = router.handle(
            &Request::get(format!("http://localhost/api/notes/{}", note1.id))
                .header("Authorization", format!("Bearer {token2}")),
        );
        assert_eq!(res.status, Status::NotFound);

        // User 1 should still be able to access their own note
        let res = router.handle(
            &Request::get(format!("http://localhost/api/notes/{}", note1.id))
                .header("Authorization", format!("Bearer {token1}")),
        );
        assert_eq!(res.status, Status::Ok);
    }

    #[test]
    fn test_notes_pinned() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create some notes with different states
        let pinned_note = Note {
            user_id: user.id,
            title: Some("Pinned Note".to_string()),
            body: "This is a pinned note".to_string(),
            is_pinned: true,
            ..Default::default()
        };
        ctx.database.insert_note(pinned_note.clone());

        let unpinned_note = Note {
            user_id: user.id,
            title: Some("Unpinned Note".to_string()),
            body: "This is an unpinned note".to_string(),
            is_pinned: false,
            ..Default::default()
        };
        ctx.database.insert_note(unpinned_note);

        // Fetch pinned notes
        let res = router.handle(
            &Request::get("http://localhost/api/notes/pinned")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert!(response.data[0].is_pinned);
        assert_eq!(response.data[0].body, "This is a pinned note");
    }

    #[test]
    fn test_notes_archived() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create archived and non-archived notes
        let archived_note = Note {
            user_id: user.id,
            title: Some("Archived Note".to_string()),
            body: "This is an archived note".to_string(),
            is_archived: true,
            ..Default::default()
        };
        ctx.database.insert_note(archived_note.clone());

        let active_note = Note {
            user_id: user.id,
            title: Some("Active Note".to_string()),
            body: "This is an active note".to_string(),
            is_archived: false,
            ..Default::default()
        };
        ctx.database.insert_note(active_note);

        // Fetch archived notes
        let res = router.handle(
            &Request::get("http://localhost/api/notes/archived")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert!(response.data[0].is_archived);
        assert_eq!(response.data[0].body, "This is an archived note");
    }

    #[test]
    fn test_notes_trashed() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create trashed and non-trashed notes
        let trashed_note = Note {
            user_id: user.id,
            title: Some("Trashed Note".to_string()),
            body: "This is a trashed note".to_string(),
            is_trashed: true,
            ..Default::default()
        };
        ctx.database.insert_note(trashed_note.clone());

        let kept_note = Note {
            user_id: user.id,
            title: Some("Kept Note".to_string()),
            body: "This is a kept note".to_string(),
            is_trashed: false,
            ..Default::default()
        };
        ctx.database.insert_note(kept_note);

        // Fetch trashed notes
        let res = router.handle(
            &Request::get("http://localhost/api/notes/trashed")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert!(response.data[0].is_trashed);
        assert_eq!(response.data[0].body, "This is a trashed note");
    }
}
