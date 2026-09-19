/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use log::info;
use small_http::{Method, Request, Response, Status};

pub(crate) use self::auth::{auth_optional_pre_layer, auth_required_pre_layer};
pub(crate) use self::spa_file_server::spa_file_server_pre_layer;
use crate::Context;

mod auth;
mod spa_file_server;

// MARK: Log layer
pub(crate) fn log_pre_layer(req: &Request, _: &mut Context) -> Result<Option<Response>> {
    info!("{} {}", req.method, req.url.path());
    Ok(None)
}

// MARK: CORS layer
pub(crate) fn cors_pre_layer(req: &Request, ctx: &mut Context) -> Result<Option<Response>> {
    if req.method == Method::Options && req.headers.get("Access-Control-Request-Method").is_some() {
        Ok(Some(
            Response::with_status(Status::NoContent)
                .header("Access-Control-Allow-Origin", ctx.server_origin.clone())
                .header(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, PATCH, DELETE, OPTIONS",
                )
                .header(
                    "Access-Control-Allow-Headers",
                    "Authorization, Content-Type",
                )
                .header("Access-Control-Max-Age", "86400"),
        ))
    } else {
        Ok(None)
    }
}

pub(crate) fn cors_post_layer(req: &Request, ctx: &Context, res: Response) -> Result<Response> {
    if !(req.method == Method::Options
        && req.headers.get("Access-Control-Request-Method").is_some())
    {
        Ok(res.header("Access-Control-Allow-Origin", ctx.server_origin.clone()))
    } else {
        Ok(res)
    }
}

pub(crate) fn security_headers_post_layer(
    req: &Request,
    _: &Context,
    res: Response,
) -> Result<Response> {
    let mut res = res
        .header("X-Content-Type-Options", "nosniff")
        .header("Referrer-Policy", "same-origin");
    if req.url.path() == "/api" || req.url.path().starts_with("/api/") {
        res = res.header("Cache-Control", "no-store");
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router;

    #[test]
    fn adds_cors_header_to_regular_responses() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx);

        let response = router.handle(&Request::get("http://localhost/api"));

        assert_eq!(
            response.headers.get("Access-Control-Allow-Origin"),
            Some("http://localhost")
        );
    }

    #[test]
    fn handles_cors_preflight_requests() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx);

        let response = router.handle(
            &Request::options("http://localhost/api/projects")
                .header("Access-Control-Request-Method", "POST"),
        );

        assert_eq!(response.status, Status::NoContent);
        assert_eq!(
            response.headers.get("Access-Control-Allow-Origin"),
            Some("http://localhost")
        );
        assert_eq!(
            response.headers.get("Access-Control-Allow-Methods"),
            Some("GET, POST, PUT, PATCH, DELETE, OPTIONS")
        );
        assert_eq!(
            response.headers.get("Access-Control-Allow-Headers"),
            Some("Authorization, Content-Type")
        );
    }

    #[test]
    fn adds_security_headers_and_disables_api_caching() {
        let ctx = Context::with_test_database().expect("create test database");
        let router = router(ctx);

        let response = router.handle(&Request::get("http://localhost/api"));

        assert_eq!(
            response.headers.get("X-Content-Type-Options"),
            Some("nosniff")
        );
        assert_eq!(response.headers.get("Referrer-Policy"), Some("same-origin"));
        assert_eq!(response.headers.get("Cache-Control"), Some("no-store"));
    }
}
