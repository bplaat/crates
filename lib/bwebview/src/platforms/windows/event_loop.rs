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
use super::window::WindowData;
use crate::{
    AppId, Event, EventLoopBuilder, KeyCode, LogicalPoint, LogicalSize, Modifiers, MouseButton,
    WindowEvent,
};

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
                // Intercept keyboard and mouse events before dispatch
                intercept_msg(&msg);
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

fn intercept_msg(msg: &MSG) {
    let window_data = unsafe {
        let ptr = GetWindowLong(msg.hwnd, GWL_USERDATA) as *mut WindowData;
        if ptr.is_null() {
            return;
        }
        &mut *ptr
    };
    let dpi = window_data.dpi;

    match msg.message {
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            let key = vk_to_keycode(msg.wParam as u32);
            let modifiers = get_modifiers();
            send_event(Event::Window(WindowEvent::KeyDown { key, modifiers }));
        }
        WM_KEYUP | WM_SYSKEYUP => {
            let key = vk_to_keycode(msg.wParam as u32);
            let modifiers = get_modifiers();
            send_event(Event::Window(WindowEvent::KeyUp { key, modifiers }));
        }
        WM_CHAR => {
            if let Some(ch) = char::from_u32(msg.wParam as u32) {
                if !ch.is_control() {
                    send_event(Event::Window(WindowEvent::Char(ch)));
                }
            }
        }
        WM_LBUTTONDOWN => {
            let pos = lparam_to_logical_point(msg.lParam, dpi);
            send_event(Event::Window(WindowEvent::MouseDown {
                button: MouseButton::Left,
                position: pos,
            }));
        }
        WM_LBUTTONUP => {
            let pos = lparam_to_logical_point(msg.lParam, dpi);
            send_event(Event::Window(WindowEvent::MouseUp {
                button: MouseButton::Left,
                position: pos,
            }));
        }
        WM_RBUTTONDOWN => {
            let pos = lparam_to_logical_point(msg.lParam, dpi);
            send_event(Event::Window(WindowEvent::MouseDown {
                button: MouseButton::Right,
                position: pos,
            }));
        }
        WM_RBUTTONUP => {
            let pos = lparam_to_logical_point(msg.lParam, dpi);
            send_event(Event::Window(WindowEvent::MouseUp {
                button: MouseButton::Right,
                position: pos,
            }));
        }
        WM_MBUTTONDOWN => {
            let pos = lparam_to_logical_point(msg.lParam, dpi);
            send_event(Event::Window(WindowEvent::MouseDown {
                button: MouseButton::Middle,
                position: pos,
            }));
        }
        WM_MBUTTONUP => {
            let pos = lparam_to_logical_point(msg.lParam, dpi);
            send_event(Event::Window(WindowEvent::MouseUp {
                button: MouseButton::Middle,
                position: pos,
            }));
        }
        WM_XBUTTONDOWN => {
            let xbutton = GET_XBUTTON_WPARAM(msg.wParam);
            let button = if xbutton == 1 {
                MouseButton::Back
            } else {
                MouseButton::Forward
            };
            let pos = lparam_to_logical_point(msg.lParam, dpi);
            send_event(Event::Window(WindowEvent::MouseDown { button, position: pos }));
        }
        WM_XBUTTONUP => {
            let xbutton = GET_XBUTTON_WPARAM(msg.wParam);
            let button = if xbutton == 1 {
                MouseButton::Back
            } else {
                MouseButton::Forward
            };
            let pos = lparam_to_logical_point(msg.lParam, dpi);
            send_event(Event::Window(WindowEvent::MouseUp { button, position: pos }));
        }
        WM_MOUSEWHEEL => {
            let delta = GET_WHEEL_DELTA_WPARAM(msg.wParam) as f32 / WHEEL_DELTA;
            send_event(Event::Window(WindowEvent::MouseWheel {
                delta_x: 0.0,
                delta_y: -delta,
            }));
        }
        WM_MOUSEHWHEEL => {
            let delta = GET_WHEEL_DELTA_WPARAM(msg.wParam) as f32 / WHEEL_DELTA;
            send_event(Event::Window(WindowEvent::MouseWheel {
                delta_x: delta,
                delta_y: 0.0,
            }));
        }
        WM_MOUSEMOVE => {
            let pos = lparam_to_logical_point(msg.lParam, dpi);
            if !window_data.tracking_mouse {
                window_data.tracking_mouse = true;
                unsafe {
                    let mut tme = TRACKMOUSEEVENT {
                        cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE,
                        hwndTrack: msg.hwnd,
                        dwHoverTime: 0,
                    };
                    TrackMouseEvent(&mut tme);
                }
                send_event(Event::Window(WindowEvent::MouseEnter));
            }
            send_event(Event::Window(WindowEvent::MouseMove(pos)));
        }
        _ => {}
    }
}

