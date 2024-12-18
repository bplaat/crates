/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_int, c_void};

pub(crate) type sqlite3 = c_void;
pub(crate) type sqlite3_stmt = c_void;

pub(crate) const SQLITE_OK: i32 = 0;
pub(crate) const SQLITE_OPEN_CREATE: i32 = 0x00000004;
pub(crate) const SQLITE_OPEN_READWRITE: i32 = 0x00000002;
pub(crate) const SQLITE_OPEN_FULLMUTEX: i32 = 0x00010000;
pub(crate) const SQLITE_ROW: i32 = 100;
pub(crate) const SQLITE_DONE: i32 = 101;
pub(crate) const SQLITE_TRANSIENT: isize = -1;
pub(crate) const SQLITE_INTEGER: i32 = 1;
pub(crate) const SQLITE_FLOAT: i32 = 2;
pub(crate) const SQLITE_TEXT: i32 = 3;
pub(crate) const SQLITE_BLOB: i32 = 4;
pub(crate) const SQLITE_NULL: i32 = 5;

extern "C" {
    // sqlite3
    pub(crate) fn sqlite3_open_v2(
        filename: *const c_char,
        ppDb: *mut *mut sqlite3,
        flags: c_int,
        zVfs: *const c_char,
    ) -> c_int;
    pub(crate) fn sqlite3_prepare_v2(
        db: *mut sqlite3,
        zSql: *const c_char,
        nByte: c_int,
        ppStmt: *mut *mut sqlite3_stmt,
        pzTail: *mut *const c_char,
    ) -> c_int;
    pub(crate) fn sqlite3_errmsg(db: *mut sqlite3) -> *const c_char;
    pub(crate) fn sqlite3_close(db: *mut sqlite3) -> c_int;

    // sqlite3_stmt
    pub(crate) fn sqlite3_step(pStmt: *mut sqlite3_stmt) -> c_int;
    pub(crate) fn sqlite3_reset(pStmt: *mut sqlite3_stmt) -> c_int;
    pub(crate) fn sqlite3_finalize(pStmt: *mut sqlite3_stmt) -> c_int;
    pub(crate) fn sqlite3_bind_null(pStmt: *mut sqlite3_stmt, i: c_int) -> c_int;
    pub(crate) fn sqlite3_bind_int64(pStmt: *mut sqlite3_stmt, i: c_int, value: i64) -> c_int;
    pub(crate) fn sqlite3_bind_double(pStmt: *mut sqlite3_stmt, i: c_int, value: f64) -> c_int;
    pub(crate) fn sqlite3_bind_text(
        pStmt: *mut sqlite3_stmt,
        i: c_int,
        z: *const c_char,
        n: c_int,
        xDel: isize,
    ) -> c_int;
    pub(crate) fn sqlite3_bind_blob(
        pStmt: *mut sqlite3_stmt,
        i: c_int,
        z: *const c_void,
        n: c_int,
        xDel: isize,
    ) -> c_int;
    pub(crate) fn sqlite3_column_type(pStmt: *mut sqlite3_stmt, iCol: c_int) -> c_int;
    pub(crate) fn sqlite3_column_int64(pStmt: *mut sqlite3_stmt, iCol: c_int) -> i64;
    pub(crate) fn sqlite3_column_double(pStmt: *mut sqlite3_stmt, iCol: c_int) -> f64;
    pub(crate) fn sqlite3_column_text(pStmt: *mut sqlite3_stmt, iCol: c_int) -> *const c_char;
    pub(crate) fn sqlite3_column_blob(pStmt: *mut sqlite3_stmt, iCol: c_int) -> *const c_void;
    pub(crate) fn sqlite3_column_bytes(pStmt: *mut sqlite3_stmt, iCol: c_int) -> c_int;
}
