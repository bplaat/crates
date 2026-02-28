/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::CString;
use std::mem::{self, size_of};
use std::process::exit;
use std::ptr::{null, null_mut};

use super::webview2::*;
use super::win32::*;
use crate::{AppId, Event, EventLoopBuilder, LogicalPoint, LogicalSize};

pub(super) static mut APP_ID: Option<AppId> = None;
static mut EVENT_HANDLER: Option<Box<dyn FnMut(Event) + 'static>> = None;
pub(super) static mut FIRST_HWND: Option<HWND> = None;

// MARK: EventLoop
pub(crate) struct PlatformEventLoop;

impl PlatformEventLoop {
    pub(crate) fn new(builder: EventLoopBuilder) -> Self {
        unsafe {
            // Ensure single instance
            if let Some(app_id) = builder.app_id {
                let mutex_name = format!(
                    "bwebview-{}.{}.{}",
                    app_id.qualifier, app_id.organization, app_id.application
                );
                let mutex_name_c = CString::new(mutex_name).expect("Can't convert to CString");
                CreateMutexA(null_mut(), TRUE, mutex_name_c.as_ptr());
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    let hwnd = FindWindowA(mutex_name_c.as_ptr(), null_mut());
                    if !hwnd.is_null() {
                        ShowWindow(hwnd, SW_RESTORE);
                        SetForegroundWindow(hwnd);
                    }
                    exit(0);
                }
                APP_ID = Some(app_id);
            }

            // Initialize COM
            CoInitializeEx(
                null_mut(),
                COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
            );

            // Enable PerMonitorV2 high DPI awareness
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

            Self
        }
    }
}

impl crate::EventLoopInterface for PlatformEventLoop {
    fn primary_monitor(&self) -> PlatformMonitor {
        let hmonitor = unsafe { MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) };
        PlatformMonitor::new(hmonitor)
    }

    fn available_monitors(&self) -> Vec<PlatformMonitor> {
        static mut MONITORS: Option<Vec<PlatformMonitor>> = None;
        unsafe extern "system" fn monitor_enum_proc(
            hmonitor: HMONITOR,
            _hdc_monitor: HDC,
            _lprc_monitor: *const RECT,
            _dw_data: LPARAM,
        ) -> BOOL {
            unsafe {
                #[allow(static_mut_refs)]
                if let Some(monitors) = &mut MONITORS {
                    monitors.push(PlatformMonitor::new(hmonitor));
                }
            }
            true.into()
        }
        unsafe {
            MONITORS = Some(Vec::new());
            EnumDisplayMonitors(null_mut(), null_mut(), monitor_enum_proc, 0);
            #[allow(static_mut_refs)]
            MONITORS.take().unwrap_or_default()
        }
    }

    fn run(self, event_handler: impl FnMut(Event) + 'static) -> ! {
        unsafe { EVENT_HANDLER = Some(Box::new(event_handler)) };

        // Start message loop
        unsafe {
            let mut msg = mem::zeroed();
            while GetMessageA(&mut msg, null_mut(), 0, 0) != 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
            CoUninitialize();
            exit(msg.wParam as i32);
        }
    }

    fn create_proxy(&self) -> PlatformEventLoopProxy {
        PlatformEventLoopProxy::new()
    }
}

pub(crate) fn send_event(event: Event) {
    unsafe {
        #[allow(static_mut_refs)]
        if let Some(handler) = &mut EVENT_HANDLER {
            handler(event);
        }
    }
}

// MARK: EventLoopProxy
const WM_SEND_MESSAGE: u32 = WM_USER + 1;

pub(crate) struct PlatformEventLoopProxy;

impl PlatformEventLoopProxy {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl crate::EventLoopProxyInterface for PlatformEventLoopProxy {
    fn send_user_event(&self, data: String) {
        if let Some(hwnd) = unsafe { FIRST_HWND } {
            let ptr =
                Box::leak(Box::new(Event::UserEvent(data))) as *mut Event as *mut std::ffi::c_void;
            unsafe { PostMessageA(hwnd, WM_SEND_MESSAGE, ptr as WPARAM, 0) };
        }
    }
}

// MARK: Monitor
pub(crate) struct PlatformMonitor {
    hmonitor: HMONITOR,
    info: MONITORINFOEXA,
}

impl PlatformMonitor {
    pub(crate) fn new(hmonitor: HMONITOR) -> Self {
        let mut info = MONITORINFOEXA {
            cbSize: size_of::<MONITORINFOEXA>() as u32,
            ..Default::default()
        };
        unsafe {
            GetMonitorInfoA(hmonitor, &mut info as *mut _ as *mut _);
        }
        Self { hmonitor, info }
    }

    pub(crate) fn rect(&self) -> RECT {
        self.info.rcMonitor.clone()
    }
}

impl crate::MonitorInterface for PlatformMonitor {
    fn name(&self) -> String {
        let byte_vec: Vec<u8> = self
            .info
            .szDevice
            .iter()
            .take_while(|&x| *x != 0)
            .map(|&x| x as u8)
            .collect();
        String::from_utf8(byte_vec).expect("Can't parse string")
    }

    fn position(&self) -> LogicalPoint {
        LogicalPoint::new(
            self.info.rcMonitor.left as f32,
            self.info.rcMonitor.top as f32,
        )
    }

    fn size(&self) -> LogicalSize {
        LogicalSize::new(
            (self.info.rcMonitor.right - self.info.rcMonitor.left) as f32,
            (self.info.rcMonitor.bottom - self.info.rcMonitor.top) as f32,
        )
    }

    fn scale_factor(&self) -> f32 {
        unsafe {
            let mut dpi_x = USER_DEFAULT_SCREEN_DPI;
            let mut dpi_y = USER_DEFAULT_SCREEN_DPI;
            let result = GetDpiForMonitor(self.hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
            if result == S_OK {
                dpi_x as f32 / USER_DEFAULT_SCREEN_DPI as f32
            } else {
                1.0
            }
        }
    }

    fn is_primary(&self) -> bool {
        self.info.rcMonitor.left == 0 && self.info.rcMonitor.top == 0
    }
}
