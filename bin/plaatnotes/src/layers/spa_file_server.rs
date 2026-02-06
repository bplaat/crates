/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use rust_embed::Embed;
use small_http::{Request, Response};

use crate::Context;

#[derive(Embed)]
#[folder = "$OUT_DIR/web"]
struct WebAssets;

pub(crate) fn spa_file_server_pre_layer(req: &Request, _: &mut Context) -> Option<Response> {
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
        Some(Response::with_header("Content-Type", mime.to_string()).body(file.data))
    } else {
        Some(
            Response::with_header("Content-Type", "text/html").body(
                WebAssets::get("index.html")
                    .expect("index.html should exists")
                    .data,
            ),
        )
    }
}
