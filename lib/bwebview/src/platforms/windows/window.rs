/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CString, c_void};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::{env, mem};

use super::event_loop::{APP_ID, EVENT_LOOP_HANDLER, FIRST_HWND, WM_SEND_MESSAGE};
#[cfg(feature = "webview")]
use super::webview2::*;
use super::win32::*;
use crate::{Key, LogicalPoint, LogicalSize, Modifiers, MouseButton, Theme, WindowBuilder, WindowHandler, WindowId};

pub(super) struct WindowData {
    pub(super) window_id: WindowId,
    pub(super) hwnd: HWND,
    pub(super) dpi: u32,
    pub(super) min_size: Option<LogicalSize>,
    pub(super) background_color: Option<u32>,
    pub(super) remember_window_state: bool,
    #[cfg(feature = "webview")]
    pub(super) controller: Option<*mut ICoreWebView2Controller>,
    pub(super) window_handler: Option<*mut dyn WindowHandler>,
    #[cfg(feature = "webview")]
    pub(super) webview_handler: Option<*mut dyn crate::WebviewHandler>,
}

pub(crate) struct PlatformWindow(pub(super) Box<WindowData>);

impl PlatformWindow {
    pub(crate) fn new(window_id: WindowId, builder: &WindowBuilder) -> Self {
        let dpi = unsafe { GetDpiForSystem() };

        // Check if window class is already registered
        let instance = unsafe { GetModuleHandleA(null_mut()) };
        let class_name = unsafe {
            if let Some(ref app_id) = APP_ID {
                format!(
                    "bwebview-{}.{}.{}",
                    app_id.qualifier, app_id.organization, app_id.application
                )
            } else {
                "bwebview".to_string()
            }
        };
        let class_name_c = CString::new(class_name).expect("Can't convert to CString");
        unsafe {
            let mut wndclass: WNDCLASSEXA = mem::zeroed();
            if GetClassInfoExA(instance, class_name_c.as_ptr(), &mut wndclass as *mut _) != TRUE {
                // Get executable icons
                let executable_path = CString::new(
                    env::current_exe()
                        .expect("Can't get current exe path")
                        .display()
                        .to_string(),
                )
                .expect("Can't convert to CString");
                let mut large_icon = HICON::default();
                let mut small_icon = HICON::default();
                ExtractIconExA(
                    executable_path.as_ptr(),
                    0,
                    &mut large_icon,
                    &mut small_icon,
                    1,
                );

                // Register window class
                let wndclass = WNDCLASSEXA {
                    cbSize: size_of::<WNDCLASSEXA>() as u32,
                    lpfnWndProc: Some(window_proc),
                    hInstance: instance,
                    hIcon: large_icon,
                    lpszClassName: class_name_c.as_ptr(),
                    hIconSm: small_icon,
                    ..Default::default()
                };
                RegisterClassExA(&wndclass);
            }
        }

        // Create window
        let hwnd = unsafe {
            let style = if builder.should_fullscreen {
                WS_POPUP
            } else if builder.resizable {
                WS_OVERLAPPEDWINDOW
            } else {
                WS_OVERLAPPEDWINDOW & !WS_THICKFRAME & !WS_MAXIMIZEBOX
            };

            // Calculate window rect based on size and position
            let monitor_rect = if let Some(monitor) = builder.monitor {
                monitor.rect()
            } else {
                RECT {
                    left: 0,
                    top: 0,
                    right: GetSystemMetrics(SM_CXSCREEN),
                    bottom: GetSystemMetrics(SM_CYSCREEN),
                }
            };

            let mut position_set = false;
            let mut x = 0;
            let mut y = 0;
            let mut width =
                (builder.size.width as i32 * dpi as i32) / USER_DEFAULT_SCREEN_DPI as i32;
            let mut height =
                (builder.size.height as i32 * dpi as i32) / USER_DEFAULT_SCREEN_DPI as i32;
            if let Some(position) = builder.position {
                position_set = true;
                x = monitor_rect.left
                    + (position.x as i32 * dpi as i32) / USER_DEFAULT_SCREEN_DPI as i32;
                y = monitor_rect.top
                    + (position.y as i32 * dpi as i32) / USER_DEFAULT_SCREEN_DPI as i32;
            }
            if builder.should_fullscreen {
                position_set = true;
                x = monitor_rect.left;
                y = monitor_rect.top;
                width = monitor_rect.right - monitor_rect.left;
                height = monitor_rect.bottom - monitor_rect.top;
            } else if builder.should_center {
                position_set = true;
                x = monitor_rect.left + ((monitor_rect.right - monitor_rect.left) - width) / 2;
                y = monitor_rect.top + ((monitor_rect.bottom - monitor_rect.top) - height) / 2;
            } else if !position_set && builder.monitor.is_some() {
                position_set = true;
                x = monitor_rect.left + (64 * dpi as i32) / USER_DEFAULT_SCREEN_DPI as i32;
                y = monitor_rect.top + (64 * dpi as i32) / USER_DEFAULT_SCREEN_DPI as i32;
            }
            let mut rect = RECT {
                left: x,
                top: y,
                right: x + width,
                bottom: y + height,
            };
            AdjustWindowRectExForDpi(&mut rect, style, FALSE, 0, dpi);

            let title = CString::new(builder.title.clone()).expect("Can't convert to CString");
            let hwnd = CreateWindowExA(
                0,
                class_name_c.as_ptr(),
                title.as_ptr(),
                style,
                if position_set {
                    rect.left
                } else {
                    CW_USEDEFAULT
                },
                if position_set {
                    rect.top
                } else {
                    CW_USEDEFAULT
                },
                rect.right - rect.left,
                rect.bottom - rect.top,
                null_mut(),
                null_mut(),
                instance,
                0,
            );
            if let Some(theme) = builder.theme {
                let enabled: BOOL = (theme == Theme::Dark).into();
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_USE_IMMERSIVE_DARK_MODE,
                    &enabled as *const _ as *const _,
                    size_of::<BOOL>() as u32,
                );
            }

            let should_show_window = if builder.remember_window_state {
                if let Ok(mut file) = File::open(config_dir().join("window.bin")) {
                    let size = size_of::<WINDOWPLACEMENT>();
                    let mut buffer = vec![0u8; size];
                    if file.read_exact(&mut buffer).is_ok() {
                        let window_placement = std::ptr::read(buffer.as_ptr() as *const _);
                        SetWindowPlacement(hwnd, &window_placement);
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            } else {
                true
            };
            if should_show_window {
                ShowWindow(hwnd, SW_SHOWDEFAULT);
            }
            UpdateWindow(hwnd);
            hwnd
        };

        // Alloc Webview data
        unsafe {
            #[allow(static_mut_refs)]
            if FIRST_HWND.is_none() {
                FIRST_HWND = Some(hwnd);
            }
        }

        let window_data = Box::new(WindowData {
            window_id,
            hwnd,
            dpi,
            min_size: builder.min_size,
            background_color: builder.background_color,
            remember_window_state: builder.remember_window_state,
            #[cfg(feature = "webview")]
            controller: None,
            window_handler: builder.window_handler,
            #[cfg(feature = "webview")]
            webview_handler: None,
        });
        unsafe {
            SetWindowLong(
                hwnd,
                GWL_USERDATA,
                window_data.as_ref() as *const _ as isize,
            )
        };

        PlatformWindow(window_data)
    }
}

impl crate::WindowInterface for PlatformWindow {
    fn set_title(&mut self, title: impl AsRef<str>) {
        let title = CString::new(title.as_ref()).expect("Can't convert to CString");
        unsafe { SetWindowTextA(self.0.hwnd, title.as_ptr()) };
    }

