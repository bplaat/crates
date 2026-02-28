/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [libsqlite3-sys](https://crates.io/crates/libsqlite3-sys) crate.

#![allow(non_camel_case_types, non_snake_case, missing_docs)]

use std::ffi::{c_char, c_int, c_void};

pub type sqlite3 = c_void;
pub type sqlite3_stmt = c_void;
pub type sqlite3_destructor_type = Option<unsafe extern "C" fn(*mut c_void)>;

pub const SQLITE_OK: i32 = 0;

pub const SQLITE_OPEN_READONLY: i32 = 0x00000001;
pub const SQLITE_OPEN_READWRITE: i32 = 0x00000002;
pub const SQLITE_OPEN_CREATE: i32 = 0x00000004;
pub const SQLITE_OPEN_FULLMUTEX: i32 = 0x00010000;

pub const SQLITE_ROW: i32 = 100;
pub const SQLITE_DONE: i32 = 101;

pub const SQLITE_INTEGER: i32 = 1;
pub const SQLITE_FLOAT: i32 = 2;
pub const SQLITE_TEXT: i32 = 3;
pub const SQLITE_BLOB: i32 = 4;
pub const SQLITE_NULL: i32 = 5;

pub fn SQLITE_TRANSIENT() -> sqlite3_destructor_type {
    Some(unsafe { std::mem::transmute::<isize, unsafe extern "C" fn(*mut c_void)>(-1) })
}

unsafe extern "C" {
    // sqlite3
    pub fn sqlite3_open_v2(
        filename: *const c_char,
        ppDb: *mut *mut sqlite3,
        flags: c_int,
        zVfs: *const c_char,
    ) -> c_int;
    pub fn sqlite3_prepare_v2(
        db: *mut sqlite3,
        zSql: *const c_char,
        nByte: c_int,
        ppStmt: *mut *mut sqlite3_stmt,
        pzTail: *mut *const c_char,
    ) -> c_int;
    pub fn sqlite3_changes(db: *mut sqlite3) -> i32;
    pub fn sqlite3_last_insert_rowid(db: *mut sqlite3) -> i64;
    pub fn sqlite3_errmsg(db: *mut sqlite3) -> *const c_char;
    pub fn sqlite3_close(db: *mut sqlite3) -> c_int;

    // sqlite3_stmt
    pub fn sqlite3_db_handle(pStmt: *mut sqlite3_stmt) -> *mut sqlite3;
    pub fn sqlite3_sql(pStmt: *mut sqlite3_stmt) -> *const c_char;
    pub fn sqlite3_step(pStmt: *mut sqlite3_stmt) -> c_int;
    pub fn sqlite3_reset(pStmt: *mut sqlite3_stmt) -> c_int;
    pub fn sqlite3_finalize(pStmt: *mut sqlite3_stmt) -> c_int;

    pub fn sqlite3_bind_parameter_index(pStmt: *mut sqlite3_stmt, zName: *const c_char) -> c_int;
    pub fn sqlite3_bind_null(pStmt: *mut sqlite3_stmt, i: c_int) -> c_int;
    pub fn sqlite3_bind_int64(pStmt: *mut sqlite3_stmt, i: c_int, value: i64) -> c_int;
    pub fn sqlite3_bind_double(pStmt: *mut sqlite3_stmt, i: c_int, value: f64) -> c_int;
    pub fn sqlite3_bind_text(
        pStmt: *mut sqlite3_stmt,
        i: c_int,
        z: *const c_char,
        n: c_int,
        xDel: sqlite3_destructor_type,
    ) -> c_int;
    pub fn sqlite3_bind_blob(
        pStmt: *mut sqlite3_stmt,
        i: c_int,
        z: *const c_void,
        n: c_int,
        xDel: sqlite3_destructor_type,
    ) -> c_int;

    pub fn sqlite3_column_count(pStmt: *mut sqlite3_stmt) -> c_int;
    pub fn sqlite3_column_name(pStmt: *mut sqlite3_stmt, iCol: c_int) -> *const c_char;
    pub fn sqlite3_column_type(pStmt: *mut sqlite3_stmt, iCol: c_int) -> c_int;
    pub fn sqlite3_column_decltype(pStmt: *mut sqlite3_stmt, iCol: c_int) -> *const c_char;
    pub fn sqlite3_column_table_name(pStmt: *mut sqlite3_stmt, iCol: c_int) -> *const c_char;
    pub fn sqlite3_column_origin_name(pStmt: *mut sqlite3_stmt, iCol: c_int) -> *const c_char;
    pub fn sqlite3_column_int64(pStmt: *mut sqlite3_stmt, iCol: c_int) -> i64;
    pub fn sqlite3_column_double(pStmt: *mut sqlite3_stmt, iCol: c_int) -> f64;
    pub fn sqlite3_column_text(pStmt: *mut sqlite3_stmt, iCol: c_int) -> *const c_char;
    pub fn sqlite3_column_blob(pStmt: *mut sqlite3_stmt, iCol: c_int) -> *const c_void;
    pub fn sqlite3_column_bytes(pStmt: *mut sqlite3_stmt, iCol: c_int) -> c_int;
}
