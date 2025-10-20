/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use log::info;
use small_http::{Method, Request, Response};

use crate::context::Context;

// MARK: Log
pub(crate) fn log_pre_layer(req: &Request, _: &mut Context) -> Option<Response> {
    info!("{} {}", req.method, req.url.path());
    None
}

// MARK: CORS
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

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::router;

    #[test]
    fn test_cors() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        let res = router.handle(&Request::get("http://localhost/"));
        assert_eq!(res.headers.get("Access-Control-Allow-Origin"), Some("*"));
    }

    #[test]
    fn test_cors_preflight() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        let res = router.handle(&Request::options("http://localhost/"));
        assert_eq!(res.headers.get("Access-Control-Allow-Origin"), Some("*"));
        assert_eq!(
            res.headers.get("Access-Control-Allow-Methods"),
            Some("GET, POST, PUT, DELETE")
        );
        assert_eq!(res.headers.get("Access-Control-Max-Age"), Some("86400"));
    }
}
