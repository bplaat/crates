/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use anyhow::Result;
use small_http::{Request, Response, Status};

use crate::api;
use crate::context::{Context, DatabaseHelpers};
use crate::models::UserRole;

pub(crate) fn deployments_show(req: &Request, ctx: &Context) -> Result<Response> {
    let auth_user = ctx.auth_user.as_ref().expect("auth missing");
    let deployment_id = match req
        .params
        .get("deployment_id")
        .and_then(|id| id.parse().ok())
    {
        Some(id) => id,
        None => return Ok(Response::new().status(Status::BadRequest)),
    };

    let deployment = match ctx.database.find_deployment_by_id(deployment_id)? {
        Some(deployment) => deployment,
        None => return Ok(Response::new().status(Status::NotFound)),
    };
    let project = match ctx.database.find_project_by_id(deployment.project_id)? {
        Some(project) => project,
        None => return Ok(Response::new().status(Status::NotFound)),
    };
    if auth_user.role != UserRole::Admin
        && !ctx
            .database
            .user_is_team_member(auth_user.id, project.team_id)?
    {
        return Ok(Response::new().status(Status::Forbidden));
    }

    Ok(Response::with_json(api::Deployment::from(deployment)))
}
