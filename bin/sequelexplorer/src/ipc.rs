/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

// MARK: IPC messages
#[derive(Deserialize, Serialize)]
#[allow(clippy::enum_variant_names)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub(crate) enum IpcMessage {
    Ready,
    MenuAction {
        action: String,
    },
    RestoreLastFile,
    OpenFile {
        path: String,
    },
    OpenFileDialog {
        request_id: u64,
    },
    OpenFileDialogResponse {
        request_id: u64,
        path: Option<String>,
    },
    OpenDatabase {
        request_id: u64,
        path: String,
    },
    OpenDatabaseResponse {
        request_id: u64,
        ok: bool,
        error: Option<String>,
    },
    ImportSql {
        request_id: u64,
    },
    ImportSqlResponse {
        request_id: u64,
        cancelled: bool,
        error: Option<String>,
    },
    ExportSql {
        request_id: u64,
        file_name: String,
    },
    ExportSqlResponse {
        request_id: u64,
        cancelled: bool,
        error: Option<String>,
    },
    OpenMysql {
        request_id: u64,
        transport: String,
        host: String,
        port: u16,
        socket: String,
        user: String,
        password: Option<Zeroizing<String>>,
        tls: bool,
        remember: bool,
        previous_connection: Option<MysqlCredentialIdentity>,
    },
    OpenMysqlResponse {
        request_id: u64,
        ok: bool,
        error: Option<String>,
        credential_saved: bool,
        credential_error: Option<String>,
    },
    SelectMysqlDatabase {
        request_id: u64,
        database: String,
    },
    SelectMysqlDatabaseResponse {
        request_id: u64,
        ok: bool,
        error: Option<String>,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MysqlCredentialIdentity {
    pub(crate) transport: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) socket: String,
    pub(crate) user: String,
}
