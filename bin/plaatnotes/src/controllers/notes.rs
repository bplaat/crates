/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
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

pub(crate) fn notes_index(req: &Request, ctx: &Context) -> Result<Response> {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Ok(Response::with_status(Status::Unauthorized)),
    };

    // Check authorization
    if !policies::can_index(user) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    // Parse request query
    let query = match req.url.query() {
        Some(query) => match serde_urlencoded::from_str::<IndexQuery>(query) {
            Ok(query) => query,
            Err(_) => return Ok(Response::with_status(Status::BadRequest)),
        },
        None => IndexQuery::default(),
    };
    if let Err(report) = query.validate() {
        return Ok(
            Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report))
        );
    }

    // Get notes for authenticated user or all notes if admin
    let (total, notes) = match user.role {
        UserRole::Admin => {
            // Admin sees all notes
            if query.query.is_empty() {
                let total = ctx
                    .database
                    .query_some::<i64>(
                        "SELECT COUNT(id) FROM notes WHERE is_trashed = 0 AND is_archived = 0 AND is_pinned = 0",
                        (),
                    )?                    ;
                let notes = query_args!(
                    Note,
                    ctx.database,
                    formatcp!(
                        "SELECT {} FROM notes WHERE is_trashed = 0 AND is_archived = 0 AND is_pinned = 0
                        ORDER BY position ASC, updated_at DESC LIMIT :limit OFFSET :offset",
                        Note::columns()
                    ),
                    Args {
                        limit: query.limit,
                        offset: (query.page - 1) * query.limit
                    }
                )
                ?
                .map(|r| r.map(Into::into))
                .collect::<Result<Vec<_>, _>>()?;
                (total, notes)
            } else {
                let total = query_args!(
                    i64,
                    ctx.database,
                    "SELECT COUNT(id) FROM notes WHERE is_trashed = 0 AND is_archived = 0 AND is_pinned = 0 AND
                    id IN (SELECT id FROM notes_fts WHERE notes_fts MATCH :fts_query)",
                    Args { fts_query: query.query.clone() }
                )
                ?
                .next()
                .transpose()?
                .unwrap_or(0);
                let notes = query_args!(
                    Note,
                    ctx.database,
                    formatcp!(
                        "SELECT {} FROM notes WHERE is_trashed = 0 AND is_archived = 0 AND is_pinned = 0 AND
                        id IN (SELECT id FROM notes_fts WHERE notes_fts MATCH :fts_query)
                        ORDER BY position ASC, updated_at DESC LIMIT :limit OFFSET :offset",
                        Note::columns()
                    ),
                    Args {
                        fts_query: query.query,
                        limit: query.limit,
                        offset: (query.page - 1) * query.limit
                    }
                )
                ?
                .map(|r| r.map(Into::into))
                .collect::<Result<Vec<_>, _>>()?;
                (total, notes)
            }
        }
        UserRole::Normal => {
            // Normal user sees only their own notes
            if query.query.is_empty() {
                let total = query_args!(
                    i64,
                    ctx.database,
                    "SELECT COUNT(id) FROM notes WHERE is_trashed = 0 AND is_archived = 0 AND is_pinned = 0 AND user_id = :user_id",
                    Args { user_id: user.id }
                )
                ?
                .next()
                .transpose()?
                .unwrap_or(0);
                let notes = query_args!(
                    Note,
                    ctx.database,
                    formatcp!(
                        "SELECT {} FROM notes WHERE is_trashed = 0 AND is_archived = 0 AND is_pinned = 0 AND user_id = :user_id
                        ORDER BY position ASC, updated_at DESC LIMIT :limit OFFSET :offset",
                        Note::columns()
                    ),
                    Args {
                        user_id: user.id,
                        limit: query.limit,
                        offset: (query.page - 1) * query.limit
                    }
                )
                ?
                .map(|r| r.map(Into::into))
                .collect::<Result<Vec<_>, _>>()?;
                (total, notes)
            } else {
                let total = query_args!(
                    i64,
                    ctx.database,
                    "SELECT COUNT(id) FROM notes WHERE is_trashed = 0 AND is_archived = 0 AND is_pinned = 0 AND user_id = :user_id AND
                    id IN (SELECT id FROM notes_fts WHERE notes_fts MATCH :fts_query)",
                    Args { user_id: user.id, fts_query: query.query.clone() }
                )
                ?
                .next()
                .transpose()?
                .unwrap_or(0);
                let notes = query_args!(
                    Note,
                    ctx.database,
                    formatcp!(
                        "SELECT {} FROM notes WHERE is_trashed = 0 AND is_archived = 0 AND is_pinned = 0 AND user_id = :user_id AND
                        id IN (SELECT id FROM notes_fts WHERE notes_fts MATCH :fts_query)
                        ORDER BY position ASC, updated_at DESC LIMIT :limit OFFSET :offset",
                        Note::columns()
                    ),
                    Args {
                        user_id: user.id,
                        fts_query: query.query,
                        limit: query.limit,
                        offset: (query.page - 1) * query.limit
                    }
                )
                ?
                .map(|r| r.map(Into::into))
                .collect::<Result<Vec<_>, _>>()?;
                (total, notes)
            }
        }
    };

    // Return notes
    Ok(Response::with_json(api::NoteIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: notes,
    }))
}

