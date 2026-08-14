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
use super::event_loop::{
    APP_ID, FIRST_HWND, WM_SEND_MESSAGE, send_event, send_theme_change, system_theme,
};
#[cfg(feature = "file_drop")]
use super::file_drop::handle_file_drop;
#[cfg(feature = "progress_bar")]
use super::progress_bar::ProgressBar;
use super::webview2::*;
use super::win32::*;
#[cfg(feature = "remember_window_state")]
use super::window_state::{restore_window_state, save_window_state};
#[cfg(feature = "progress_bar")]
use crate::WindowsProgressBarState;
use crate::{
    CloseRequest, LogicalPoint, LogicalSize, Theme, WindowBuilder, WindowEvent, WindowEvents,
};
use crate::{KeyboardEvent, MouseEvent, WheelEvent};

fn load_cursor(cursor: crate::Cursor) -> HCURSOR {
    use crate::Cursor;

    let resource = match cursor {
        Cursor::Default => IDC_ARROW,
        Cursor::Pointer => IDC_HAND,
        Cursor::Crosshair => IDC_CROSS,
        Cursor::Text => IDC_IBEAM,
        Cursor::Grab | Cursor::Grabbing => IDC_SIZEALL,
    };
    unsafe { LoadCursorW(null_mut(), resource) }
}

pub(super) struct WindowData {
    pub(super) hwnd: HWND,
    pub(super) dpi: u32,
    pub(super) min_size: Option<LogicalSize>,
    pub(super) background_color: Option<u32>,
    pub(super) theme: Theme,
    pub(super) follows_system_theme: bool,
    pub(super) cursor: HCURSOR,
    fullscreen: bool,
    windowed_placement: WINDOWPLACEMENT,
    windowed_style: u32,
    pub(super) events: WindowEvents,
    pub(super) last_mouse: Option<LogicalPoint>,
    pub(super) tracks_mouse_leave: bool,
    primary_button_down: bool,
    #[cfg(feature = "remember_window_state")]
    pub(super) remember_window_state: bool,
    #[cfg(feature = "file_drop")]
    pub(super) allow_file_drop: bool,
    pub(super) resize_callback: Option<Box<dyn Fn(i32, i32)>>,
    #[cfg(feature = "progress_bar")]
    progress_bar: ProgressBar,
}

pub(crate) struct PlatformWindow(pub(super) Box<WindowData>);

