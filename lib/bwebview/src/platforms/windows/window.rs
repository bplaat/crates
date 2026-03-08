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

use super::event_loop::{APP_ID, FIRST_HWND, WM_SEND_MESSAGE, send_event};
use super::webview2::*;
use super::win32::*;
use crate::{LogicalPoint, LogicalSize, Theme, WindowBuilder, WindowEvent, WindowId};

pub(super) struct WindowData {
    pub(super) window_id: WindowId,
    pub(super) hwnd: HWND,
    pub(super) dpi: u32,
    pub(super) min_size: Option<LogicalSize>,
    pub(super) background_color: Option<u32>,
    #[cfg(feature = "remember_window_state")]
    pub(super) remember_window_state: bool,
    pub(super) controller: Option<*mut ICoreWebView2Controller>,
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

            #[cfg(feature = "remember_window_state")]
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
            #[cfg(not(feature = "remember_window_state"))]
            let should_show_window = true;
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
            #[cfg(feature = "remember_window_state")]
            remember_window_state: builder.remember_window_state,
            controller: None,
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
            send_event(crate::Event::Window(_self.window_id, WindowEvent::Created));
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
            send_event(crate::Event::Window(
                _self.window_id,
                WindowEvent::Moved(LogicalPoint::new(
                    (x * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                    (y * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                )),
            ));
            0
        }
        WM_SIZE => {
            let width = (l_param as u16) as i32;
            let height = ((l_param >> 16) as u16) as i32;
            send_event(crate::Event::Window(
                _self.window_id,
                WindowEvent::Resized(LogicalSize::new(
                    (width * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                    (height * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                )),
            ));
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
        WM_SEND_MESSAGE => {
            let ptr = w_param as *mut c_void;
            let event = unsafe { Box::from_raw(ptr as *mut crate::Event) };
            send_event(*event);
            0
        }
        WM_CLOSE => {
            #[cfg(feature = "remember_window_state")]
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

            send_event(crate::Event::Window(_self.window_id, WindowEvent::Closed));
            unsafe { DestroyWindow(hwnd) };
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
