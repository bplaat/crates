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

const CSP_VALUE: &str = concat!(
    "default-src 'self'; ",
    "script-src 'self'; ",
    "script-src-attr 'none'; ",
    "style-src 'self'; ",
    "style-src-attr 'unsafe-inline'; ",
    "img-src 'self' data:; ",
    "connect-src 'self'; ",
    "object-src 'none'; ",
    "frame-src 'none'; ",
    "frame-ancestors 'none'; ",
    "base-uri 'self'; ",
    "form-action 'self' https://github.com; ",
    "worker-src 'none'"
);

fn security_headers(res: Response) -> Response {
    res.header("Content-Security-Policy", CSP_VALUE)
        .header("X-Content-Type-Options", "nosniff")
        .header("Referrer-Policy", "same-origin")
        .header("Cross-Origin-Opener-Policy", "same-origin")
        .header("Cross-Origin-Resource-Policy", "same-origin")
}

pub(crate) fn spa_pre_layer(req: &Request, _: &mut Context) -> Option<Result<Response>> {
    let path = req.url.path();
    if path.starts_with("/api") {
        return None;
    }
    if path == "/swagger-ui" {
        return Some(Ok(Response::with_redirect("/swagger-ui/index.html")));
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
    Some(Ok(security_headers(
        Response::new()
            .header("Content-Type", mime.to_string())
            .body(data),
    )))
}
