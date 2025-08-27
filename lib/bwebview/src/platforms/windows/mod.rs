/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(clippy::upper_case_acronyms)]

use std::ffi::{CString, c_void};
use std::fs::File;
use std::io::Read;
use std::process::exit;
use std::ptr::null_mut;
use std::sync::mpsc;
use std::{env, mem};

use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_COLOR, CreateCoreWebView2EnvironmentWithOptions, ICoreWebView2Controller,
    ICoreWebView2Controller2,
};
use webview2_com::{
    CreateCoreWebView2ControllerCompletedHandler, CreateCoreWebView2EnvironmentCompletedHandler,
    NavigationCompletedEventHandler, NavigationStartingEventHandler,
    NewWindowRequestedEventHandler,
};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, MAX_PATH, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute};
use windows::Win32::Graphics::Gdi::{
    CreateSolidBrush, EnumDisplayMonitors, FillRect, GetMonitorInfoA, HDC, HMONITOR,
    InvalidateRect, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MONITORINFOEXA, MonitorFromPoint,
    UpdateWindow,
};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameA, GetModuleHandleA};
use windows::Win32::UI::HiDpi::{
    AdjustWindowRectExForDpi, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForMonitor,
    GetDpiForSystem, MDT_EFFECTIVE_DPI, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Shell::{
    ExtractIconExA, FOLDERID_RoamingAppData, KF_FLAG_DEFAULT, SHGetKnownFolderPath, ShellExecuteW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExA, DefWindowProcA, DestroyWindow, DispatchMessageA, GWL_STYLE,
    GWL_USERDATA, GetClassInfoExA, GetClientRect, GetMessageA, GetSystemMetrics, GetWindowRect,
    HICON, MINMAXINFO, MSG, PostMessageA, PostQuitMessage, RegisterClassExA, SM_CXSCREEN,
    SM_CYSCREEN, SW_SHOWDEFAULT, SW_SHOWNORMAL, SWP_NOACTIVATE, SWP_NOREPOSITION, SWP_NOSIZE,
    SWP_NOZORDER, SetWindowPos, SetWindowTextA, ShowWindow, TranslateMessage,
    USER_DEFAULT_SCREEN_DPI, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLOSE, WM_CREATE, WM_DESTROY,
    WM_DPICHANGED, WM_ERASEBKGND, WM_GETMINMAXINFO, WM_MOVE, WM_SIZE, WM_USER, WNDCLASSEXA,
    WS_OVERLAPPEDWINDOW, WS_POPUP, WS_THICKFRAME,
};
use windows::core::{BOOL, HSTRING, Interface, PCSTR, PWSTR, w};

use self::utils::*;
use crate::{Event, LogicalPoint, LogicalSize, Theme, WebviewBuilder};

mod utils;

// MARK: EventLoop
pub(crate) struct PlatformEventLoop;

static mut FIRST_HWND: Option<HWND> = None;
static mut EVENT_HANDLER: Option<Box<dyn FnMut(Event) + 'static>> = None;

impl PlatformEventLoop {
    pub(crate) fn new() -> Self {
        // Enable PerMonitorV2 high DPI awareness
        _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        Self
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
            _hdc: HDC,
            _lprc_clip: *mut RECT,
            _lparam: LPARAM,
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
            _ = EnumDisplayMonitors(None, None, Some(monitor_enum_proc), LPARAM(0));
            #[allow(static_mut_refs)]
            MONITORS.take().unwrap_or_default()
        }
    }

    fn run(self, event_handler: impl FnMut(Event) + 'static) -> ! {
        unsafe { EVENT_HANDLER = Some(Box::new(event_handler)) };

        // Start message loop
        unsafe {
            let mut msg = MSG::default();
            while GetMessageA(&mut msg, None, 0, 0).into() {
                _ = TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
            exit(msg.wParam.0 as i32);
        }
    }

    fn create_proxy(&self) -> PlatformEventLoopProxy {
        PlatformEventLoopProxy::new()
    }
}

fn send_event(event: Event) {
    unsafe {
        #[allow(static_mut_refs)]
        if let Some(handler) = &mut EVENT_HANDLER {
            handler(event);
        }
    }
}

// MARK: PlatformEventLoopProxy
const WM_SEND_MESSAGE: u32 = WM_USER + 1;

pub(crate) struct PlatformEventLoopProxy;

impl PlatformEventLoopProxy {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl crate::EventLoopProxyInterface for PlatformEventLoopProxy {
    fn send_user_event(&self, data: String) {
        if let Some(hwnd) = unsafe { FIRST_HWND } {
            let ptr = Box::leak(Box::new(Event::UserEvent(data))) as *mut Event as *mut c_void;
            _ = unsafe {
                PostMessageA(Some(hwnd), WM_SEND_MESSAGE, WPARAM(ptr as usize), LPARAM(0))
            };
        }
    }
}

// MARK: PlatformMonitor
pub(crate) struct PlatformMonitor {
    hmonitor: HMONITOR,
    info: MONITORINFOEXA,
}

impl PlatformMonitor {
    pub(crate) fn new(hmonitor: HMONITOR) -> Self {
        let mut info = MONITORINFOEXA {
            monitorInfo: MONITORINFO {
                cbSize: size_of::<MONITORINFOEXA>() as u32,
                ..Default::default()
            },
            ..Default::default()
        };
        unsafe {
            _ = GetMonitorInfoA(hmonitor, &mut info as *mut _ as *mut _);
        }
        Self { hmonitor, info }
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
            self.info.monitorInfo.rcMonitor.left as f32,
            self.info.monitorInfo.rcMonitor.top as f32,
        )
    }

    fn size(&self) -> LogicalSize {
        LogicalSize::new(
            (self.info.monitorInfo.rcMonitor.right - self.info.monitorInfo.rcMonitor.left) as f32,
            (self.info.monitorInfo.rcMonitor.bottom - self.info.monitorInfo.rcMonitor.top) as f32,
        )
    }

    fn scale_factor(&self) -> f32 {
        unsafe {
            let mut dpi_x = USER_DEFAULT_SCREEN_DPI;
            let mut dpi_y = USER_DEFAULT_SCREEN_DPI;
            let result = GetDpiForMonitor(self.hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
            if result.is_ok() {
                dpi_x as f32 / USER_DEFAULT_SCREEN_DPI as f32
            } else {
                1.0
            }
        }
    }

    fn is_primary(&self) -> bool {
        self.info.monitorInfo.rcMonitor.left == 0 && self.info.monitorInfo.rcMonitor.top == 0
    }
}

// MARK: Webview
struct WebviewData {
    hwnd: HWND,
    dpi: u32,
    min_size: Option<LogicalSize>,
    background_color: Option<u32>,
    #[cfg(feature = "remember_window_state")]
    remember_window_state: bool,
    controller: Option<ICoreWebView2Controller>,
}

pub(crate) struct PlatformWebview(Box<WebviewData>);

impl PlatformWebview {
    pub(crate) fn new(builder: WebviewBuilder) -> Self {
        let dpi = unsafe { GetDpiForSystem() };

        // Check if window class is already registered
        let instance = unsafe { GetModuleHandleA(None) }.expect("Can't get module handle");
        let class_name = PCSTR(c"window".as_ptr() as _);
        unsafe {
            let mut wndclass = WNDCLASSEXA::default();
            if GetClassInfoExA(Some(instance.into()), class_name, &mut wndclass as *mut _).is_err()
            {
                // Get executable icons
                let mut module_path = [0u8; MAX_PATH as usize];
                _ = GetModuleFileNameA(instance.into(), &mut module_path);
                let mut large_icon = HICON::default();
                let mut small_icon = HICON::default();
                ExtractIconExA(
                    PCSTR::from_raw(module_path.as_ptr()),
                    0,
                    Some(&mut large_icon),
                    Some(&mut small_icon),
                    1,
                );

                // Register window class
                let wndclass = WNDCLASSEXA {
                    cbSize: size_of::<WNDCLASSEXA>() as u32,
                    lpfnWndProc: Some(window_proc),
                    hInstance: instance.into(),
                    hIcon: large_icon,
                    lpszClassName: class_name,
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
                WS_OVERLAPPEDWINDOW & !WS_THICKFRAME
            };

            // Calculate window rect based on size and position
            let monitor_rect = if let Some(monitor) = builder.monitor {
                monitor.info.monitorInfo.rcMonitor
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
            _ = AdjustWindowRectExForDpi(&mut rect, style, false, WINDOW_EX_STYLE(0), dpi);

            let title = CString::new(builder.title).expect("Can't convert to CString");
            let hwnd = CreateWindowExA(
                WINDOW_EX_STYLE(0),
                class_name,
                PCSTR(title.as_ptr() as _),
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
                None,
                None,
                Some(instance.into()),
                None,
            )
            .expect("Can't create window");
            if let Some(theme) = builder.theme {
                let enabled: BOOL = (theme == Theme::Dark).into();
                _ = DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_USE_IMMERSIVE_DARK_MODE,
                    &enabled as *const _ as *const _,
                    size_of::<BOOL>() as u32,
                );
            }

            #[cfg(feature = "remember_window_state")]
            let should_show_window = if builder.remember_window_state {
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
                _ = ShowWindow(hwnd, SW_SHOWDEFAULT);
            }
            _ = UpdateWindow(hwnd);
            hwnd
        };

        // Create Webview2
        let controller = unsafe {
            if let Some(color) = builder.background_color {
                env::set_var(
                    "WEBVIEW2_DEFAULT_BACKGROUND_COLOR",
                    format!("0xFF{:06X}", color & 0xFFFFFF),
                );
            }
            let environment = {
                let (tx, rx) = mpsc::channel();
                _ = CreateCoreWebView2EnvironmentWithOptions(
                    PWSTR::default(),
                    &HSTRING::from(Self::userdata_folder()),
                    None,
                    &CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(
                        move |error_code, environment| {
                            if let Err(e) = error_code {
                                panic!("Failed to create WebView2 environment: {e:?}");
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
            _ = GetClientRect(hwnd, &mut rect);
            _ = controller.SetBounds(rect);
            if builder.background_color.is_some() {
                let controller2 = controller
                    .cast::<ICoreWebView2Controller2>()
                    .expect("Should be some");
                _ = controller2.SetDefaultBackgroundColor(COREWEBVIEW2_COLOR {
                    A: 0x0,
                    R: 0x0,
                    G: 0x0,
                    B: 0x0,
                });
            }

            let webview = controller.CoreWebView2().expect("Should be some");

            _ = webview.add_NavigationStarting(
                &NavigationStartingEventHandler::create(Box::new(move |_sender, _args| {
                    send_event(Event::PageLoadStarted);
                    Ok(())
                })),
                null_mut(),
            );
            _ = webview.add_NavigationCompleted(
                &NavigationCompletedEventHandler::create(Box::new(move |_sender, _args| {
                    send_event(Event::PageLoadFinished);
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
                        let args = args.expect("Should be some");
                        let mut message = PWSTR::default();
                        _ = args.TryGetWebMessageAsString(&mut message);
                        let message = convert_pwstr_to_string(message);
                        if message.starts_with("ipc") {
                            let message = message.trim_start_matches("ipc");
                            send_event(Event::PageMessageReceived(message.to_string()));
                        } else if message.starts_with("console") {
                            let message = message.trim_start_matches("console");
                            println!("{message}");
                        }
                        Ok(())
                    },
                )),
                null_mut(),
            );

            if let Some(url) = &builder.should_load_url {
                _ = webview.Navigate(&HSTRING::from(url));
            }
            if let Some(html) = &builder.should_load_html {
                _ = webview.NavigateToString(&HSTRING::from(html));
            }
            controller
        };

        #[allow(static_mut_refs)]
        unsafe {
            if FIRST_HWND.is_none() {
                FIRST_HWND = Some(hwnd);
            }
        }

        let webview_data = Box::new(WebviewData {
            hwnd,
            dpi,
            min_size: builder.min_size,
            background_color: builder.background_color,
            #[cfg(feature = "remember_window_state")]
            remember_window_state: builder.remember_window_state,
            controller: Some(controller),
        });
        unsafe {
            SetWindowLong(
                hwnd,
                GWL_USERDATA,
                webview_data.as_ref() as *const _ as isize,
            )
        };
        Self(webview_data)
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
}

impl crate::WebviewInterface for PlatformWebview {
    fn set_title(&mut self, title: impl AsRef<str>) {
        let title = CString::new(title.as_ref()).expect("Can't convert to CString");
        _ = unsafe { SetWindowTextA(self.0.hwnd, PCSTR(title.as_ptr() as _)) };
    }

    fn position(&self) -> LogicalPoint {
        let mut rect = RECT::default();
        _ = unsafe { GetWindowRect(self.0.hwnd, &mut rect) };
        LogicalPoint::new(
            (rect.left * USER_DEFAULT_SCREEN_DPI as i32 / self.0.dpi as i32) as f32,
            (rect.top * USER_DEFAULT_SCREEN_DPI as i32 / self.0.dpi as i32) as f32,
        )
    }

    fn size(&self) -> LogicalSize {
        let mut rect = RECT::default();
        _ = unsafe { GetWindowRect(self.0.hwnd, &mut rect) };
        LogicalSize::new(
            ((rect.right - rect.left) * USER_DEFAULT_SCREEN_DPI as i32 / self.0.dpi as i32) as f32,
            ((rect.bottom - rect.top) * USER_DEFAULT_SCREEN_DPI as i32 / self.0.dpi as i32) as f32,
        )
    }

    fn set_position(&mut self, point: LogicalPoint) {
        _ = unsafe {
            SetWindowPos(
                self.0.hwnd,
                None,
                point.x as i32 * self.0.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32,
                point.y as i32 * self.0.dpi as i32 / USER_DEFAULT_SCREEN_DPI as i32,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }

    fn set_size(&mut self, size: LogicalSize) {
        _ = unsafe {
            SetWindowPos(
                self.0.hwnd,
                None,
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
            let style = WINDOW_STYLE(GetWindowLong(self.0.hwnd, GWL_STYLE) as u32);
            SetWindowLong(
                self.0.hwnd,
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

    fn set_theme(&mut self, theme: Theme) {
        unsafe {
            let enabled: BOOL = (theme == Theme::Dark).into();
            _ = DwmSetWindowAttribute(
                self.0.hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &enabled as *const _ as *const _,
                size_of::<BOOL>() as u32,
            );
        }
    }

    fn set_background_color(&mut self, color: u32) {
        self.0.background_color = Some(color);
        _ = unsafe { InvalidateRect(Some(self.0.hwnd), None, true) };
    }

    fn url(&self) -> Option<String> {
        unsafe {
            if let Some(controller) = &self.0.controller {
                let webview = controller.CoreWebView2().expect("Should be some");
                let mut uri = PWSTR::default();
                _ = webview.Source(&mut uri);
                Some(convert_pwstr_to_string(uri))
            } else {
                None
            }
        }
    }

    fn load_url(&mut self, url: impl AsRef<str>) {
        unsafe {
            if let Some(controller) = &self.0.controller {
                let webview = controller.CoreWebView2().expect("Should be some");
                _ = webview.Navigate(&HSTRING::from(url.as_ref()));
            }
        }
    }

    fn load_html(&mut self, html: impl AsRef<str>) {
        unsafe {
            if let Some(controller) = &self.0.controller {
                let webview = controller.CoreWebView2().expect("Should be some");
                _ = webview.NavigateToString(&HSTRING::from(html.as_ref()));
            }
        }
    }

    fn evaluate_script(&mut self, script: impl AsRef<str>) {
        unsafe {
            if let Some(controller) = &self.0.controller {
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
        let ptr = GetWindowLong(hwnd, GWL_USERDATA) as *mut WebviewData;
        if ptr.is_null() {
            return DefWindowProcA(hwnd, msg, w_param, l_param);
        }
        &mut *ptr
    };
    match msg {
        WM_CREATE => {
            send_event(Event::WindowCreated);
            LRESULT(0)
        }
        WM_ERASEBKGND => {
            if let Some(color) = _self.background_color {
                let hdc = HDC(w_param.0 as *mut c_void);
                let mut client_rect = RECT::default();
                _ = unsafe { GetClientRect(hwnd, &mut client_rect) };
                let brush = unsafe {
                    CreateSolidBrush(COLORREF(
                        ((color & 0xFF) << 16) | (color & 0xFF00) | ((color >> 16) & 0xFF),
                    ))
                };
                _ = unsafe { FillRect(hdc, &client_rect, brush) };
                LRESULT(1)
            } else {
                LRESULT(0)
            }
        }
        WM_MOVE => {
            let x = l_param.0 as u16 as i32;
            let y = (l_param.0 >> 16) as u16 as i32;
            send_event(Event::WindowMoved(LogicalPoint::new(
                (x * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                (y * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
            )));
            LRESULT(0)
        }
        WM_SIZE => {
            let width = (l_param.0 as u16) as i32;
            let height = ((l_param.0 >> 16) as u16) as i32;
            send_event(Event::WindowResized(LogicalSize::new(
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
        WM_SEND_MESSAGE => {
            let ptr = w_param.0 as *mut c_void;
            let event = unsafe { Box::from_raw(ptr as *mut Event) };
            send_event(*event);
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
                        format!("{}/window.bin", PlatformWebview::userdata_folder());
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

            send_event(Event::WindowClosed);
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
