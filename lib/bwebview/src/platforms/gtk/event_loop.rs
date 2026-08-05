/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CStr, CString, c_char, c_void};
use std::fs::File;
use std::mem::MaybeUninit;
use std::process::exit;
use std::ptr::{null, null_mut};
use std::{env, fs, iter};

use super::headers::*;
use crate::{AppId, Event, EventLoopBuilder, LogicalPoint, LogicalSize, Theme};

// MARK: EventLoop
pub(crate) struct PlatformEventLoop {
    theme: Theme,
}

pub(super) static mut APP_ID: Option<AppId> = None;
static mut EVENT_HANDLER: Option<Box<dyn FnMut(Event) + 'static>> = None;
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

impl PlatformEventLoop {
    pub(crate) fn new(builder: EventLoopBuilder) -> Self {
        let app_id_name = builder.app_id.as_ref().map(|app_id| {
            CString::new(format!(
                "{}.{}.{}",
                app_id.qualifier, app_id.organization, app_id.application
            ))
            .expect("Can't convert app id to CString")
        });

        // Ensure single instance
        // FIXME: Use GtkApplication for this
        if let Some(app_id) = builder.app_id {
            let lock_file = env::temp_dir()
                .join(format!(
                    "{}.{}.{}",
                    app_id.qualifier, app_id.organization, app_id.application
                ))
                .join(".lock");
            if let Some(parent) = lock_file.parent() {
                fs::create_dir_all(parent).expect("Failed to create lock file directory");
            }
            let file = File::create(&lock_file).expect("Failed to open lock file");
            if file.try_lock().is_err() {
                exit(0);
            }
            std::mem::forget(file);
            unsafe { APP_ID = Some(app_id) };
        }

        // Init GTK
        unsafe {
            if let Some(ref app_id) = app_id_name {
                g_set_prgname(app_id.as_ptr());
            }

            let args = env::args()
                .map(|arg| CString::new(arg.as_str()).expect("Can't convert to CString"))
                .collect::<Vec<CString>>();
            let mut argc = args.len() as i32;
            let mut argv: Vec<*mut c_char> = args
                .iter()
                .map(|arg| arg.as_ptr() as *mut c_char)
                .chain(iter::once(null_mut()))
                .collect();
            let mut argv_ptr = argv.as_mut_ptr();
            gtk_init(&mut argc, &mut argv_ptr);

            if let Some(ref app_id) = app_id_name {
                gtk_window_set_default_icon_name(app_id.as_ptr());
            }
        }
        Self {
            theme: system_theme(),
        }
    }
}

// MARK: Theme
fn system_theme() -> Theme {
    let settings = unsafe { gtk_settings_get_default() };
    let mut prefer_dark = 0i32;
    let mut theme_name: *mut c_char = null_mut();
    unsafe {
        g_object_get(
            settings as *mut GObject,
            c"gtk-application-prefer-dark-theme".as_ptr(),
            &mut prefer_dark,
            c"gtk-theme-name".as_ptr(),
            &mut theme_name,
            null::<c_void>(),
        );
    }
    let theme_name_is_dark = if theme_name.is_null() {
        false
    } else {
        let is_dark = unsafe { CStr::from_ptr(theme_name) }
            .to_string_lossy()
            .to_ascii_lowercase()
            .contains("dark");
        unsafe { g_free(theme_name as *mut c_void) };
        is_dark
    };
    if prefer_dark != 0 || theme_name_is_dark {
        Theme::Dark
    } else {
        Theme::Light
    }
}

impl crate::EventLoopInterface for PlatformEventLoop {
    fn theme(&self) -> Theme {
        self.theme
    }

    fn primary_monitor(&self) -> PlatformMonitor {
        cfg_select! {
            gtk3_22 => unsafe {
                let mut m = gdk_display_get_primary_monitor(gdk_display_get_default());
                if m.is_null() {
                    m = gdk_display_get_monitor(gdk_display_get_default(), 0);
                }
                PlatformMonitor::new(m)
            },
            _ => unsafe {
                let screen = gdk_screen_get_default();
                let idx = gdk_screen_get_primary_monitor(screen);
                PlatformMonitor::new(idx)
            },
        }
    }

    fn available_monitors(&self) -> Vec<PlatformMonitor> {
        cfg_select! {
            gtk3_22 => unsafe {
                let display = gdk_display_get_default();
                (0..gdk_display_get_n_monitors(display))
                    .map(|i| PlatformMonitor::new(gdk_display_get_monitor(display, i)))
                    .collect()
            },
            _ => unsafe {
                let screen = gdk_screen_get_default();
                (0..gdk_screen_get_n_monitors(screen))
                    .map(PlatformMonitor::new)
                    .collect()
            },
        }
    }

    fn run(self, event_handler: impl FnMut(Event) + 'static) -> ! {
        unsafe { EVENT_HANDLER = Some(Box::new(event_handler)) };

        // Start event loop
        unsafe { gtk_main() };
        exit(0);
    }