fn lparam_to_logical_point(lparam: LPARAM, dpi: u32) -> LogicalPoint {
    let x = GET_X_LPARAM(lparam);
    let y = GET_Y_LPARAM(lparam);
    LogicalPoint::new(
        (x * USER_DEFAULT_SCREEN_DPI as i32 / dpi as i32) as f32,
        (y * USER_DEFAULT_SCREEN_DPI as i32 / dpi as i32) as f32,
    )
}

fn get_modifiers() -> Modifiers {
    Modifiers {
        shift: unsafe { GetKeyState(VK_SHIFT as i32) } < 0,
        ctrl: unsafe { GetKeyState(VK_CONTROL as i32) } < 0,
        alt: unsafe { GetKeyState(VK_MENU as i32) } < 0,
        meta: unsafe { GetKeyState(VK_LWIN as i32) } < 0
            || unsafe { GetKeyState(VK_RWIN as i32) } < 0,
    }
}

fn vk_to_keycode(vk: u32) -> KeyCode {
    match vk {
        0x41 => KeyCode::A,
        0x42 => KeyCode::B,
        0x43 => KeyCode::C,
        0x44 => KeyCode::D,
        0x45 => KeyCode::E,
        0x46 => KeyCode::F,
        0x47 => KeyCode::G,
        0x48 => KeyCode::H,
        0x49 => KeyCode::I,
        0x4A => KeyCode::J,
        0x4B => KeyCode::K,
        0x4C => KeyCode::L,
        0x4D => KeyCode::M,
        0x4E => KeyCode::N,
        0x4F => KeyCode::O,
        0x50 => KeyCode::P,
        0x51 => KeyCode::Q,
        0x52 => KeyCode::R,
        0x53 => KeyCode::S,
        0x54 => KeyCode::T,
        0x55 => KeyCode::U,
        0x56 => KeyCode::V,
        0x57 => KeyCode::W,
        0x58 => KeyCode::X,
        0x59 => KeyCode::Y,
        0x5A => KeyCode::Z,
        0x30 => KeyCode::Key0,
        0x31 => KeyCode::Key1,
        0x32 => KeyCode::Key2,
        0x33 => KeyCode::Key3,
        0x34 => KeyCode::Key4,
        0x35 => KeyCode::Key5,
        0x36 => KeyCode::Key6,
        0x37 => KeyCode::Key7,
        0x38 => KeyCode::Key8,
        0x39 => KeyCode::Key9,
        vk if vk >= VK_F1 && vk <= VK_F12 => match vk - VK_F1 {
            0 => KeyCode::F1,
            1 => KeyCode::F2,
            2 => KeyCode::F3,
            3 => KeyCode::F4,
            4 => KeyCode::F5,
            5 => KeyCode::F6,
            6 => KeyCode::F7,
            7 => KeyCode::F8,
            8 => KeyCode::F9,
            9 => KeyCode::F10,
            10 => KeyCode::F11,
            _ => KeyCode::F12,
        },
        VK_BACK => KeyCode::Backspace,
        VK_TAB => KeyCode::Tab,
        VK_RETURN => KeyCode::Enter,
        VK_ESCAPE => KeyCode::Escape,
        VK_SPACE => KeyCode::Space,
        VK_DELETE => KeyCode::Delete,
        VK_INSERT => KeyCode::Insert,
        VK_LEFT => KeyCode::Left,
        VK_RIGHT => KeyCode::Right,
        VK_UP => KeyCode::Up,
        VK_DOWN => KeyCode::Down,
        VK_HOME => KeyCode::Home,
        VK_END => KeyCode::End,
        VK_PRIOR => KeyCode::PageUp,
        VK_NEXT => KeyCode::PageDown,
        VK_SHIFT | VK_LSHIFT | VK_RSHIFT => KeyCode::Shift,
        VK_CONTROL | VK_LCONTROL | VK_RCONTROL => KeyCode::Control,
        VK_MENU | VK_LMENU | VK_RMENU => KeyCode::Alt,
        VK_LWIN | VK_RWIN => KeyCode::Meta,
        VK_CAPITAL => KeyCode::CapsLock,
        _ => KeyCode::Unknown(vk),
    }
}

// MARK: EventLoopProxy
pub(super) const WM_SEND_MESSAGE: u32 = WM_USER + 1;

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
