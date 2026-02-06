/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

/// MySQL column types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    /// Decimal column type.
    Decimal,
    /// Tiny integer (1 byte).
    TinyInt,
    /// Small integer (2 bytes).
    SmallInt,
    /// Integer (4 bytes).
    Int,
    /// Float column type.
    Float,
    /// Double precision float.
    Double,
    /// Null column type.
    Null,
    /// Timestamp column type.
    Timestamp,
    /// Big integer (8 bytes).
    BigInt,
    /// Medium integer (3 bytes).
    MediumInt,
    /// Date column type.
    Date,
    /// Time column type.
    Time,
    /// DateTime column type.
    DateTime,
    /// Year column type.
    Year,
    /// New date column type.
    NewDate,
    /// Variable character string.
    VarChar,
    /// Bit column type.
    Bit,
    /// New decimal column type.
    NewDecimal,
    /// Enum column type.
    Enum,
    /// Set column type.
    Set,
    /// Tiny blob (small binary data).
    TinyBlob,
    /// Medium blob (medium binary data).
    MediumBlob,
    /// Long blob (large binary data).
    LongBlob,
    /// Blob (binary data).
    Blob,
    /// Variable length string.
    VarString,
    /// Fixed length string.
    String,
    /// Geometry column type.
    GeometryType,
}

impl ColumnType {
    /// Convert MySQL type byte to ColumnType.
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(ColumnType::Decimal),
            0x01 => Some(ColumnType::TinyInt),
            0x02 => Some(ColumnType::SmallInt),
            0x03 => Some(ColumnType::Int),
            0x04 => Some(ColumnType::Float),
            0x05 => Some(ColumnType::Double),
            0x06 => Some(ColumnType::Null),
            0x07 => Some(ColumnType::Timestamp),
            0x08 => Some(ColumnType::BigInt),
            0x09 => Some(ColumnType::MediumInt),
            0x0a => Some(ColumnType::Date),
            0x0b => Some(ColumnType::Time),
            0x0c => Some(ColumnType::DateTime),
            0x0d => Some(ColumnType::Year),
            0x0e => Some(ColumnType::NewDate),
            0x0f => Some(ColumnType::VarChar),
            0x10 => Some(ColumnType::Bit),
            0xf6 => Some(ColumnType::NewDecimal),
            0xf7 => Some(ColumnType::Enum),
            0xf8 => Some(ColumnType::Set),
            0xf9 => Some(ColumnType::TinyBlob),
            0xfa => Some(ColumnType::MediumBlob),
            0xfb => Some(ColumnType::LongBlob),
            0xfc => Some(ColumnType::Blob),
            0xfd => Some(ColumnType::VarString),
            0xfe => Some(ColumnType::String),
            0xff => Some(ColumnType::GeometryType),
            _ => None,
        }
    }
}

/// Server status flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusFlags(u16);

impl StatusFlags {
    /// Check if transaction is in progress.
    pub fn in_transaction(self) -> bool {
        (self.0 & 0x0001) != 0
    }

    /// Check if autocommit is enabled.
    pub fn autocommit(self) -> bool {
        (self.0 & 0x0002) != 0
    }

    /// Check if more results exist.
    pub fn more_results_exist(self) -> bool {
        (self.0 & 0x0008) != 0
    }
}

/// Capability flags for server.
#[derive(Debug, Clone, Copy)]
pub struct CapabilityFlags(u32);

impl CapabilityFlags {
    /// Create new capability flags from raw u32.
    pub fn new(flags: u32) -> Self {
        CapabilityFlags(flags)
    }

    /// Check if SSL is supported.
    pub fn supports_ssl(self) -> bool {
        (self.0 & 0x0800) != 0
    }

    /// Check if CLIENT_PLUGIN_AUTH is supported.
    pub fn supports_plugin_auth(self) -> bool {
        (self.0 & 0x00080000) != 0
    }

    /// Check if CLIENT_SECURE_CONNECTION is supported.
    pub fn supports_secure_connection(self) -> bool {
        (self.0 & 0x8000) != 0
    }

    /// Get raw flags value.
    pub fn raw(self) -> u32 {
        self.0
    }
}

/// Column definition from server.
#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    /// Column catalog.
    pub catalog: String,
    /// Database name.
    pub database: String,
    /// Table name.
    pub table: String,
    /// Original table name.
    pub org_table: String,
    /// Column name.
    pub name: String,
    /// Original column name.
    pub org_name: String,
    /// Column type.
    pub column_type: ColumnType,
    /// Character set.
    pub character_set: u16,
    /// Column length.
    pub column_length: u32,
    /// Flags.
    pub flags: u16,
    /// Decimals.
    pub decimals: u8,
}
