/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CStr, CString, c_char, c_void};
use std::fs::File;
use std::mem::MaybeUninit;
use std::os::unix::io::AsRawFd;
use std::process::exit;
use std::ptr::{null, null_mut};
use std::{env, fs, iter};

use super::headers::*;
use crate::{
    AppId, Event, EventLoopBuilder, InjectionTime, LogicalPoint, LogicalSize, WebviewBuilder,
};

// MARK: EventLoop
pub(crate) struct PlatformEventLoop;

static mut APP_ID: Option<AppId> = None;
static mut EVENT_HANDLER: Option<Box<dyn FnMut(Event) + 'static>> = None;

impl PlatformEventLoop {
    pub(crate) fn new(builder: EventLoopBuilder) -> Self {
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
            if unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) } != 0 {
                exit(0);
            }
            std::mem::forget(file);
            unsafe { APP_ID = Some(app_id) };
        }

        // Init GTK
        unsafe {
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
        }

        Self
    }
}

impl crate::EventLoopInterface for PlatformEventLoop {
    fn primary_monitor(&self) -> PlatformMonitor {
        unsafe {
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

fn send_event(event: Event) {
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
    pub(crate) fn new() -> Self {
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

// MARK: Monitor
pub(crate) struct PlatformMonitor {
    monitor: *mut GdkMonitor,
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

        // The GTK monitors are not offset by primary monitor position,
        // so we need to calculate the position relative to the primary monitor.
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
