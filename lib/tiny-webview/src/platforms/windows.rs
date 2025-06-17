/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// FIXME: Add WebView2 IPC support
// FIXME: Add high dpi support
// FIXME: Add remember window position and size

#![allow(clippy::upper_case_acronyms)]

use std::ffi::CString;
use std::mem;
use std::process::exit;
use std::ptr::null_mut;
use std::sync::mpsc;

use webview2_com::Microsoft::Web::WebView2::Win32::{
    CreateCoreWebView2Environment, ICoreWebView2Controller,
};
use webview2_com::{
    CreateCoreWebView2ControllerCompletedHandler, CreateCoreWebView2EnvironmentCompletedHandler,
};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute};
use windows::Win32::Graphics::Gdi::UpdateWindow;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExA, DefWindowProcA, DispatchMessageA, GWL_STYLE, GWL_USERDATA,
    GetClientRect, GetMessageA, GetSystemMetrics, GetWindowRect, MINMAXINFO, MSG, PostQuitMessage,
    RegisterClassExA, SM_CXSCREEN, SW_SHOWDEFAULT, SWP_NOACTIVATE, SWP_NOREPOSITION, SWP_NOSIZE,
    SWP_NOZORDER, SetWindowPos, SetWindowTextA, ShowWindow, TranslateMessage,
    WINDOW_LONG_PTR_INDEX, WINDOW_STYLE, WM_DESTROY, WM_GETMINMAXINFO, WM_SIZE, WNDCLASSEXA,
    WS_OVERLAPPEDWINDOW, WS_THICKFRAME,
};
use windows::core::{BOOL, HSTRING, PCSTR};

use crate::{Event, LogicalPoint, LogicalSize, WebviewBuilder};

/// Webview
pub(crate) struct Webview {
    builder: Option<WebviewBuilder>,
    hwnd: HWND,
    controller: Option<ICoreWebView2Controller>,
    min_size: Option<LogicalSize>,
}

impl Webview {
    pub(crate) fn new(builder: WebviewBuilder) -> Self {
        let min_size = builder.min_size;
        Self {
            builder: Some(builder),
            hwnd: HWND(null_mut()),
            controller: None,
            min_size,
        }
    }
}

