/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;
use std::mem::{self, size_of};
use std::process::exit;
use std::ptr::{null, null_mut};

use super::webview2::*;
use super::win32::*;
use crate::{AppId, Event, EventLoopBuilder, LogicalPoint, LogicalSize, Theme};

pub(super) static mut APP_ID: Option<AppId> = None;
static mut EVENT_HANDLER: Option<Box<dyn FnMut(Event) + 'static>> = None;
pub(super) static mut FIRST_HWND: Option<HWND> = None;
pub(super) static mut TASKBAR_BUTTON_CREATED: u32 = 0;

// MARK: EventLoop
pub(crate) struct PlatformEventLoop {
    theme: Theme,
}

impl PlatformEventLoop {
    pub(crate) fn new(builder: EventLoopBuilder) -> Self {
        unsafe {
            // Ensure single instance
            if let Some(app_id) = builder.app_id {
                let mutex_name = format!(
                    "bwebview-{}.{}.{}",
                    app_id.qualifier, app_id.organization, app_id.application
                );
                let mutex_name_w = mutex_name.to_wide_string();
                CreateMutexW(null_mut(), TRUE, mutex_name_w.as_ptr());
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    let hwnd = FindWindowW(mutex_name_w.as_ptr(), null());
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

            // Explorer creates the taskbar button asynchronously. Shell APIs must
            // not be used until the window receives this registered message.
            TASKBAR_BUTTON_CREATED = RegisterWindowMessageW(wide!("TaskbarButtonCreated").as_ptr());

            enable_high_dpi_awareness();

            Self {
                theme: system_theme(),
            }
        }
    }
}

// Enable PerMonitorV2 when available without requiring Windows 10 1703
unsafe fn enable_high_dpi_awareness() {
    type SetProcessDpiAwarenessContext = unsafe extern "system" fn(isize) -> BOOL;

    let user32 = unsafe { GetModuleHandleW(wide!("user32.dll").as_ptr()) };
    if !user32.is_null() {
        let proc = unsafe { GetProcAddress(user32, c"SetProcessDpiAwarenessContext".as_ptr()) };
        if !proc.is_null() {
            // SAFETY: proc points to SetProcessDpiAwarenessContext with this signature.
            let set_context: SetProcessDpiAwarenessContext = unsafe { mem::transmute(proc) };
            if unsafe { set_context(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) } != FALSE {
                return;
            }
        }
    }

    // Fall back to PerMonitor on older Windows versions
    unsafe {
        SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE);
    }
}

// MARK: Theme
pub(super) fn system_theme() -> Theme {
    let mut apps_use_light_theme = 1u32;
    let mut size = size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            wide!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize").as_ptr(),
            wide!("AppsUseLightTheme").as_ptr(),
            RRF_RT_REG_DWORD,
            null_mut(),
            &mut apps_use_light_theme as *mut _ as *mut c_void,
            &mut size,
        )
    };
    if status == ERROR_SUCCESS && apps_use_light_theme == 0 {
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
            while GetMessageW(&mut msg, null_mut(), 0, 0) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
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
pub(super) const WM_SEND_MESSAGE: u32 = WM_USER + 1;

pub(crate) struct PlatformEventLoopProxy;

impl PlatformEventLoopProxy {
    pub(crate) const fn new() -> Self {
        Self
    }
}

impl crate::EventLoopProxyInterface for PlatformEventLoopProxy {
    fn send_user_event(&self, data: String) {
        if let Some(hwnd) = unsafe { FIRST_HWND } {
            let ptr = Box::leak(Box::new(Event::UserEvent(data))) as *mut Event as *mut c_void;
            unsafe { PostMessageW(hwnd, WM_SEND_MESSAGE, ptr as WPARAM, 0) };
        }
    }
}

// MARK: Monitor
pub(crate) struct PlatformMonitor {
    hmonitor: HMONITOR,
    info: MONITORINFOEXW,
}

impl PlatformMonitor {
    pub(crate) fn new(hmonitor: HMONITOR) -> Self {
        let mut info = MONITORINFOEXW {
            cbSize: size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        };
        unsafe {
            GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut _);
        }
        Self { hmonitor, info }
    }

    pub(crate) fn rect(&self) -> RECT {
        self.info.rcMonitor.clone()
    }
}

impl crate::MonitorInterface for PlatformMonitor {
    fn name(&self) -> String {
        let len = self
            .info
            .szDevice
            .iter()
            .position(|&unit| unit == 0)
            .unwrap_or(self.info.szDevice.len());
        String::from_utf16_lossy(&self.info.szDevice[..len])
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
