/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use small_http::{Method, Request, Response, Status};

use crate::context::Context;

pub(crate) fn cors_pre_layer(req: &Request, ctx: &mut Context) -> Option<Result<Response>> {
    if req.method == Method::Options && req.headers.get("Access-Control-Request-Method").is_some() {
        Some(Ok(Response::new()
            .status(Status::NoContent)
            .header("Access-Control-Allow-Origin", &ctx.server_origin)
            .header(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, PATCH, DELETE, OPTIONS",
            )
            .header("Access-Control-Allow-Headers", "Authorization")
            .header("Access-Control-Max-Age", "86400")))
    } else {
        None
    }
}

pub(crate) fn cors_post_layer(req: &Request, ctx: &Context, res: Response) -> Result<Response> {
    if !(req.method == Method::Options
        && req.headers.get("Access-Control-Request-Method").is_some())
    {
        Ok(res.header("Access-Control-Allow-Origin", &ctx.server_origin))
    } else {
        Ok(res)
    }
}
