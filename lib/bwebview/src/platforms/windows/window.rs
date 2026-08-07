/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{OsString, c_void};
use std::fs::File;
use std::io::Read;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::{env, mem};

use super::event_loop::{
    APP_ID, FIRST_HWND, TASKBAR_BUTTON_CREATED, WM_SEND_MESSAGE, send_event, system_theme,
};
use super::webview2::*;
use super::win32::*;
use crate::{
    CloseRequest, LogicalPoint, LogicalSize, Theme, WindowBuilder, WindowEvent,
    WindowsProgressBarState,
};

pub(super) struct WindowData {
    pub(super) hwnd: HWND,
    pub(super) dpi: u32,
    pub(super) min_size: Option<LogicalSize>,
    pub(super) background_color: Option<u32>,
    pub(super) remember_window_state: bool,
    pub(super) allow_file_drop: bool,
    pub(super) resize_callback: Option<Box<dyn Fn(i32, i32)>>,
    taskbar_button_ready: bool,
    progress_bar: (Option<f32>, WindowsProgressBarState),
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
            remember_window_state: builder.remember_window_state,
            allow_file_drop: builder.allow_file_drop,
            resize_callback: None,
            taskbar_button_ready: false,
            progress_bar: (None, WindowsProgressBarState::Normal),
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
            if builder.allow_file_drop {
                DragAcceptFiles(hwnd, TRUE);
            }
            set_titlebar_theme(hwnd, builder.theme.unwrap_or_else(system_theme));

            let restored_window_state = if builder.remember_window_state {
                if let Ok(mut file) = File::open(config_dir().join("window.bin")) {
                    let size = size_of::<WINDOWPLACEMENT>();
                    let mut buffer = vec![0u8; size];
                    if file.read_exact(&mut buffer).is_ok() {
                        let mut window_placement: WINDOWPLACEMENT =
                            std::ptr::read(buffer.as_ptr() as *const _);
                        window_placement.length = size as u32;
                        SetWindowPlacement(hwnd, &window_placement);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };
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
        unsafe { set_titlebar_theme(self.0.hwnd, theme) }
    }

    fn set_background_color(&mut self, color: u32) {
        self.0.background_color = Some(color);
        unsafe { InvalidateRect(self.0.hwnd, null_mut(), TRUE) };
    }

    fn windows_set_progress_bar(&mut self, progress: Option<f32>, state: WindowsProgressBarState) {
        self.0.progress_bar = (progress, state);
        if self.0.taskbar_button_ready {
            set_progress_bar(self.0.hwnd, progress, state);
        }
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
        message if message == unsafe { TASKBAR_BUTTON_CREATED } => {
            _self.taskbar_button_ready = true;
            set_progress_bar(hwnd, _self.progress_bar.0, _self.progress_bar.1);
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
        WM_MOVE => {
            let x = l_param as u16 as i32;
            let y = (l_param >> 16) as u16 as i32;
            send_event(crate::Event::Window(WindowEvent::Move(LogicalPoint::new(
                (x * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                (y * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
            ))));
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
            let event = unsafe { Box::from_raw(ptr as *mut crate::Event) };
            send_event(*event);
            0
        }
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

unsafe fn handle_file_drop(drop: HDROP) {
    let count = unsafe { DragQueryFileW(drop, u32::MAX, null_mut(), 0) };
    for index in 0..count {
        let length = unsafe { DragQueryFileW(drop, index, null_mut(), 0) };
        let mut buffer = vec![0; length as usize + 1];
        unsafe { DragQueryFileW(drop, index, buffer.as_mut_ptr(), buffer.len() as u32) };
        send_event(crate::Event::Window(WindowEvent::DroppedFile(
            OsString::from_wide(&buffer[..length as usize]).into(),
        )));
    }
    unsafe { DragFinish(drop) };
}

fn save_window_state(hwnd: HWND) {
    unsafe {
        use std::io::Write;
        let mut window_placement: WINDOWPLACEMENT = mem::zeroed();
        window_placement.length = size_of::<WINDOWPLACEMENT>() as u32;
        GetWindowPlacement(hwnd, &mut window_placement);
        if let Ok(mut file) = File::create(config_dir().join("window.bin")) {
            _ = file.write_all(std::slice::from_raw_parts(
                &window_placement as *const _ as *const u8,
                size_of::<WINDOWPLACEMENT>(),
            ));
        }
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

// MARK: Windows progress bar
fn set_progress_bar(hwnd: HWND, progress: Option<f32>, state: WindowsProgressBarState) {
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
        if (vtable.hr_init)(taskbar) >= 0 {
            let taskbar_state = match (progress, state) {
                (None, _) => 0,
                (Some(_), WindowsProgressBarState::Normal) => 2,
                (Some(_), WindowsProgressBarState::Error) => 4,
                (Some(_), WindowsProgressBarState::Paused) => 8,
                (Some(_), WindowsProgressBarState::Indeterminate) => 1,
            };
            if let (Some(progress), false) = (
                progress,
                matches!(state, WindowsProgressBarState::Indeterminate),
            ) {
                let progress = progress.clamp(0.0, 1.0);
                _ = (vtable.set_progress_value)(taskbar, hwnd, (progress * 1000.0) as u64, 1000);
            }
            _ = (vtable.set_progress_state)(taskbar, hwnd, taskbar_state);
        }
        _ = (vtable.release)(taskbar);
    }
}
