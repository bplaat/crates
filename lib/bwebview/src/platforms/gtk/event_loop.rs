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
use crate::{AppId, Event, EventLoopBuilder, LogicalPoint, LogicalSize};

// MARK: EventLoop
pub(crate) struct PlatformEventLoop;

pub(super) static mut APP_ID: Option<AppId> = None;
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
        #[cfg(gtk3_22)]
        unsafe {
            let mut m = gdk_display_get_primary_monitor(gdk_display_get_default());
            if m.is_null() {
                m = gdk_display_get_monitor(gdk_display_get_default(), 0);
            }
            PlatformMonitor::new(m)
        }
        #[cfg(not(gtk3_22))]
        unsafe {
            let screen = gdk_screen_get_default();
            let idx = gdk_screen_get_primary_monitor(screen);
            PlatformMonitor::new(idx)
        }
    }

    fn available_monitors(&self) -> Vec<PlatformMonitor> {
        #[cfg(gtk3_22)]
        unsafe {
            let display = gdk_display_get_default();
            let mut monitors = Vec::new();
            for i in 0..gdk_display_get_n_monitors(display) {
                monitors.push(PlatformMonitor::new(gdk_display_get_monitor(display, i)));
            }
            monitors
        }
        #[cfg(not(gtk3_22))]
        unsafe {
            let screen = gdk_screen_get_default();
            (0..gdk_screen_get_n_monitors(screen))
                .map(|i| PlatformMonitor::new(i))
                .collect()
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
    #[cfg(gtk3_22)]
    pub(crate) monitor: *mut GdkMonitor,
    #[cfg(not(gtk3_22))]
    pub(crate) index: i32,
}

impl PlatformMonitor {
    #[cfg(gtk3_22)]
    pub(crate) fn new(monitor: *mut GdkMonitor) -> Self {
        Self { monitor }
    }

    #[cfg(not(gtk3_22))]
    pub(crate) fn new(index: i32) -> Self {
        Self { index }
    }
}

impl PlatformMonitor {
    // Returns the screen rectangle for this monitor.
    pub(super) fn rect(&self) -> GdkRectangle {
        #[cfg(gtk3_22)]
        {
            let mut rect = MaybeUninit::<GdkRectangle>::uninit();
            unsafe { gdk_monitor_get_geometry(self.monitor, rect.as_mut_ptr()) };
            unsafe { rect.assume_init() }
        }
        #[cfg(not(gtk3_22))]
        {
            let screen = unsafe { gdk_screen_get_default() };
            let mut rect = MaybeUninit::<GdkRectangle>::uninit();
            unsafe { gdk_screen_get_monitor_geometry(screen, self.index, rect.as_mut_ptr()) };
            unsafe { rect.assume_init() }
        }
    }
}

impl crate::MonitorInterface for PlatformMonitor {
    fn name(&self) -> String {
        #[cfg(gtk3_22)]
        unsafe {
            CStr::from_ptr(gdk_monitor_get_model(self.monitor))
                .to_string_lossy()
                .into_owned()
        }
        #[cfg(not(gtk3_22))]
        unsafe {
            let ptr = gdk_screen_get_monitor_plug_name(gdk_screen_get_default(), self.index);
            if ptr.is_null() {
                format!("Monitor {}", self.index)
            } else {
                let name = CStr::from_ptr(ptr).to_string_lossy().into_owned();
                g_free(ptr as *mut c_void);
                name
            }
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
        #[cfg(gtk3_22)]
        unsafe { gdk_monitor_get_scale_factor(self.monitor) as f32 }
        #[cfg(not(gtk3_22))]
        unsafe {
            gdk_screen_get_monitor_scale_factor(gdk_screen_get_default(), self.index) as f32
        }
    }

    fn is_primary(&self) -> bool {
        #[cfg(gtk3_22)]
        unsafe { gdk_monitor_is_primary(self.monitor) }
        #[cfg(not(gtk3_22))]
        unsafe { gdk_screen_get_primary_monitor(gdk_screen_get_default()) == self.index }
    }
}

// Returns the screen rectangle of the primary monitor.
pub(super) fn primary_monitor_rect() -> GdkRectangle {
    #[cfg(gtk3_22)]
    unsafe {
        let display = gdk_display_get_default();
        let mut m = gdk_display_get_primary_monitor(display);
        if m.is_null() {
            m = gdk_display_get_monitor(display, 0);
        }
        let mut r = MaybeUninit::<GdkRectangle>::uninit();
        gdk_monitor_get_geometry(m, r.as_mut_ptr());
        r.assume_init()
    }
    #[cfg(not(gtk3_22))]
    unsafe {
        let screen = gdk_screen_get_default();
        let idx = gdk_screen_get_primary_monitor(screen);
        // Normalize: -1 means no primary set, fall back to monitor 0.
        let idx = if idx < 0 { 0 } else { idx };
        let mut r = MaybeUninit::<GdkRectangle>::uninit();
        gdk_screen_get_monitor_geometry(screen, idx, r.as_mut_ptr());
        r.assume_init()
    }
}
