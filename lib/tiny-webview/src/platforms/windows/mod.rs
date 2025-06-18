/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(clippy::upper_case_acronyms)]

use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::process::exit;
use std::ptr::null_mut;
use std::sync::mpsc;
use std::{env, mem};

use webview2_com::Microsoft::Web::WebView2::Win32::{
    CreateCoreWebView2EnvironmentWithOptions, ICoreWebView2Controller,
};
use webview2_com::{
    CreateCoreWebView2ControllerCompletedHandler, CreateCoreWebView2EnvironmentCompletedHandler,
    NavigationCompletedEventHandler, NavigationStartingEventHandler,
    NewWindowRequestedEventHandler,
};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, MAX_PATH, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute};
use windows::Win32::Graphics::Gdi::UpdateWindow;
use windows::Win32::System::LibraryLoader::{GetModuleFileNameA, GetModuleHandleA};
use windows::Win32::UI::HiDpi::{
    AdjustWindowRectExForDpi, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForSystem,
    SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Shell::{
    ExtractIconExA, FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, SHGetKnownFolderPath, ShellExecuteW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExA, DefWindowProcA, DestroyWindow, DispatchMessageA, GWL_STYLE,
    GWL_USERDATA, GetClientRect, GetMessageA, GetSystemMetrics, GetWindowRect, HICON, MINMAXINFO,
    MSG, PostQuitMessage, RegisterClassExA, SM_CXSCREEN, SM_CYSCREEN, SW_SHOWDEFAULT,
    SW_SHOWNORMAL, SWP_NOACTIVATE, SWP_NOREPOSITION, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
    SetWindowTextA, ShowWindow, TranslateMessage, USER_DEFAULT_SCREEN_DPI, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_CLOSE, WM_CREATE, WM_DESTROY, WM_DPICHANGED, WM_GETMINMAXINFO, WM_MOVE,
    WM_SIZE, WNDCLASSEXA, WS_OVERLAPPEDWINDOW, WS_THICKFRAME,
};
use windows::core::{BOOL, HSTRING, PCSTR, PWSTR, w};

use self::utils::*;
use crate::{Event, LogicalPoint, LogicalSize, WebviewBuilder};

mod utils;

/// Webview
pub(crate) struct Webview {
    builder: Option<WebviewBuilder>,
    hwnd: HWND,
    dpi: u32,
    min_size: Option<LogicalSize>,
    #[cfg(feature = "remember_window_state")]
    remember_window_state: bool,
    controller: Option<ICoreWebView2Controller>,
    event_handler: Option<fn(&mut Webview, Event)>,
}

impl Webview {
    pub(crate) fn new(builder: WebviewBuilder) -> Self {
        let min_size = builder.min_size;
        #[cfg(feature = "remember_window_state")]
        let remember_window_state = builder.remember_window_state;
        Self {
            builder: Some(builder),
            hwnd: HWND(null_mut()),
            dpi: USER_DEFAULT_SCREEN_DPI,
            min_size,
            #[cfg(feature = "remember_window_state")]
            remember_window_state,
            controller: None,
            event_handler: None,
        }
    }

    fn userdata_folder() -> String {
        unsafe {
            let appdata_path = convert_pwstr_to_string(
                SHGetKnownFolderPath(&FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, None)
                    .expect("Should be some"),
            );
            format!(
                "{}/{}",
                appdata_path,
                env::current_exe()
                    .expect("Can't get current process name")
                    .file_name()
                    .expect("Can't get current process name")
                    .to_string_lossy()
                    .strip_suffix(".exe")
                    .expect("Should strip .exe")
            )
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

        // Get executable icons
        let instance = unsafe { GetModuleHandleA(None) }.expect("Can't get module handle");
        let mut module_path = [0u8; MAX_PATH as usize];
        _ = unsafe { GetModuleFileNameA(instance.into(), &mut module_path) };
        let mut large_icon = HICON::default();
        let mut small_icon = HICON::default();
        unsafe {
            ExtractIconExA(
                PCSTR::from_raw(module_path.as_ptr()),
                0,
                Some(&mut large_icon),
                Some(&mut small_icon),
                1,
            );
        }

        // Register window class
        let wndclass = WNDCLASSEXA {
            cbSize: size_of::<WNDCLASSEXA>() as u32,
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            hIcon: large_icon,
            lpszClassName: PCSTR(c"window".as_ptr() as _),
            hIconSm: small_icon,
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

            if cfg!(feature = "remember_window_state") {
                if builder.remember_window_state {
                    let window_placement_path = format!("{}/window.bin", Self::userdata_folder());
                    if let Ok(mut file) = File::open(window_placement_path) {
                        let size =
                            size_of::<windows::Win32::UI::WindowsAndMessaging::WINDOWPLACEMENT>();
                        let mut buffer = vec![0u8; size];
                        if file.read_exact(&mut buffer).is_ok() {
                            let window_placement = std::ptr::read(buffer.as_ptr() as *const _);
                            _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPlacement(
                                hwnd,
                                &window_placement,
                            );
                        }
                    } else {
                        _ = ShowWindow(hwnd, SW_SHOWDEFAULT);
                    }
                } else {
                    _ = ShowWindow(hwnd, SW_SHOWDEFAULT);
                }
            } else {
                _ = ShowWindow(hwnd, SW_SHOWDEFAULT);
            }
            _ = UpdateWindow(hwnd);
            hwnd
        };

        // Create Webview2
        unsafe {
            let _self = self as *mut Webview;
            let environment = {
                let (tx, rx) = mpsc::channel();
                _ = CreateCoreWebView2EnvironmentWithOptions(
                    PWSTR::default(),
                    &HSTRING::from(Self::userdata_folder()),
                    None,
                    &CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(
                        move |error_code, environment| {
                            if let Err(e) = error_code {
                                panic!("Failed to create WebView2 environment: {:?}", e);
                            }
                            tx.send(environment.expect("Should be some"))
                                .expect("Should send environment");
                            Ok(())
                        },
                    )),
                );
                rx.recv().expect("Should receive environment")
            };

            let controller = {
                let (tx, rx) = mpsc::channel();
                let hwnd = self.hwnd;
                CreateCoreWebView2ControllerCompletedHandler::wait_for_async_operation(
                    Box::new(move |handler| {
                        _ = environment.CreateCoreWebView2Controller(hwnd, &handler);
                        Ok(())
                    }),
                    Box::new(move |error_code, controller| {
                        error_code?;
                        tx.send(controller.expect("WebView2 controller"))
                            .expect("Should send controller");
                        Ok(())
                    }),
                )
                .expect("Failed to create WebView2 controller");
                rx.recv().expect("Should receive controller")
            };

            let mut rect = RECT::default();
            _ = GetClientRect(self.hwnd, &mut rect);
            _ = controller.SetBounds(rect);

            let webview = controller.CoreWebView2().expect("Should be some");

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

            if cfg!(feature = "ipc") {
                _ = webview.AddScriptToExecuteOnDocumentCreated(
                    w!("window.ipc = new EventTarget();\
                        window.ipc.postMessage = message => window.chrome.webview.postMessage(`ipc${typeof message !== 'string' ? JSON.stringify(message) : message}`);\
                        console.log = message => window.chrome.webview.postMessage(`console${typeof message !== 'string' ? JSON.stringify(message) : message}`);"),
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
                            let message = convert_pwstr_to_string(message);
                            if message.starts_with("ipc") {
                                let message = message.trim_start_matches("ipc");
                                _self.send_event(Event::PageMessageReceived(message.to_string()));
                            } else if message.starts_with("console") {
                                let message = message.trim_start_matches("console");
                                println!("{}", message);
                            }
                            Ok(())
                        },
                    )),
                    null_mut(),
                );
            }

            self.controller = Some(controller);
            if let Some(url) = &builder.should_load_url {
                self.load_url(url);
            }
            if let Some(html) = &builder.should_load_html {
                self.load_html(html);
            }
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
            #[cfg(feature = "remember_window_state")]
            if _self.remember_window_state {
                unsafe {
                    use std::io::Write;
                    let mut window_placement = Default::default();
                    _ = windows::Win32::UI::WindowsAndMessaging::GetWindowPlacement(
                        hwnd,
                        &mut window_placement,
                    );
                    let window_placement_path =
                        format!("{}/window.bin", Webview::userdata_folder());
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(window_placement_path)
                    {
                        _ = file.write_all(std::slice::from_raw_parts(
                            &window_placement as *const _ as *const u8,
                            size_of::<windows::Win32::UI::WindowsAndMessaging::WINDOWPLACEMENT>(),
                        ));
                    }
                }
            }

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

// Also link to advapi32.dll for WebView2
#[link(name = "advapi32")]
unsafe extern "C" {}
