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
use crate::{
    AppId, Event, EventLoopBuilder, LogicalPoint, LogicalSize, ProgressBarState, Theme,
};

pub(super) static mut APP_ID: Option<AppId> = None;
static mut EVENT_HANDLER: Option<Box<dyn FnMut(Event) + 'static>> = None;
pub(super) static mut FIRST_HWND: Option<HWND> = None;
pub(super) static mut TASKBAR_BUTTON_CREATED: u32 = 0;
static mut TASKBAR_BUTTON_READY: bool = false;
static mut PENDING_PROGRESS_BAR: ProgressBarState = ProgressBarState::None;

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
            TASKBAR_BUTTON_CREATED =
                RegisterWindowMessageW(wide!("TaskbarButtonCreated").as_ptr());

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

    fn set_progress_bar(&self, state: ProgressBarState) {
        set_progress_bar(state);
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
pub(super) const WM_SET_PROGRESS_BAR: u32 = WM_USER + 2;

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

    fn set_progress_bar(&self, state: ProgressBarState) {
        if let Some(hwnd) = unsafe { FIRST_HWND } {
            let ptr = Box::into_raw(Box::new(state));
            if unsafe { PostMessageW(hwnd, WM_SET_PROGRESS_BAR, ptr as WPARAM, 0) } == FALSE {
                // The window may have closed between reading FIRST_HWND and posting.
                unsafe { drop(Box::from_raw(ptr)) };
            }
        }
    }
}

// MARK: Taskbar progress
const CLSID_TASKBAR_LIST: GUID = GUID {
    data1: 0x56FDF344,
    data2: 0xFD6D,
    data3: 0x11D0,
    data4: [0x95, 0x8A, 0x00, 0x60, 0x97, 0xC9, 0xA0, 0x90],
};
const IID_TASKBAR_LIST3: GUID = GUID {
    data1: 0xEA1AFB91,
    data2: 0x9E28,
    data3: 0x4B86,
    data4: [0x90, 0xE9, 0x9E, 0x9F, 0x8A, 0x5E, 0xEF, 0xAF],
};

#[repr(C)]
struct TaskbarList3 {
    vtable: *const TaskbarList3Vtable,
}

#[repr(C)]
struct TaskbarList3Vtable {
    query_interface: usize,
    add_ref: usize,
    release: unsafe extern "system" fn(*mut TaskbarList3) -> u32,
    hr_init: unsafe extern "system" fn(*mut TaskbarList3) -> HRESULT,
    add_tab: usize,
    delete_tab: usize,
    activate_tab: usize,
    set_active_alt: usize,
    mark_fullscreen_window: usize,
    set_progress_value:
        unsafe extern "system" fn(*mut TaskbarList3, HWND, u64, u64) -> HRESULT,
    set_progress_state: unsafe extern "system" fn(*mut TaskbarList3, HWND, u32) -> HRESULT,
}

pub(super) fn set_progress_bar(state: ProgressBarState) {
    unsafe { PENDING_PROGRESS_BAR = state };
    if !unsafe { TASKBAR_BUTTON_READY } {
        return;
    }
    let Some(hwnd) = (unsafe { FIRST_HWND }) else {
        return;
    };
    let mut taskbar = null_mut::<TaskbarList3>();
    let result = unsafe {
        CoCreateInstance(
            &CLSID_TASKBAR_LIST,
            null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_TASKBAR_LIST3,
            &mut taskbar as *mut _ as *mut *mut c_void,
        )
    };
    if result < 0 || taskbar.is_null() {
        return;
    }
    unsafe {
        let vtable = &*(*taskbar).vtable;
        let init_result = (vtable.hr_init)(taskbar);
        if init_result >= 0 {
            let taskbar_state = match state {
                ProgressBarState::None => 0,
                ProgressBarState::Indeterminate => 1,
                ProgressBarState::Normal(_) => 2,
                ProgressBarState::Error(_) => 4,
                ProgressBarState::Paused(_) => 8,
            };
            if let Some(progress) = state.progress() {
                _ =
                    (vtable.set_progress_value)(taskbar, hwnd, (progress * 1000.0) as u64, 1000);
            }
            // An explicit normal state is required to clear paused/error states.
            _ = (vtable.set_progress_state)(taskbar, hwnd, taskbar_state);
        }
        _ = (vtable.release)(taskbar);
    }
}

pub(super) fn taskbar_button_created() {
    unsafe { TASKBAR_BUTTON_READY = true };
    set_progress_bar(unsafe { PENDING_PROGRESS_BAR });
}

#[cfg(test)]
mod taskbar_tests {
    use super::IID_TASKBAR_LIST3;

    #[test]
    fn taskbar_list3_iid_matches_windows_sdk() {
        assert_eq!(IID_TASKBAR_LIST3.data1, 0xEA1AFB91);
        assert_eq!(IID_TASKBAR_LIST3.data2, 0x9E28);
        assert_eq!(IID_TASKBAR_LIST3.data3, 0x4B86);
        assert_eq!(
            IID_TASKBAR_LIST3.data4,
            [0x90, 0xE9, 0x9E, 0x9F, 0x8A, 0x5E, 0xEF, 0xAF]
        );
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
