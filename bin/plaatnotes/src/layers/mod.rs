/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use log::info;
use small_http::{Method, Request, Response};

pub(crate) use self::spa_file_server::spa_file_server_pre_layer;
use crate::Context;

mod spa_file_server;

// MARK: Log layer
pub(crate) fn log_pre_layer(req: &Request, _: &mut Context) -> Option<Response> {
    info!("{} {}", req.method, req.url.path());
    None
}

// MARK: CORS layer
pub(crate) fn cors_pre_layer(req: &Request, _: &mut Context) -> Option<Response> {
    if req.method == Method::Options {
        Some(Response::new())
    } else {
        None
    }
}

pub(crate) fn cors_post_layer(_: &Request, _: &mut Context, res: Response) -> Response {
    res.header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE")
        .header("Access-Control-Max-Age", "86400")
}
