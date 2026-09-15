/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use rust_embed::Embed;
use small_http::{Request, Response};

use crate::Context;

#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
struct WebAssets;

const CSP_VALUE: &str = concat!(
    "default-src 'self'; ",
    "script-src 'self'; ",
    "script-src-attr 'none'; ",
    "style-src 'self'; ",
    "style-src-attr 'none'; ",
    "img-src 'self' data:; ",
    "connect-src 'self'; ",
    "object-src 'none'; ",
    "frame-src 'none'; ",
    "frame-ancestors 'none'; ",
    "base-uri 'self'; ",
    "form-action 'self'",
);

pub(crate) fn spa_file_server_pre_layer(
    request: &Request,
    _: &mut Context,
) -> Result<Option<Response>> {
    let path = request.url.path();
    if path.starts_with("/api") {
        return Ok(None);
    }

    let asset_path = if path == "/" {
        "index.html".to_string()
    } else {
        path.trim_start_matches('/').to_string()
    };
    let asset = WebAssets::get(&asset_path).or_else(|| WebAssets::get("index.html"));
    let Some(asset) = asset else {
        return Ok(None);
    };
    let mime = if asset_path == "index.html" || WebAssets::get(&asset_path).is_none() {
        "text/html".to_string()
    } else {
        mime_guess::from_path(&asset_path)
            .first_or_octet_stream()
            .to_string()
    };
    Ok(Some(
        Response::with_header("Content-Type", mime)
            .header("Content-Security-Policy", CSP_VALUE)
            .header("Cache-Control", "no-cache")
            .body(asset.data),
    ))
}
