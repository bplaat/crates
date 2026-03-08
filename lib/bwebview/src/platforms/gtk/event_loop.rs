/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CStr, CString, c_char, c_void};
use std::mem::MaybeUninit;
use std::process::exit;
use std::ptr::{null, null_mut};
use std::{env, iter};

use super::headers::*;
use crate::{AppId, EventLoopBuilder, EventLoopHandler, LogicalPoint, LogicalSize};

// MARK: EventLoop
pub(crate) struct PlatformEventLoop {
    app_id: Option<AppId>,
    event_loop_handler: Option<*mut dyn EventLoopHandler>,
}

pub(super) static mut APP_ID: Option<AppId> = None;
static mut GTK_APPLICATION: *mut GtkApplication = null_mut();
static mut EVENT_LOOP_HANDLER: Option<*mut dyn EventLoopHandler> = None;

impl PlatformEventLoop {
    pub(crate) fn new(builder: EventLoopBuilder) -> Self {
        Self {
            app_id: builder.app_id,
            event_loop_handler: builder.event_loop_handler,
        }
    }
}

impl crate::EventLoopInterface for PlatformEventLoop {
    fn primary_monitor(&self) -> PlatformMonitor {
        unsafe {
            // GTK must be initialized before monitor queries; init lazily here
            let mut primary_monitor = gdk_display_get_primary_monitor(gdk_display_get_default());
            if primary_monitor.is_null() {
                primary_monitor = gdk_display_get_monitor(gdk_display_get_default(), 0);
            }
            PlatformMonitor::new(primary_monitor)
        }
    }

    fn available_monitors(&self) -> Vec<PlatformMonitor> {
        unsafe {
            let display = gdk_display_get_default();
            let mut monitors = Vec::new();
            for i in 0..gdk_display_get_n_monitors(display) {
                monitors.push(PlatformMonitor::new(gdk_display_get_monitor(display, i)));
            }
            monitors
        }
    }

    fn run(self) -> ! {
        unsafe {
            EVENT_LOOP_HANDLER = self.event_loop_handler;

            // Build app_id string for GtkApplication
            let app_id_str = if let Some(ref app_id) = self.app_id {
                APP_ID = Some(app_id.clone());
                format!(
                    "{}.{}.{}",
                    app_id.qualifier, app_id.organization, app_id.application
                )
            } else {
                "org.bwebview.app".to_string()
            };
            let app_id_c = CString::new(app_id_str).expect("Can't convert app_id to CString");

            let app = gtk_application_new(app_id_c.as_ptr(), G_APPLICATION_FLAGS_NONE);
            GTK_APPLICATION = app;

            // Connect "activate" signal to on_init
            g_signal_connect_data(
                app as *mut GObject,
                c"activate".as_ptr(),
                app_activate as *const c_void,
                null(),
                null(),
                G_CONNECT_DEFAULT,
            );

            // Run the application
            let args = env::args()
                .map(|arg| CString::new(arg.as_str()).expect("Can't convert to CString"))
                .collect::<Vec<CString>>();
            let mut argv: Vec<*mut c_char> = args
                .iter()
                .map(|arg| arg.as_ptr() as *mut c_char)
                .chain(iter::once(null_mut()))
                .collect();
            let ret = g_application_run(app, args.len() as i32, argv.as_mut_ptr());
            exit(ret);
        }
    }

    fn quit() {
        unsafe {
            #[allow(static_mut_refs)]
            if !GTK_APPLICATION.is_null() {
                g_application_quit(GTK_APPLICATION);
            }
        }
    }

    fn create_proxy(&self) -> PlatformEventLoopProxy {
        PlatformEventLoopProxy::new()
    }
}

extern "C" fn app_activate(_app: *mut GtkApplication, _user_data: *const c_void) {
    unsafe {
        #[allow(static_mut_refs)]
        if let Some(h_ptr) = EVENT_LOOP_HANDLER {
            (*h_ptr).on_init();
        }
    }
}

// MARK: EventLoopProxy
pub(crate) struct PlatformEventLoopProxy;

impl PlatformEventLoopProxy {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl crate::EventLoopProxyInterface for PlatformEventLoopProxy {
    fn send_user_event(&self, data: String) {
        let ptr = Box::leak(Box::new(data)) as *mut String as *mut c_void;
        unsafe { g_idle_add(send_user_event_callback, ptr) };
    }
}

extern "C" fn send_user_event_callback(ptr: *mut c_void) -> i32 {
    let data = unsafe { Box::from_raw(ptr as *mut String) };
    unsafe {
        #[allow(static_mut_refs)]
        if let Some(h_ptr) = EVENT_LOOP_HANDLER {
            (*h_ptr).on_user_event(*data);
        }
    }
    0
}

// MARK: Monitor
pub(crate) struct PlatformMonitor {
    pub(crate) monitor: *mut GdkMonitor,
}

impl PlatformMonitor {
    pub(crate) fn new(monitor: *mut GdkMonitor) -> Self {
        Self { monitor }
    }
}

impl crate::MonitorInterface for PlatformMonitor {
    fn name(&self) -> String {
        unsafe {
            let name_ptr = gdk_monitor_get_model(self.monitor);
            CStr::from_ptr(name_ptr).to_string_lossy().into_owned()
        }
    }

    fn position(&self) -> LogicalPoint {
        let mut rect = MaybeUninit::<GdkRectangle>::uninit();
        unsafe {
            gdk_monitor_get_geometry(self.monitor, rect.as_mut_ptr());
        }
        let rect = unsafe { rect.assume_init() };

        let primary_monitor_rect = unsafe {
            let mut primary_monitor = gdk_display_get_primary_monitor(gdk_display_get_default());
            if primary_monitor.is_null() {
                primary_monitor = gdk_display_get_monitor(gdk_display_get_default(), 0);
            }
            let mut primary_monitor_rect = MaybeUninit::<GdkRectangle>::uninit();
            gdk_monitor_get_geometry(primary_monitor, primary_monitor_rect.as_mut_ptr());
            primary_monitor_rect.assume_init()
        };
        LogicalPoint::new(
            (rect.x - primary_monitor_rect.x) as f32,
            (rect.y - primary_monitor_rect.y) as f32,
        )
    }

    fn size(&self) -> LogicalSize {
        let mut rect = MaybeUninit::<GdkRectangle>::uninit();
        unsafe {
            gdk_monitor_get_geometry(self.monitor, rect.as_mut_ptr());
        }
        let rect = unsafe { rect.assume_init() };
        LogicalSize::new(rect.width as f32, rect.height as f32)
    }

    fn scale_factor(&self) -> f32 {
        unsafe { gdk_monitor_get_scale_factor(self.monitor) as f32 }
    }

    fn is_primary(&self) -> bool {
        unsafe { gdk_monitor_is_primary(self.monitor) }
    }
}