fn calculate_window_rect(
    builder: &WindowBuilder,
    monitor_rect: &RECT,
    style: u32,
    dpi: u32,
    fullscreen: bool,
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
    if fullscreen {
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
    if builder.should_fullscreen && !fullscreen && !position_set {
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        rect.left = monitor_rect.left + (monitor_rect.right - monitor_rect.left - width) / 2;
        rect.top = monitor_rect.top + (monitor_rect.bottom - monitor_rect.top - height) / 2;
        rect.right = rect.left + width;
        rect.bottom = rect.top + height;
    }
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
            cursor: load_cursor(builder.cursor),
            fullscreen: builder.should_fullscreen,
            windowed_placement: WINDOWPLACEMENT::default(),
            windowed_style: 0,
            events: WindowEvents::NONE,
            last_mouse: None,
            tracks_mouse_leave: false,
            primary_button_down: false,
            #[cfg(feature = "remember_window_state")]
            remember_window_state: builder.remember_window_state,
            #[cfg(feature = "file_drop")]
            allow_file_drop: builder.allow_file_drop,
            resize_callback: None,
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
            let windowed_style = if builder.resizable {
                WS_OVERLAPPEDWINDOW
            } else {
                WS_OVERLAPPEDWINDOW & !WS_THICKFRAME & !WS_MAXIMIZEBOX
            };
            let style = if builder.should_fullscreen {
                WS_POPUP
            } else {
                windowed_style
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
            let (rect, position_set) = calculate_window_rect(
                builder,
                &monitor_rect,
                style,
                initial_dpi,
                builder.should_fullscreen,
            );
            let (windowed_rect, _) =
                calculate_window_rect(builder, &monitor_rect, windowed_style, initial_dpi, false);
            window_data.windowed_placement = WINDOWPLACEMENT {
                length: size_of::<WINDOWPLACEMENT>() as u32,
                showCmd: SW_SHOWNORMAL as u32,
                rcNormalPosition: windowed_rect,
                ..Default::default()
            };
            window_data.windowed_style = windowed_style;

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
            let restored_window_state = builder.remember_window_state
                && !builder.should_fullscreen
                && restore_window_state(hwnd);
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
                let (rect, position_set) = calculate_window_rect(
                    builder,
                    &monitor_rect,
                    style,
                    dpi,
                    builder.should_fullscreen,
                );
                let (windowed_rect, _) =
                    calculate_window_rect(builder, &monitor_rect, windowed_style, dpi, false);
                window_data.windowed_placement.rcNormalPosition = windowed_rect;
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
        unsafe { GetClientRect(self.0.hwnd, &mut rect) };
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
        let style = if self.0.fullscreen {
            self.0.windowed_style
        } else {
            unsafe { GetWindowLong(self.0.hwnd, GWL_STYLE) as u32 }
        };
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: size.width as i32 * self.0.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32,
            bottom: size.height as i32 * self.0.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32,
        };
        unsafe { AdjustWindowRectExForDpi(&mut rect, style, FALSE, 0, self.0.dpi) };
        if self.0.fullscreen {
            let normal = &mut self.0.windowed_placement.rcNormalPosition;
            normal.right = normal.left + rect.right - rect.left;
            normal.bottom = normal.top + rect.bottom - rect.top;
            return;
        }
        unsafe {
            SetWindowPos(
                self.0.hwnd,
                null_mut(),
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOREPOSITION | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }

    fn set_min_size(&mut self, min_size: Option<LogicalSize>) {
        self.0.min_size = min_size;
    }

    fn set_resizable(&mut self, resizable: bool) {
        if self.0.fullscreen {
            if resizable {
                self.0.windowed_style |= WS_THICKFRAME | WS_MAXIMIZEBOX;
            } else {
                self.0.windowed_style &= !WS_THICKFRAME & !WS_MAXIMIZEBOX;
            }
            return;
        }
        unsafe {
            let style = GetWindowLong(self.0.hwnd, GWL_STYLE) as u32;
            SetWindowLong(
                self.0.hwnd,
                GWL_STYLE,
                if resizable {
                    style | WS_THICKFRAME | WS_MAXIMIZEBOX
                } else {
                    style & !WS_THICKFRAME & !WS_MAXIMIZEBOX
                } as isize,
            );
            SetWindowPos(
                self.0.hwnd,
                null_mut(),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
        }
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        if fullscreen == self.0.fullscreen {
            return;
        }
        unsafe {
            if fullscreen {
                self.0.windowed_placement.length = size_of::<WINDOWPLACEMENT>() as u32;
                GetWindowPlacement(self.0.hwnd, &mut self.0.windowed_placement);
                self.0.windowed_style = GetWindowLong(self.0.hwnd, GWL_STYLE) as u32;
                ShowWindow(self.0.hwnd, SW_RESTORE);
                let monitor = MonitorFromWindow(self.0.hwnd, MONITOR_DEFAULTTONEAREST);
                let mut info = MONITORINFOEXW {
                    cbSize: size_of::<MONITORINFOEXW>() as u32,
                    ..Default::default()
                };
                GetMonitorInfoW(monitor, &mut info);
                SetWindowLong(self.0.hwnd, GWL_STYLE, WS_POPUP as isize);
                SetWindowPos(
                    self.0.hwnd,
                    null_mut(),
                    info.rcMonitor.left,
                    info.rcMonitor.top,
                    info.rcMonitor.right - info.rcMonitor.left,
                    info.rcMonitor.bottom - info.rcMonitor.top,
                    SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                );
            } else {
                SetWindowLong(self.0.hwnd, GWL_STYLE, self.0.windowed_style as isize);
                SetWindowPlacement(self.0.hwnd, &self.0.windowed_placement);
                SetWindowPos(
                    self.0.hwnd,
                    null_mut(),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                );
            }
        }
        self.0.fullscreen = fullscreen;
    }

    fn set_theme(&mut self, theme: Theme) {
        self.0.theme = theme;
        self.0.follows_system_theme = false;
        unsafe { set_titlebar_theme(self.0.hwnd, theme) }
    }

    fn follow_system_theme(&mut self) {
        self.0.follows_system_theme = true;
        self.0.theme = system_theme();
        unsafe {
            set_titlebar_theme(self.0.hwnd, self.0.theme);
            InvalidateRect(self.0.hwnd, null_mut(), TRUE);
        }
    }

    fn set_background_color(&mut self, color: u32) {
        self.0.background_color = Some(color);
        unsafe { InvalidateRect(self.0.hwnd, null_mut(), TRUE) };
    }

    fn set_cursor(&mut self, cursor: crate::Cursor) {
        self.0.cursor = load_cursor(cursor);
        unsafe {
            SetCursor(self.0.cursor);
            install_webview_input_hooks(self.0.as_mut());
        }
    }

    fn enable_events(&mut self, events: WindowEvents) {
        self.0.events |= events;
        if events.contains(WindowEvents::MOUSE)
            || events.contains(WindowEvents::WHEEL)
            || events.contains(WindowEvents::KEYBOARD)
        {
            unsafe { install_webview_input_hooks(self.0.as_mut()) };
        }
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
        WM_SETCURSOR if l_param as u16 as i32 == HTCLIENT => {
            unsafe { SetCursor(_self.cursor) };
            1
        }
        WM_SETTINGCHANGE | WM_THEMECHANGED if _self.follows_system_theme => {
            let theme = system_theme();
            if theme != _self.theme {
                _self.theme = theme;
                unsafe { set_titlebar_theme(hwnd, theme) };
                if _self.events.contains(WindowEvents::THEME_CHANGE) {
                    send_theme_change(theme);
                }
                unsafe { InvalidateRect(hwnd, null_mut(), TRUE) };
            }
            0
        }
        WM_ACTIVATE => {
            if w_param as u16 == WA_INACTIVE {
                _self.primary_button_down = false;
            }
            if _self.events.contains(WindowEvents::FOCUS) {
                send_event(crate::Event::Window(if w_param as u16 == WA_INACTIVE {
                    WindowEvent::Blur
                } else {
                    WindowEvent::Focus
                }));
            }
            0
        }
        WM_MOUSEMOVE if _self.events.contains(WindowEvents::MOUSE) => {
            let event = win_mouse_event(hwnd, _self, w_param, l_param, -1, 0);
            if !_self.tracks_mouse_leave {
                let mut tracking = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                unsafe { TrackMouseEvent(&mut tracking) };
                _self.tracks_mouse_leave = true;
                send_event(crate::Event::Window(WindowEvent::MouseEnter(event.clone())));
            }
            send_event(crate::Event::Window(WindowEvent::MouseMove(event)));
            0
        }
        WM_MOUSELEAVE if _self.events.contains(WindowEvents::MOUSE) => {
            _self.tracks_mouse_leave = false;
            if let Some(point) = _self.last_mouse {
                let event = win_mouse_event_at(hwnd, _self, point, 0, -1, 0);
                send_event(crate::Event::Window(WindowEvent::MouseLeave(event)));
            }
            _self.last_mouse = None;
            0
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN
            if _self.events.contains(WindowEvents::MOUSE) =>
        {
            let button = win_button(msg);
            if button == 0 {
                _self.primary_button_down = true;
            }
            let event = win_mouse_event(hwnd, _self, w_param, l_param, button, 1);
            send_event(crate::Event::Window(WindowEvent::MouseDown(event)));
            0
        }
        WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP
            if _self.events.contains(WindowEvents::MOUSE) =>
        {
            let event = win_mouse_event(hwnd, _self, w_param, l_param, win_button(msg), 1);
            send_event(crate::Event::Window(WindowEvent::MouseUp(event.clone())));
            if event.button == 0 && std::mem::take(&mut _self.primary_button_down) {
                send_event(crate::Event::Window(WindowEvent::Click(event)));
            }
            0
        }
        WM_MOUSEWHEEL | WM_MOUSEHWHEEL if _self.events.contains(WindowEvents::WHEEL) => {
            let mut screen = POINT {
                x: l_param as i16 as i32,
                y: (l_param >> 16) as i16 as i32,
            };
            let screen_x = screen.x as f32 * USER_DEFAULT_SCREEN_DPI as f32 / _self.dpi as f32;
            let screen_y = screen.y as f32 * USER_DEFAULT_SCREEN_DPI as f32 / _self.dpi as f32;
            unsafe { ScreenToClient(hwnd, &mut screen) };
            let point = LogicalPoint::new(
                screen.x as f32 * USER_DEFAULT_SCREEN_DPI as f32 / _self.dpi as f32,
                screen.y as f32 * USER_DEFAULT_SCREEN_DPI as f32 / _self.dpi as f32,
            );
            let mut mouse = win_mouse_event_at(hwnd, _self, point, 0, -1, 0);
            mouse.screen_x = screen_x;
            mouse.screen_y = screen_y;
            let delta = (w_param >> 16) as i16 as f32 / 120.0;
            send_event(crate::Event::Window(WindowEvent::Wheel(WheelEvent {
                mouse,
                delta_x: if msg == WM_MOUSEHWHEEL { delta } else { 0.0 },
                delta_y: if msg == WM_MOUSEWHEEL { -delta } else { 0.0 },
                delta_z: 0.0,
                delta_mode: 1,
            })));
            0
        }
        WM_KEYDOWN if _self.events.contains(WindowEvents::KEYBOARD) => {
            send_event(crate::Event::Window(WindowEvent::KeyDown(
                win_keyboard_event(w_param, l_param),
            )));
            0
        }
        WM_KEYUP if _self.events.contains(WindowEvents::KEYBOARD) => {
            send_event(crate::Event::Window(WindowEvent::KeyUp(
                win_keyboard_event(w_param, l_param),
            )));
            0
        }
        WM_SYSKEYDOWN | WM_SYSKEYUP if _self.events.contains(WindowEvents::KEYBOARD) => {
            let event = win_keyboard_event(w_param, l_param);
            send_event(crate::Event::Window(if msg == WM_SYSKEYDOWN {
                WindowEvent::KeyDown(event)
            } else {
                WindowEvent::KeyUp(event)
            }));
            unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) }
        }
        WM_MOVE => {
            let x = l_param as i16 as i32;
            let y = (l_param >> 16) as i16 as i32;
            if _self.events.contains(WindowEvents::MOVE) {
                send_event(crate::Event::Window(WindowEvent::Move(LogicalPoint::new(
                    (x * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                    (y * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                ))));
            }
            0
        }
        WM_SIZE => {
            let width = (l_param as u16) as i32;
            let height = ((l_param >> 16) as u16) as i32;
            if _self.events.contains(WindowEvents::RESIZE) {
                send_event(crate::Event::Window(WindowEvent::Resize(LogicalSize::new(
                    (width * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                    (height * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                ))));
            }
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
            let event = unsafe { Box::from_raw(ptr as *mut crate::Event) };
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

const fn win_button(message: u32) -> i16 {
    match message {
        WM_LBUTTONDOWN | WM_LBUTTONUP => 0,
        WM_MBUTTONDOWN | WM_MBUTTONUP => 1,
        _ => 2,
    }
}

fn modifier(key: i32) -> bool {
    unsafe { GetKeyState(key) < 0 }
}

const WEBVIEW_INPUT_SUBCLASS_ID: usize = 0x4257_4556;

pub(super) unsafe fn install_webview_input_hooks(window: *mut WindowData) {
    let window_data = unsafe { &*window };
    let default_cursor = unsafe { LoadCursorW(null_mut(), IDC_ARROW) };
    if window_data.cursor == default_cursor
        && !window_data.events.contains(WindowEvents::MOUSE)
        && !window_data.events.contains(WindowEvents::WHEEL)
        && !window_data.events.contains(WindowEvents::KEYBOARD)
    {
        return;
    }

    unsafe extern "system" fn install(hwnd: HWND, data: LPARAM) -> BOOL {
        unsafe {
            SetWindowSubclass(
                hwnd,
                webview_input_proc,
                WEBVIEW_INPUT_SUBCLASS_ID,
                data as usize,
            );
        }
        TRUE
    }

    unsafe { EnumChildWindows((*window).hwnd, install, window as LPARAM) };
}

fn child_mouse_lparam(child: HWND, window: HWND, l_param: LPARAM) -> LPARAM {
    let mut point = POINT {
        x: l_param as i16 as i32,
        y: (l_param >> 16) as i16 as i32,
    };
    unsafe {
        ClientToScreen(child, &mut point);
        ScreenToClient(window, &mut point);
    }
    ((point.y as u16 as u32) << 16 | point.x as u16 as u32) as LPARAM
}

unsafe extern "system" fn webview_input_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
    _id: usize,
    data: usize,
) -> isize {
    let window = unsafe { &mut *(data as *mut WindowData) };
    let root = window.hwnd;
    match msg {
        WM_SETCURSOR if l_param as u16 as i32 == HTCLIENT => {
            unsafe { SetCursor(window.cursor) };
            return 1;
        }
        WM_MOUSEMOVE if window.events.contains(WindowEvents::MOUSE) => {
            let l_param = child_mouse_lparam(hwnd, root, l_param);
            let event = win_mouse_event(root, window, w_param, l_param, -1, 0);
            if !window.tracks_mouse_leave {
                let mut tracking = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                unsafe { TrackMouseEvent(&mut tracking) };
                window.tracks_mouse_leave = true;
                send_event(crate::Event::Window(WindowEvent::MouseEnter(event.clone())));
            }
            send_event(crate::Event::Window(WindowEvent::MouseMove(event)));
        }
        WM_MOUSELEAVE if window.events.contains(WindowEvents::MOUSE) => {
            window.tracks_mouse_leave = false;
            if let Some(point) = window.last_mouse {
                let event = win_mouse_event_at(root, window, point, 0, -1, 0);
                send_event(crate::Event::Window(WindowEvent::MouseLeave(event)));
            }
            window.last_mouse = None;
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN
            if window.events.contains(WindowEvents::MOUSE) =>
        {
            let button = win_button(msg);
            if button == 0 {
                window.primary_button_down = true;
            }
            let l_param = child_mouse_lparam(hwnd, root, l_param);
            let event = win_mouse_event(root, window, w_param, l_param, button, 1);
            send_event(crate::Event::Window(WindowEvent::MouseDown(event)));
        }
        WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP
            if window.events.contains(WindowEvents::MOUSE) =>
        {
            let l_param = child_mouse_lparam(hwnd, root, l_param);
            let event = win_mouse_event(root, window, w_param, l_param, win_button(msg), 1);
            send_event(crate::Event::Window(WindowEvent::MouseUp(event.clone())));
            if event.button == 0 && std::mem::take(&mut window.primary_button_down) {
                send_event(crate::Event::Window(WindowEvent::Click(event)));
            }
        }
        WM_MOUSEWHEEL | WM_MOUSEHWHEEL if window.events.contains(WindowEvents::WHEEL) => {
            unsafe { window_proc(root, msg, w_param, l_param) };
        }
        WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP
            if window.events.contains(WindowEvents::KEYBOARD) =>
        {
            let event = win_keyboard_event(w_param, l_param);
            send_event(crate::Event::Window(
                if matches!(msg, WM_KEYDOWN | WM_SYSKEYDOWN) {
                    WindowEvent::KeyDown(event)
                } else {
                    WindowEvent::KeyUp(event)
                },
            ));
        }
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, msg, w_param, l_param) }
}

fn win_mouse_event(
    hwnd: HWND,
    data: &mut WindowData,
    w_param: WPARAM,
    l_param: LPARAM,
    button: i16,
    detail: u16,
) -> MouseEvent {
    let point = LogicalPoint::new(
        l_param as i16 as f32 * USER_DEFAULT_SCREEN_DPI as f32 / data.dpi as f32,
        (l_param >> 16) as i16 as f32 * USER_DEFAULT_SCREEN_DPI as f32 / data.dpi as f32,
    );
    win_mouse_event_at(hwnd, data, point, w_param, button, detail)
}

fn win_mouse_event_at(
    hwnd: HWND,
    data: &mut WindowData,
    point: LogicalPoint,
    w_param: WPARAM,
    button: i16,
    detail: u16,
) -> MouseEvent {
    let movement = data.last_mouse.map_or(LogicalPoint::new(0.0, 0.0), |last| {
        LogicalPoint::new(point.x - last.x, point.y - last.y)
    });
    data.last_mouse = Some(point);
    let mut screen = POINT {
        x: (point.x * data.dpi as f32 / USER_DEFAULT_SCREEN_DPI as f32) as i32,
        y: (point.y * data.dpi as f32 / USER_DEFAULT_SCREEN_DPI as f32) as i32,
    };
    unsafe { ClientToScreen(hwnd, &mut screen) };
    MouseEvent {
        client_x: point.x,
        client_y: point.y,
        screen_x: screen.x as f32 * USER_DEFAULT_SCREEN_DPI as f32 / data.dpi as f32,
        screen_y: screen.y as f32 * USER_DEFAULT_SCREEN_DPI as f32 / data.dpi as f32,
        movement_x: movement.x,
        movement_y: movement.y,
        button,
        buttons: ((w_param & 1) | ((w_param & 16) >> 3) | ((w_param & 2) << 1)) as u16,
        detail,
        alt_key: modifier(0x12),
        ctrl_key: modifier(0x11),
        meta_key: modifier(0x5B) || modifier(0x5C),
        shift_key: modifier(0x10),
    }
}

fn win_keyboard_event(w_param: WPARAM, l_param: LPARAM) -> KeyboardEvent {
    let key_code = w_param as u32;
    let key = match key_code {
        0x08 => "Backspace".into(),
        0x09 => "Tab".into(),
        0x0D => "Enter".into(),
        0x10 | 0xA0 | 0xA1 => "Shift".into(),
        0x11 | 0xA2 | 0xA3 => "Control".into(),
        0x12 | 0xA4 | 0xA5 => "Alt".into(),
        0x1B => "Escape".into(),
        0x20 => " ".into(),
        0x21 => "PageUp".into(),
        0x22 => "PageDown".into(),
        0x23 => "End".into(),
        0x24 => "Home".into(),
        0x25 => "ArrowLeft".into(),
        0x26 => "ArrowUp".into(),
        0x27 => "ArrowRight".into(),
        0x28 => "ArrowDown".into(),
        0x2D => "Insert".into(),
        0x2E => "Delete".into(),
        0x5B | 0x5C => "Meta".into(),
        _ => win_printable_key(key_code, l_param),
    };
    let code = win_physical_code(l_param).to_owned();
    let location = if code.starts_with("Numpad") {
        3
    } else if code.ends_with("Left") {
        1
    } else if code.ends_with("Right") {
        2
    } else {
        0
    };
    KeyboardEvent {
        key,
        code,
        location,
        repeat: l_param & (1 << 30) != 0,
        is_composing: false,
        alt_key: modifier(0x12),
        ctrl_key: modifier(0x11),
        meta_key: modifier(0x5B) || modifier(0x5C),
        shift_key: modifier(0x10),
    }
}

const fn win_physical_code(l_param: LPARAM) -> &'static str {
    let scan = (l_param >> 16) as u8;
    let extended = l_param & (1 << 24) != 0;
    match (scan, extended) {
        (0x01, _) => "Escape",
        (0x02, _) => "Digit1",
        (0x03, _) => "Digit2",
        (0x04, _) => "Digit3",
        (0x05, _) => "Digit4",
        (0x06, _) => "Digit5",
        (0x07, _) => "Digit6",
        (0x08, _) => "Digit7",
        (0x09, _) => "Digit8",
        (0x0a, _) => "Digit9",
        (0x0b, _) => "Digit0",
        (0x0c, _) => "Minus",
        (0x0d, _) => "Equal",
        (0x0e, _) => "Backspace",
        (0x0f, _) => "Tab",
        (0x10, _) => "KeyQ",
        (0x11, _) => "KeyW",
        (0x12, _) => "KeyE",
        (0x13, _) => "KeyR",
        (0x14, _) => "KeyT",
        (0x15, _) => "KeyY",
        (0x16, _) => "KeyU",
        (0x17, _) => "KeyI",
        (0x18, _) => "KeyO",
        (0x19, _) => "KeyP",
        (0x1a, _) => "BracketLeft",
        (0x1b, _) => "BracketRight",
        (0x1c, false) => "Enter",
        (0x1c, true) => "NumpadEnter",
        (0x1d, false) => "ControlLeft",
        (0x1d, true) => "ControlRight",
        (0x1e, _) => "KeyA",
        (0x1f, _) => "KeyS",
        (0x20, _) => "KeyD",
        (0x21, _) => "KeyF",
        (0x22, _) => "KeyG",
        (0x23, _) => "KeyH",
        (0x24, _) => "KeyJ",
        (0x25, _) => "KeyK",
        (0x26, _) => "KeyL",
        (0x27, _) => "Semicolon",
        (0x28, _) => "Quote",
        (0x29, _) => "Backquote",
        (0x2a, _) => "ShiftLeft",
        (0x2c, _) => "KeyZ",
        (0x2d, _) => "KeyX",
        (0x2e, _) => "KeyC",
        (0x2f, _) => "KeyV",
        (0x30, _) => "KeyB",
        (0x31, _) => "KeyN",
        (0x32, _) => "KeyM",
        (0x33, _) => "Comma",
        (0x34, _) => "Period",
        (0x35, false) => "Slash",
        (0x35, true) => "NumpadDivide",
        (0x36, _) => "ShiftRight",
        (0x38, false) => "AltLeft",
        (0x38, true) => "AltRight",
        (0x39, _) => "Space",
        (0x47, true) => "Home",
        (0x48, true) => "ArrowUp",
        (0x49, true) => "PageUp",
        (0x4b, true) => "ArrowLeft",
        (0x4d, true) => "ArrowRight",
        (0x4f, true) => "End",
        (0x50, true) => "ArrowDown",
        (0x51, true) => "PageDown",
        (0x52, true) => "Insert",
        (0x53, true) => "Delete",
        (0x5b, true) => "MetaLeft",
        (0x5c, true) => "MetaRight",
        _ => "Unidentified",
    }
}

fn win_printable_key(key_code: u32, l_param: LPARAM) -> String {
    let mut state = [0u8; 256];
    let mut buffer = [0u16; 8];
    let length = unsafe {
        GetKeyboardState(state.as_mut_ptr());
        ToUnicodeEx(
            key_code,
            ((l_param >> 16) & 0xff) as u32,
            state.as_ptr(),
            buffer.as_mut_ptr(),
            buffer.len() as i32,
            1 << 2,
            GetKeyboardLayout(0),
        )
    };
    if length > 0 {
        String::from_utf16_lossy(&buffer[..length as usize])
    } else if length < 0 {
        "Dead".into()
    } else {
        "Unidentified".into()
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

#[cfg(test)]
mod tests {
    use super::win_physical_code;

    const fn key_l_param(scan_code: u8, extended: bool) -> isize {
        ((scan_code as isize) << 16) | ((extended as isize) << 24)
    }

    #[test]
    fn physical_keys_match_dom_codes() {
        assert_eq!(win_physical_code(key_l_param(0x1e, false)), "KeyA");
        assert_eq!(win_physical_code(key_l_param(0x02, false)), "Digit1");
        assert_eq!(win_physical_code(key_l_param(0x1d, true)), "ControlRight");
        assert_eq!(win_physical_code(key_l_param(0x48, true)), "ArrowUp");
    }
}
