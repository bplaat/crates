/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::env;
use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr::{null, null_mut};

use super::event_loop::APP_ID;
use super::headers::*;

static mut UNITY_CONNECTION: *mut GDBusConnection = null_mut();
static mut UNITY_NAME_WATCH: u32 = 0;
static mut UNITY_PROGRESS: Option<f32> = None;
static mut UNITY_NODE_INFO: *mut GDBusNodeInfo = null_mut();
static mut UNITY_OBJECT_REGISTRATION: u32 = 0;
static mut UNITY_VTABLE: GDBusInterfaceVTable = GDBusInterfaceVTable {
    method_call: Some(unity_method_call),
    get_property: null(),
    set_property: null(),
};

// MARK: Unity launcher entry
pub(super) fn update_progress_bar(progress: Option<f32>) {
    update_unity_progress(progress);
}

#[allow(static_mut_refs)]
fn update_unity_progress(progress: Option<f32>) {
    unsafe { UNITY_PROGRESS = progress };
    emit_unity_progress(progress);
}

#[allow(static_mut_refs)]
fn emit_unity_progress(progress: Option<f32>) {
    let desktop_id = unity_desktop_id();
    let Ok(app_uri) = CString::new(format!("application://{desktop_id}")) else {
        return;
    };

    unsafe {
        if UNITY_CONNECTION.is_null() {
            let mut error = null_mut();
            UNITY_CONNECTION = g_bus_get_sync(G_BUS_TYPE_SESSION, null_mut(), &mut error);
            if !error.is_null() {
                g_error_free(error);
            }
        }
        if UNITY_CONNECTION.is_null() {
            return;
        }
        if UNITY_NAME_WATCH == 0 {
            UNITY_NAME_WATCH = g_bus_watch_name_on_connection(
                UNITY_CONNECTION,
                c"com.canonical.Unity".as_ptr(),
                0,
                Some(unity_name_appeared),
                None,
                null_mut(),
                null(),
            );
        }
        register_unity_query_object();

        let parameters = unity_parameters(progress, app_uri.as_ptr());
        let parameters = g_variant_ref_sink(parameters);
        let mut error = null_mut();
        _ = g_dbus_connection_emit_signal(
            UNITY_CONNECTION,
            null(),
            c"/com/canonical/Unity/LauncherEntry".as_ptr(),
            c"com.canonical.Unity.LauncherEntry".as_ptr(),
            c"Update".as_ptr(),
            parameters,
            &mut error,
        );
        if !error.is_null() {
            g_error_free(error);
        }
        g_variant_unref(parameters);
    }
}

#[allow(static_mut_refs)]
fn unity_desktop_id() -> String {
    unsafe {
        APP_ID.as_ref().map(|app_id| {
            format!(
                "{}.{}.{}.desktop",
                app_id.qualifier, app_id.organization, app_id.application
            )
        })
    }
    .or_else(|| {
        let launched_pid = env::var("GIO_LAUNCHED_DESKTOP_FILE_PID")
            .ok()
            .and_then(|pid| pid.parse::<u32>().ok());
        (launched_pid == Some(std::process::id()))
            .then(|| env::var_os("GIO_LAUNCHED_DESKTOP_FILE"))
            .flatten()
            .and_then(|path| {
                std::path::Path::new(&path)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
    })
    .or_else(|| {
        env::current_exe()
            .ok()
            .and_then(|path| {
                path.file_stem()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .map(|name| format!("{name}.desktop"))
    })
    .unwrap_or_else(|| "bwebview.desktop".to_owned())
}

unsafe fn unity_parameters(progress: Option<f32>, app_uri: *const c_char) -> *mut GVariant {
    let dictionary_type = unsafe { g_variant_type_new(c"a{sv}".as_ptr()) };
    let properties = unsafe { g_variant_builder_new(dictionary_type) };
    unsafe { g_variant_type_free(dictionary_type) };
    let visible = progress.is_some();
    let progress = f64::from(progress.unwrap_or_default().clamp(0.0, 1.0));
    unsafe {
        g_variant_builder_add(
            properties,
            c"{sv}".as_ptr(),
            c"progress".as_ptr(),
            g_variant_new_double(progress),
        );
        g_variant_builder_add(
            properties,
            c"{sv}".as_ptr(),
            c"progress-visible".as_ptr(),
            g_variant_new_boolean(i32::from(visible)),
        );
        let parameters = g_variant_new(c"(sa{sv})".as_ptr(), app_uri, properties);
        g_variant_builder_unref(properties);
        parameters
    }
}

unsafe fn register_unity_query_object() {
    if unsafe { UNITY_OBJECT_REGISTRATION != 0 } {
        return;
    }
    let mut error = null_mut();
    if unsafe { UNITY_NODE_INFO.is_null() } {
        unsafe {
            UNITY_NODE_INFO = g_dbus_node_info_new_for_xml(
                c"<node><interface name='com.canonical.Unity.LauncherEntry'><method name='Query'><arg type='s' direction='out'/><arg type='a{sv}' direction='out'/></method></interface></node>".as_ptr(),
                &mut error,
            );
        }
        if !error.is_null() {
            unsafe { g_error_free(error) };
            return;
        }
    }
    let interface = unsafe {
        g_dbus_node_info_lookup_interface(
            UNITY_NODE_INFO,
            c"com.canonical.Unity.LauncherEntry".as_ptr(),
        )
    };
    if interface.is_null() {
        return;
    }
    error = null_mut();
    unsafe {
        UNITY_OBJECT_REGISTRATION = g_dbus_connection_register_object(
            UNITY_CONNECTION,
            c"/com/canonical/Unity/LauncherEntry".as_ptr(),
            interface,
            &raw const UNITY_VTABLE,
            null_mut(),
            null(),
            &mut error,
        );
    }
    if !error.is_null() {
        unsafe { g_error_free(error) };
    }
}

extern "C" fn unity_method_call(
    _connection: *mut GDBusConnection,
    _sender: *const c_char,
    _object_path: *const c_char,
    _interface_name: *const c_char,
    method_name: *const c_char,
    _parameters: *mut GVariant,
    invocation: *mut GDBusMethodInvocation,
    _data: *mut c_void,
) {
    if unsafe { CStr::from_ptr(method_name) }.to_bytes() != b"Query" {
        return;
    }
    let desktop_id = unity_desktop_id();
    let Ok(app_uri) = CString::new(format!("application://{desktop_id}")) else {
        return;
    };
    unsafe {
        g_dbus_method_invocation_return_value(
            invocation,
            unity_parameters(UNITY_PROGRESS, app_uri.as_ptr()),
        );
    }
}

extern "C" fn unity_name_appeared(
    _connection: *mut GDBusConnection,
    _name: *const c_char,
    _owner: *const c_char,
    _data: *mut c_void,
) {
    emit_unity_progress(unsafe { UNITY_PROGRESS });
}
