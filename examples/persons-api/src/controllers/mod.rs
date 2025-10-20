/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use small_http::{Request, Response, Status};

pub(crate) use self::persons::*;
use crate::Context;

pub(crate) mod persons;

pub(crate) fn home(_: &Request, _: &Context) -> Response {
    Response::with_body(concat!("Persons v", env!("CARGO_PKG_VERSION")))
}

pub(crate) fn not_found(_: &Request, _: &Context) -> Response {
    Response::with_status(Status::NotFound).body("404 Not found")
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::router;

    #[test]
    fn test_home() {
        let ctx = Context::with_test_database();
        let router = router(ctx.clone());

        let res = router.handle(&Request::get("http://localhost/"));
        assert_eq!(res.status, Status::Ok);
        assert!(res.body.starts_with(b"Persons v"));
    }
}