    fn create_proxy(&self) -> PlatformEventLoopProxy {
        PlatformEventLoopProxy::new()
    }
}

pub(super) fn send_event(event: Event) {
    unsafe {
        #[allow(static_mut_refs)]
        if let Some(handler) = &mut EVENT_HANDLER {
            handler(event);
        }
    }
}

// MARK: EventLoopProxy
pub(crate) struct PlatformEventLoopProxy;

impl PlatformEventLoopProxy {
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl crate::EventLoopProxyInterface for PlatformEventLoopProxy {
    fn send_user_event(&self, data: String) {
        let ptr = Box::leak(Box::new(Event::UserEvent(data))) as *mut Event as *mut c_void;
        unsafe { g_idle_add(send_event_callback, ptr) };
    }
}

extern "C" fn send_event_callback(ptr: *mut c_void) -> i32 {
    let event = unsafe { Box::from_raw(ptr as *mut Event) };
    send_event(*event);
    0
}

// MARK: GTK progress bar
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
        APP_ID
            .as_ref()
            .map(|app_id| {
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

// MARK: Monitor
cfg_select! {
    gtk3_22 => {
        pub(crate) struct PlatformMonitor {
            pub(crate) monitor: *mut GdkMonitor,
        }

        impl PlatformMonitor {
            pub(crate) const fn new(monitor: *mut GdkMonitor) -> Self {
                Self { monitor }
            }
        }
    }
    _ => {
        pub(crate) struct PlatformMonitor {
            pub(crate) index: i32,
        }

        impl PlatformMonitor {
            pub(crate) const fn new(index: i32) -> Self {
                Self { index }
            }
        }
    }
}

impl PlatformMonitor {
    // Returns the screen rectangle for this monitor.
    pub(super) fn rect(&self) -> GdkRectangle {
        cfg_select! {
            gtk3_22 => {
                let mut rect = MaybeUninit::<GdkRectangle>::uninit();
                unsafe { gdk_monitor_get_geometry(self.monitor, rect.as_mut_ptr()) };
                unsafe { rect.assume_init() }
            }
            _ => {
                let screen = unsafe { gdk_screen_get_default() };
                let mut rect = MaybeUninit::<GdkRectangle>::uninit();
                unsafe { gdk_screen_get_monitor_geometry(screen, self.index, rect.as_mut_ptr()) };
                unsafe { rect.assume_init() }
            }
        }
    }
}

impl crate::MonitorInterface for PlatformMonitor {
    fn name(&self) -> String {
        cfg_select! {
            gtk3_22 => unsafe {
                CStr::from_ptr(gdk_monitor_get_model(self.monitor))
                    .to_string_lossy()
                    .into_owned()
            },
            _ => unsafe {
                let ptr = gdk_screen_get_monitor_plug_name(gdk_screen_get_default(), self.index);
                if ptr.is_null() {
                    format!("Monitor {}", self.index)
                } else {
                    let name = CStr::from_ptr(ptr).to_string_lossy().into_owned();
                    g_free(ptr as *mut c_void);
                    name
                }
            },
        }
    }

    fn position(&self) -> LogicalPoint {
        let rect = self.rect();
        let primary_rect = primary_monitor_rect();
        LogicalPoint::new(
            (rect.x - primary_rect.x) as f32,
            (rect.y - primary_rect.y) as f32,
        )
    }

    fn size(&self) -> LogicalSize {
        let rect = self.rect();
        LogicalSize::new(rect.width as f32, rect.height as f32)
    }

    fn scale_factor(&self) -> f32 {
        cfg_select! {
            gtk3_22 => unsafe { gdk_monitor_get_scale_factor(self.monitor) as f32 },
            _ => unsafe {
                gdk_screen_get_monitor_scale_factor(gdk_screen_get_default(), self.index) as f32
            },
        }
    }

    fn is_primary(&self) -> bool {
        cfg_select! {
            gtk3_22 => unsafe { gdk_monitor_is_primary(self.monitor) },
            _ => unsafe { gdk_screen_get_primary_monitor(gdk_screen_get_default()) == self.index },
        }
    }
}

// Returns the screen rectangle of the primary monitor.
pub(super) fn primary_monitor_rect() -> GdkRectangle {
    cfg_select! {
        gtk3_22 => unsafe {
            let display = gdk_display_get_default();
            let mut m = gdk_display_get_primary_monitor(display);
            if m.is_null() {
                m = gdk_display_get_monitor(display, 0);
            }
            let mut r = MaybeUninit::<GdkRectangle>::uninit();
            gdk_monitor_get_geometry(m, r.as_mut_ptr());
            r.assume_init()
        },
        _ => unsafe {
            let screen = gdk_screen_get_default();
            let idx = gdk_screen_get_primary_monitor(screen);
            // Normalize: -1 means no primary set, fall back to monitor 0.
            let idx = if idx < 0 { 0 } else { idx };
            let mut r = MaybeUninit::<GdkRectangle>::uninit();
            gdk_screen_get_monitor_geometry(screen, idx, r.as_mut_ptr());
            r.assume_init()
        },
    }
}
