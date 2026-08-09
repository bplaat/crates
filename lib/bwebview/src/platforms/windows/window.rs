/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::env;
use std::ffi::c_void;
use std::path::PathBuf;
use std::ptr::{null, null_mut};

#[cfg(feature = "progress_bar")]
use super::event_loop::TASKBAR_BUTTON_CREATED;
use super::event_loop::{APP_ID, FIRST_HWND, WM_SEND_MESSAGE, send_event, system_theme};
#[cfg(feature = "file_drop")]
use super::file_drop::handle_file_drop;
#[cfg(feature = "progress_bar")]
use super::progress_bar::ProgressBar;
#[cfg(feature = "webview")]
use super::webview2::*;
use super::win32::*;
#[cfg(feature = "remember_window_state")]
use super::window_state::{restore_window_state, save_window_state};
#[cfg(feature = "progress_bar")]
use crate::WindowsProgressBarState;
use crate::{
    CloseRequest, KeyboardEvent, LogicalPoint, LogicalSize, MouseEvent, Theme, WheelEvent,
    WindowBuilder, WindowEvent,
};

pub(super) struct WindowData {
    pub(super) hwnd: HWND,
    pub(super) dpi: u32,
    pub(super) min_size: Option<LogicalSize>,
    pub(super) background_color: Option<u32>,
    pub(super) theme: Theme,
    pub(super) follows_system_theme: bool,
    pub(super) cursor: HCURSOR,
    #[cfg(feature = "remember_window_state")]
    pub(super) remember_window_state: bool,
    #[cfg(feature = "file_drop")]
    pub(super) allow_file_drop: bool,
    pub(super) resize_callback: Option<Box<dyn Fn(i32, i32)>>,
    pub(super) last_mouse: Option<LogicalPoint>,
    #[cfg(feature = "canvas")]
    pub(super) canvas_data: Option<*mut super::canvas::CanvasData>,
    #[cfg(feature = "progress_bar")]
    progress_bar: ProgressBar,
}

pub(crate) struct PlatformWindow(pub(super) Box<WindowData>);

