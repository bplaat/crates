/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
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
    "form-action 'self'; ",
    "worker-src 'none'",
);

pub(crate) fn spa_file_server_pre_layer(
    req: &Request,
    _: &mut Context,
) -> Option<Result<Response>> {
    let path = req.url.path();
    if path.starts_with("/api") {
        return None;
    }

    let mut path = path.to_string();
    if path.ends_with('/') {
        path = format!("{path}index.html");
    }
    if let Some(file) = WebAssets::get(path.trim_start_matches('/')) {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        Some(Ok(Response::with_header("Content-Type", mime.to_string())
            .header("Content-Security-Policy", CSP_VALUE)
            .body(file.data)))
    } else {
        Some(match WebAssets::get("index.html") {
            Some(file) => Ok(Response::with_header("Content-Type", "text/html")
                .header("Content-Security-Policy", CSP_VALUE)
                .body(file.data)),
            None => Err(anyhow::anyhow!("index.html not found in embedded assets")),
        })
    }
}
