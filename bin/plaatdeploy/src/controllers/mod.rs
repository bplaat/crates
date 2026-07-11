/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use serde::Deserialize;
use small_http::{Request, Response, Status};
use validate::Validate;

mod auth;
mod deployments;
mod home;
mod projects;
mod sessions;
mod team_github;
mod teams;
mod users;
mod webhook;

pub(crate) use auth::{auth_login, auth_logout, auth_validate};
pub(crate) use deployments::deployments_show;
pub(crate) use home::home;
pub(crate) use projects::{
    projects_create, projects_delete, projects_deploy, projects_deployments, projects_index,
    projects_show, projects_update,
};
pub(crate) use sessions::{sessions_delete, sessions_index};
pub(crate) use team_github::{
    teams_github_branches, teams_github_delete, teams_github_repositories, teams_github_show,
    teams_github_update,
};
pub(crate) use teams::{
    teams_create, teams_delete, teams_index, teams_members_create, teams_members_delete,
    teams_members_update, teams_show, teams_update,
};
pub(crate) use users::{
    users_change_password, users_create, users_delete, users_index, users_show, users_update,
};
pub(crate) use webhook::webhook_github;

use crate::api;
use crate::consts::DEFAULT_LIMIT;

impl From<validate::Report> for api::Report {
    fn from(report: validate::Report) -> Self {
        Self(report.0)
    }
}

#[derive(Deserialize, Validate)]
pub(crate) struct PaginationQuery {
    #[serde(default = "PaginationQuery::default_page")]
    #[validate(range(min = 1))]
    pub page: i64,
    #[serde(default = "PaginationQuery::default_limit")]
    #[validate(range(min = 1, max = 50))]
    pub limit: i64,
}

impl PaginationQuery {
    fn default_page() -> i64 {
        1
    }
    fn default_limit() -> i64 {
        DEFAULT_LIMIT
    }
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: Self::default_page(),
            limit: Self::default_limit(),
        }
    }
}

pub(crate) fn parse_pagination(req: &Request) -> Result<PaginationQuery, Response> {
    let q = match req.url.query() {
        Some(q) => match serde_urlencoded::from_str::<PaginationQuery>(q) {
            Ok(q) => q,
            Err(_) => return Err(Response::with_status(Status::BadRequest)),
        },
        None => PaginationQuery::default(),
    };
    if let Err(report) = q.validate() {
        return Err(Response::with_status(Status::BadRequest).json(api::Report::from(report)));
    }
    Ok(q)
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

macro_rules! parse_body {
    ($req:expr, $api_type:ty, $internal_type:ty) => {{
        use validate::Validate as _;
        let body = match $req.parse_body::<$api_type>() {
            Ok(body) => Into::<$internal_type>::into(body),
            Err(err) => {
                return Ok(small_http::Response::with_status(small_http::Status::from(
                    err,
                )));
            }
        };
        if let Err(report) = body.validate() {
            return Ok(
                small_http::Response::with_status(small_http::Status::BadRequest)
                    .json(crate::api::Report::from(report)),
            );
        }
        body
    }};
}

macro_rules! parse_body_ctx {
    ($req:expr, $api_type:ty, $internal_type:ty, $ctx:expr) => {{
        use validate::Validate as _;
        let body = match $req.parse_body::<$api_type>() {
            Ok(body) => Into::<$internal_type>::into(body),
            Err(err) => {
                return Ok(small_http::Response::with_status(small_http::Status::from(
                    err,
                )));
            }
        };
        if let Err(report) = body.validate_with($ctx) {
            return Ok(
                small_http::Response::with_status(small_http::Status::BadRequest)
                    .json(crate::api::Report::from(report)),
            );
        }
        body
    }};
}

pub(crate) use parse_body;
pub(crate) use parse_body_ctx;
pub(crate) use require_auth;