fn calculate_window_rect(
    builder: &WindowBuilder,
    monitor_rect: &RECT,
    style: u32,
    dpi: u32,
) -> (RECT, bool) {
    let mut position_set = false;
    let mut x = 0;
    let mut y = 0;
    let mut width = (builder.size.width as i32 * dpi as i32) / USER_DEFAULT_SCREEN_DPI as i32;
    let mut height = (builder.size.height as i32 * dpi as i32) / USER_DEFAULT_SCREEN_DPI as i32;
    if let Some(position) = builder.position {
        position_set = true;
        x = monitor_rect.left + (position.x as i32 * dpi as i32) / USER_DEFAULT_SCREEN_DPI as i32;
        y = monitor_rect.top + (position.y as i32 * dpi as i32) / USER_DEFAULT_SCREEN_DPI as i32;
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
    // SAFETY: rect is valid and style is a supported window style.
    unsafe { AdjustWindowRectExForDpi(&mut rect, style, FALSE, 0, dpi) };
    (rect, position_set)
}

impl PlatformWindow {
    pub(crate) fn new(builder: &WindowBuilder) -> Self {
        let initial_dpi = unsafe { GetDpiForSystem() };
        let mut window_data = Box::new(WindowData {
            hwnd: null_mut(),
            dpi: initial_dpi,
            min_size: builder.min_size,
            background_color: builder.background_color,
            theme: builder.theme.unwrap_or_else(system_theme),
            follows_system_theme: builder.theme.is_none(),
            cursor: unsafe { LoadCursorW(null_mut(), 32512 as *const u16) },
            #[cfg(feature = "remember_window_state")]
            remember_window_state: builder.remember_window_state,
            #[cfg(feature = "file_drop")]
            allow_file_drop: builder.allow_file_drop,
            resize_callback: None,
            last_mouse: None,
            #[cfg(feature = "canvas")]
            canvas_data: None,
            #[cfg(feature = "progress_bar")]
            progress_bar: ProgressBar::default(),
        });

        // Check if window class is already registered
        let instance = unsafe { GetModuleHandleW(null()) };
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
        let class_name = class_name.to_wide_string();
        unsafe {
            let mut wndclass = WNDCLASSEXW {
                cbSize: size_of::<WNDCLASSEXW>() as u32,
                ..Default::default()
            };
            if GetClassInfoExW(instance, class_name.as_ptr(), &mut wndclass as *mut _) != TRUE {
                // Get executable icons
                let executable_path = env::current_exe()
                    .expect("Can't get current exe path")
                    .display()
                    .to_string()
                    .to_wide_string();
                let mut large_icon = HICON::default();
                let mut small_icon = HICON::default();
                ExtractIconExW(
                    executable_path.as_ptr(),
                    0,
                    &mut large_icon,
                    &mut small_icon,
                    1,
                );

                // Register window class
                let wndclass = WNDCLASSEXW {
                    cbSize: size_of::<WNDCLASSEXW>() as u32,
                    lpfnWndProc: Some(window_proc),
                    hInstance: instance,
                    hIcon: large_icon,
                    hbrBackground: GetSysColorBrush(COLOR_WINDOW) as usize,
                    lpszClassName: class_name.as_ptr(),
                    hIconSm: small_icon,
                    ..Default::default()
                };
                RegisterClassExW(&wndclass);
            }
        }

        // Create window
        let (hwnd, restored_window_state) = unsafe {
            let style = if builder.should_fullscreen {
                WS_POPUP
            } else if builder.resizable {
                WS_OVERLAPPEDWINDOW
            } else {
                WS_OVERLAPPEDWINDOW & !WS_THICKFRAME & !WS_MAXIMIZEBOX
            };

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
            let (rect, position_set) =
                calculate_window_rect(builder, &monitor_rect, style, initial_dpi);

            let title = builder.title.to_wide_string();
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
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
                window_data.as_mut() as *mut WindowData as LPARAM,
            );
            window_data.hwnd = hwnd;
            #[cfg(feature = "file_drop")]
            if builder.allow_file_drop {
                DragAcceptFiles(hwnd, TRUE);
            }
            set_titlebar_theme(hwnd, builder.theme.unwrap_or_else(system_theme));

            #[cfg(feature = "remember_window_state")]
            let restored_window_state = builder.remember_window_state && restore_window_state(hwnd);
            #[cfg(not(feature = "remember_window_state"))]
            let restored_window_state = false;
            let window_dpi = GetDpiForWindow(hwnd);
            let dpi = if window_dpi == 0 {
                initial_dpi
            } else {
                window_dpi
            };
            window_data.dpi = dpi;
            if !restored_window_state && dpi != initial_dpi {
                let (rect, position_set) =
                    calculate_window_rect(builder, &monitor_rect, style, dpi);
                SetWindowPos(
                    hwnd,
                    null_mut(),
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    SWP_NOZORDER | SWP_NOACTIVATE | if position_set { 0 } else { SWP_NOMOVE },
                );
            }
            (hwnd, restored_window_state)
        };

        unsafe {
            #[allow(static_mut_refs)]
            if FIRST_HWND.is_none() {
                FIRST_HWND = Some(hwnd);
            }
        }

        unsafe {
            if !restored_window_state {
                ShowWindow(hwnd, SW_SHOWDEFAULT);
            }
            UpdateWindow(hwnd);
        }

        PlatformWindow(window_data)
    }
}

impl crate::WindowInterface for PlatformWindow {
    fn close(&mut self) {
        #[cfg(feature = "remember_window_state")]
        if self.0.remember_window_state {
            save_window_state(self.0.hwnd);
        }
        unsafe { DestroyWindow(self.0.hwnd) };
    }

    fn set_title(&mut self, title: impl AsRef<str>) {
        let title = title.as_ref().to_wide_string();
        unsafe { SetWindowTextW(self.0.hwnd, title.as_ptr()) };
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
        self.0.theme = theme;
        self.0.follows_system_theme = false;
        unsafe { set_titlebar_theme(self.0.hwnd, theme) }
    }

    fn set_background_color(&mut self, color: u32) {
        self.0.background_color = Some(color);
        unsafe { InvalidateRect(self.0.hwnd, null_mut(), TRUE) };
    }

    fn set_cursor(&mut self, cursor: crate::CursorIcon) {
        use crate::CursorIcon;
        let resource = match cursor {
            CursorIcon::Default => 32512,
            CursorIcon::Pointer => 32649,
            CursorIcon::Crosshair => 32515,
            CursorIcon::Text => 32513,
            CursorIcon::Grab | CursorIcon::Grabbing => 32646,
        };
        self.0.cursor = unsafe { LoadCursorW(null_mut(), resource as *const u16) };
        unsafe { SetCursor(self.0.cursor) };
    }

    #[cfg(feature = "progress_bar")]
    fn windows_set_progress_bar(&mut self, progress: Option<f32>, state: WindowsProgressBarState) {
        self.0.progress_bar.set(self.0.hwnd, progress, state);
    }
}

