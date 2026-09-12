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
use super::event_loop::{APP_ID, send_event, system_theme};
#[cfg(feature = "file_drop")]
use super::file_drop::handle_file_drop;
use super::headers::*;
use super::input::{windows_key_event, windows_modifiers};
#[cfg(feature = "progress_bar")]
use super::progress_bar::ProgressBar;
#[cfg(feature = "remember_window_state")]
use super::window_state::{restore_window_state, save_window_state};
#[cfg(feature = "progress_bar")]
use crate::WindowsProgressBarState;
use crate::{
    ButtonState, CloseRequest, LogicalPoint, LogicalSize, MouseButton, PointerLockError,
    ScrollDelta, Theme, WindowBuilder, WindowEvent,
};

pub(super) struct WindowData {
    host: std::rc::Rc<crate::content::ContentHost>,
    pub(crate) window_id: crate::WindowId,
    pub(super) hwnd: HWND,
    pub(super) dpi: u32,
    pub(super) min_size: Option<LogicalSize>,
    pub(super) background_color: Option<u32>,
    theme: Option<Theme>,
    #[cfg(feature = "remember_window_state")]
    pub(super) remember_window_state: bool,
    #[cfg(feature = "file_drop")]
    pub(super) allow_file_drop: bool,
    #[cfg(feature = "progress_bar")]
    progress_bar: ProgressBar,
}

