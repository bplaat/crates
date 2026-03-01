/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::io::{Cursor, Read};
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::context::{Context, DatabaseHelpers};
use crate::models::Note;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeepNote {
    title: Option<String>,
    #[serde(default)]
    text_content: String,
    #[serde(default)]
    is_pinned: bool,
    #[serde(default)]
    is_archived: bool,
    #[serde(default)]
    is_trashed: bool,
    created_timestamp_usec: i64,
    user_edited_timestamp_usec: i64,
}

fn import_note_from_json(json_bytes: &[u8], ctx: &Context, user_id: uuid::Uuid, count: &mut usize) {
    let keep_note: KeepNote = match serde_json::from_slice(json_bytes) {
        Ok(n) => n,
        Err(_) => return,
    };
    let title = keep_note.title.as_deref().unwrap_or("").trim().to_string();
    let body = keep_note.text_content.trim().to_string();
    if title.is_empty() && body.is_empty() {
        return;
    }
    let note = Note {
        user_id,
        title: if title.is_empty() { None } else { Some(title) },
        body,
        is_pinned: keep_note.is_pinned,
        is_archived: keep_note.is_archived,
        is_trashed: keep_note.is_trashed,
        created_at: DateTime::from_timestamp_secs(keep_note.created_timestamp_usec / 1_000_000)
            .unwrap_or_else(Utc::now),
        updated_at: DateTime::from_timestamp_secs(keep_note.user_edited_timestamp_usec / 1_000_000)
            .unwrap_or_else(Utc::now),
        ..Default::default()
    };
    ctx.database.insert_note(note);
    *count += 1;
}

/// Import all Google Keep notes from a zip archive in memory.
/// Returns the number of notes successfully imported.
pub(crate) fn import_from_zip_bytes(zip_bytes: &[u8], ctx: &Context, user_id: uuid::Uuid) -> usize {
    let mut count = 0;
    let mut archive = match zip::ZipArchive::new(Cursor::new(zip_bytes)) {
        Ok(a) => a,
        Err(_) => return 0,
    };
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let file_name = file.name().to_string();
        let parts: Vec<&str> = file_name.split('/').collect();
        let is_keep_json = parts
            .windows(2)
            .any(|w| w[0] == "Keep" && w[1].ends_with(".json"));
        if !is_keep_json {
            continue;
        }
        let mut json_bytes = Vec::new();
        if file.read_to_end(&mut json_bytes).is_err() {
            continue;
        }
        import_note_from_json(&json_bytes, ctx, user_id, &mut count);
    }
    count
}

pub(crate) fn run(path: &str, email: &str, ctx: &Context) {
    let user_id = ctx
        .database
        .query_some::<uuid::Uuid>("SELECT id FROM users WHERE email = ?", email.to_string())
        .unwrap_or_else(|_| panic!("No user found with email: {email}"));

    let mut count = 0;

    if path.ends_with(".zip") {
        let zip_bytes = std::fs::read(path).unwrap_or_else(|_| panic!("Can't read zip: {path}"));
        count = import_from_zip_bytes(&zip_bytes, ctx, user_id);
    } else {
        let keep_dir = {
            let keep_sub = Path::new(path).join("Keep");
            if keep_sub.is_dir() {
                keep_sub
            } else {
                Path::new(path).to_path_buf()
            }
        };
        let entries = std::fs::read_dir(&keep_dir)
            .unwrap_or_else(|_| panic!("Can't read directory: {}", keep_dir.display()));
        for entry in entries {
            let entry = entry.expect("Can't read directory entry");
            let entry_path = entry.path();
            if entry_path.extension().and_then(|e| e.to_str()) == Some("json") {
                let json_bytes = std::fs::read(&entry_path).expect("Can't read file");
                import_note_from_json(&json_bytes, ctx, user_id, &mut count);
            }
        }
    }

    println!("Imported {count} notes");
}