unsafe fn set_titlebar_theme(hwnd: HWND, theme: Theme) {
    let enabled: BOOL = (theme == Theme::Dark).into();
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &enabled as *const _ as *const _,
            size_of::<BOOL>() as u32,
        );
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let _self = unsafe {
        let ptr = if msg == WM_NCCREATE {
            let create = &*(l_param as *const CREATESTRUCTW);
            let ptr = create.lpCreateParams.cast::<WindowData>();
            SetWindowLong(hwnd, GWL_USERDATA, ptr as isize);
            ptr
        } else {
            GetWindowLong(hwnd, GWL_USERDATA) as *mut WindowData
        };
        let Some(window_data) = ptr.as_mut() else {
            return DefWindowProcW(hwnd, msg, w_param, l_param);
        };
        window_data
    };
    match msg {
        #[cfg(feature = "progress_bar")]
        message if message == unsafe { TASKBAR_BUTTON_CREATED } => {
            _self.progress_bar.button_created(hwnd);
            0
        }
        WM_CREATE => {
            send_event(crate::Event::Window(WindowEvent::Create));
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
        WM_SETCURSOR => {
            unsafe { SetCursor(_self.cursor) };
            1
        }
        WM_SETTINGCHANGE | WM_THEMECHANGED if _self.follows_system_theme => {
            let theme = system_theme();
            if theme != _self.theme {
                _self.theme = theme;
                unsafe { set_titlebar_theme(hwnd, theme) };
                send_event(crate::Event::Window(WindowEvent::ThemeChange(theme)));
                unsafe { InvalidateRect(hwnd, null_mut(), TRUE) };
            }
            0
        }
        #[cfg(feature = "canvas")]
        WM_TIMER if w_param == super::canvas::CANVAS_ANIMATION_TIMER => {
            unsafe {
                KillTimer(hwnd, super::canvas::CANVAS_ANIMATION_TIMER);
                InvalidateRect(hwnd, null_mut(), FALSE);
            }
            0
        }
        #[cfg(feature = "canvas")]
        WM_PAINT if _self.canvas_data.is_some() => {
            unsafe { ValidateRect(hwnd, null()) };
            super::canvas::draw(unsafe { &mut *_self.canvas_data.expect("canvas data") });
            0
        }
        WM_MOVE => {
            let x = l_param as u16 as i32;
            let y = (l_param >> 16) as u16 as i32;
            send_event(crate::Event::Window(WindowEvent::Move(LogicalPoint::new(
                (x * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                (y * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
            ))));
            0
        }
        WM_SETFOCUS => {
            send_event(crate::Event::Window(WindowEvent::Focus));
            0
        }
        WM_KILLFOCUS => {
            send_event(crate::Event::Window(WindowEvent::Blur));
            0
        }
        WM_MOUSEMOVE => {
            let event = win_mouse_event(hwnd, _self, w_param, l_param, -1, 0);
            send_event(crate::Event::Window(WindowEvent::MouseMove(event)));
            0
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
            let button = if msg == WM_LBUTTONDOWN {
                0
            } else if msg == WM_MBUTTONDOWN {
                1
            } else {
                2
            };
            send_event(crate::Event::Window(WindowEvent::MouseDown(
                win_mouse_event(hwnd, _self, w_param, l_param, button, 1),
            )));
            0
        }
        WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP => {
            let button = if msg == WM_LBUTTONUP {
                0
            } else if msg == WM_MBUTTONUP {
                1
            } else {
                2
            };
            let event = win_mouse_event(hwnd, _self, w_param, l_param, button, 1);
            send_event(crate::Event::Window(WindowEvent::MouseUp(event.clone())));
            send_event(crate::Event::Window(WindowEvent::Click(event)));
            0
        }
        WM_MOUSEWHEEL => {
            let mut event = win_mouse_event(hwnd, _self, 0, l_param, -1, 0);
            event.screen_x = l_param as i16 as f32;
            event.screen_y = (l_param >> 16) as i16 as f32;
            send_event(crate::Event::Window(WindowEvent::Wheel(WheelEvent {
                mouse: event,
                delta_x: 0.0,
                delta_y: -((w_param >> 16) as i16 as f32),
                delta_z: 0.0,
                delta_mode: 0,
            })));
            0
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            send_event(crate::Event::Window(WindowEvent::KeyDown(
                win_keyboard_event(w_param, l_param),
            )));
            0
        }
        WM_KEYUP | WM_SYSKEYUP => {
            send_event(crate::Event::Window(WindowEvent::KeyUp(
                win_keyboard_event(w_param, l_param),
            )));
            0
        }
        WM_SIZE => {
            let width = (l_param as u16) as i32;
            let height = ((l_param >> 16) as u16) as i32;
            send_event(crate::Event::Window(WindowEvent::Resize(LogicalSize::new(
                (width * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                (height * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
            ))));
            if let Some(cb) = &_self.resize_callback {
                cb(width, height);
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
        WM_SEND_MESSAGE => {
            let ptr = w_param as *mut c_void;
            let event = unsafe { Box::from_raw(ptr as *mut crate::Event<'static>) };
            send_event(*event);
            0
        }
        #[cfg(feature = "file_drop")]
        WM_DROPFILES => {
            unsafe { handle_file_drop(w_param as HDROP) };
            0
        }
        WM_CLOSE => {
            let request = CloseRequest::new();
            send_event(crate::Event::Window(WindowEvent::CloseRequested(
                request.clone(),
            )));
            if !request.is_prevented() {
                #[cfg(feature = "remember_window_state")]
                if _self.remember_window_state {
                    save_window_state(hwnd);
                }
                unsafe { DestroyWindow(hwnd) };
            }
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) },
    }
}

fn modifier(key: i32) -> bool {
    unsafe { GetKeyState(key) < 0 }
}
fn win_mouse_event(
    hwnd: HWND,
    data: &mut WindowData,
    w_param: WPARAM,
    l_param: LPARAM,
    button: i16,
    detail: u16,
) -> MouseEvent {
    let x = l_param as i16 as f32 * USER_DEFAULT_SCREEN_DPI as f32 / data.dpi as f32;
    let y = (l_param >> 16) as i16 as f32 * USER_DEFAULT_SCREEN_DPI as f32 / data.dpi as f32;
    let (movement_x, movement_y) = data.last_mouse.map_or((0.0, 0.0), |p| (x - p.x, y - p.y));
    data.last_mouse = Some(LogicalPoint::new(x, y));
    let mut screen = POINT {
        x: (x * data.dpi as f32 / USER_DEFAULT_SCREEN_DPI as f32) as i32,
        y: (y * data.dpi as f32 / USER_DEFAULT_SCREEN_DPI as f32) as i32,
    };
    unsafe { ClientToScreen(hwnd, &mut screen) };
    MouseEvent {
        client_x: x,
        client_y: y,
        screen_x: screen.x as f32,
        screen_y: screen.y as f32,
        movement_x,
        movement_y,
        button,
        buttons: ((w_param & 1) | ((w_param & 2) << 1) | ((w_param & 16) >> 3)) as u16,
        detail,
        alt_key: modifier(0x12),
        ctrl_key: modifier(0x11),
        meta_key: modifier(0x5B) || modifier(0x5C),
        shift_key: modifier(0x10),
    }
}
fn win_keyboard_event(w_param: WPARAM, l_param: LPARAM) -> KeyboardEvent {
    let vk = w_param as u32;
    let shift = modifier(0x10);
    let key = match vk {
        0x08 => "Backspace".into(),
        0x09 => "Tab".into(),
        0x0D => "Enter".into(),
        0x1B => "Escape".into(),
        0x20 => " ".into(),
        0x25 => "ArrowLeft".into(),
        0x26 => "ArrowUp".into(),
        0x27 => "ArrowRight".into(),
        0x28 => "ArrowDown".into(),
        v @ 0x41..=0x5A => char::from_u32(if shift { v } else { v + 32 })
            .unwrap_or_default()
            .to_string(),
        v @ 0x30..=0x39 => char::from_u32(v).unwrap_or_default().to_string(),
        _ => "Unidentified".into(),
    };
    let code = match vk {
        v @ 0x41..=0x5A => format!("Key{}", char::from_u32(v).unwrap_or_default()),
        v @ 0x30..=0x39 => format!("Digit{}", char::from_u32(v).unwrap_or_default()),
        0x0D => "Enter".into(),
        0x1B => "Escape".into(),
        0x20 => "Space".into(),
        0x25 => "ArrowLeft".into(),
        0x26 => "ArrowUp".into(),
        0x27 => "ArrowRight".into(),
        0x28 => "ArrowDown".into(),
        _ => "Unidentified".into(),
    };
    KeyboardEvent {
        key,
        code,
        location: 0,
        repeat: l_param & (1 << 30) != 0,
        is_composing: false,
        alt_key: modifier(0x12),
        ctrl_key: modifier(0x11),
        meta_key: modifier(0x5B) || modifier(0x5C),
        shift_key: shift,
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