thread_local! {
    static WINDOW_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn post_content_redraw(window: *mut c_void) -> bool {
    unsafe { InvalidateRect(window, null(), FALSE) != FALSE }
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
    pub(crate) const fn id(&self) -> crate::WindowId {
        self.0.window_id
    }

    pub(crate) fn new(
        builder: &WindowBuilder,
        host: std::rc::Rc<crate::content::ContentHost>,
    ) -> Self {
        let initial_dpi = unsafe { GetDpiForSystem() };
        let mut window_data = Box::new(WindowData {
            host: host.clone(),
            window_id: builder.window_id,
            hwnd: null_mut(),
            dpi: initial_dpi,
            min_size: builder.min_size,
            background_color: builder.background_color,
            theme: builder.theme,
            #[cfg(feature = "remember_window_state")]
            remember_window_state: builder.remember_window_state,
            #[cfg(feature = "file_drop")]
            allow_file_drop: builder.allow_file_drop,
            #[cfg(feature = "progress_bar")]
            progress_bar: ProgressBar::default(),
        });

        // Check if window class is already registered
        let instance = unsafe { GetModuleHandleW(null()) };
        let class_name = unsafe {
            if let Some(ref app_id) = APP_ID {
                format!(
                    "bwindow-{}.{}.{}",
                    app_id.qualifier, app_id.organization, app_id.application
                )
            } else {
                "bwindow".to_string()
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
            host.set_handle(crate::NativeWindowHandle::Win32(hwnd));
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
        self.0.theme = Some(theme);
        let host = self.0.host.clone();
        unsafe { set_titlebar_theme(self.0.hwnd, theme) };
        host.theme_changed(theme);
    }

    fn set_background_color(&mut self, color: u32) {
        self.0.background_color = Some(color);
        unsafe { InvalidateRect(self.0.hwnd, null_mut(), TRUE) };
    }

    fn request_pointer_lock(&mut self) -> Result<(), PointerLockError> {
        let device = RAWINPUTDEVICE {
            usUsagePage: 1,
            usUsage: 2,
            dwFlags: 0,
            hwndTarget: self.0.hwnd,
        };
        if unsafe { RegisterRawInputDevices(&device, 1, size_of::<RAWINPUTDEVICE>() as u32) }
            == FALSE
        {
            return Err(PointerLockError(
                "Could not register raw mouse input".into(),
            ));
        }
        let mut point = POINT { x: 0, y: 0 };
        if unsafe { GetCursorPos(&mut point) } == FALSE {
            unregister_raw_mouse();
            return Err(PointerLockError(
                "Could not read the pointer position".into(),
            ));
        }
        let clip = RECT {
            left: point.x,
            top: point.y,
            right: point.x + 1,
            bottom: point.y + 1,
        };
        if unsafe { ClipCursor(&clip) } == FALSE {
            unregister_raw_mouse();
            return Err(PointerLockError("Could not confine the pointer".into()));
        }
        unsafe {
            ShowCursor(FALSE);
            SetCapture(self.0.hwnd);
        }
        Ok(())
    }

    fn exit_pointer_lock(&mut self) {
        release_pointer_lock();
    }

    #[cfg(feature = "progress_bar")]
    fn windows_set_progress_bar(&mut self, progress: Option<f32>, state: WindowsProgressBarState) {
        self.0.progress_bar.set(self.0.hwnd, progress, state);
    }
}

pub(crate) fn release_pointer_lock() {
    unsafe {
        ClipCursor(null());
        ShowCursor(TRUE);
        ReleaseCapture();
    }
    unregister_raw_mouse();
}

fn unregister_raw_mouse() {
    let device = RAWINPUTDEVICE {
        usUsagePage: 1,
        usUsage: 2,
        dwFlags: RIDEV_REMOVE,
        hwndTarget: null_mut(),
    };
    unsafe { RegisterRawInputDevices(&device, 1, size_of::<RAWINPUTDEVICE>() as u32) };
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
        WM_SETCURSOR if _self.host.pointer_locked() && l_param as u16 == HTCLIENT => 1,
        WM_SETFOCUS => {
            send_event(crate::Event::Window(_self.window_id, WindowEvent::Focus));
            0
        }
        WM_KILLFOCUS => {
            if w_param == 0 || unsafe { IsChild(hwnd, w_param as HWND) } == FALSE {
                send_event(crate::Event::Window(_self.window_id, WindowEvent::Blur));
            }
            0
        }
        WM_INPUT => {
            if _self.host.pointer_locked()
                && let Some(movement) = unsafe { raw_mouse_movement(l_param) }
            {
                _self.host.event_sender().send(WindowEvent::MouseMove {
                    position: LogicalPoint::default(),
                    movement,
                    modifiers: windows_modifiers(),
                });
            }
            unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) }
        }
        WM_MOUSEMOVE => {
            if !_self.host.pointer_locked() {
                let mut track = TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    ..Default::default()
                };
                unsafe { TrackMouseEvent(&mut track) };
                _self.host.event_sender().send(WindowEvent::MouseMove {
                    position: unsafe { pointer_position(hwnd, l_param) },
                    movement: LogicalPoint::default(),
                    modifiers: windows_modifiers(),
                });
            }
            0
        }
        WM_MOUSELEAVE => {
            _self.host.event_sender().send(WindowEvent::MouseLeave);
            0
        }
        WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN
        | WM_MBUTTONUP | WM_XBUTTONDOWN | WM_XBUTTONUP => {
            let pressed = matches!(
                msg,
                WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN
            );
            if pressed {
                unsafe {
                    SetFocus(hwnd);
                    SetCapture(hwnd);
                }
            } else if w_param & 0x73 == 0 && !_self.host.pointer_locked() {
                unsafe { ReleaseCapture() };
            }
            let event = crate::MouseEvent {
                button: mouse_button(msg, w_param),
                position: unsafe { pointer_position(hwnd, l_param) },
                modifiers: windows_modifiers(),
            };
            _self.host.event_sender().send(if pressed {
                WindowEvent::MouseDown(event)
            } else {
                WindowEvent::MouseUp(event)
            });
            mouse_button_result(msg)
        }
        WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
            let delta = (w_param >> 16) as u16 as i16 as f64 / 120.0;
            _self.host.event_sender().send(WindowEvent::Wheel(
                if msg == WM_MOUSEWHEEL {
                    ScrollDelta::Lines(0.0, -delta)
                } else {
                    ScrollDelta::Lines(delta, 0.0)
                },
                windows_modifiers(),
            ));
            0
        }
        WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => {
            let pressed = matches!(msg, WM_KEYDOWN | WM_SYSKEYDOWN);
            let event = windows_key_event(
                w_param as u32,
                l_param,
                if pressed {
                    ButtonState::Pressed
                } else {
                    ButtonState::Released
                },
            );
            _self.host.event_sender().send(if pressed {
                WindowEvent::KeyDown(event)
            } else {
                WindowEvent::KeyUp(event)
            });
            if matches!(msg, WM_KEYDOWN | WM_KEYUP) {
                0
            } else {
                unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) }
            }
        }
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            unsafe { BeginPaint(hwnd, &mut paint) };
            unsafe { EndPaint(hwnd, &paint) };
            // Redraw delivery can run application code and destroy this window.
            // End the raw WindowData borrow before entering that callback.
            let host = _self.host.clone();
            host.request_redraw();
            0
        }
        // WM_SETTINGCHANGE and WM_THEMECHANGED, including the application's color mode.
        0x001A | 0x031A => {
            let theme = _self.theme.unwrap_or_else(system_theme);
            let host = _self.host.clone();
            unsafe { set_titlebar_theme(hwnd, theme) };
            host.theme_changed(theme);
            0
        }
        #[cfg(feature = "progress_bar")]
        message if message == unsafe { TASKBAR_BUTTON_CREATED } => {
            _self.progress_bar.button_created(hwnd);
            0
        }
        WM_CREATE => {
            WINDOW_COUNT.with(|count| count.set(count.get() + 1));
            send_event(crate::Event::Window(_self.window_id, WindowEvent::Create));
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
            let x = l_param as u16 as i16 as i32;
            let y = (l_param >> 16) as u16 as i16 as i32;
            send_event(crate::Event::Window(
                _self.window_id,
                WindowEvent::Move(LogicalPoint::new(
                    (x * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                    (y * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                )),
            ));
            0
        }
        WM_SIZE => {
            let width = (l_param as u16) as i32;
            let height = ((l_param >> 16) as u16) as i32;
            let host = _self.host.clone();
            let id = _self.window_id;
            let dpi = _self.dpi;
            // Native content resizing may reenter application code and drop the window.
            host.resize(width as u32, height as u32);
            if host.is_closed() {
                return 0;
            }
            send_event(crate::Event::Window(
                id,
                WindowEvent::Resize(LogicalSize::new(
                    (width * USER_DEFAULT_SCREEN_DPI as i32 / dpi as i32) as f32,
                    (height * USER_DEFAULT_SCREEN_DPI as i32 / dpi as i32) as f32,
                )),
            ));
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
        #[cfg(feature = "file_drop")]
        WM_DROPFILES => {
            unsafe { handle_file_drop(_self.window_id, w_param as HDROP) };
            0
        }
        WM_CLOSE => {
            let request = CloseRequest::new();
            #[cfg(feature = "remember_window_state")]
            let remember = _self.remember_window_state;
            let id = _self.window_id;
            crate::dispatch::send_then(
                crate::Event::Window(id, WindowEvent::CloseRequested(request.clone())),
                move || {
                    // Another queued close may already have destroyed this native window.
                    let data = unsafe { GetWindowLong(hwnd, GWL_USERDATA) as *const WindowData };
                    if data.is_null()
                        || unsafe { (*data).window_id } != id
                        || request.default_prevented()
                    {
                        return;
                    }
                    #[cfg(feature = "remember_window_state")]
                    if remember {
                        save_window_state(hwnd);
                    }
                    unsafe { (*data).host.close() };
                    unsafe { DestroyWindow(hwnd) };
                },
            );
            0
        }
        WM_DESTROY => {
            _self.host.close();
            WINDOW_COUNT.with(|count| {
                count.set(count.get() - 1);
                if count.get() == 0 {
                    unsafe { PostQuitMessage(0) };
                }
            });
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) },
    }
}

