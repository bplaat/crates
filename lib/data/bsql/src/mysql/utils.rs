/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::io;

use crate::StatementError;

pub(super) fn put_lenenc_int(output: &mut Vec<u8>, value: u64) {
    match value {
        0..=250 => output.push(value as u8),
        251..=0xffff => {
            output.push(0xfc);
            output.extend_from_slice(&(value as u16).to_le_bytes());
        }
        0x1_0000..=0xff_ffff => {
            output.push(0xfd);
            output.extend_from_slice(&value.to_le_bytes()[..3]);
        }
        _ => {
            output.push(0xfe);
            output.extend_from_slice(&value.to_le_bytes());
        }
    }
}

pub(super) fn put_lenenc_bytes(output: &mut Vec<u8>, value: &[u8]) {
    put_lenenc_int(output, value.len() as u64);
    output.extend_from_slice(value);
}

pub(super) fn nonempty_lossy(value: &[u8]) -> Option<String> {
    (!value.is_empty()).then(|| String::from_utf8_lossy(value).into_owned())
}

pub(super) fn protocol_error(message: impl Into<String>) -> StatementError {
    StatementError::broken_connection(format!("MySQL protocol error: {}", message.into()))
}

pub(super) fn statement_io(error: io::Error) -> StatementError {
    StatementError::broken_connection(format!("MySQL I/O error: {error}"))
}
