/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use small_http::{Request, Response, Status};

use crate::api;
use crate::context::{Context, DatabaseHelpers};

pub(crate) fn deployments_show(req: &Request, ctx: &Context) -> Result<Response> {
    let deployment_id = match req
        .params
        .get("deployment_id")
        .and_then(|id| id.parse().ok())
    {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    match ctx.database.find_deployment_by_id(deployment_id)? {
        Some(deployment) => Ok(Response::with_json(api::Deployment::from(deployment))),
        None => Ok(Response::new().status(Status::NotFound)),
    }
}
