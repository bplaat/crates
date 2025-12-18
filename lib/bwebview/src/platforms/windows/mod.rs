/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CString, c_void};
use std::fs::File;
use std::io::Read;
use std::process::exit;
use std::ptr::{null, null_mut};
use std::{env, mem};

use self::webview2::*;
use self::win32::*;
#[cfg(feature = "custom_protocol")]
use crate::CustomProtocol;
use crate::{Event, EventLoopBuilder, LogicalPoint, LogicalSize, Theme, WebviewBuilder};

mod webview2;
mod win32;

// MARK: EventLoop
pub(crate) struct PlatformEventLoop;

static mut APP_ID: Option<String> = None;
static mut EVENT_HANDLER: Option<Box<dyn FnMut(Event) + 'static>> = None;
static mut FIRST_HWND: Option<HWND> = None;

impl PlatformEventLoop {
    pub(crate) fn new(builder: EventLoopBuilder) -> Self {
        unsafe {
            // Ensure single instance
            if let Some(app_id) = builder.app_id {
                let mutex_name = format!("bwebview-{app_id}");
                let mutex_name_c = CString::new(mutex_name).expect("Can't convert to CString");
                CreateMutexA(null_mut(), TRUE, mutex_name_c.as_ptr());
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    let hwnd = FindWindowA(mutex_name_c.as_ptr(), null_mut());
                    if !hwnd.is_null() {
                        ShowWindow(hwnd, SW_RESTORE);
                        SetForegroundWindow(hwnd);
                    }
                    exit(0);
                }
                APP_ID = Some(app_id);
            }

            // Initialize COM
            CoInitializeEx(
                null_mut(),
                COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
            );

            // Enable PerMonitorV2 high DPI awareness
            SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

            Self
        }
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
            _hdc_monitor: HDC,
            _lprc_monitor: *const RECT,
            _dw_data: LPARAM,
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
            EnumDisplayMonitors(null_mut(), null_mut(), monitor_enum_proc, 0);
            #[allow(static_mut_refs)]
            MONITORS.take().unwrap_or_default()
        }
    }

    fn run(self, event_handler: impl FnMut(Event) + 'static) -> ! {
        unsafe { EVENT_HANDLER = Some(Box::new(event_handler)) };

        // Start message loop
        unsafe {
            let mut msg = mem::zeroed();
            while GetMessageA(&mut msg, null_mut(), 0, 0) != 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
            CoUninitialize();
            exit(msg.wParam as i32);
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
            unsafe { PostMessageA(hwnd, WM_SEND_MESSAGE, ptr as WPARAM, 0) };
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
            cbSize: size_of::<MONITORINFOEXA>() as u32,
            ..Default::default()
        };
        unsafe {
            GetMonitorInfoA(hmonitor, &mut info as *mut _ as *mut _);
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
            self.info.rcMonitor.left as f32,
            self.info.rcMonitor.top as f32,
        )
    }

    fn size(&self) -> LogicalSize {
        LogicalSize::new(
            (self.info.rcMonitor.right - self.info.rcMonitor.left) as f32,
            (self.info.rcMonitor.bottom - self.info.rcMonitor.top) as f32,
        )
    }

    fn scale_factor(&self) -> f32 {
        unsafe {
            let mut dpi_x = USER_DEFAULT_SCREEN_DPI;
            let mut dpi_y = USER_DEFAULT_SCREEN_DPI;
            let result = GetDpiForMonitor(self.hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
            if result == S_OK {
                dpi_x as f32 / USER_DEFAULT_SCREEN_DPI as f32
            } else {
                1.0
            }
        }
    }

    fn is_primary(&self) -> bool {
        self.info.rcMonitor.left == 0 && self.info.rcMonitor.top == 0
    }
}

// MARK: Webview
struct WebviewData {
    hwnd: HWND,
    dpi: u32,
    min_size: Option<LogicalSize>,
    background_color: Option<u32>,
    should_load_url: Option<String>,
    should_load_html: Option<String>,
    #[cfg(feature = "custom_protocol")]
    custom_protocols: Vec<CustomProtocol>,
    #[cfg(feature = "remember_window_state")]
    remember_window_state: bool,
    environment: Option<*mut ICoreWebView2Environment>,
    controller: Option<*mut ICoreWebView2Controller>,
    webview: Option<*mut ICoreWebView2>,
}

