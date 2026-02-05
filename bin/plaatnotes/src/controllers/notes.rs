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
use crate::controllers::{get_auth_user, not_found};
use crate::models::note::policies;
use crate::models::user::UserRole;
use crate::models::{IndexQuery, Note, User};

pub(crate) fn notes_index(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let user = match get_auth_user(req, ctx) {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Check authorization
    if !policies::can_index(&user) {
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
                "SELECT COUNT(id) FROM notes WHERE body LIKE :search_query",
                Args {
                    search_query: search_query.clone()
                }
            )
            .next()
            .unwrap_or(0);
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
            (total, notes)
        }
        UserRole::Normal => {
            // Normal user sees only their own notes
            let total = query_args!(
                i64,
                ctx.database,
                "SELECT COUNT(id) FROM notes WHERE user_id = :user_id AND body LIKE :search_query",
                Args {
                    user_id: user.id,
                    search_query: search_query.clone()
                }
            )
            .next()
            .unwrap_or(0);
            let notes = query_args!(
                Note,
                ctx.database,
                formatcp!(
                    "SELECT {} FROM notes WHERE user_id = :user_id AND body LIKE :search_query ORDER BY updated_at DESC LIMIT :limit OFFSET :offset",
                    Note::columns()
                ),
                Args {
                    user_id: user.id,
                    search_query: search_query,
                    limit: query.limit,
                    offset: (query.page - 1) * query.limit
                }
            )
            .map(Into::<api::Note>::into)
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
    // Check authentication
    let user = match get_auth_user(req, ctx) {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Check authorization
    if !policies::can_create(&user) {
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
        body: body.body,
        ..Default::default()
    };
    ctx.database.insert_note(note.clone());

    // Return created note
    Response::with_json(Into::<api::Note>::into(note))
}

pub(crate) fn notes_show(req: &Request, ctx: &Context) -> Response {
    // Check authentication
    let user = match get_auth_user(req, ctx) {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Get note (admins can access any note, normal users only their own)
    let note = match fetch_note_for_user(req, ctx, &user) {
        Some(note) => note,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_show(&user, &note) {
        return Response::with_status(Status::Forbidden);
    }

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
    // Check authentication
    let user = match get_auth_user(req, ctx) {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Get note (admins can access any note, normal users only their own)
    let mut note = match fetch_note_for_user(req, ctx, &user) {
        Some(note) => note,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_update(&user, &note) {
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
    // Check authentication
    let user = match get_auth_user(req, ctx) {
        Some(user) => user,
        None => return Response::with_status(Status::Unauthorized),
    };

    // Get note (admins can access any note, normal users only their own)
    let note = match fetch_note_for_user(req, ctx, &user) {
        Some(note) => note,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_delete(&user, &note) {
        return Response::with_status(Status::Forbidden);
    }

    // Delete note
    ctx.database
        .execute("DELETE FROM notes WHERE id = ?", note.id);

    // Success response
    Response::new()
}

// MARK: Utils
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
            .next()
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
            .next()
        }
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::context::test_helpers::create_test_user_with_session;
    use crate::router;

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
            body: "Meeting notes from today".to_string(),
            ..Default::default()
        });
        ctx.database.insert_note(Note {
            user_id: user.id,
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
                .body("body=This+is+a+new+note"),
        );
        assert_eq!(res.status, Status::Ok);
        let note = serde_json::from_slice::<api::Note>(&res.body).unwrap();
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
            body: "Original note content".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Update note
        let res = router.handle(
            &Request::put(format!("http://localhost/api/notes/{}", note.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("body=Updated+note+content"),
        );
        assert_eq!(res.status, Status::Ok);
        let note = serde_json::from_slice::<api::Note>(&res.body).unwrap();
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
            body: "Original note content".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Update note with validation errors (empty body)
        let res = router.handle(
            &Request::put(format!("http://localhost/api/notes/{}", note.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("body="),
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
        use crate::context::test_helpers::create_test_user_with_session_and_role;
        use crate::models::user::UserRole;

        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create admin user
        let (_admin, admin_token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create first normal user and their note
        let (user1, _) = create_test_user_with_session(&ctx);
        let user1_note = Note {
            user_id: user1.id,
            body: "User 1's note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(user1_note);

        // Create second normal user and their note
        let (user2, _) = create_test_user_with_session(&ctx);
        let user2_note = Note {
            user_id: user2.id,
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
        use crate::context::test_helpers::create_test_user_with_session_and_role;
        use crate::models::user::UserRole;

        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create admin user
        let (_admin, admin_token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create normal user and their note
        let (user, _) = create_test_user_with_session(&ctx);
        let note = Note {
            user_id: user.id,
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
        use crate::context::test_helpers::create_test_user_with_session_and_role;
        use crate::models::user::UserRole;

        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create admin user
        let (_admin, admin_token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create normal user and their note
        let (user, _) = create_test_user_with_session(&ctx);
        let note = Note {
            user_id: user.id,
            body: "Original content".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone());

        // Admin should be able to update the user's note
        let res = router.handle(
            &Request::put(format!("http://localhost/api/notes/{}", note.id))
                .header("Authorization", format!("Bearer {admin_token}"))
                .body("body=Admin+updated+this"),
        );
        assert_eq!(res.status, Status::Ok);
        let updated_note = serde_json::from_slice::<api::Note>(&res.body).unwrap();
        assert_eq!(updated_note.body, "Admin updated this");
    }

    #[test]
    fn test_notes_delete_admin_can_delete_any_note() {
        use crate::context::test_helpers::create_test_user_with_session_and_role;
        use crate::models::user::UserRole;

        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create admin user
        let (_admin, admin_token) = create_test_user_with_session_and_role(&ctx, UserRole::Admin);

        // Create normal user and their note
        let (user, _) = create_test_user_with_session(&ctx);
        let note = Note {
            user_id: user.id,
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
            body: "User 1's private note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note1.clone());

        // Create second user and their note
        let (user2, token2) = create_test_user_with_session(&ctx);

        let note2 = Note {
            user_id: user2.id,
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
        use std::time::Duration;

        use crate::models::{Session, User};

        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        // Create first user and their note
        let (user1, token1) = create_test_user_with_session(&ctx);
        let note1 = Note {
            user_id: user1.id,
            body: "User 1's private note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note1.clone());

        // Create second user
        let user2 = User {
            first_name: "User2".to_string(),
            last_name: "Test".to_string(),
            email: "user2@example.com".to_string(),
            password: pbkdf2::password_hash("password123"),
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
}
