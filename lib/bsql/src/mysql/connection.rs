/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::io::{Read, Write};
#[cfg(unix)]
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use crate::connection::{Connection, InnerConnection};
use crate::ConnectionError;

pub(super) enum MysqlTransport {
    Tcp {
        host: String,
        port: u16,
    },
    #[cfg(unix)]
    Unix {
        path: PathBuf,
    },
}

pub(super) struct MysqlOptions {
    pub(super) transport: MysqlTransport,
    pub(super) user: String,
    pub(super) password: String,
    pub(super) database: Option<String>,
    pub(super) tls: bool,
    pub(super) timeout: Duration,
}

impl MysqlOptions {
    fn tcp(
        host: impl Into<String>,
        port: u16,
        user: impl Into<String>,
        password: impl Into<String>,
        database: Option<&str>,
        tls: bool,
    ) -> Self {
        Self {
            transport: MysqlTransport::Tcp {
                host: host.into(),
                port,
            },
            user: user.into(),
            password: password.into(),
            database: database.map(str::to_owned),
            tls,
            timeout: Duration::from_secs(10),
        }
    }

    #[cfg(unix)]
    fn unix(
        path: impl Into<PathBuf>,
        user: impl Into<String>,
        password: impl Into<String>,
        database: Option<&str>,
    ) -> Self {
        Self {
            transport: MysqlTransport::Unix { path: path.into() },
            user: user.into(),
            password: password.into(),
            database: database.map(str::to_owned),
            tls: false,
            timeout: Duration::from_secs(10),
        }
    }
}

pub(crate) trait Stream: Read + Write + Send {}
impl<T: Read + Write + Send> Stream for T {}
pub(crate) type OpenedStream = (Box<dyn Stream>, Option<String>, bool);

pub(crate) struct Client {
    pub(crate) stream: Box<dyn Stream>,
    pub(crate) affected_rows: u64,
    pub(crate) last_insert_id: u64,
    pub(crate) capabilities: u32,
}

impl Connection {
    fn connect_mysql(options: MysqlOptions) -> Result<Self, ConnectionError> {
        let client = Client::connect(&options).map_err(ConnectionError::new)?;
        Ok(Self::from_inner(InnerConnection::Mysql(Mutex::new(client))))
    }

    /// Connect to a MySQL server over TCP using the classic client/server protocol.
    pub fn open_mysql_tcp(
        host: impl Into<String>,
        port: u16,
        user: impl Into<String>,
        password: impl Into<String>,
        database: Option<&str>,
        tls: bool,
    ) -> Result<Self, ConnectionError> {
        Self::connect_mysql(MysqlOptions::tcp(host, port, user, password, database, tls))
    }

    /// Connect to a MySQL server through a Unix domain socket.
    #[cfg(unix)]
    pub fn open_mysql_unix(
        path: impl Into<PathBuf>,
        user: impl Into<String>,
        password: impl Into<String>,
        database: Option<&str>,
    ) -> Result<Self, ConnectionError> {
        Self::connect_mysql(MysqlOptions::unix(path, user, password, database))
    }
}
