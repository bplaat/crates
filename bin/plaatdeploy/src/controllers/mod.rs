/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use serde::Deserialize;
use small_http::{Request, Response, Status};

mod auth;
mod deployments;
mod github_setup;
mod home;
mod projects;
mod sessions;
mod team_github;
mod teams;
mod users;
mod webhook;

pub(crate) use auth::{auth_login, auth_logout, auth_validate};
pub(crate) use deployments::deployments_show;
pub(crate) use github_setup::github_setup;
pub(crate) use home::home;
pub(crate) use projects::{
    projects_create, projects_delete, projects_deploy, projects_deployments, projects_index,
    projects_show, projects_update,
};
pub(crate) use sessions::{sessions_delete, sessions_index};
pub(crate) use team_github::{
    teams_github_branches, teams_github_delete, teams_github_repositories,
    teams_github_setup_start, teams_github_show, teams_github_update,
};
pub(crate) use teams::{
    teams_create, teams_delete, teams_index, teams_members_create, teams_members_delete,
    teams_members_update, teams_show, teams_update,
};
pub(crate) use users::{
    users_change_password, users_create, users_delete, users_index, users_show, users_update,
};
pub(crate) use webhook::webhook_github;

use crate::consts::DEFAULT_LIMIT;

#[derive(Deserialize)]
pub(crate) struct PaginationQuery {
    #[serde(default = "PaginationQuery::default_page")]
    pub page: i64,
    #[serde(default = "PaginationQuery::default_limit")]
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
    Ok(PaginationQuery {
        page: q.page.max(1),
        limit: q.limit.clamp(1, 50),
    })
}
