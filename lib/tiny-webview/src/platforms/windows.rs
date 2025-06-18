/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

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
    NavigationCompletedEventHandler, NavigationStartingEventHandler,
    NewWindowRequestedEventHandler,
};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute};
use windows::Win32::Graphics::Gdi::UpdateWindow;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::HiDpi::{
    AdjustWindowRectExForDpi, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForSystem,
    SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExA, DefWindowProcA, DestroyWindow, DispatchMessageA, GWL_STYLE,
    GWL_USERDATA, GetClientRect, GetMessageA, GetSystemMetrics, GetWindowRect, MINMAXINFO, MSG,
    PostQuitMessage, RegisterClassExA, SM_CXSCREEN, SM_CYSCREEN, SW_SHOWDEFAULT, SW_SHOWNORMAL,
    SWP_NOACTIVATE, SWP_NOREPOSITION, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos, SetWindowTextA,
    ShowWindow, TranslateMessage, USER_DEFAULT_SCREEN_DPI, WINDOW_EX_STYLE, WINDOW_LONG_PTR_INDEX,
    WINDOW_STYLE, WM_CLOSE, WM_CREATE, WM_DESTROY, WM_DPICHANGED, WM_GETMINMAXINFO, WM_MOVE,
    WM_SIZE, WNDCLASSEXA, WS_OVERLAPPEDWINDOW, WS_THICKFRAME,
};
use windows::core::{BOOL, HSTRING, PCSTR, PWSTR, w};

use crate::{Event, LogicalPoint, LogicalSize, WebviewBuilder};

/// Webview
pub(crate) struct Webview {
    builder: Option<WebviewBuilder>,
    hwnd: HWND,
    dpi: u32,
    min_size: Option<LogicalSize>,
    controller: Option<ICoreWebView2Controller>,
    event_handler: Option<fn(&mut Webview, Event)>,
}

impl Webview {
    pub(crate) fn new(builder: WebviewBuilder) -> Self {
        let min_size = builder.min_size;
        Self {
            builder: Some(builder),
            hwnd: HWND(null_mut()),
            dpi: USER_DEFAULT_SCREEN_DPI,
            min_size,
            controller: None,
            event_handler: None,
        }
    }

    fn send_event(&mut self, event: Event) {
        self.event_handler.expect("Should be some")(self, event);
    }
}