    fn position(&self) -> LogicalPoint {
        let mut rect = RECT::default();
        unsafe { GetWindowRect(self.0.hwnd, &mut rect) };
        LogicalPoint::new(
            (rect.left * USER_DEFAULT_SCREEN_DPI as i32 / self.0.dpi as i32) as f32,
            (rect.top * USER_DEFAULT_SCREEN_DPI as i32 / self.0.dpi as i32) as f32,
        )
    }

    fn size(&self) -> LogicalSize {
        let mut rect = RECT::default();
        unsafe { GetWindowRect(self.0.hwnd, &mut rect) };
        LogicalSize::new(
            ((rect.right - rect.left) * USER_DEFAULT_SCREEN_DPI as i32 / self.0.dpi as i32) as f32,
            ((rect.bottom - rect.top) * USER_DEFAULT_SCREEN_DPI as i32 / self.0.dpi as i32) as f32,
        )
    }

    fn set_position(&mut self, point: LogicalPoint) {
        unsafe {
            SetWindowPos(
                self.0.hwnd,
                null_mut(),
                point.x as i32 * self.0.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32,
                point.y as i32 * self.0.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }

    fn set_size(&mut self, size: LogicalSize) {
        unsafe {
            SetWindowPos(
                self.0.hwnd,
                null_mut(),
                0,
                0,
                size.width as i32 * self.0.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32,
                size.height as i32 * self.0.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32,
                SWP_NOREPOSITION | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }

    fn set_min_size(&mut self, min_size: LogicalSize) {
        self.0.min_size = Some(min_size);
    }

    fn set_resizable(&mut self, resizable: bool) {
        unsafe {
            let style = GetWindowLong(self.0.hwnd, GWL_STYLE) as u32;
            SetWindowLong(
                self.0.hwnd,
                GWL_STYLE,
                if resizable {
                    style & !WS_THICKFRAME
                } else {
                    style | WS_THICKFRAME
                } as isize,
            );
        }
    }

    fn set_theme(&mut self, theme: Theme) {
        unsafe {
            let enabled: BOOL = (theme == Theme::Dark).into();
            DwmSetWindowAttribute(
                self.0.hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &enabled as *const _ as *const _,
                size_of::<BOOL>() as u32,
            );
        }
    }

    fn set_background_color(&mut self, color: u32) {
        self.0.background_color = Some(color);
        unsafe { InvalidateRect(self.0.hwnd, null_mut(), TRUE) };
        #[cfg(feature = "webview")]
        if let Some(controller) = self.0.controller {
            unsafe {
                let mut controller2: *mut ICoreWebView2Controller2 = null_mut();
                (*controller).QueryInterface(
                    &IID_ICoreWebView2Controller2,
                    &mut controller2 as *mut _ as *mut *mut c_void,
                );
                if !controller2.is_null() {
                    (*controller2).put_DefaultBackgroundColor(COREWEBVIEW2_COLOR {
                        A: 0,
                        R: ((color >> 16) & 0xFF) as u8,
                        G: ((color >> 8) & 0xFF) as u8,
                        B: (color & 0xFF) as u8,
                    });
                }
            }
        }
    }
}


// --- Win32 VK -> Key ---
fn vk_to_key(vk: u32) -> Key {
    match vk {
        0x41 => Key::A, 0x42 => Key::B, 0x43 => Key::C, 0x44 => Key::D,
        0x45 => Key::E, 0x46 => Key::F, 0x47 => Key::G, 0x48 => Key::H,
        0x49 => Key::I, 0x4A => Key::J, 0x4B => Key::K, 0x4C => Key::L,
        0x4D => Key::M, 0x4E => Key::N, 0x4F => Key::O, 0x50 => Key::P,
        0x51 => Key::Q, 0x52 => Key::R, 0x53 => Key::S, 0x54 => Key::T,
        0x55 => Key::U, 0x56 => Key::V, 0x57 => Key::W, 0x58 => Key::X,
        0x59 => Key::Y, 0x5A => Key::Z,
        0x30 => Key::Digit0, 0x31 => Key::Digit1, 0x32 => Key::Digit2,
        0x33 => Key::Digit3, 0x34 => Key::Digit4, 0x35 => Key::Digit5,
        0x36 => Key::Digit6, 0x37 => Key::Digit7, 0x38 => Key::Digit8, 0x39 => Key::Digit9,
        0x70 => Key::F1, 0x71 => Key::F2, 0x72 => Key::F3, 0x73 => Key::F4,
        0x74 => Key::F5, 0x75 => Key::F6, 0x76 => Key::F7, 0x77 => Key::F8,
        0x78 => Key::F9, 0x79 => Key::F10, 0x7A => Key::F11, 0x7B => Key::F12,
        0x7C => Key::F13, 0x7D => Key::F14,
        0x1B => Key::Escape, 0x0D => Key::Enter, 0x08 => Key::Backspace,
        0x09 => Key::Tab, 0x20 => Key::Space, 0x2E => Key::Delete, 0x2D => Key::Insert,
        0x26 => Key::ArrowUp, 0x28 => Key::ArrowDown,
        0x25 => Key::ArrowLeft, 0x27 => Key::ArrowRight,
        0x24 => Key::Home, 0x23 => Key::End,
        0x21 => Key::PageUp, 0x22 => Key::PageDown,
        0x10 => Key::Shift, 0x11 => Key::Control, 0x12 => Key::Alt,
        0x5B | 0x5C => Key::Meta, 0x14 => Key::CapsLock,
        0x60 => Key::Numpad0, 0x61 => Key::Numpad1, 0x62 => Key::Numpad2,
        0x63 => Key::Numpad3, 0x64 => Key::Numpad4, 0x65 => Key::Numpad5,
        0x66 => Key::Numpad6, 0x67 => Key::Numpad7, 0x68 => Key::Numpad8, 0x69 => Key::Numpad9,
        0x6B => Key::NumpadAdd, 0x6D => Key::NumpadSubtract,
        0x6A => Key::NumpadMultiply, 0x6F => Key::NumpadDivide,
        0x6E => Key::NumpadDecimal,
        0xBD => Key::Minus, 0xBB => Key::Equal, 0xDB => Key::BracketLeft,
        0xDD => Key::BracketRight, 0xDC => Key::Backslash, 0xBA => Key::Semicolon,
        0xDE => Key::Quote, 0xBC => Key::Comma, 0xBE => Key::Period, 0xBF => Key::Slash,
        0xC0 => Key::Backtick,
        _ => Key::Unknown,
    }
}

fn get_key_modifiers() -> Modifiers {
    let mut mods = Modifiers::empty();
    unsafe {
        if GetKeyState(VK_SHIFT as i32) as u16 & 0x8000 != 0 { mods = mods | Modifiers::SHIFT; }
        if GetKeyState(VK_CONTROL as i32) as u16 & 0x8000 != 0 { mods = mods | Modifiers::CTRL; }
        if GetKeyState(VK_MENU as i32) as u16 & 0x8000 != 0 { mods = mods | Modifiers::ALT; }
        if GetKeyState(VK_LWIN as i32) as u16 & 0x8000 != 0
            || GetKeyState(VK_RWIN as i32) as u16 & 0x8000 != 0 {
            mods = mods | Modifiers::META;
        }
    }
    mods
}

unsafe fn make_temp_window(data: &mut WindowData) -> std::mem::ManuallyDrop<crate::Window> {
    std::mem::ManuallyDrop::new(crate::Window {
        id: data.window_id,
        platform: PlatformWindow(Box::from_raw(data as *mut WindowData)),
        window_handler: data.window_handler,
    })
}

fn dispatch_mouse_down(data: &mut WindowData, l_param: LPARAM, button: MouseButton) {
    if let Some(h_ptr) = data.window_handler {
        unsafe {
            let handler = &mut *h_ptr;
            let x = (l_param as u16) as f64;
            let y = ((l_param >> 16) as u16) as f64;
            let lx = x * USER_DEFAULT_SCREEN_DPI as f64 / data.dpi as f64;
            let ly = y * USER_DEFAULT_SCREEN_DPI as f64 / data.dpi as f64;
            let mut window = make_temp_window(data);
            handler.on_mouse_down(&mut window, button, lx, ly);
            std::mem::forget(window.platform.0);
        }
    }
}

fn dispatch_mouse_up(data: &mut WindowData, l_param: LPARAM, button: MouseButton) {
    if let Some(h_ptr) = data.window_handler {
        unsafe {
            let handler = &mut *h_ptr;
            let x = (l_param as u16) as f64;
            let y = ((l_param >> 16) as u16) as f64;
            let lx = x * USER_DEFAULT_SCREEN_DPI as f64 / data.dpi as f64;
            let ly = y * USER_DEFAULT_SCREEN_DPI as f64 / data.dpi as f64;
            let mut window = make_temp_window(data);
            handler.on_mouse_up(&mut window, button, lx, ly);
            std::mem::forget(window.platform.0);
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let _self = unsafe {
        let ptr = GetWindowLong(hwnd, GWL_USERDATA) as *mut WindowData;
        if ptr.is_null() {
            return DefWindowProcA(hwnd, msg, w_param, l_param);
        }
        &mut *ptr
    };
    match msg {
        WM_CREATE => {
            0
        }
        WM_ERASEBKGND => {
            if let Some(color) = _self.background_color {
                let hdc = w_param as HDC;
                let mut client_rect = RECT::default();
                unsafe { GetClientRect(hwnd, &mut client_rect) };
                let brush = unsafe {
                    CreateSolidBrush(
                        ((color & 0xFF) << 16) | (color & 0xFF00) | ((color >> 16) & 0xFF),
                    )
                };
                unsafe { FillRect(hdc, &client_rect, brush) };
                unsafe { DeleteObject(brush) };
                1
            } else {
                0
            }
        }
        WM_MOVE => {
            if let Some(h_ptr) = _self.window_handler {
                let handler = &mut *h_ptr;
                let x = l_param as u16 as i32;
                let y = (l_param >> 16) as u16 as i32;
                let lx = x * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32;
                let ly = y * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32;
                let mut window = make_temp_window(_self);
                handler.on_move(&mut window, lx, ly);
                std::mem::forget(window.platform.0);
            }
            0
        }
        WM_SIZE => {
            let width = (l_param as u16) as i32;
            let height = ((l_param >> 16) as u16) as i32;
            if let Some(h_ptr) = _self.window_handler {
                let handler = &mut *h_ptr;
                let lw = (width * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as u32;
                let lh = (height * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as u32;
                let mut window = make_temp_window(_self);
                handler.on_resize(&mut window, lw, lh);
                std::mem::forget(window.platform.0);
            }
            #[cfg(feature = "webview")]
            if let Some(controller) = _self.controller {
                unsafe {
                    (*controller).put_Bounds(RECT {
                        left: 0,
                        top: 0,
                        right: width,
                        bottom: height,
                    })
                };
            }
            0
        }
        WM_DPICHANGED => {
            _self.dpi = (w_param >> 16) as u32;
            let window_rect = unsafe { &*(l_param as *const RECT) };
            unsafe {
                SetWindowPos(
                    hwnd,
                    null_mut(),
                    window_rect.left,
                    window_rect.top,
                    window_rect.right - window_rect.left,
                    window_rect.bottom - window_rect.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                )
            };
            0
        }
        WM_GETMINMAXINFO => {
            if let Some(min_size) = _self.min_size {
                let min_width =
                    min_size.width as i32 * _self.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32;
                let min_height =
                    min_size.height as i32 * _self.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32;
                unsafe {
                    let minmax_info: *mut MINMAXINFO = l_param as *mut MINMAXINFO;
                    (*minmax_info).ptMinTrackSize.x = min_width;
                    (*minmax_info).ptMinTrackSize.y = min_height;
                }
            }
            0
        }
        WM_SETFOCUS => {
            if let Some(h_ptr) = _self.window_handler {
                let handler = &mut *h_ptr;
                let mut window = make_temp_window(_self);
                handler.on_focus(&mut window);
                std::mem::forget(window.platform.0);
            }
            0
        }
        WM_KILLFOCUS => {
            if let Some(h_ptr) = _self.window_handler {
                let handler = &mut *h_ptr;
                let mut window = make_temp_window(_self);
                handler.on_blur(&mut window);
                std::mem::forget(window.platform.0);
            }
            0
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            if let Some(h_ptr) = _self.window_handler {
                let handler = &mut *h_ptr;
                let key = vk_to_key(w_param as u32);
                let mods = get_key_modifiers();
                let mut window = make_temp_window(_self);
                handler.on_key_down(&mut window, key, mods);
                std::mem::forget(window.platform.0);
            }
            DefWindowProcA(hwnd, msg, w_param, l_param)
        }
        WM_KEYUP | WM_SYSKEYUP => {
            if let Some(h_ptr) = _self.window_handler {
                let handler = &mut *h_ptr;
                let key = vk_to_key(w_param as u32);
                let mods = get_key_modifiers();
                let mut window = make_temp_window(_self);
                handler.on_key_up(&mut window, key, mods);
                std::mem::forget(window.platform.0);
            }
            DefWindowProcA(hwnd, msg, w_param, l_param)
        }
        WM_MOUSEMOVE => {
            if let Some(h_ptr) = _self.window_handler {
                let handler = &mut *h_ptr;
                let x = (l_param as u16) as f64;
                let y = ((l_param >> 16) as u16) as f64;
                let lx = x * USER_DEFAULT_SCREEN_DPI as f64 / _self.dpi as f64;
                let ly = y * USER_DEFAULT_SCREEN_DPI as f64 / _self.dpi as f64;
                let mut window = make_temp_window(_self);
                handler.on_mouse_move(&mut window, lx, ly);
                std::mem::forget(window.platform.0);
            }
            0
        }
        WM_LBUTTONDOWN => {
            dispatch_mouse_down(_self, l_param, MouseButton::Left);
            0
        }
        WM_RBUTTONDOWN => {
            dispatch_mouse_down(_self, l_param, MouseButton::Right);
            0
        }
        WM_MBUTTONDOWN => {
            dispatch_mouse_down(_self, l_param, MouseButton::Middle);
            0
        }
        WM_LBUTTONUP => {
            dispatch_mouse_up(_self, l_param, MouseButton::Left);
            0
        }
        WM_RBUTTONUP => {
            dispatch_mouse_up(_self, l_param, MouseButton::Right);
            0
        }
        WM_MBUTTONUP => {
            dispatch_mouse_up(_self, l_param, MouseButton::Middle);
            0
        }
        WM_MOUSEWHEEL => {
            if let Some(h_ptr) = _self.window_handler {
                let handler = &mut *h_ptr;
                let delta = (w_param >> 16) as i16 as f64 / 120.0 * 3.0;
                let mut window = make_temp_window(_self);
                handler.on_wheel(&mut window, 0.0, -delta);
                std::mem::forget(window.platform.0);
            }
            0
        }
        WM_MOUSEHWHEEL => {
            if let Some(h_ptr) = _self.window_handler {
                let handler = &mut *h_ptr;
                let delta = (w_param >> 16) as i16 as f64 / 120.0 * 3.0;
                let mut window = make_temp_window(_self);
                handler.on_wheel(&mut window, delta, 0.0);
                std::mem::forget(window.platform.0);
            }
            0
        }
        WM_SEND_MESSAGE => {
            let ptr = w_param as *mut c_void;
            let data = unsafe { Box::from_raw(ptr as *mut String) };
            unsafe {
                #[allow(static_mut_refs)]
                if let Some(h_ptr) = EVENT_LOOP_HANDLER {
                    (*h_ptr).on_user_event(*data);
                }
            }
            0
        }
        WM_CLOSE => {
            if _self.remember_window_state {
                unsafe {
                    use std::io::Write;
                    let mut window_placement = mem::zeroed();
                    GetWindowPlacement(hwnd, &mut window_placement);
                    if let Ok(mut file) = File::create(config_dir().join("window.bin")) {
                        _ = file.write_all(std::slice::from_raw_parts(
                            &window_placement as *const _ as *const u8,
                            size_of::<WINDOWPLACEMENT>(),
                        ));
                    }
                }
            }

            let allow = if let Some(h_ptr) = _self.window_handler {
                let handler = &mut *h_ptr;
                let mut window = make_temp_window(_self);
                let result = handler.on_close(&mut window);
                std::mem::forget(window.platform.0);
                result
            } else {
                true
            };
            if allow {
                unsafe { DestroyWindow(hwnd) };
            }
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcA(hwnd, msg, w_param, l_param) },
    }
}

pub(super) fn config_dir() -> PathBuf {
    let project_dirs = unsafe {
        if let Some(ref app_id) = APP_ID {
            directories::ProjectDirs::from(
                &app_id.qualifier,
                &app_id.organization,
                &app_id.application,
            )
        } else {
            directories::ProjectDirs::from_path(PathBuf::from(
                env::current_exe()
                    .expect("Can't get current process name")
                    .file_name()
                    .expect("Can't get current process name")
                    .to_string_lossy()
                    .strip_suffix(".exe")
                    .expect("Should strip .exe"),
            ))
        }
    }
    .expect("Can't get dirs");
    project_dirs.config_dir()
}
