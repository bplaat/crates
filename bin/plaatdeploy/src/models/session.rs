/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use bsqlite::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::api;

// MARK: Session
#[derive(Clone, FromRow)]
pub(crate) struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub ip_address: String,
    pub ip_latitude: Option<f64>,
    pub ip_longitude: Option<f64>,
    pub ip_country: Option<String>,
    pub ip_city: Option<String>,
    pub client_name: Option<String>,
    pub client_version: Option<String>,
    pub client_os: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Session {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            user_id: Uuid::nil(),
            token: String::new(),
            ip_address: String::new(),
            ip_latitude: None,
            ip_longitude: None,
            ip_country: None,
            ip_city: None,
            client_name: None,
            client_version: None,
            client_os: None,
            expires_at: now,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<Session> for api::Session {
    fn from(session: Session) -> Self {
        Self {
            id: session.id,
            user_id: session.user_id,
            ip: api::SessionIp {
                address: session.ip_address,
                latitude: session.ip_latitude,
                longitude: session.ip_longitude,
                country: session.ip_country,
                city: session.ip_city,
            },
            client: api::SessionClient {
                name: session.client_name,
                version: session.client_version,
                os: session.client_os,
            },
            expires_at: session.expires_at,
            created_at: session.created_at,
            updated_at: session.updated_at,
        }
    }
}