impl crate::Webview for Webview {
    fn run(&mut self, event_handler: fn(&mut Webview, Event)) -> ! {
        self.event_handler = Some(event_handler);

        let builder = self.builder.take().expect("Should be some");

        // Enable PerMonitorV2 high DPI awareness
        _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        self.dpi = unsafe { GetDpiForSystem() };

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
        self.hwnd = unsafe {
            let style = if builder.resizable {
                WS_OVERLAPPEDWINDOW
            } else {
                WS_OVERLAPPEDWINDOW & !WS_THICKFRAME
            };

            // Calculate window rect based on size and position
            let mut x = 0;
            let mut y = 0;
            let width =
                (builder.size.width as i32 * self.dpi as i32) / USER_DEFAULT_SCREEN_DPI as i32;
            let height =
                (builder.size.height as i32 * self.dpi as i32) / USER_DEFAULT_SCREEN_DPI as i32;
            if let Some(position) = builder.position {
                x = position.x as i32;
                y = position.y as i32;
            }
            if builder.should_center {
                let screen_width = GetSystemMetrics(SM_CXSCREEN);
                let screen_height = GetSystemMetrics(SM_CYSCREEN);
                x = (screen_width - width) / 2;
                y = (screen_height - height) / 2;
            }
            let mut rect = RECT {
                left: x,
                top: y,
                right: x + width,
                bottom: y + height,
            };
            _ = AdjustWindowRectExForDpi(&mut rect, style, false, WINDOW_EX_STYLE(0), self.dpi);

            let title = CString::new(builder.title).expect("Can't convert to CString");
            let hwnd = CreateWindowExA(
                WINDOW_EX_STYLE(0),
                wndclass.lpszClassName,
                PCSTR(title.as_ptr() as _),
                style,
                if builder.position.is_some() || builder.should_center {
                    rect.left
                } else {
                    CW_USEDEFAULT
                },
                if builder.position.is_some() || builder.should_center {
                    rect.top
                } else {
                    CW_USEDEFAULT
                },
                rect.right - rect.left,
                rect.bottom - rect.top,
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
            _ = ShowWindow(hwnd, SW_SHOWDEFAULT);
            _ = UpdateWindow(hwnd);
            hwnd
        };

        // Create Webview2
        let environment = {
            // FIXME: Clean up
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
            // FIXME: Clean up
            let hwnd = self.hwnd;
            let (tx, rx) = mpsc::channel();
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

                let webview = controller.CoreWebView2().expect("Should be some");
                let _self = self as *mut Webview;

                _ = webview.add_NavigationStarting(
                    &NavigationStartingEventHandler::create(Box::new(move |_sender, _args| {
                        let _self = &mut *_self;
                        _self.send_event(Event::PageLoadStarted);
                        Ok(())
                    })),
                    null_mut(),
                );
                _ = webview.add_NavigationCompleted(
                    &NavigationCompletedEventHandler::create(Box::new(move |_sender, _args| {
                        let _self = &mut *_self;
                        _self.send_event(Event::PageLoadFinished);
                        Ok(())
                    })),
                    null_mut(),
                );
                _ = webview.add_NewWindowRequested(
                    &NewWindowRequestedEventHandler::create(Box::new(|_sender, args| {
                        let args = args.expect("Should be some");
                        _ = args.SetHandled(true);
                        let mut uri = PWSTR::default();
                        _ = args.Uri(&mut uri);
                        _ = ShellExecuteW(None, w!("open"), uri, None, None, SW_SHOWNORMAL);
                        Ok(())
                    })),
                    null_mut(),
                );

                #[cfg(feature = "ipc")]
                {
                    _ = webview.AddScriptToExecuteOnDocumentCreated(
                        w!("window.ipc=new EventTarget();window.ipc.postMessage=message=>window.chrome.webview.postMessage(message);"),
                        &webview2_com::AddScriptToExecuteOnDocumentCreatedCompletedHandler::create(Box::new(|_sender, _args| {
                            Ok(())
                        })));
                    _ = webview.add_WebMessageReceived(
                        &webview2_com::WebMessageReceivedEventHandler::create(Box::new(
                            move |_sender, args| {
                                let _self = &mut *_self;
                                let args = args.expect("Should be some");
                                let mut message = PWSTR::default();
                                _ = args.TryGetWebMessageAsString(&mut message);

                                // Convert PWSTR to String
                                let mut len = 0;
                                while *message.0.add(len) != 0 {
                                    len += 1;
                                }
                                let message = String::from_utf16_lossy(std::slice::from_raw_parts(
                                    message.0, len,
                                ));
                                _self.send_event(Event::PageMessageReceived(message));
                                Ok(())
                            },
                        )),
                        null_mut(),
                    );
                }
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
            let mut msg = MSG::default();
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
        LogicalPoint::new(
            (rect.left * USER_DEFAULT_SCREEN_DPI as i32 / self.dpi as i32) as f32,
            (rect.top * USER_DEFAULT_SCREEN_DPI as i32 / self.dpi as i32) as f32,
        )
    }

    fn size(&self) -> LogicalSize {
        let mut rect = RECT::default();
        _ = unsafe { GetWindowRect(self.hwnd, &mut rect) };
        LogicalSize::new(
            ((rect.right - rect.left) * USER_DEFAULT_SCREEN_DPI as i32 / self.dpi as i32) as f32,
            ((rect.bottom - rect.top) * USER_DEFAULT_SCREEN_DPI as i32 / self.dpi as i32) as f32,
        )
    }

    fn set_position(&mut self, point: LogicalPoint) {
        _ = unsafe {
            SetWindowPos(
                self.hwnd,
                None,
                point.x as i32 * self.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32,
                point.y as i32 * self.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }

    fn set_size(&mut self, size: LogicalSize) {
        _ = unsafe {
            SetWindowPos(
                self.hwnd,
                None,
                0,
                0,
                size.width as i32 * self.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32,
                size.height as i32 * self.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32,
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
        WM_CREATE => {
            _self.send_event(Event::WindowCreated);
            LRESULT(0)
        }
        WM_MOVE => {
            let x = l_param.0 as u16 as i32;
            let y = (l_param.0 >> 16) as u16 as i32;
            _self.send_event(Event::WindowMoved(LogicalPoint::new(
                (x * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                (y * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
            )));
            LRESULT(0)
        }
        WM_SIZE => {
            let width = (l_param.0 as u16) as i32;
            let height = ((l_param.0 >> 16) as u16) as i32;
            _self.send_event(Event::WindowResized(LogicalSize::new(
                (width * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                (height * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
            )));
            if let Some(controller) = &_self.controller {
                _ = unsafe {
                    controller.SetBounds(RECT {
                        left: 0,
                        top: 0,
                        right: width,
                        bottom: height,
                    })
                };
            }
            LRESULT(0)
        }
        WM_DPICHANGED => {
            _self.dpi = (w_param.0 >> 16) as u32;
            let window_rect = unsafe { &*(l_param.0 as *const RECT) };
            _ = unsafe {
                SetWindowPos(
                    hwnd,
                    None,
                    window_rect.left,
                    window_rect.top,
                    window_rect.right - window_rect.left,
                    window_rect.bottom - window_rect.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                )
            };
            LRESULT(0)
        }
        WM_GETMINMAXINFO => {
            unsafe {
                if let Some(min_size) = _self.min_size {
                    let min_width =
                        min_size.width as i32 * _self.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32;
                    let min_height =
                        min_size.height as i32 * _self.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32;
                    let minmax_info: *mut MINMAXINFO = mem::transmute(l_param);
                    (*minmax_info).ptMinTrackSize.x = min_width;
                    (*minmax_info).ptMinTrackSize.y = min_height;
                }
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            _self.send_event(Event::WindowClosed);
            _ = unsafe { DestroyWindow(hwnd) };
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcA(hwnd, msg, w_param, l_param) },
    }
}

// Utils
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