#[derive(Validate, FromStruct)]
#[from_struct(api::NoteCreateBody)]
struct NoteCreateBody {
    #[validate(length(min = 1))]
    title: Option<String>,
    #[validate(length(min = 1))]
    body: String,
    is_pinned: Option<bool>,
}

pub(crate) fn notes_create(req: &Request, ctx: &Context) -> Result<Response> {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Ok(Response::with_status(Status::Unauthorized)),
    };

    // Check authorization
    if !policies::can_create(user) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::NoteCreateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => Into::<NoteCreateBody>::into(body),
        Err(_) => return Ok(Response::with_status(Status::BadRequest)),
    };
    if let Err(report) = body.validate() {
        return Ok(
            Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report))
        );
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
    ctx.database.insert_note(note.clone())?;

    // Return created note
    Ok(Response::with_json(Into::<api::Note>::into(note)))
}

pub(crate) fn notes_show(_req: &Request, ctx: &Context) -> Result<Response> {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Ok(Response::with_status(Status::Unauthorized)),
    };

    // Get note (admins can access any note, normal users only their own)
    let note = match fetch_note_for_user(_req, ctx, user)? {
        Some(note) => note,
        None => return not_found(_req, ctx),
    };

    // Check authorization
    if !policies::can_show(user, &note) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    // Return note
    Ok(Response::with_json(Into::<api::Note>::into(note)))
}

#[derive(Validate, FromStruct)]
#[from_struct(api::NoteUpdateBody)]
struct NoteUpdateBody {
    #[validate(length(min = 1))]
    title: Option<String>,
    #[validate(length(min = 1))]
    body: String,
    is_pinned: bool,
    is_archived: bool,
    is_trashed: bool,
}

pub(crate) fn notes_update(req: &Request, ctx: &Context) -> Result<Response> {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Ok(Response::with_status(Status::Unauthorized)),
    };

    // Get note (admins can access any note, normal users only their own)
    let mut note = match fetch_note_for_user(req, ctx, user)? {
        Some(note) => note,
        None => return not_found(req, ctx),
    };

    // Check authorization
    if !policies::can_update(user, &note) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    // Parse and validate body
    let body = match serde_urlencoded::from_bytes::<api::NoteUpdateBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => Into::<NoteUpdateBody>::into(body),
        Err(_) => return Ok(Response::with_status(Status::BadRequest)),
    };
    if let Err(report) = body.validate() {
        return Ok(
            Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report))
        );
    }

    // Update note
    let prev_is_archived = note.is_archived;
    let prev_is_trashed = note.is_trashed;
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
    )?;

    // When archiving or unarchiving, put the note first and shift all others
    if note.is_archived != prev_is_archived && !note.is_trashed {
        let filter = if note.is_archived {
            "is_archived = 1 AND is_trashed = 0"
        } else {
            "is_archived = 0 AND is_trashed = 0 AND is_pinned = 0"
        };
        execute_args!(
            ctx.database,
            &format!(
                "UPDATE notes SET position = position + 1 WHERE id != :id AND {filter} AND user_id = :user_id"
            ),
            Args {
                id: note.id,
                user_id: user.id
            }
        )?;
        execute_args!(
            ctx.database,
            "UPDATE notes SET position = 0 WHERE id = :id",
            Args { id: note.id }
        )?;
        note.position = 0;
    }

    // When trashing or untrashing, reset position in the destination category
    if note.is_trashed != prev_is_trashed {
        let filter = if note.is_trashed {
            // Trashed notes form their own ordered list
            "is_trashed = 1"
        } else if note.is_pinned {
            "is_trashed = 0 AND is_pinned = 1"
        } else if note.is_archived {
            "is_trashed = 0 AND is_archived = 1"
        } else {
            "is_trashed = 0 AND is_archived = 0 AND is_pinned = 0"
        };
        execute_args!(
            ctx.database,
            &format!(
                "UPDATE notes SET position = position + 1 WHERE id != :id AND {filter} AND user_id = :user_id"
            ),
            Args {
                id: note.id,
                user_id: user.id
            }
        )?;
        execute_args!(
            ctx.database,
            "UPDATE notes SET position = 0 WHERE id = :id",
            Args { id: note.id }
        )?;
        note.position = 0;
    }

    // Return updated note
    Ok(Response::with_json(Into::<api::Note>::into(note)))
}

pub(crate) fn notes_delete(_req: &Request, ctx: &Context) -> Result<Response> {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Ok(Response::with_status(Status::Unauthorized)),
    };

    // Get note (admins can access any note, normal users only their own)
    let note = match fetch_note_for_user(_req, ctx, user)? {
        Some(note) => note,
        None => return not_found(_req, ctx),
    };

    // Check authorization
    if !policies::can_delete(user, &note) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    // Delete note
    ctx.database
        .execute("DELETE FROM notes WHERE id = ?", note.id)?;

    // Success response
    Ok(Response::new())
}

