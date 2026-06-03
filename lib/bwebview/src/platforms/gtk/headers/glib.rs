/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unused)]

use std::ffi::{c_char, c_void};

// MARK: Libc
pub(crate) const LOCK_EX: i32 = 2;
pub(crate) const LOCK_NB: i32 = 4;
unsafe extern "C" {
    pub(crate) fn flock(fd: i32, op: i32) -> i32;
}

// MARK: GObject
#[repr(C)]
pub(crate) struct GObject([u8; 0]);
pub(crate) const G_CONNECT_DEFAULT: i32 = 0;
#[link(name = "gobject-2.0")]
unsafe extern "C" {
    pub(crate) fn g_object_new(
        object_type: *mut c_void,
        first_property_name: *const c_char,
        ...
    ) -> *mut GObject;
    pub(crate) fn g_object_set(instance: *mut GObject, first_property_name: *const c_char, ...);
    pub(crate) fn g_signal_connect_data(
        instance: *mut GObject,
        detailed_signal: *const c_char,
        c_handler: *const c_void,
        data: *const c_void,
        destroy_data: *const c_void,
        connect_flags: i32,
    );
    pub(crate) fn g_object_unref(object: *mut GObject);
}

// MARK: GLib
#[repr(C)]
pub(crate) struct GError([u8; 0]);
#[repr(C)]
pub(crate) struct GKeyFile([u8; 0]);
#[repr(C)]
pub(crate) struct GSList {
    pub(crate) data: *mut c_void,
    pub(crate) next: *mut GSList,
}
#[link(name = "glib-2.0")]
unsafe extern "C" {
    pub(crate) fn g_error_free(error: *mut GError);
    pub(crate) fn g_key_file_new() -> *mut GKeyFile;
    pub(crate) fn g_key_file_load_from_file(
        key_file: *mut GKeyFile,
        file: *const c_char,
        flags: i32,
        error: *mut *mut GError,
    ) -> bool;
    pub(crate) fn g_key_file_get_integer(
        key_file: *mut GKeyFile,
        group_name: *const c_char,
        key: *const c_char,
        error: *mut *mut c_void,
    ) -> i32;
    pub(crate) fn g_key_file_get_boolean(
        key_file: *mut GKeyFile,
        group_name: *const c_char,
        key: *const c_char,
        error: *mut *mut c_void,
    ) -> bool;
    pub(crate) fn g_key_file_set_integer(
        key_file: *mut GKeyFile,
        group_name: *const c_char,
        key: *const c_char,
        value: i32,
    );
    pub(crate) fn g_key_file_set_boolean(
        key_file: *mut GKeyFile,
        group_name: *const c_char,
        key: *const c_char,
        value: bool,
    );
    pub(crate) fn g_key_file_save_to_file(
        key_file: *mut GKeyFile,
        file: *const c_char,
        error: *mut *mut c_void,
    ) -> bool;
    pub(crate) fn g_key_file_free(key_file: *mut GKeyFile);
    pub(crate) fn g_idle_add(function: extern "C" fn(*mut c_void) -> i32, data: *mut c_void)
    -> u32;
    pub(crate) fn g_free(mem: *mut c_void);
    pub(crate) fn g_slist_free_full(
        list: *mut GSList,
        free_func: unsafe extern "C" fn(*mut c_void),
    );
}

// MARK: GIO
#[repr(C)]
pub(crate) struct GInputStream([u8; 0]);
#[link(name = "gio-2.0")]
unsafe extern "C" {
    pub(crate) fn g_memory_input_stream_new_from_data(
        data: *const c_void,
        len: usize,
        destroy: *const c_void,
    ) -> *mut GInputStream;
    pub(crate) fn g_input_stream_read_all(
        stream: *mut GInputStream,
        buffer: *mut c_void,
        count: usize,
        bytes_read: *mut usize,
        cancellable: *mut c_void,
        error: *mut *mut GError,
    ) -> bool;
}
