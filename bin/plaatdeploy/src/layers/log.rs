/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use small_http::{Request, Response};

use crate::context::Context;

pub(crate) fn log_pre_layer(req: &Request, _: &mut Context) -> Option<Result<Response>> {
    log::info!("{} {}", req.method, req.url.path());
    None
}