pub(crate) fn notes_pinned(req: &Request, ctx: &Context) -> Result<Response> {
    notes_filtered(req, ctx, "is_trashed = 0 AND is_pinned")
}

pub(crate) fn notes_archived(req: &Request, ctx: &Context) -> Result<Response> {
    notes_filtered(req, ctx, "is_trashed = 0 AND is_archived")
}

pub(crate) fn notes_trashed(req: &Request, ctx: &Context) -> Result<Response> {
    notes_filtered(req, ctx, "is_trashed")
}

pub(crate) fn notes_trashed_clear(_req: &Request, ctx: &Context) -> Result<Response> {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Ok(Response::with_status(Status::Unauthorized)),
    };

    // Delete all trashed notes for this user (admins delete all, normal users only their own)
    match user.role {
        UserRole::Admin => {
            ctx.database
                .execute("DELETE FROM notes WHERE is_trashed = 1", ())?;
        }
        UserRole::Normal => {
            execute_args!(
                ctx.database,
                "DELETE FROM notes WHERE is_trashed = 1 AND user_id = :user_id",
                Args { user_id: user.id }
            )?;
        }
    }

    Ok(Response::new())
}

// MARK: Utils
fn notes_filtered(req: &Request, ctx: &Context, filter: &str) -> Result<Response> {
    // Check authentication
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Ok(Response::with_status(Status::Unauthorized)),
    };

    // Check authorization
    if !policies::can_index(user) {
        return Ok(Response::with_status(Status::Forbidden));
    }

    // Parse request query
    let query = match req.url.query() {
        Some(query) => match serde_urlencoded::from_str::<IndexQuery>(query) {
            Ok(query) => query,
            Err(_) => return Ok(Response::with_status(Status::BadRequest)),
        },
        None => IndexQuery::default(),
    };
    if let Err(report) = query.validate() {
        return Ok(
            Response::with_status(Status::BadRequest).json(Into::<api::Report>::into(report))
        );
    }

    // Get filtered notes
    let (total, notes) = match user.role {
        UserRole::Admin => {
            // Admin sees all filtered notes
            if query.query.is_empty() {
                let total = ctx.database.query_some::<i64>(
                    &format!("SELECT COUNT(id) FROM notes WHERE {filter} = 1"),
                    (),
                )?;
                let notes = query_args!(
                    Note,
                    ctx.database,
                    format!(
                        "SELECT {} FROM notes WHERE {filter} = 1 ORDER BY position ASC, updated_at DESC LIMIT :limit OFFSET :offset",
                        Note::columns()
                    ),
                    Args {
                        limit: query.limit,
                        offset: (query.page - 1) * query.limit
                    }
                )
                ?
                .map(|r| r.map(Into::into))
                .collect::<Result<Vec<_>, _>>()?;
                (total, notes)
            } else {
                let total = query_args!(
                    i64,
                    ctx.database,
                    &format!("SELECT COUNT(id) FROM notes WHERE {filter} = 1 AND id IN (SELECT id FROM notes_fts WHERE notes_fts MATCH :fts_query)"),
                    Args { fts_query: query.query.clone() }
                )
                ?
                .next()
                .transpose()?
                .unwrap_or(0);
                let notes = query_args!(
                    Note,
                    ctx.database,
                    format!(
                        "SELECT {} FROM notes WHERE {filter} = 1 AND id IN (SELECT id FROM notes_fts WHERE notes_fts MATCH :fts_query)
                        ORDER BY position ASC, updated_at DESC LIMIT :limit OFFSET :offset",
                        Note::columns()
                    ),
                    Args {
                        fts_query: query.query,
                        limit: query.limit,
                        offset: (query.page - 1) * query.limit
                    }
                )
                ?
                .map(|r| r.map(Into::into))
                .collect::<Result<Vec<_>, _>>()?;
                (total, notes)
            }
        }
        UserRole::Normal => {
            // Normal user sees only their own filtered notes
            if query.query.is_empty() {
                let total = query_args!(
                    i64,
                    ctx.database,
                    &format!(
                        "SELECT COUNT(id) FROM notes WHERE {filter} = 1 AND user_id = :user_id"
                    ),
                    Args { user_id: user.id }
                )?
                .next()
                .transpose()?
                .unwrap_or(0);
                let notes = query_args!(
                    Note,
                    ctx.database,
                    format!(
                        "SELECT {} FROM notes WHERE {filter} = 1 AND user_id = :user_id
                        ORDER BY position ASC, updated_at DESC LIMIT :limit OFFSET :offset",
                        Note::columns()
                    ),
                    Args {
                        user_id: user.id,
                        limit: query.limit,
                        offset: (query.page - 1) * query.limit
                    }
                )?
                .map(|r| r.map(Into::into))
                .collect::<Result<Vec<_>, _>>()?;
                (total, notes)
            } else {
                let total = query_args!(
                    i64,
                    ctx.database,
                    &format!(
                        "SELECT COUNT(id) FROM notes WHERE {filter} = 1 AND user_id = :user_id AND
                        id IN (SELECT id FROM notes_fts WHERE notes_fts MATCH :fts_query)"
                    ),
                    Args {
                        user_id: user.id,
                        fts_query: query.query.clone()
                    }
                )?
                .next()
                .transpose()?
                .unwrap_or(0);
                let notes = query_args!(
                    Note,
                    ctx.database,
                    format!(
                        "SELECT {} FROM notes WHERE {filter} = 1 AND user_id = :user_id AND
                        id IN (SELECT id FROM notes_fts WHERE notes_fts MATCH :fts_query)
                        ORDER BY position ASC, updated_at DESC LIMIT :limit OFFSET :offset",
                        Note::columns()
                    ),
                    Args {
                        user_id: user.id,
                        fts_query: query.query,
                        limit: query.limit,
                        offset: (query.page - 1) * query.limit
                    }
                )?
                .map(|r| r.map(Into::into))
                .collect::<Result<Vec<_>, _>>()?;
                (total, notes)
            }
        }
    };

    // Return filtered notes
    Ok(Response::with_json(api::NoteIndexResponse {
        pagination: api::Pagination {
            page: query.page,
            limit: query.limit,
            total,
        },
        data: notes,
    }))
}

