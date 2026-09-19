/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

pub(crate) use account::{
    account_change_password, account_github_delete, account_github_update, account_update,
};
use anyhow::Result;
pub(crate) use auth::{auth_login, auth_logout, auth_validate};
pub(crate) use projects::{
    github_repositories, github_repository_scan, github_webhook, project_deployments,
    projects_create, projects_delete, projects_deploy, projects_index, projects_show,
};
pub(crate) use sessions::{sessions_delete, sessions_index};
use small_http::{Request, Response};

use crate::Context;

mod account;
mod auth;
mod projects;
mod sessions;

pub(crate) fn home(_: &Request, _: &Context) -> Result<Response> {
    Ok(Response::with_body(concat!(
        "Plaat Deploy API v",
        env!("CARGO_PKG_VERSION")
    )))
}

macro_rules! parse_body {
    ($req:expr, $api_type:ty, $internal_type:ty) => {{
        use validate::Validate as _;
        let body = match $req.parse_body::<$api_type>() {
            Ok(body) => Into::<$internal_type>::into(body),
            Err(error) => {
                return Ok(small_http::Response::with_status(small_http::Status::from(
                    error,
                )));
            }
        };
        if let Err(report) = body.validate() {
            return Ok(
                small_http::Response::with_status(small_http::Status::BadRequest)
                    .json(crate::api::Report(report.0)),
            );
        }
        body
    }};
}

macro_rules! require_auth {
    ($ctx:expr) => {
        match $ctx.auth_user.as_ref() {
            Some(user) => user,
            None => {
                return Ok(small_http::Response::with_status(
                    small_http::Status::Unauthorized,
                ))
            }
        }
    };
}

pub(crate) use parse_body;
pub(crate) use require_auth;
