/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use small_http::{Request, Response};

use crate::context::Context;

pub(crate) fn home(_req: &Request, _ctx: &Context) -> Result<Response> {
    Ok(Response::new().body(concat!("plaatdeploy v", env!("CARGO_PKG_VERSION"))))
}