fn get_note_by_id(req: &Request, _ctx: &Context) -> Option<Uuid> {
    req.params
        .get("note_id")
        .expect("note_id param should be present")
        .parse::<Uuid>()
        .ok()
}

fn fetch_note_for_user(req: &Request, ctx: &Context, user: &User) -> Result<Option<Note>> {
    let note_id = match get_note_by_id(req, ctx) {
        Some(id) => id,
        None => return Ok(None),
    };

    match user.role {
        UserRole::Admin => {
            // Admin can fetch any note
            Ok(query_args!(
                Note,
                ctx.database,
                formatcp!(
                    "SELECT {} FROM notes WHERE id = :note_id LIMIT 1",
                    Note::columns()
                ),
                Args { note_id: note_id }
            )?
            .next()
            .transpose()?)
        }
        UserRole::Normal => {
            // Normal user can only fetch their own notes
            Ok(query_args!(
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
            )?
            .next()
            .transpose()?)
        }
    }
}

fn notes_reorder_for(ctx: &Context, user: &User, ids_str: &str, filter: &str) -> Result<()> {
    // Parse provided IDs in order (these are assigned positions 0, 1, 2, …)
    let provided_ids: Vec<Uuid> = ids_str
        .split(',')
        .filter_map(|s| s.trim().parse::<Uuid>().ok())
        .collect();

    // Fetch all note IDs in this category ordered by current position
    let all_ids: Vec<Uuid> = query_args!(
        Note,
        ctx.database,
        format!(
            "SELECT {} FROM notes WHERE {filter} AND user_id = :user_id ORDER BY position ASC, updated_at DESC",
            Note::columns()
        ),
        Args { user_id: user.id }
    )?
    .filter_map(|r| r.ok())
    .map(|n| n.id)
    .collect();

    // Notes not in the provided list follow in their existing relative order
    let rest_ids: Vec<Uuid> = all_ids
        .into_iter()
        .filter(|id| !provided_ids.contains(id))
        .collect();

    // Final sequence: provided notes first (in given order), then the rest
    for (position, note_id) in provided_ids
        .into_iter()
        .chain(rest_ids.into_iter())
        .enumerate()
    {
        execute_args!(
            ctx.database,
            "UPDATE notes SET position = :position WHERE id = :note_id AND user_id = :user_id",
            Args {
                position: position as i64,
                note_id: note_id,
                user_id: user.id,
            }
        )?;
    }
    Ok(())
}

fn notes_reorder_handler(req: &Request, ctx: &Context, filter: &str) -> Result<Response> {
    let user = match &ctx.auth_user {
        Some(user) => user,
        None => return Ok(Response::with_status(Status::Unauthorized)),
    };
    let body = match serde_urlencoded::from_bytes::<api::NoteReorderBody>(
        req.body.as_deref().unwrap_or(&[]),
    ) {
        Ok(body) => body,
        Err(_) => return Ok(Response::with_status(Status::BadRequest)),
    };
    notes_reorder_for(ctx, user, &body.ids, filter)?;
    Ok(Response::with_status(Status::NoContent))
}

pub(crate) fn notes_reorder(req: &Request, ctx: &Context) -> Result<Response> {
    notes_reorder_handler(
        req,
        ctx,
        "is_trashed = 0 AND is_archived = 0 AND is_pinned = 0",
    )
}

pub(crate) fn notes_pinned_reorder(req: &Request, ctx: &Context) -> Result<Response> {
    notes_reorder_handler(req, ctx, "is_trashed = 0 AND is_pinned = 1")
}