impl crate::Webview for Webview {
    fn run(&mut self, _event_handler: fn(&mut Webview, Event)) -> ! {
        let builder = self.builder.take().expect("Should be some");

        // Register window class
        let instance = unsafe { GetModuleHandleA(None) }.expect("Can't get module handle");
        let wndclass = WNDCLASSEXA {
            cbSize: size_of::<WNDCLASSEXA>() as u32,
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            lpszClassName: PCSTR(c"window".as_ptr() as _),
            ..Default::default()
        };
        unsafe { RegisterClassExA(&wndclass) };

        // Create window
        // FIXME: Use single AdjustWindowRectExForDpi to calc window rect
        self.hwnd = unsafe {
            let title = CString::new(builder.title).expect("Can't convert to CString");

            let hwnd = CreateWindowExA(
                Default::default(),
                wndclass.lpszClassName,
                PCSTR(title.as_ptr() as _),
                if builder.resizable {
                    WS_OVERLAPPEDWINDOW
                } else {
                    WS_OVERLAPPEDWINDOW & !WS_THICKFRAME
                },
                if let Some(pos) = builder.position {
                    pos.x as i32
                } else {
                    CW_USEDEFAULT
                },
                if let Some(pos) = builder.position {
                    pos.y as i32
                } else {
                    CW_USEDEFAULT
                },
                builder.size.width as i32,
                builder.size.height as i32,
                None,
                None,
                Some(instance.into()),
                None,
            )
            .expect("Can't create window");
            SetWindowLong(hwnd, GWL_USERDATA, self as *mut Webview as isize);

            if builder.should_force_dark_mode {
                let enabled: BOOL = true.into();
                _ = DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_USE_IMMERSIVE_DARK_MODE,
                    &enabled as *const _ as *const _,
                    size_of::<BOOL>() as u32,
                );
            }

            // Center the window on the screen
            if builder.position.is_none() || builder.should_center {
                let mut rect = RECT::default();
                _ = GetWindowRect(hwnd, &mut rect);
                let screen_width = GetSystemMetrics(SM_CXSCREEN);
                let screen_height =
                    GetSystemMetrics(windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN);
                let x = (screen_width - (rect.right - rect.left)) / 2;
                let y = (screen_height - (rect.bottom - rect.top)) / 2;
                _ = SetWindowPos(
                    hwnd,
                    None,
                    x,
                    y,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }

            _ = ShowWindow(hwnd, SW_SHOWDEFAULT);
            _ = UpdateWindow(hwnd);
            hwnd
        };

        // Create Webview2
        let environment = {
            let (tx, rx) = mpsc::channel();

            CreateCoreWebView2EnvironmentCompletedHandler::wait_for_async_operation(
                Box::new(|environmentcreatedhandler| unsafe {
                    CreateCoreWebView2Environment(&environmentcreatedhandler)
                        .map_err(webview2_com::Error::WindowsError)
                }),
                Box::new(move |error_code, environment| {
                    error_code?;
                    tx.send(environment.expect("WebView2 environment"))
                        .expect("send over mpsc channel");
                    Ok(())
                }),
            )
            .expect("Failed to create WebView2 environment");

            rx.recv().expect("Failed to receive WebView2 environment")
        };

        self.controller = Some({
            let (tx, rx) = mpsc::channel();

            // FIXME: Clean up
            let hwnd = self.hwnd;
            CreateCoreWebView2ControllerCompletedHandler::wait_for_async_operation(
                Box::new(move |handler| unsafe {
                    environment
                        .CreateCoreWebView2Controller(hwnd, &handler)
                        .map_err(webview2_com::Error::WindowsError)
                }),
                Box::new(move |error_code, controller| {
                    error_code?;
                    tx.send(controller.expect("WebView2 controller"))
                        .expect("send over mpsc channel");
                    Ok(())
                }),
            )
            .expect("Failed to create WebView2 controller");

            rx.recv().expect("Failed to receive WebView2 cont roller")
        });

        unsafe {
            if let Some(controller) = &self.controller {
                let mut rect = RECT::default();
                _ = GetClientRect(self.hwnd, &mut rect);
                _ = controller.SetBounds(rect);
            }
        };
        if let Some(url) = &builder.should_load_url {
            self.load_url(url);
        }
        if let Some(html) = &builder.should_load_html {
            self.load_html(html);
        }

        // Start event loop
        unsafe {
            let mut msg: MSG = mem::zeroed();
            while GetMessageA(&mut msg, None, 0, 0).into() {
                _ = TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
            exit(msg.wParam.0 as i32);
        }
    }

    fn set_title(&mut self, title: impl AsRef<str>) {
        let title = CString::new(title.as_ref()).expect("Can't convert to CString");
        _ = unsafe { SetWindowTextA(self.hwnd, PCSTR(title.as_ptr() as _)) };
    }

    fn position(&self) -> LogicalPoint {
        let mut rect = RECT::default();
        _ = unsafe { GetWindowRect(self.hwnd, &mut rect) };
        LogicalPoint::new(rect.left as f32, rect.top as f32)
    }

    fn size(&self) -> LogicalSize {
        let mut rect = RECT::default();
        _ = unsafe { GetWindowRect(self.hwnd, &mut rect) };
        LogicalSize::new(
            (rect.right - rect.left) as f32,
            (rect.bottom - rect.top) as f32,
        )
    }

    fn set_position(&mut self, point: LogicalPoint) {
        _ = unsafe {
            SetWindowPos(
                self.hwnd,
                None,
                point.x as i32,
                point.y as i32,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }

    fn set_size(&mut self, _size: LogicalSize) {
        _ = unsafe {
            SetWindowPos(
                self.hwnd,
                None,
                0,
                0,
                _size.width as i32,
                _size.height as i32,
                SWP_NOREPOSITION | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }

    fn set_min_size(&mut self, min_size: LogicalSize) {
        self.min_size = Some(min_size);
    }

    fn set_resizable(&mut self, resizable: bool) {
        unsafe {
            let style = WINDOW_STYLE(GetWindowLong(self.hwnd, GWL_STYLE) as u32);
            SetWindowLong(
                self.hwnd,
                GWL_STYLE,
                if resizable {
                    style & !WS_THICKFRAME
                } else {
                    style | WS_THICKFRAME
                }
                .0 as isize,
            );
        }
    }

    fn load_url(&mut self, url: impl AsRef<str>) {
        unsafe {
            if let Some(controller) = &self.controller {
                let webview = controller.CoreWebView2().expect("Should be some");
                _ = webview.Navigate(&HSTRING::from(url.as_ref()));
            }
        }
    }

    fn load_html(&mut self, html: impl AsRef<str>) {
        unsafe {
            if let Some(controller) = &self.controller {
                let webview = controller.CoreWebView2().expect("Should be some");
                _ = webview.NavigateToString(&HSTRING::from(html.as_ref()));
            }
        }
    }

    fn evaluate_script(&mut self, script: impl AsRef<str>) {
        unsafe {
            if let Some(controller) = &self.controller {
                let webview = controller.CoreWebView2().expect("Should be some");
                _ = webview.ExecuteScript(&HSTRING::from(script.as_ref()), None);
            }
        }
    }

    #[cfg(feature = "ipc")]
    fn send_ipc_message(&mut self, _message: impl AsRef<str>) {
        todo!()
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let _self = unsafe {
        let ptr = GetWindowLong(hwnd, GWL_USERDATA) as *mut Webview;
        if ptr.is_null() {
            return DefWindowProcA(hwnd, msg, w_param, l_param);
        }
        &mut *ptr
    };
    match msg {
        WM_SIZE => {
            unsafe {
                if let Some(controller) = &_self.controller {
                    let mut rect = RECT::default();
                    _ = GetClientRect(hwnd, &mut rect);
                    _ = controller.SetBounds(rect);
                }
            }
            LRESULT(0)
        }
        WM_GETMINMAXINFO => {
            unsafe {
                if let Some(min_size) = _self.min_size {
                    let minmax_info: *mut MINMAXINFO = mem::transmute(l_param);
                    (*minmax_info).ptMinTrackSize.x = min_size.width as i32;
                    (*minmax_info).ptMinTrackSize.y = min_size.height as i32;
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcA(hwnd, msg, w_param, l_param) },
    }
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
unsafe fn GetWindowLong(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    (unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowLongA(hwnd, index) }) as isize
}
#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
unsafe fn GetWindowLong(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrA(hwnd, index) }
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
unsafe fn SetWindowLong(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    (unsafe { windows::Win32::UI::WindowsAndMessaging::SetWindowLongA(hwnd, index, value as i32) })
        as isize
}
#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
unsafe fn SetWindowLong(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    unsafe { windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrA(hwnd, index, value) }
}
