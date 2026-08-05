/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unused)]

use std::ffi::{c_char, c_void};

// MARK: GObject
#[repr(C)]
pub(crate) struct GObject([u8; 0]);
pub(crate) const G_CONNECT_DEFAULT: i32 = 0;
unsafe extern "C" {
    pub(crate) fn g_object_new(
        object_type: *mut c_void,
        first_property_name: *const c_char,
        ...
    ) -> *mut GObject;
    pub(crate) fn g_object_get(instance: *mut GObject, first_property_name: *const c_char, ...);
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
pub(crate) struct GVariant([u8; 0]);
#[repr(C)]
pub(crate) struct GVariantBuilder([u8; 0]);
#[repr(C)]
pub(crate) struct GVariantType([u8; 0]);
#[repr(C)]
pub(crate) struct GSList {
    pub(crate) data: *mut c_void,
    pub(crate) next: *mut GSList,
}
unsafe extern "C" {
    pub(crate) fn g_set_prgname(prgname: *const c_char);
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
    pub(crate) fn g_variant_type_new(type_string: *const c_char) -> *mut GVariantType;
    pub(crate) fn g_variant_type_free(type_: *mut GVariantType);
    pub(crate) fn g_variant_builder_new(type_: *const GVariantType) -> *mut GVariantBuilder;
    pub(crate) fn g_variant_builder_add(
        builder: *mut GVariantBuilder,
        format_string: *const c_char,
        ...
    );
    pub(crate) fn g_variant_builder_unref(builder: *mut GVariantBuilder);
    pub(crate) fn g_variant_new(format_string: *const c_char, ...) -> *mut GVariant;
    pub(crate) fn g_variant_new_boolean(value: i32) -> *mut GVariant;
    pub(crate) fn g_variant_new_double(value: f64) -> *mut GVariant;
    pub(crate) fn g_variant_ref_sink(value: *mut GVariant) -> *mut GVariant;
    pub(crate) fn g_variant_unref(value: *mut GVariant);
}

// MARK: GIO
#[repr(C)]
pub(crate) struct GDBusConnection([u8; 0]);
#[repr(C)]
pub(crate) struct GDBusInterfaceInfo([u8; 0]);
#[repr(C)]
pub(crate) struct GDBusMethodInvocation([u8; 0]);
#[repr(C)]
pub(crate) struct GDBusNodeInfo([u8; 0]);
pub(crate) const G_BUS_TYPE_SESSION: i32 = 2;
pub(crate) type GDBusMethodCallFunc = extern "C" fn(
    *mut GDBusConnection,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *mut GVariant,
    *mut GDBusMethodInvocation,
    *mut c_void,
);
#[repr(C)]
pub(crate) struct GDBusInterfaceVTable {
    pub(crate) method_call: Option<GDBusMethodCallFunc>,
    pub(crate) get_property: *const c_void,
    pub(crate) set_property: *const c_void,
}
#[repr(C)]
pub(crate) struct GInputStream([u8; 0]);
unsafe extern "C" {
    pub(crate) fn g_bus_get_sync(
        bus_type: i32,
        cancellable: *mut c_void,
        error: *mut *mut GError,
    ) -> *mut GDBusConnection;
    pub(crate) fn g_dbus_connection_emit_signal(
        connection: *mut GDBusConnection,
        destination_bus_name: *const c_char,
        object_path: *const c_char,
        interface_name: *const c_char,
        signal_name: *const c_char,
        parameters: *mut GVariant,
        error: *mut *mut GError,
    ) -> i32;
    pub(crate) fn g_bus_watch_name_on_connection(
        connection: *mut GDBusConnection,
        name: *const c_char,
        flags: u32,
        name_appeared_handler: Option<
            extern "C" fn(*mut GDBusConnection, *const c_char, *const c_char, *mut c_void),
        >,
        name_vanished_handler: Option<
            extern "C" fn(*mut GDBusConnection, *const c_char, *mut c_void),
        >,
        user_data: *mut c_void,
        user_data_free_func: *const c_void,
    ) -> u32;
    pub(crate) fn g_dbus_connection_register_object(
        connection: *mut GDBusConnection,
        object_path: *const c_char,
        interface_info: *mut GDBusInterfaceInfo,
        vtable: *const GDBusInterfaceVTable,
        user_data: *mut c_void,
        user_data_free_func: *const c_void,
        error: *mut *mut GError,
    ) -> u32;
    pub(crate) fn g_dbus_method_invocation_return_value(
        invocation: *mut GDBusMethodInvocation,
        parameters: *mut GVariant,
    );
    pub(crate) fn g_dbus_node_info_new_for_xml(
        xml_data: *const c_char,
        error: *mut *mut GError,
    ) -> *mut GDBusNodeInfo;
    pub(crate) fn g_dbus_node_info_lookup_interface(
        info: *mut GDBusNodeInfo,
        name: *const c_char,
    ) -> *mut GDBusInterfaceInfo;
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