pub(crate) struct PlatformWebview(Box<WebviewData>);

impl PlatformWebview {
    pub(crate) fn new(builder: WebviewBuilder) -> Self {
        let dpi = unsafe { GetDpiForSystem() };

        // Check if window class is already registered
        let instance = unsafe { GetModuleHandleA(null_mut()) };
        let class_name = unsafe {
            if let Some(ref app_id) = APP_ID {
                format!("bwebview-{app_id}")
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
                monitor.info.rcMonitor.clone()
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

            let title = CString::new(builder.title).expect("Can't convert to CString");
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
                let window_placement_path = format!("{}/window.bin", Self::userdata_folder());
                if let Ok(mut file) = File::open(window_placement_path) {
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

        let webview_data = Box::new(WebviewData {
            hwnd,
            dpi,
            min_size: builder.min_size,
            background_color: builder.background_color,
            should_load_url: builder.should_load_url,
            should_load_html: builder.should_load_html,
            #[cfg(feature = "custom_protocol")]
            custom_protocols: builder.custom_protocols,
            #[cfg(feature = "remember_window_state")]
            remember_window_state: builder.remember_window_state,
            environment: None,
            controller: None,
            webview: None,
        });
        unsafe {
            SetWindowLong(
                hwnd,
                GWL_USERDATA,
                webview_data.as_ref() as *const _ as isize,
            )
        };

        // Init Webview2 creation
        unsafe {
            if let Some(color) = builder.background_color {
                env::set_var(
                    "WEBVIEW2_DEFAULT_BACKGROUND_COLOR",
                    format!("0xFF{:06X}", color & 0xFFFFFF),
                );
            }

            let exectuable_path = env::current_exe().expect("Can't get current exe path");
            let executable_path = exectuable_path
                .file_name()
                .expect("Can't get current exe file name")
                .to_string_lossy();
            let executable_name = executable_path
                .rsplit_once('.')
                .expect("Should strip .exe")
                .0;
            let data_folder = dirs::config_dir()
                .expect("Can't find config dir")
                .join(executable_name)
                .display()
                .to_string();

            static VTBL: ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandlerVtbl =
                ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandlerVtbl {
                    QueryInterface: unimplemented_query_interface,
                    AddRef: unimplemented_add_ref,
                    Release: unimplemented_release,
                    Invoke: environment_created,
                };
            let completed_handler = Box::into_raw(Box::new(
                ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler {
                    lpVtbl: &VTBL,
                    user_data: webview_data.as_ref() as *const _ as *mut _,
                },
            ));
            if CreateCoreWebView2EnvironmentWithOptions(
                null(),
                data_folder.to_wide_string().as_ptr(),
                null_mut(),
                completed_handler,
            ) != S_OK
            {
                MessageBoxA(
                    null_mut(),
                    c"Failed to create WebView2 environment".as_ptr(),
                    c"Error".as_ptr(),
                    MB_OK,
                );
                exit(1);
            }
        }

        Self(webview_data)
    }

    fn userdata_folder() -> String {
        format!(
            "{}/{}",
            dirs::config_dir()
                .expect("Can't get config directory")
                .display(),
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

impl crate::WebviewInterface for PlatformWebview {
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
    }

    fn url(&self) -> Option<String> {
        unsafe {
            if let Some(webview) = self.0.webview {
                let mut uri = LPWSTR::default();
                (*webview).get_Source(uri.as_mut_ptr());
                Some(uri.to_string())
            } else {
                None
            }
        }
    }

    fn load_url(&mut self, url: impl AsRef<str>) {
        unsafe {
            if let Some(webview) = self.0.webview {
                #[cfg(feature = "custom_protocol")]
                let url = replace_custom_protocol_in_url(url.as_ref(), &self.0.custom_protocols);
                #[cfg(not(feature = "custom_protocol"))]
                let url: &str = url.as_ref();
                (*webview).Navigate(url.to_wide_string().as_ptr());
            }
        }
    }

    fn load_html(&mut self, html: impl AsRef<str>) {
        unsafe {
            if let Some(webview) = self.0.webview {
                (*webview).NavigateToString(html.as_ref().to_wide_string().as_ptr());
            }
        }
    }

    fn evaluate_script(&mut self, script: impl AsRef<str>) {
        unsafe {
            if let Some(webview) = self.0.webview {
                (*webview).ExecuteScript(script.as_ref().to_wide_string().as_ptr(), null_mut());
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
            send_event(Event::WindowMoved(LogicalPoint::new(
                (x * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                (y * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
            )));
            0
        }
        WM_SIZE => {
            let width = (l_param as u16) as i32;
            let height = ((l_param >> 16) as u16) as i32;
            send_event(Event::WindowResized(LogicalSize::new(
                (width * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
                (height * USER_DEFAULT_SCREEN_DPI as i32 / _self.dpi as i32) as f32,
            )));
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
            let event = unsafe { Box::from_raw(ptr as *mut Event) };
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
                            size_of::<WINDOWPLACEMENT>(),
                        ));
                    }
                }
            }

            send_event(Event::WindowClosed);
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

extern "system" fn unimplemented_query_interface(
    _this: *mut c_void,
    _riid: *const GUID,
    _ppv_object: *mut *mut c_void,
) -> HRESULT {
    E_NOINTERFACE
}
extern "system" fn unimplemented_add_ref(_this: *mut c_void) -> HRESULT {
    E_NOTIMPL
}
extern "system" fn unimplemented_release(_this: *mut c_void) -> HRESULT {
    E_NOTIMPL
}

extern "system" fn environment_created(
    _this: *mut ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
    _result: HRESULT,
    environment: *mut ICoreWebView2Environment,
) -> HRESULT {
    unsafe {
        let _self = &mut *((*_this).user_data as *mut WebviewData);

        (*environment).AddRef();
        _self.environment = Some(environment);

        static VTBL: ICoreWebView2CreateCoreWebView2ControllerCompletedHandlerVtbl =
            ICoreWebView2CreateCoreWebView2ControllerCompletedHandlerVtbl {
                QueryInterface: unimplemented_query_interface,
                AddRef: unimplemented_add_ref,
                Release: unimplemented_release,
                Invoke: controller_created,
            };
        let creation_completed_handler = Box::into_raw(Box::new(
            ICoreWebView2CreateCoreWebView2ControllerCompletedHandler {
                lpVtbl: &VTBL,
                user_data: (*_this).user_data,
            },
        ));
        (*environment).CreateCoreWebView2Controller(_self.hwnd, creation_completed_handler);

        S_OK
    }
}

extern "system" fn controller_created(
    _this: *mut ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
    _result: HRESULT,
    controller: *mut ICoreWebView2Controller,
) -> HRESULT {
    unsafe {
        let _self = &mut *((*_this).user_data as *mut WebviewData);
        (*controller).AddRef();
        _self.controller = Some(controller);

        // Set bounds
        let mut rect: RECT = mem::zeroed();
        GetClientRect(_self.hwnd, &mut rect);
        (*controller).put_Bounds(rect);

        // Get webview
        let mut webview: *mut ICoreWebView2 = null_mut();
        (*controller).get_CoreWebView2(&mut webview);
        _self.webview = Some(webview);

        // Set transparent background if needed
        if _self.background_color.is_some() {
            let mut controller2: *mut ICoreWebView2Controller2 = null_mut();
            (*controller).QueryInterface(
                &IID_ICoreWebView2Controller2,
                &mut controller2 as *mut _ as *mut *mut c_void,
            );
            (*controller2).put_DefaultBackgroundColor(0x00000000);
        }

        // Set user agent
        let useragent = format!(
            "Mozilla/5.0 (Windows NT; {}) bwebview/{}",
            env::consts::ARCH,
            env!("CARGO_PKG_VERSION"),
        );
        let mut settings = null_mut();
        (*webview).get_Settings(&mut settings);

        let mut settings2: *mut ICoreWebView2Settings2 = null_mut();
        (*settings).QueryInterface(
            &IID_ICoreWebView2Settings2,
            &mut settings2 as *mut _ as *mut *mut c_void,
        );
        (*settings2).put_UserAgent(useragent.to_wide_string().as_ptr());

        // Set custom protocols
        #[cfg(feature = "custom_protocol")]
        {
            for custom_protocol in &_self.custom_protocols {
                (*webview).AddWebResourceRequestedFilter(
                    format!("http://{}.localhost/*", custom_protocol.scheme)
                        .to_wide_string()
                        .as_ptr(),
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                );
            }

            static VTBL: ICoreWebView2WebResourceRequestedEventHandlerVtbl =
                ICoreWebView2WebResourceRequestedEventHandlerVtbl {
                    QueryInterface: unimplemented_query_interface,
                    AddRef: unimplemented_add_ref,
                    Release: unimplemented_release,
                    Invoke: web_resource_requested,
                };
            let web_resource_requested_handler =
                Box::into_raw(Box::new(ICoreWebView2WebResourceRequestedEventHandler {
                    lpVtbl: &VTBL,
                    user_data: (*_this).user_data,
                }));
            (*webview).add_WebResourceRequested(web_resource_requested_handler, null_mut());
        }

        // Setup event handlers
        {
            static VTBL: ICoreWebView2NavigationStartingEventHandlerVtbl =
                ICoreWebView2NavigationStartingEventHandlerVtbl {
                    QueryInterface: unimplemented_query_interface,
                    AddRef: unimplemented_add_ref,
                    Release: unimplemented_release,
                    Invoke: navigation_starting,
                };
            let navigation_starting_handler =
                Box::into_raw(Box::new(ICoreWebView2NavigationStartingEventHandler {
                    lpVtbl: &VTBL,
                }));
            (*webview).add_NavigationStarting(navigation_starting_handler, null_mut());
        }
        {
            static VTBL: ICoreWebView2NavigationCompletedEventHandlerVtbl =
                ICoreWebView2NavigationCompletedEventHandlerVtbl {
                    QueryInterface: unimplemented_query_interface,
                    AddRef: unimplemented_add_ref,
                    Release: unimplemented_release,
                    Invoke: navigation_completed,
                };
            let navigation_completed_handler =
                Box::into_raw(Box::new(ICoreWebView2NavigationCompletedEventHandler {
                    lpVtbl: &VTBL,
                }));
            (*webview).add_NavigationCompleted(navigation_completed_handler, null_mut());
        }
        {
            static VTBL: ICoreWebView2DocumentTitleChangedEventHandlerVtbl =
                ICoreWebView2DocumentTitleChangedEventHandlerVtbl {
                    QueryInterface: unimplemented_query_interface,
                    AddRef: unimplemented_add_ref,
                    Release: unimplemented_release,
                    Invoke: document_title_changed,
                };
            let document_title_changed_handler =
                Box::into_raw(Box::new(ICoreWebView2DocumentTitleChangedEventHandler {
                    lpVtbl: &VTBL,
                }));
            (*webview).add_DocumentTitleChanged(document_title_changed_handler, null_mut());
        }
        {
            static VTBL: ICoreWebView2NewWindowRequestedEventHandlerVtbl =
                ICoreWebView2NewWindowRequestedEventHandlerVtbl {
                    QueryInterface: unimplemented_query_interface,
                    AddRef: unimplemented_add_ref,
                    Release: unimplemented_release,
                    Invoke: new_window_requested,
                };
            let new_window_requested_handler =
                Box::into_raw(Box::new(ICoreWebView2NewWindowRequestedEventHandler {
                    lpVtbl: &VTBL,
                }));
            (*webview).add_NewWindowRequested(new_window_requested_handler, null_mut());
        }

        // Setup ipc and console logging
        const IPC_SCRIPT: &str = "window.ipc = new EventTarget();\
            window.ipc.postMessage = message => window.chrome.webview.postMessage('i' + (typeof message !== 'string' ? JSON.stringify(message) : message));";
        #[cfg(feature = "log")]
        const CONSOLE_SCRIPT: &str = "for (const level of ['error', 'warn', 'info', 'debug', 'trace', 'log'])\
            window.console[level] = (...args) => window.chrome.webview.postMessage('c' + level.charAt(0) + args.map(arg => typeof arg !== 'string' ? JSON.stringify(arg) : arg).join(' '));";
        #[cfg(not(feature = "log"))]
        let script = IPC_SCRIPT;
        #[cfg(feature = "log")]
        let script = format!("{IPC_SCRIPT}\n{CONSOLE_SCRIPT}");
        (*webview)
            .AddScriptToExecuteOnDocumentCreated(script.to_wide_string().as_ptr(), null_mut());

        static VTBL: ICoreWebView2WebMessageReceivedEventHandlerVtbl =
            ICoreWebView2WebMessageReceivedEventHandlerVtbl {
                QueryInterface: unimplemented_query_interface,
                AddRef: unimplemented_add_ref,
                Release: unimplemented_release,
                Invoke: web_message_received,
            };
        let message_received_handler =
            Box::into_raw(Box::new(ICoreWebView2WebMessageReceivedEventHandler {
                lpVtbl: &VTBL,
            }));
        (*webview).add_WebMessageReceived(message_received_handler, null_mut());

        // Load initial contents
        if let Some(url) = &_self.should_load_url {
            #[cfg(feature = "custom_protocol")]
            let url = replace_custom_protocol_in_url(url, &_self.custom_protocols);
            #[cfg(not(feature = "custom_protocol"))]
            let url: &str = url.as_ref();
            (*webview).Navigate(url.to_wide_string().as_ptr());
        }
        if let Some(html) = &_self.should_load_html {
            (*webview).NavigateToString(html.to_wide_string().as_ptr());
        }

        S_OK
    }
}

extern "system" fn navigation_starting(
    _this: *mut ICoreWebView2NavigationStartingEventHandler,
    _sender: *mut ICoreWebView2,
    _args: *mut c_void,
) -> HRESULT {
    send_event(Event::PageLoadStarted);
    S_OK
}

extern "system" fn navigation_completed(
    _this: *mut ICoreWebView2NavigationCompletedEventHandler,
    _sender: *mut ICoreWebView2,
    _args: *mut c_void,
) -> HRESULT {
    send_event(Event::PageLoadFinished);
    S_OK
}

extern "system" fn document_title_changed(
    _this: *mut ICoreWebView2DocumentTitleChangedEventHandler,
    _sender: *mut ICoreWebView2,
    _args: *mut c_void,
) -> HRESULT {
    unsafe {
        let mut title = LPWSTR::default();
        (*_sender).get_DocumentTitle(title.as_mut_ptr());
        send_event(Event::PageTitleChanged(title.to_string()));
    }
    S_OK
}

extern "system" fn new_window_requested(
    _this: *mut ICoreWebView2NewWindowRequestedEventHandler,
    _sender: *mut ICoreWebView2,
    args: *mut ICoreWebView2NewWindowRequestedEventArgs,
) -> HRESULT {
    unsafe {
        (*args).put_Handled(TRUE);
        let mut uri = LPWSTR::default();
        (*args).get_Uri(uri.as_mut_ptr());
        let uri = CString::new(uri.to_string()).expect("Can't convert to CString");
        ShellExecuteA(
            null_mut(),
            c"open".as_ptr(),
            uri.as_ptr(),
            null_mut(),
            null_mut(),
            SW_SHOWNORMAL,
        );
    }
    S_OK
}

extern "system" fn web_message_received(
    _this: *mut ICoreWebView2WebMessageReceivedEventHandler,
    _sender: *mut ICoreWebView2,
    args: *mut ICoreWebView2WebMessageReceivedEventArgs,
) -> HRESULT {
    let mut message = LPWSTR::default();
    unsafe { (*args).TryGetWebMessageAsString(message.as_mut_ptr()) };
    let message = message.to_string();
    let (r#type, message) = message.split_at(1);

    #[cfg(feature = "log")]
    if r#type == "c" {
        let (level, message) = message.split_at(1);
        match level {
            "e" => log::error!("{message}"),
            "w" => log::warn!("{message}"),
            "i" | "l" => log::info!("{message}"),
            "d" => log::debug!("{message}"),
            "t" => log::trace!("{message}"),
            _ => unimplemented!(),
        }
    }
    if r#type == "i" {
        send_event(Event::PageMessageReceived(message.to_string()));
    }

    S_OK
}

#[cfg(feature = "custom_protocol")]
extern "system" fn web_resource_requested(
    _this: *mut ICoreWebView2WebResourceRequestedEventHandler,
    _sender: *mut ICoreWebView2,
    args: *mut ICoreWebView2WebResourceRequestedEventArgs,
) -> HRESULT {
    let _self = unsafe { &mut *((*_this).user_data as *mut WebviewData) };

    let mut webview2_request = null_mut();
    unsafe { (*args).get_Request(&mut webview2_request) };
    let http_request = webview2_request_to_http_request(webview2_request);
    unsafe { (*webview2_request).Release() };

    for custom_protocol in &_self.custom_protocols {
        if http_request.url.host() == Some(&format!("{}.localhost", &custom_protocol.scheme)) {
            let response = (custom_protocol.handler)(&http_request);

            let webview2_response = http_response_to_webview2_response(
                response,
                _self.environment.expect("Should be some"),
            );
            unsafe { (*args).put_Response(webview2_response) };
            unsafe { (*webview2_response).Release() };

            return S_OK;
        }
    }
    panic!("No handler found for custom protocol");
}

#[cfg(feature = "custom_protocol")]
fn replace_custom_protocol_in_url(url: &str, custom_protocols: &[CustomProtocol]) -> String {
    for custom_protocol in custom_protocols {
        if url.starts_with(&format!("{}://", &custom_protocol.scheme)) {
            return url.replace(
                &format!("{}://", &custom_protocol.scheme),
                &format!("http://{}.localhost/", &custom_protocol.scheme),
            );
        }
    }
    url.to_string()
}

#[cfg(feature = "custom_protocol")]
fn webview2_request_to_http_request(
    request: *mut ICoreWebView2WebResourceRequest,
) -> small_http::Request {
    unsafe {
        use std::str::FromStr;

        let mut method = LPWSTR::default();
        (*request).get_Method(method.as_mut_ptr());
        let method = method.to_string();

        let mut uri = LPWSTR::default();
        (*request).get_Uri(uri.as_mut_ptr());
        let uri = uri.to_string();

        let mut req = small_http::Request::with_method_and_url(
            small_http::Method::from_str(&method).unwrap_or(small_http::Method::Get),
            &uri,
        );
        {
            let mut headers = null_mut();
            (*request).get_Headers(&mut headers);
            let mut iterator = null_mut();
            (*headers).GetIterator(&mut iterator);
            let mut has_current: BOOL = FALSE;
            (*iterator).get_HasCurrentHeader(&mut has_current);
            while has_current == TRUE {
                let mut name = LPWSTR::default();
                let mut value = LPWSTR::default();
                (*iterator).GetCurrentHeader(name.as_mut_ptr(), value.as_mut_ptr());
                req = req.header(name.to_string(), value.to_string());
                (*iterator).MoveNext(&mut has_current);
            }
            (*iterator).Release();
            (*headers).Release();
        }
        {
            let mut body_stream = null_mut();
            (*request).get_Content(&mut body_stream);
            if !body_stream.is_null() {
                let mut stat: STATSTG = mem::zeroed();
                (*body_stream).Stat(&mut stat as *mut _, STATFLAG_NONAME);
                let size = stat.cbSize as usize;
                let mut buffer = vec![0u8; size];
                let mut read: u32 = 0;
                (*body_stream).Read(buffer.as_mut_ptr() as *mut c_void, size as u32, &mut read);
                req = req.body(buffer);
                (*body_stream).Release();
            }
        }
        req
    }
}

#[cfg(feature = "custom_protocol")]
fn http_response_to_webview2_response(
    response: small_http::Response,
    environment: *mut ICoreWebView2Environment,
) -> *mut ICoreWebView2WebResourceResponse {
    unsafe {
        let body_stream = SHCreateMemStream(response.body.as_ptr(), response.body.len() as u32);

        let mut webview2_response = null_mut();
        (*environment).CreateWebResourceResponse(
            body_stream,
            response.status as u32,
            response.status.to_string().to_wide_string().as_ptr(),
            response
                .headers
                .iter()
                .map(|(name, value)| format!("{name}: {value}"))
                .collect::<Vec<_>>()
                .join("\n")
                .to_wide_string()
                .as_ptr(),
            &mut webview2_response,
        );
        (*body_stream).Release();
        webview2_response
    }
}
