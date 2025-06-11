/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// FIXME: Add Webview2
// FIXME: Add high dpi support
// FIXME: Add remember window position and size
// FIXME: Add forced dark theme support
// FIXME: Add 32-bit target support

#![allow(clippy::upper_case_acronyms)]

use std::ffi::{CString, c_char, c_void};
use std::mem;
use std::process::exit;
use std::ptr::null_mut;

use crate::{Event, LogicalPoint, LogicalSize, WebviewBuilder};

/// Webview
pub(crate) struct Webview {
    builder: Option<WebviewBuilder>,
    window: *mut c_void,
    min_size: Option<LogicalSize>,
}

impl Webview {
    pub(crate) fn new(builder: WebviewBuilder) -> Self {
        let min_size = builder.min_size;
        Self {
            builder: Some(builder),
            window: null_mut(),
            min_size,
        }
    }
}

impl crate::Webview for Webview {
    fn run(&mut self, _event_handler: fn(&mut Webview, Event)) -> ! {
        let builder = self.builder.take().expect("Should be some");

        // Register window class
        let instance = unsafe { GetModuleHandleA(std::ptr::null()) };
        let wndclass = WNDCLASSEXA {
            cb_size: size_of::<WNDCLASSEXA>() as u32,
            style: 0,
            lpfn_wnd_proc: wndproc,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: instance,
            h_icon: unsafe { LoadIconA(null_mut(), IDI_APPLICATION as *const c_char) },
            h_cursor: unsafe { LoadCursorA(null_mut(), IDC_ARROW as *const c_char) },
            h_brush: null_mut(),
            lpsz_menu_name: std::ptr::null(),
            lpsz_class_name: c"window".as_ptr(),
            h_icon_sm: unsafe { LoadIconA(null_mut(), IDI_APPLICATION as *const c_char) },
        };
        unsafe { RegisterClassExA(&wndclass) };

        // Create window
        self.window = unsafe {
            let title = CString::new(builder.title).expect("Can't convert to CString");
            let window = CreateWindowExA(
                0,
                wndclass.lpsz_class_name,
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                if let Some(pos) = builder.position {
                    pos.x as i32
                } else {
                    CW_USEDEFAULT as i32
                },
                if let Some(pos) = builder.position {
                    pos.y as i32
                } else {
                    CW_USEDEFAULT as i32
                },
                builder.size.width as i32,
                builder.size.height as i32,
                null_mut(),
                null_mut(),
                instance,
                null_mut(),
            );
            SetWindowLongPtrA(window, GWL_USERDATA, self as *mut Webview as isize);

            // Center the window on the screen
            if builder.position.is_none() || builder.should_center {
                let mut rect = RECT::default();
                GetWindowRect(window, &mut rect);
                let screen_width = GetSystemMetrics(SM_CXSCREEN);
                let screen_height = GetSystemMetrics(SM_CYSCREEN);
                let x = (screen_width - (rect.right - rect.left)) / 2;
                let y = (screen_height - (rect.bottom - rect.top)) / 2;
                SetWindowPos(
                    window,
                    null_mut(),
                    x,
                    y,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }

            ShowWindow(window, SW_SHOWDEFAULT);
            UpdateWindow(window);
            window
        };

        // Start event loop
        unsafe {
            let mut msg: MSG = mem::zeroed();
            while GetMessageA(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
            exit(msg.w_param as i32);
        }
    }

    fn set_title(&mut self, title: impl AsRef<str>) {
        let title = CString::new(title.as_ref()).expect("Can't convert to CString");
        unsafe { SetWindowTextA(self.window, title.as_ptr()) };
    }

    fn position(&self) -> LogicalPoint {
        let mut rect = RECT::default();
        unsafe { GetWindowRect(self.window, &mut rect) };
        LogicalPoint::new(rect.left as f32, rect.top as f32)
    }

    fn size(&self) -> LogicalSize {
        let mut rect = RECT::default();
        unsafe { GetWindowRect(self.window, &mut rect) };
        LogicalSize::new(
            (rect.right - rect.left) as f32,
            (rect.bottom - rect.top) as f32,
        )
    }

    fn set_position(&mut self, point: LogicalPoint) {
        unsafe {
            SetWindowPos(
                self.window,
                null_mut(),
                point.x as i32,
                point.y as i32,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
        };
    }

    fn set_size(&mut self, _size: LogicalSize) {
        unsafe {
            SetWindowPos(
                self.window,
                null_mut(),
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
            let style = GetWindowLongPtrA(self.window, GWL_STYLE) as u32;
            SetWindowLongPtrA(
                self.window,
                GWL_STYLE,
                if resizable {
                    style & !WS_THICKFRAME
                } else {
                    style | WS_THICKFRAME
                } as isize,
            );
        }
    }

    fn load_url(&mut self, _url: impl AsRef<str>) {}

    fn load_html(&mut self, _html: impl AsRef<str>) {}

    fn evaluate_script(&mut self, _script: impl AsRef<str>) {}

    #[cfg(feature = "ipc")]
    fn send_ipc_message(&mut self, _message: impl AsRef<str>) {}
}

extern "C" fn wndproc(hwnd: *mut c_void, msg: u32, w_param: usize, l_param: isize) -> isize {
    let _self = unsafe {
        let ptr = GetWindowLongPtrA(hwnd, GWL_USERDATA) as *mut Webview;
        if ptr.is_null() {
            return DefWindowProcA(hwnd, msg, w_param, l_param);
        }
        &mut *ptr
    };

    match msg {
        WM_GETMINMAXINFO => {
            unsafe {
                if let Some(min_size) = _self.min_size {
                    let minmax_info = l_param as *mut MINMAXINFO;
                    (*minmax_info).pt_min_track_size.x = min_size.width as i32;
                    (*minmax_info).pt_min_track_size.y = min_size.height as i32;
                }
            }
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcA(hwnd, msg, w_param, l_param) },
    }
}

// MARK: Win32 headers
const CW_USEDEFAULT: u32 = 0x80000000;
const GWL_STYLE: i32 = -16;
const GWL_USERDATA: i32 = -21;
const IDC_ARROW: u16 = 32512;
const IDI_APPLICATION: u16 = 32512;
const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;
const SWP_NOACTIVATE: u32 = 0x0010;
const SWP_NOREPOSITION: u32 = 0x0200;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOZORDER: u32 = 0x0004;
const SW_SHOWDEFAULT: i32 = 10;
const WM_DESTROY: u32 = 0x0002;
const WM_GETMINMAXINFO: u32 = 0x0024;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const WS_THICKFRAME: u32 = 0x00040000;

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Default)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct MINMAXINFO {
    pt_reserved: POINT,
    pt_max_size: POINT,
    pt_max_position: POINT,
    pt_min_track_size: POINT,
    pt_max_track_size: POINT,
}

#[repr(C)]
struct MSG {
    hwnd: *mut c_void,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt: POINT,
}

#[repr(C)]
struct WNDCLASSEXA {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: extern "C" fn(*mut c_void, u32, usize, isize) -> isize,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: *mut c_void,
    h_icon: *mut c_void,
    h_cursor: *mut c_void,
    h_brush: *mut c_void,
    lpsz_menu_name: *const c_char,
    lpsz_class_name: *const c_char,
    h_icon_sm: *mut c_void,
}

#[link(name = "kernel32")]
unsafe extern "C" {
    fn GetModuleHandleA(lp_module_name: *const c_char) -> *mut c_void;
}

#[link(name = "user32")]
unsafe extern "C" {
    fn CreateWindowExA(
        dw_ex_style: u32,
        lp_class_name: *const c_char,
        lp_window_name: *const c_char,
        dw_style: u32,
        x: i32,
        y: i32,
        n_width: i32,
        n_height: i32,
        h_wnd_parent: *mut c_void,
        h_menu: *mut c_void,
        h_instance: *mut c_void,
        lp_param: *mut c_void,
    ) -> *mut c_void;
    fn DefWindowProcA(h_wnd: *mut c_void, msg: u32, w_param: usize, l_param: isize) -> isize;
    fn DispatchMessageA(lp_msg: *const MSG) -> i32;
    fn GetMessageA(
        lp_msg: *mut MSG,
        h_wnd: *mut c_void,
        w_msg_filter_min: u32,
        w_msg_filter_max: u32,
    ) -> i32;
    fn GetSystemMetrics(n_index: i32) -> i32;
    fn GetWindowLongPtrA(h_wnd: *mut c_void, n_index: i32) -> isize;
    fn GetWindowRect(h_wnd: *mut c_void, lp_rect: *mut RECT) -> i32;
    fn LoadCursorA(h_instance: *mut c_void, lp_cursor_name: *const c_char) -> *mut c_void;
    fn LoadIconA(h_instance: *mut c_void, lp_icon_name: *const c_char) -> *mut c_void;
    fn PostQuitMessage(code: i32);
    fn RegisterClassExA(lpwcx: *const WNDCLASSEXA) -> u16;
    fn SetWindowLongPtrA(h_wnd: *mut c_void, n_index: i32, dw_new_long: isize) -> isize;
    fn SetWindowPos(
        h_wnd: *mut c_void,
        h_wnd_insert_after: *mut c_void,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        u_flags: u32,
    ) -> i32;
    fn SetWindowTextA(h_wnd: *mut c_void, lp_string: *const c_char) -> i32;
    fn ShowWindow(h_wnd: *mut c_void, n_cmd_show: i32) -> i32;
    fn TranslateMessage(lp_msg: *const MSG) -> i32;
    fn UpdateWindow(h_wnd: *mut c_void) -> i32;
}
