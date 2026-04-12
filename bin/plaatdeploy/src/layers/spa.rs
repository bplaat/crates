/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! SPA file server layer - serves embedded Preact app for non-/api paths

use anyhow::Result;
use rust_embed::Embed;
use small_http::{Request, Response};

use crate::context::Context;

#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
struct WebAssets;

pub(crate) fn spa_pre_layer(req: &Request, _: &mut Context) -> Option<Result<Response>> {
    let path = req.url.path();
    if path.starts_with("/api") {
        return None;
    }

    let mut asset_path = path.to_string();
    if asset_path.ends_with('/') {
        asset_path = format!("{asset_path}index.html");
    }

    // Try exact path, then fall back to index.html for SPA routing
    let (data, mime_path) = if let Some(file) = WebAssets::get(asset_path.trim_start_matches('/')) {
        (file.data, asset_path.clone())
    } else if let Some(file) = WebAssets::get("index.html") {
        (file.data, "index.html".to_string())
    } else {
        return None;
    };

    let mime = mime_guess::from_path(&mime_path).first_or_octet_stream();
    Some(Ok(Response::new()
        .header("Content-Type", mime.to_string())
        .body(data)))
}