pub(crate) fn notes_archived_reorder(req: &Request, ctx: &Context) -> Result<Response> {
    notes_reorder_handler(req, ctx, "is_trashed = 0 AND is_archived = 1")
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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_note(note.clone()).unwrap();

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
    fn test_notes_index_excludes_pinned() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create a regular note and a pinned note
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Regular Note".to_string()),
                body: "Regular".to_string(),
                is_pinned: false,
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Pinned Note".to_string()),
                body: "Pinned".to_string(),
                is_pinned: true,
                ..Default::default()
            })
            .unwrap();

        // /notes should only return the non-pinned note
        let res = router.handle(
            &Request::get("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].body, "Regular");
    }

    #[test]
    fn test_notes_index_search() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create multiple notes
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Meeting Notes".to_string()),
                body: "Meeting notes from today".to_string(),
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Shopping List".to_string()),
                body: "Shopping list for tomorrow".to_string(),
                ..Default::default()
            })
            .unwrap();

        // Search for "meeting" finds by body
        let res = router.handle(
            &Request::get("http://localhost/api/notes?q=meeting")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].body, "Meeting notes from today");

        // Search for "Shopping" finds by title
        let res = router.handle(
            &Request::get("http://localhost/api/notes?q=Shopping")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].title, Some("Shopping List".to_string()));
    }

    #[test]
    fn test_notes_index_search_by_title_only() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Note with a unique title but generic body
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("ProjectAlpha".to_string()),
                body: "Some content".to_string(),
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: None,
                body: "Some other content".to_string(),
                ..Default::default()
            })
            .unwrap();

        // "ProjectAlpha" only appears in the title of the first note
        let res = router.handle(
            &Request::get("http://localhost/api/notes?q=ProjectAlpha")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].title, Some("ProjectAlpha".to_string()));
    }

    #[test]
    fn test_notes_index_fts5_search() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Alice Smith".to_string()),
                body: "Notes from Alice".to_string(),
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Alice Johnson".to_string()),
                body: "Notes from Alice".to_string(),
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Bob Smith".to_string()),
                body: "Notes from Bob".to_string(),
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Carol White".to_string()),
                body: "Notes from Carol".to_string(),
                ..Default::default()
            })
            .unwrap();

        // Prefix search
        let res = router.handle(
            &Request::get("http://localhost/api/notes?q=Al*")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 2);

        // AND search
        let res = router.handle(
            &Request::get("http://localhost/api/notes?q=Alice AND Smith")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].title, Some("Alice Smith".to_string()));

        // OR search
        let res = router.handle(
            &Request::get("http://localhost/api/notes?q=Alice OR Bob")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 3);

        // NOT search
        let res = router.handle(
            &Request::get("http://localhost/api/notes?q=Alice NOT Smith")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].title, Some("Alice Johnson".to_string()));

        // Phrase search
        let res = router.handle(
            &Request::get(r#"http://localhost/api/notes?q="Alice Smith""#)
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].title, Some("Alice Smith".to_string()));

        // Column-scoped search (body field only)
        let res = router.handle(
            &Request::get("http://localhost/api/notes?q=body:Carol")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].title, Some("Carol White".to_string()));
    }

    #[test]
    fn test_notes_index_pagination() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create multiple notes
        for i in 1..=30 {
            ctx.database
                .insert_note(Note {
                    user_id: user.id,
                    title: Some(format!("Note {i}")),
                    body: format!("Note number {i}"),
                    ..Default::default()
                })
                .unwrap();
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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create note
        let note = Note {
            user_id: user.id,
            title: Some("Important".to_string()),
            body: "My important note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone()).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create note
        let note = Note {
            user_id: user.id,
            title: Some("Original Title".to_string()),
            body: "Original note content".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone()).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create note
        let note = Note {
            user_id: user.id,
            title: Some("Original Title".to_string()),
            body: "Original note content".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone()).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create note
        let note = Note {
            user_id: user.id,
            title: Some("To Delete".to_string()),
            body: "Note to be deleted".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note.clone()).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_note(user1_note).unwrap();

        // Create second normal user and their note
        let (user2, _) = create_test_user_with_session(&ctx);
        let user2_note = Note {
            user_id: user2.id,
            title: Some("User 2 Note".to_string()),
            body: "User 2's note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(user2_note).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_note(note.clone()).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_note(note.clone()).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_note(note.clone()).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());

        // Create first user and their note
        let (user1, token1) = create_test_user_with_session(&ctx);
        let note1 = Note {
            user_id: user1.id,
            title: Some("User 1 Note".to_string()),
            body: "User 1's private note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note1.clone()).unwrap();

        // Create second user and their note
        let (user2, token2) = create_test_user_with_session(&ctx);

        let note2 = Note {
            user_id: user2.id,
            title: Some("User 2 Note".to_string()),
            body: "User 2's private note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note2.clone()).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());

        // Create first user and their note
        let (user1, token1) = create_test_user_with_session(&ctx);
        let note1 = Note {
            user_id: user1.id,
            title: Some("Private Note".to_string()),
            body: "User 1's private note".to_string(),
            ..Default::default()
        };
        ctx.database.insert_note(note1.clone()).unwrap();

        // Create second user
        let user2 = User {
            first_name: "User2".to_string(),
            last_name: "Test".to_string(),
            email: "user2@example.com".to_string(),
            password: crate::test_utils::TEST_PASSWORD_HASH.to_string(),
            ..Default::default()
        };
        ctx.database.insert_user(user2.clone()).unwrap();
        let token2 = format!("test-token-{}", user2.id);
        let session2 = Session {
            user_id: user2.id,
            token: token2.clone(),
            expires_at: Utc::now() + Duration::from_secs(3600),
            ..Default::default()
        };
        ctx.database.insert_session(session2).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_note(pinned_note.clone()).unwrap();

        let unpinned_note = Note {
            user_id: user.id,
            title: Some("Unpinned Note".to_string()),
            body: "This is an unpinned note".to_string(),
            is_pinned: false,
            ..Default::default()
        };
        ctx.database.insert_note(unpinned_note).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_note(archived_note.clone()).unwrap();

        let active_note = Note {
            user_id: user.id,
            title: Some("Active Note".to_string()),
            body: "This is an active note".to_string(),
            is_archived: false,
            ..Default::default()
        };
        ctx.database.insert_note(active_note).unwrap();

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
        let ctx = Context::with_test_database().expect("Can't create test database");
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
        ctx.database.insert_note(trashed_note.clone()).unwrap();

        let kept_note = Note {
            user_id: user.id,
            title: Some("Kept Note".to_string()),
            body: "This is a kept note".to_string(),
            is_trashed: false,
            ..Default::default()
        };
        ctx.database.insert_note(kept_note).unwrap();

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

    #[test]
    fn test_notes_pinned_search() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create two pinned notes with different content
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Pinned Alpha".to_string()),
                body: "Content about alpha".to_string(),
                is_pinned: true,
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Pinned Beta".to_string()),
                body: "Content about beta".to_string(),
                is_pinned: true,
                ..Default::default()
            })
            .unwrap();
        // A non-pinned note that also matches the query – must not appear
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Unpinned Alpha".to_string()),
                body: "Content about alpha".to_string(),
                is_pinned: false,
                ..Default::default()
            })
            .unwrap();

        // ?q=alpha should return only the pinned alpha note
        let res = router.handle(
            &Request::get("http://localhost/api/notes/pinned?q=alpha")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert!(response.data[0].is_pinned);
        assert_eq!(response.data[0].title, Some("Pinned Alpha".to_string()));

        // ?q=beta should return only the pinned beta note
        let res = router.handle(
            &Request::get("http://localhost/api/notes/pinned?q=beta")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].title, Some("Pinned Beta".to_string()));

        // Empty query returns all pinned notes
        let res = router.handle(
            &Request::get("http://localhost/api/notes/pinned")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 2);
    }

    #[test]
    fn test_notes_archived_search() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create two archived notes with different content
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Archived Recipe".to_string()),
                body: "How to bake bread".to_string(),
                is_archived: true,
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Archived Travel".to_string()),
                body: "Trip to Paris".to_string(),
                is_archived: true,
                ..Default::default()
            })
            .unwrap();
        // Non-archived note that also matches – must not appear
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Active Recipe".to_string()),
                body: "How to bake bread".to_string(),
                is_archived: false,
                ..Default::default()
            })
            .unwrap();

        // ?q=recipe should return only the archived recipe note
        let res = router.handle(
            &Request::get("http://localhost/api/notes/archived?q=recipe")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert!(response.data[0].is_archived);
        assert_eq!(response.data[0].title, Some("Archived Recipe".to_string()));

        // ?q=paris finds by body content
        let res = router.handle(
            &Request::get("http://localhost/api/notes/archived?q=paris")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].title, Some("Archived Travel".to_string()));

        // No match returns empty list
        let res = router.handle(
            &Request::get("http://localhost/api/notes/archived?q=zzznomatch")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 0);
    }

    #[test]
    fn test_notes_trashed_search() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create two trashed notes with different content
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Trashed Invoice".to_string()),
                body: "Invoice for January".to_string(),
                is_trashed: true,
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Trashed Draft".to_string()),
                body: "Draft blog post".to_string(),
                is_trashed: true,
                ..Default::default()
            })
            .unwrap();
        // Non-trashed note that also matches – must not appear
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Active Invoice".to_string()),
                body: "Invoice for February".to_string(),
                is_trashed: false,
                ..Default::default()
            })
            .unwrap();

        // ?q=invoice should return only the trashed invoice
        let res = router.handle(
            &Request::get("http://localhost/api/notes/trashed?q=invoice")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert!(response.data[0].is_trashed);
        assert_eq!(response.data[0].title, Some("Trashed Invoice".to_string()));

        // ?q=draft finds by body
        let res = router.handle(
            &Request::get("http://localhost/api/notes/trashed?q=draft")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].title, Some("Trashed Draft".to_string()));

        // No match returns empty list
        let res = router.handle(
            &Request::get("http://localhost/api/notes/trashed?q=zzznomatch")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 0);
    }

    #[test]
    fn test_notes_trashed_clear() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create two trashed notes and one non-trashed note
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Trashed A".to_string()),
                body: "First trashed note".to_string(),
                is_trashed: true,
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Trashed B".to_string()),
                body: "Second trashed note".to_string(),
                is_trashed: true,
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user.id,
                title: Some("Active".to_string()),
                body: "Non-trashed note".to_string(),
                is_trashed: false,
                ..Default::default()
            })
            .unwrap();

        // Clear trash
        let res = router.handle(
            &Request::delete("http://localhost/api/notes/trashed/clear")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);

        // Trashed endpoint should now be empty
        let res = router.handle(
            &Request::get("http://localhost/api/notes/trashed")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert!(response.data.is_empty());

        // Non-trashed note must still exist
        let res = router.handle(
            &Request::get("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let response = serde_json::from_slice::<api::NoteIndexResponse>(&res.body).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].title, Some("Active".to_string()));
    }

    #[test]
    fn test_notes_trashed_clear_only_own_notes() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user1, token1) = create_test_user_with_session(&ctx);
        let (user2, _token2) = create_test_user_with_session(&ctx);

        // Create a trashed note for user1 and user2
        ctx.database
            .insert_note(Note {
                user_id: user1.id,
                title: Some("User1 Trashed".to_string()),
                body: "User1 trashed note".to_string(),
                is_trashed: true,
                ..Default::default()
            })
            .unwrap();
        ctx.database
            .insert_note(Note {
                user_id: user2.id,
                title: Some("User2 Trashed".to_string()),
                body: "User2 trashed note".to_string(),
                is_trashed: true,
                ..Default::default()
            })
            .unwrap();

        // User1 clears their trash
        let res = router.handle(
            &Request::delete("http://localhost/api/notes/trashed/clear")
                .header("Authorization", format!("Bearer {token1}")),
        );
        assert_eq!(res.status, Status::Ok);

        // User2's trashed note must still exist in the database
        let remaining: i64 = ctx
            .database
            .query("SELECT COUNT(id) FROM notes WHERE is_trashed = 1", ())
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        assert_eq!(remaining, 1);
    }

    #[test]
    fn test_notes_reorder() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create three notes
        let note1 = Note {
            user_id: user.id,
            title: Some("Note 1".to_string()),
            body: "First note".to_string(),
            position: 0,
            ..Default::default()
        };
        ctx.database.insert_note(note1.clone()).unwrap();
        let note2 = Note {
            user_id: user.id,
            title: Some("Note 2".to_string()),
            body: "Second note".to_string(),
            position: 1,
            ..Default::default()
        };
        ctx.database.insert_note(note2.clone()).unwrap();
        let note3 = Note {
            user_id: user.id,
            title: Some("Note 3".to_string()),
            body: "Third note".to_string(),
            position: 2,
            ..Default::default()
        };
        ctx.database.insert_note(note3.clone()).unwrap();

        // Reorder notes: 3, 1, 2
        let ids = format!("{},{},{}", note3.id, note1.id, note2.id);
        let res = router.handle(
            &Request::put("http://localhost/api/notes/reorder")
                .header("Authorization", format!("Bearer {token}"))
                .body(format!("ids={ids}")),
        );
        assert_eq!(res.status, Status::NoContent);

        // Fetch notes and verify order
        let res = router.handle(
            &Request::get("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(notes.len(), 3);
        assert_eq!(notes[0].id, note3.id);
        assert_eq!(notes[1].id, note1.id);
        assert_eq!(notes[2].id, note2.id);
    }

    #[test]
    fn test_notes_reorder_partial() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        // Create four notes (simulates two loaded pages of 2)
        let note1 = Note {
            user_id: user.id,
            title: Some("Note 1".to_string()),
            body: "First".to_string(),
            position: 0,
            ..Default::default()
        };
        ctx.database.insert_note(note1.clone()).unwrap();
        let note2 = Note {
            user_id: user.id,
            title: Some("Note 2".to_string()),
            body: "Second".to_string(),
            position: 1,
            ..Default::default()
        };
        ctx.database.insert_note(note2.clone()).unwrap();
        let note3 = Note {
            user_id: user.id,
            title: Some("Note 3".to_string()),
            body: "Third".to_string(),
            position: 2,
            ..Default::default()
        };
        ctx.database.insert_note(note3.clone()).unwrap();
        let note4 = Note {
            user_id: user.id,
            title: Some("Note 4".to_string()),
            body: "Fourth".to_string(),
            position: 3,
            ..Default::default()
        };
        ctx.database.insert_note(note4.clone()).unwrap();

        // User reorders only the first "page" (note1, note2) swapping them to (note2, note1)
        let ids = format!("{},{}", note2.id, note1.id);
        let res = router.handle(
            &Request::put("http://localhost/api/notes/reorder")
                .header("Authorization", format!("Bearer {token}"))
                .body(format!("ids={ids}")),
        );
        assert_eq!(res.status, Status::NoContent);

        // Full list should be: note2, note1, note3, note4 (unloaded notes keep relative order)
        let res = router.handle(
            &Request::get("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {token}")),
        );
        assert_eq!(res.status, Status::Ok);
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(notes.len(), 4);
        assert_eq!(notes[0].id, note2.id);
        assert_eq!(notes[1].id, note1.id);
        assert_eq!(notes[2].id, note3.id);
        assert_eq!(notes[3].id, note4.id);
    }

    #[test]
    fn test_notes_reorder_unauthenticated() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());

        let res = router.handle(&Request::put("http://localhost/api/notes/reorder").body("ids="));
        assert_eq!(res.status, Status::Unauthorized);
    }

    #[test]
    fn test_notes_reorder_ignores_other_users_notes() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());

        let (user1, token1) = create_test_user_with_session(&ctx);
        let (user2, _token2) = create_test_user_with_session(&ctx);

        let note1 = Note {
            user_id: user1.id,
            body: "User 1 note".to_string(),
            position: 0,
            ..Default::default()
        };
        ctx.database.insert_note(note1.clone()).unwrap();
        let note2 = Note {
            user_id: user2.id,
            body: "User 2 note".to_string(),
            position: 0,
            ..Default::default()
        };
        ctx.database.insert_note(note2.clone()).unwrap();

        // User 1 tries to include user 2's note in reorder — should be silently ignored
        let ids = format!("{},{}", note2.id, note1.id);
        let res = router.handle(
            &Request::put("http://localhost/api/notes/reorder")
                .header("Authorization", format!("Bearer {token1}"))
                .body(format!("ids={ids}")),
        );
        assert_eq!(res.status, Status::NoContent);

        // User 1's note position should have changed (position 1 since note2.id is first but ignored)
        let res = router.handle(
            &Request::get("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {token1}")),
        );
        let notes = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, note1.id);
    }

    #[test]
    fn test_notes_reorder_pinned_category() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        let normal1 = Note {
            user_id: user.id,
            body: "Normal 1".to_string(),
            position: 0,
            ..Default::default()
        };
        ctx.database.insert_note(normal1.clone()).unwrap();
        let pinned1 = Note {
            user_id: user.id,
            body: "Pinned 1".to_string(),
            is_pinned: true,
            position: 0,
            ..Default::default()
        };
        ctx.database.insert_note(pinned1.clone()).unwrap();
        let pinned2 = Note {
            user_id: user.id,
            body: "Pinned 2".to_string(),
            is_pinned: true,
            position: 1,
            ..Default::default()
        };
        ctx.database.insert_note(pinned2.clone()).unwrap();

        // Reorder only pinned: swap pinned2 before pinned1
        let ids = format!("{},{}", pinned2.id, pinned1.id);
        let res = router.handle(
            &Request::put("http://localhost/api/notes/pinned/reorder")
                .header("Authorization", format!("Bearer {token}"))
                .body(format!("ids={ids}")),
        );
        assert_eq!(res.status, Status::NoContent);

        // Pinned list order should reflect the reorder
        let res = router.handle(
            &Request::get("http://localhost/api/notes/pinned")
                .header("Authorization", format!("Bearer {token}")),
        );
        let pinned = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(pinned.len(), 2);
        assert_eq!(pinned[0].id, pinned2.id);
        assert_eq!(pinned[1].id, pinned1.id);

        // Normal list should be unaffected
        let res = router.handle(
            &Request::get("http://localhost/api/notes")
                .header("Authorization", format!("Bearer {token}")),
        );
        let normal = serde_json::from_slice::<api::NoteIndexResponse>(&res.body)
            .unwrap()
            .data;
        assert_eq!(normal.len(), 1);
        assert_eq!(normal[0].id, normal1.id);
    }

    #[test]
    fn test_notes_trash_resets_position() {
        let ctx = Context::with_test_database().expect("Can't create test database");
        let router = router(ctx.clone());
        let (user, token) = create_test_user_with_session(&ctx);

        let note1 = Note {
            user_id: user.id,
            body: "Note 1".to_string(),
            position: 5,
            ..Default::default()
        };
        ctx.database.insert_note(note1.clone()).unwrap();

        // Trash the note
        let res = router.handle(
            &Request::put(format!("http://localhost/api/notes/{}", note1.id))
                .header("Authorization", format!("Bearer {token}"))
                .body("body=Note+1&isPinned=false&isArchived=false&isTrashed=true"),
        );
        assert_eq!(res.status, Status::Ok);
        let updated = serde_json::from_slice::<api::Note>(&res.body).unwrap();
        assert_eq!(updated.position, 0);
    }
}
