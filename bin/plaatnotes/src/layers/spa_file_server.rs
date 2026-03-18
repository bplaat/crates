/*
 * Copyright (c) 2025 Bastiaan van der Plaat
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
    "style-src 'self' 'unsafe-inline'; ",
    "img-src 'self' data:; ",
    "connect-src 'self'; ",
    "object-src 'none'; ",
    "frame-ancestors 'none'",
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
        let res = Response::with_header("Content-Type", mime.to_string()).body(file.data);
        Some(Ok(if mime.type_() == "text" && mime.subtype() == "html" {
            res.header("Content-Security-Policy", CSP_VALUE)
        } else {
            res
        }))
    } else {
        Some(Ok(Response::with_header("Content-Type", "text/html")
            .header("Content-Security-Policy", CSP_VALUE)
            .body(
                WebAssets::get("index.html")
                    .expect("index.html should exists")
                    .data,
            )))
    }
}