const fn mouse_button(message: u32, wparam: WPARAM) -> MouseButton {
    match message {
        WM_LBUTTONDOWN | WM_LBUTTONUP => MouseButton::Left,
        WM_RBUTTONDOWN | WM_RBUTTONUP => MouseButton::Right,
        WM_MBUTTONDOWN | WM_MBUTTONUP => MouseButton::Middle,
        _ => MouseButton::Other(((wparam >> 16) & 0xffff) as u16),
    }
}

const fn mouse_button_result(message: u32) -> LRESULT {
    if matches!(message, WM_XBUTTONDOWN | WM_XBUTTONUP) {
        1
    } else {
        0
    }
}

unsafe fn pointer_position(hwnd: HWND, lparam: LPARAM) -> LogicalPoint {
    let scale = unsafe { GetDpiForWindow(hwnd) }.max(USER_DEFAULT_SCREEN_DPI) as f32
        / USER_DEFAULT_SCREEN_DPI as f32;
    LogicalPoint::new(
        lparam as u16 as i16 as f32 / scale,
        (lparam >> 16) as u16 as i16 as f32 / scale,
    )
}

unsafe fn raw_mouse_movement(input: LPARAM) -> Option<LogicalPoint> {
    let header_size = if size_of::<usize>() == 8 { 24 } else { 16 };
    let mut size = 0;
    if unsafe {
        GetRawInputData(
            input as HRAWINPUT,
            RID_INPUT,
            null_mut(),
            &mut size,
            header_size,
        )
    } == u32::MAX
        || size < header_size + 20
    {
        return None;
    }
    let mut bytes = vec![0u8; size as usize];
    if unsafe {
        GetRawInputData(
            input as HRAWINPUT,
            RID_INPUT,
            bytes.as_mut_ptr().cast(),
            &mut size,
            header_size,
        )
    } == u32::MAX
        || u32::from_ne_bytes(bytes[..4].try_into().ok()?) != RIM_TYPEMOUSE
    {
        return None;
    }
    let x = i32::from_ne_bytes(
        bytes[header_size as usize + 12..header_size as usize + 16]
            .try_into()
            .ok()?,
    );
    let y = i32::from_ne_bytes(
        bytes[header_size as usize + 16..header_size as usize + 20]
            .try_into()
            .ok()?,
    );
    (x != 0 || y != 0).then(|| LogicalPoint::new(x as f32, y as f32))
}

pub(crate) fn config_dir() -> PathBuf {
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
                    .file_stem()
                    .expect("Can't get current process name"),
            ))
        }
    }
    .expect("Can't get dirs");
    project_dirs.config_dir()
}
