/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

/// Value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// Undefined value
    Undefined,
    /// Null value
    Null,
    /// Boolean value
    Boolean(bool),
    /// Number value
    Number(i64),
    /// String value
    String(String),
}
