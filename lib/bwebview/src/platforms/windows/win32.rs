/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::upper_case_acronyms)]
#![allow(unused)]

use std::ffi::{c_char, c_void};

// MARK: Base types
pub(crate) type BOOL = i32;
pub(crate) const TRUE: BOOL = 1;
pub(crate) const FALSE: BOOL = 0;
pub(crate) type w_char = u16;
pub(crate) type HANDLE = *mut c_void;

#[repr(C)]
pub(crate) struct FILETIME {
    pub(crate) dwLowDateTime: u32,
    pub(crate) dwHighDateTime: u32,
}

// MARK: kernel32.dll
pub(crate) type HMODULE = *const c_void;

pub(crate) const ERROR_ALREADY_EXISTS: u32 = 183;

#[link(name = "kernel32")]
unsafe extern "system" {
    pub(crate) fn GetModuleHandleA(lpModuleName: *const u8) -> HMODULE;
    pub(crate) fn GetLastError() -> u32;
    pub(crate) fn CreateMutexA(
        lpMutexAttributes: *mut c_void,
        bInitialOwner: BOOL,
        lpName: *const c_char,
    ) -> HANDLE;
}

// MARK: gdi32.dll
pub(crate) type HBRUSH = HANDLE;
pub(crate) type HDC = HANDLE;

#[link(name = "gdi32")]
unsafe extern "system" {
    pub(crate) fn FillRect(hdc: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    pub(crate) fn CreateSolidBrush(color: u32) -> HBRUSH;
    pub(crate) fn DeleteObject(ho: *mut c_void) -> BOOL;
}

// MARK: user32.dll
pub(crate) type HWND = HANDLE;
pub(crate) type HCURSOR = HANDLE;
pub(crate) type HICON = HANDLE;
pub(crate) type HMENU = HANDLE;
pub(crate) type HMONITOR = HANDLE;
pub(crate) type ATOM = u16;
pub(crate) type WPARAM = usize;
pub(crate) type LPARAM = isize;
pub(crate) type LRESULT = isize;

#[repr(C)]
#[derive(Default)]
pub(crate) struct WNDCLASSEXA {
    pub(crate) cbSize: u32,
    pub(crate) style: u32,
    pub(crate) lpfnWndProc: Option<
        unsafe extern "system" fn(
            window: HWND,
            message: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT,
    >,
    pub(crate) cbClsExtra: i32,
    pub(crate) cbWndExtra: i32,
    pub(crate) hInstance: HMODULE,
    pub(crate) hIcon: HICON,
    pub(crate) hCursor: HCURSOR,
    pub(crate) hbrBackground: usize,
    pub(crate) lpszMenuName: *const c_char,
    pub(crate) lpszClassName: *const c_char,
    pub(crate) hIconSm: HICON,
}

#[repr(C)]
pub(crate) struct MSG {
    pub(crate) hwnd: HWND,
    pub(crate) message: u32,
    pub(crate) wParam: WPARAM,
    pub(crate) lParam: LPARAM,
    pub(crate) time: u32,
    pub(crate) pt_x: i32,
    pub(crate) pt_y: i32,
}

#[repr(C)]
pub(crate) struct POINT {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[repr(C)]
#[derive(Default, Clone)]
pub(crate) struct RECT {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

#[repr(C)]
pub(crate) struct MINMAXINFO {
    pub(crate) ptReserved: POINT,
    pub(crate) ptMaxSize: POINT,
    pub(crate) ptMaxPosition: POINT,
    pub(crate) ptMinTrackSize: POINT,
    pub(crate) ptMaxTrackSize: POINT,
}

#[repr(C)]
#[derive(Default)]
pub(crate) struct MONITORINFOEXA {
    pub(crate) cbSize: u32,
    pub(crate) rcMonitor: RECT,
    pub(crate) rcWork: RECT,
    pub(crate) dwFlags: u32,
    pub(crate) szDevice: [c_char; 32],
}

#[repr(C)]
pub(crate) struct WINDOWPLACEMENT {
    pub(crate) length: u32,
    pub(crate) flags: u32,
    pub(crate) showCmd: u32,
    pub(crate) ptMinPosition: POINT,
    pub(crate) ptMaxPosition: POINT,
    pub(crate) rcNormalPosition: RECT,
}

pub(crate) const WS_POPUP: u32 = 0x80000000;
pub(crate) const WS_THICKFRAME: u32 = 0x00040000;
pub(crate) const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
pub(crate) const WS_MAXIMIZEBOX: u32 = 0x00010000;

pub(crate) const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;

pub(crate) const WM_CREATE: u32 = 0x0001;
pub(crate) const WM_DESTROY: u32 = 0x0002;
pub(crate) const WM_MOVE: u32 = 0x0003;
pub(crate) const WM_SIZE: u32 = 0x0005;
pub(crate) const WM_CLOSE: u32 = 0x0010;
pub(crate) const WM_ERASEBKGND: u32 = 0x0014;
pub(crate) const WM_GETMINMAXINFO: u32 = 0x0024;
pub(crate) const WM_DPICHANGED: u32 = 0x02E0;
pub(crate) const WM_USER: u32 = 0x0400;

pub(crate) const SW_SHOWNORMAL: i32 = 1;
pub(crate) const SW_RESTORE: i32 = 9;
pub(crate) const SW_SHOWDEFAULT: i32 = 10;

pub(crate) const MB_OK: u32 = 0x00000000;

pub(crate) const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: isize = -4isize;

pub(crate) const MONITOR_DEFAULTTOPRIMARY: u32 = 0x00000001;

pub(crate) const USER_DEFAULT_SCREEN_DPI: u32 = 96;
pub(crate) const MDT_EFFECTIVE_DPI: i32 = 0;

pub(crate) const SM_CXSCREEN: i32 = 0;
pub(crate) const SM_CYSCREEN: i32 = 1;

pub(crate) const GWL_STYLE: i32 = -16;
pub(crate) const GWL_USERDATA: i32 = -21;

pub(crate) const SWP_NOSIZE: u32 = 0x0001;
pub(crate) const SWP_NOZORDER: u32 = 0x0004;
pub(crate) const SWP_NOACTIVATE: u32 = 0x0010;
pub(crate) const SWP_NOREPOSITION: u32 = 0x0200;

#[link(name = "user32")]
unsafe extern "system" {
    pub(crate) fn ExtractIconExA(
        lpszFile: *const c_char,
        nIconIndex: i32,
        phiconLarge: *mut HICON,
        phiconSmall: *mut HICON,
        nIcons: u32,
    ) -> u32;
    pub(crate) fn GetClassInfoExA(
        hInstance: HMODULE,
        lpClassName: *const c_char,
        lpWndClass: *mut WNDCLASSEXA,
    ) -> BOOL;
    pub(crate) fn RegisterClassExA(lpWndClass: *const WNDCLASSEXA) -> ATOM;
    pub(crate) fn CreateWindowExA(
        dwExStyle: u32,
        lpClassName: *const c_char,
        lpWindowName: *const c_char,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HMODULE,
        lpParam: LPARAM,
    ) -> HWND;
    pub(crate) fn SetWindowTextA(hWnd: HWND, lpString: *const c_char) -> i32;
    pub(crate) fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> i32;
    pub(crate) fn UpdateWindow(hWnd: HWND) -> i32;
    pub(crate) fn GetWindowRect(hWnd: HWND, rect: *mut RECT) -> i32;
    pub(crate) fn GetClientRect(hWnd: HWND, rect: *mut RECT) -> i32;
    pub(crate) fn SetWindowPos(
        hWnd: HWND,
        hWndInsertAfter: HWND,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> BOOL;
    pub(crate) fn GetMessageA(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: u32,
        wMsgFilterMax: u32,
    ) -> i32;
    pub(crate) fn TranslateMessage(lpMsg: *const MSG) -> i32;
    pub(crate) fn DispatchMessageA(lpMsg: *const MSG) -> isize;
    pub(crate) fn DefWindowProcA(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;
    pub(crate) fn PostQuitMessage(nExitCode: i32);
    pub(crate) fn SetProcessDpiAwarenessContext(value: isize) -> i32;
    pub(crate) fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
    pub(crate) fn MessageBoxA(
        hWnd: HWND,
        lpText: *const c_char,
        lpCaption: *const c_char,
        uType: u32,
    ) -> i32;
    pub(crate) fn GetWindowLongA(hwnd: HWND, index: i32) -> i32;
    pub(crate) fn GetWindowLongPtrA(hwnd: HWND, index: i32) -> isize;
    pub(crate) fn SetWindowLongA(hwnd: HWND, index: i32, value: i32) -> i32;
    pub(crate) fn SetWindowLongPtrA(hwnd: HWND, index: i32, value: isize) -> isize;
    pub(crate) fn MonitorFromPoint(pt: POINT, dwFlags: u32) -> *mut c_void;
    pub(crate) fn EnumDisplayMonitors(
        hdc: HDC,
        lprcClip: *const RECT,
        lpfnEnum: unsafe extern "system" fn(
            hMonitor: HMONITOR,
            hdcMonitor: HDC,
            lprcMonitor: *const RECT,
            dwData: LPARAM,
        ) -> BOOL,
        dwData: LPARAM,
    ) -> BOOL;
    pub(crate) fn PostMessageA(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    pub(crate) fn GetMonitorInfoA(hMonitor: HMONITOR, lpmi: *mut MONITORINFOEXA) -> BOOL;
    pub(crate) fn GetDpiForSystem() -> u32;
    pub(crate) fn GetSystemMetrics(nIndex: i32) -> i32;
    pub(crate) fn AdjustWindowRectExForDpi(
        lpRect: *mut RECT,
        dwStyle: u32,
        bMenu: BOOL,
        dwExStyle: u32,
        dpi: u32,
    ) -> BOOL;
    pub(crate) fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub(crate) fn GetWindowPlacement(hWnd: HWND, lpwndpl: *mut WINDOWPLACEMENT) -> BOOL;
    pub(crate) fn SetWindowPlacement(hWnd: HWND, lpwndpl: *const WINDOWPLACEMENT) -> BOOL;
    pub(crate) fn FindWindowA(lpClassName: *const c_char, lpWindowName: *const c_char) -> HWND;
    pub(crate) fn SetForegroundWindow(hWnd: HWND) -> BOOL;
}

#[cfg(target_pointer_width = "32")]
pub(crate) unsafe fn GetWindowLong(hwnd: HWND, index: i32) -> isize {
    (unsafe { GetWindowLongA(hwnd, index) }) as isize
}
#[cfg(target_pointer_width = "64")]
pub(crate) unsafe fn GetWindowLong(hwnd: HWND, index: i32) -> isize {
    unsafe { GetWindowLongPtrA(hwnd, index) }
}

#[cfg(target_pointer_width = "32")]
pub(crate) unsafe fn SetWindowLong(hwnd: HWND, index: i32, value: isize) -> isize {
    (unsafe { SetWindowLongA(hwnd, index, value as i32) }) as isize
}
#[cfg(target_pointer_width = "64")]
pub(crate) unsafe fn SetWindowLong(hwnd: HWND, index: i32, value: isize) -> isize {
    unsafe { SetWindowLongPtrA(hwnd, index, value) }
}

// MARK: shell32.dll
#[link(name = "shell32")]
unsafe extern "system" {
    pub(crate) fn ShellExecuteA(
        hwnd: HWND,
        lpOperation: *const c_char,
        lpFile: *const c_char,
        lpParameters: *const c_char,
        lpDirectory: *const c_char,
        nShowCmd: i32,
    ) -> isize;
}

// MARK: shcore.dll
#[link(name = "shcore")]
unsafe extern "system" {
    pub(crate) fn GetDpiForMonitor(
        hmonitor: HMONITOR,
        dpiType: i32,
        dpiX: *mut u32,
        dpiY: *mut u32,
    ) -> HRESULT;
}

// MARK: dwmapi.dll
pub(crate) const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;

#[link(name = "dwmapi")]
unsafe extern "system" {
    pub(crate) fn DwmSetWindowAttribute(
        hwnd: HWND,
        dwAttribute: u32,
        pvAttribute: *const c_void,
        cbAttribute: u32,
    ) -> i32;
}

// MARK: ole32.dll
pub(crate) type HRESULT = i32;

#[repr(C)]
pub(crate) struct GUID {
    pub(crate) data1: u32,
    pub(crate) data2: u16,
    pub(crate) data3: u16,
    pub(crate) data4: [u8; 8],
}

#[repr(C)]
pub(crate) struct STATSTG {
    pub(crate) pwcsName: *mut u16,
    pub(crate) type_: u32,
    pub(crate) cbSize: u64,
    pub(crate) mtime: FILETIME,
    pub(crate) ctime: FILETIME,
    pub(crate) atime: FILETIME,
    pub(crate) grfMode: u32,
    pub(crate) grfLocksSupported: u32,
    pub(crate) clsid: GUID,
    pub(crate) grfStateBits: u32,
    pub(crate) reserved: u32,
}

pub(crate) const COINIT_APARTMENTTHREADED: u32 = 0x2;
pub(crate) const COINIT_DISABLE_OLE1DDE: u32 = 0x4;

pub(crate) const S_OK: HRESULT = 0;
pub(crate) const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT;
pub(crate) const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT;

#[link(name = "ole32")]
unsafe extern "system" {
    pub(crate) fn CoInitializeEx(pvReserved: *mut c_void, dwCoInit: u32) -> HRESULT;
    pub(crate) fn CoUninitialize();
    pub(crate) fn CoTaskMemFree(pv: *mut c_void);
}

// MARK: IStream
pub(crate) const STATFLAG_NONAME: u32 = 1;

#[repr(C)]
pub(crate) struct IStream {
    pub(crate) lpVtbl: *const IStream_Vtbl,
}

impl IStream {
    pub(crate) unsafe fn Release(&self) -> HRESULT {
        unsafe { ((*self.lpVtbl).Release)(self as *const _ as *mut _) }
    }

    pub(crate) unsafe fn Read(&self, pv: *mut c_void, cb: u32, pcbRead: *mut u32) -> HRESULT {
        unsafe { ((*self.lpVtbl).Read)(self as *const _ as *mut _, pv, cb, pcbRead) }
    }

    pub(crate) unsafe fn Stat(&self, pstatstg: *mut STATSTG, grfStatFlag: u32) -> HRESULT {
        unsafe { ((*self.lpVtbl).Stat)(self as *const _ as *mut _, pstatstg, grfStatFlag) }
    }
}

#[repr(C)]
pub(crate) struct IStream_Vtbl {
    pub(crate) QueryInterface: unsafe extern "system" fn(
        This: *mut IStream,
        riid: *const GUID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    pub(crate) AddRef: unsafe extern "system" fn(This: *mut IStream) -> HRESULT,
    pub(crate) Release: unsafe extern "system" fn(This: *mut IStream) -> HRESULT,
    pub(crate) Read: unsafe extern "system" fn(
        This: *mut IStream,
        pv: *mut c_void,
        cb: u32,
        pcbRead: *mut u32,
    ) -> HRESULT,
    padding1: [usize; 8],
    pub(crate) Stat: unsafe extern "system" fn(
        This: *mut IStream,
        pstatstg: *mut STATSTG,
        grfStatFlag: u32,
    ) -> HRESULT,
}

// shlwapi.dll
#[link(name = "shlwapi")]
unsafe extern "system" {
    pub(crate) fn SHCreateMemStream(pInit: *const u8, cbInit: u32) -> *mut IStream;
}

// MARK: Utils
pub(crate) trait ToWideString {
    fn to_wide_string(&self) -> Vec<u16>;
}
impl ToWideString for &str {
    fn to_wide_string(&self) -> Vec<u16> {
        let mut v: Vec<u16> = self.encode_utf16().collect();
        v.push(0);
        v
    }
}
impl ToWideString for String {
    fn to_wide_string(&self) -> Vec<u16> {
        let mut v: Vec<u16> = self.encode_utf16().collect();
        v.push(0);
        v
    }
}

pub(crate) struct LPWSTR(*mut w_char);
impl Default for LPWSTR {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}
impl LPWSTR {
    pub(crate) fn as_mut_ptr(&mut self) -> *mut *mut w_char {
        &mut self.0
    }
}
impl std::fmt::Display for LPWSTR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_null() {
            return Ok(());
        }
        let mut len = 0;
        unsafe {
            while *self.0.add(len) != 0 {
                len += 1;
            }
        }
        let str = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(self.0, len) });
        write!(f, "{str}")
    }
}
impl Drop for LPWSTR {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CoTaskMemFree(self.0 as *mut c_void) };
        }
    }
}

// MARK: Common Item Dialog
pub(crate) const FOS_OVERWRITEPROMPT: u32 = 0x00000002;
pub(crate) const FOS_NOCHANGEDIR: u32 = 0x00000008;
pub(crate) const FOS_ALLOWMULTISELECT: u32 = 0x00000200;
pub(crate) const FOS_PATHMUSTEXIST: u32 = 0x00000800;
pub(crate) const FOS_FILEMUSTEXIST: u32 = 0x00001000;
pub(crate) const SIGDN_FILESYSPATH: u32 = 0x80058000;
pub(crate) const CLSCTX_INPROC_SERVER: u32 = 1;

pub(crate) const CLSID_FileOpenDialog: GUID = GUID {
    data1: 0xDC1C5A9C,
    data2: 0xE88A,
    data3: 0x4dde,
    data4: [0xA5, 0xA1, 0x60, 0xF8, 0x2A, 0x20, 0xAE, 0xF7],
};
pub(crate) const CLSID_FileSaveDialog: GUID = GUID {
    data1: 0xC0B4E2F3,
    data2: 0xBA21,
    data3: 0x4773,
    data4: [0x8D, 0xBA, 0x33, 0x5E, 0xC9, 0x46, 0xEB, 0x8B],
};
pub(crate) const IID_IFileOpenDialog: GUID = GUID {
    data1: 0xD57C7288,
    data2: 0xD4AD,
    data3: 0x4768,
    data4: [0xBE, 0x02, 0x9D, 0x96, 0x95, 0x32, 0xD9, 0x60],
};
pub(crate) const IID_IFileSaveDialog: GUID = GUID {
    data1: 0x84BCCD23,
    data2: 0x5FDE,
    data3: 0x4CDB,
    data4: [0xAE, 0xA4, 0xAF, 0x64, 0xB8, 0x3D, 0x78, 0xAB],
};
pub(crate) const IID_IShellItem: GUID = GUID {
    data1: 0x43826D1E,
    data2: 0xE718,
    data3: 0x42EE,
    data4: [0xBC, 0x55, 0xA1, 0xE2, 0x61, 0xC3, 0x7B, 0xFE],
};

#[repr(C)]
pub(crate) struct COMDLG_FILTERSPEC {
    pub(crate) pszName: *const u16,
    pub(crate) pszSpec: *const u16,
}

// IShellItem
#[repr(C)]
pub(crate) struct IShellItem {
    pub(crate) vtbl: *const IShellItemVtbl,
}

impl IShellItem {
    pub(crate) unsafe fn Release(&self) -> u32 {
        unsafe { ((*self.vtbl).Release)(self as *const _ as *mut _) }
    }

    pub(crate) unsafe fn GetDisplayName(&self, sigdn: u32, ppszName: *mut *mut u16) -> HRESULT {
        unsafe { ((*self.vtbl).GetDisplayName)(self as *const _ as *mut _, sigdn, ppszName) }
    }
}

#[repr(C)]
pub(crate) struct IShellItemVtbl {
    _unk: [usize; 2], // QueryInterface, AddRef
    pub(crate) Release: unsafe extern "system" fn(*mut c_void) -> u32,
    _bind_parent: [usize; 2], // BindToHandler, GetParent
    pub(crate) GetDisplayName:
        unsafe extern "system" fn(*mut c_void, u32, *mut *mut u16) -> HRESULT,
    // GetAttributes, Compare
}

// IShellItemArray
#[repr(C)]
pub(crate) struct IShellItemArray {
    pub(crate) vtbl: *const IShellItemArrayVtbl,
}

impl IShellItemArray {
    pub(crate) unsafe fn Release(&self) -> u32 {
        unsafe { ((*self.vtbl).Release)(self as *const _ as *mut _) }
    }

    pub(crate) unsafe fn GetCount(&self, pdwNumItems: *mut u32) -> HRESULT {
        unsafe { ((*self.vtbl).GetCount)(self as *const _ as *mut _, pdwNumItems) }
    }

    pub(crate) unsafe fn GetItemAt(&self, dwIndex: u32, ppsi: *mut *mut IShellItem) -> HRESULT {
        unsafe { ((*self.vtbl).GetItemAt)(self as *const _ as *mut _, dwIndex, ppsi) }
    }
}

#[repr(C)]
pub(crate) struct IShellItemArrayVtbl {
    _unk: [usize; 2], // QueryInterface, AddRef
    pub(crate) Release: unsafe extern "system" fn(*mut c_void) -> u32,
    _mid: [usize; 4], // BindToHandler, GetPropertyStore, GetPropertyDescriptionList, GetAttributes
    pub(crate) GetCount: unsafe extern "system" fn(*mut c_void, *mut u32) -> HRESULT,
    pub(crate) GetItemAt:
        unsafe extern "system" fn(*mut c_void, u32, *mut *mut IShellItem) -> HRESULT,
    // EnumItems
}

// IFileOpenDialog
#[repr(C)]
pub(crate) struct IFileOpenDialog {
    pub(crate) vtbl: *const IFileOpenDialogVtbl,
}

impl IFileOpenDialog {
    pub(crate) unsafe fn Release(&self) -> u32 {
        unsafe { ((*self.vtbl).Release)(self as *const _ as *mut _) }
    }

    pub(crate) unsafe fn Show(&self, hwnd: HWND) -> HRESULT {
        unsafe { ((*self.vtbl).Show)(self as *const _ as *mut _, hwnd) }
    }

    pub(crate) unsafe fn SetFileTypes(
        &self,
        cFileTypes: u32,
        rgFilterSpec: *const COMDLG_FILTERSPEC,
    ) -> HRESULT {
        unsafe { ((*self.vtbl).SetFileTypes)(self as *const _ as *mut _, cFileTypes, rgFilterSpec) }
    }

    pub(crate) unsafe fn SetOptions(&self, fos: u32) -> HRESULT {
        unsafe { ((*self.vtbl).SetOptions)(self as *const _ as *mut _, fos) }
    }

    pub(crate) unsafe fn SetFolder(&self, psi: *mut IShellItem) -> HRESULT {
        unsafe { ((*self.vtbl).SetFolder)(self as *const _ as *mut _, psi as *mut c_void) }
    }

    pub(crate) unsafe fn SetTitle(&self, pszTitle: *const u16) -> HRESULT {
        unsafe { ((*self.vtbl).SetTitle)(self as *const _ as *mut _, pszTitle) }
    }

    pub(crate) unsafe fn GetResult(&self, ppsi: *mut *mut IShellItem) -> HRESULT {
        unsafe { ((*self.vtbl).GetResult)(self as *const _ as *mut _, ppsi) }
    }

    pub(crate) unsafe fn GetResults(&self, ppsia: *mut *mut IShellItemArray) -> HRESULT {
        unsafe { ((*self.vtbl).GetResults)(self as *const _ as *mut _, ppsia) }
    }
}

#[repr(C)]
pub(crate) struct IFileOpenDialogVtbl {
    _unk: [usize; 2], // QueryInterface, AddRef
    pub(crate) Release: unsafe extern "system" fn(*mut c_void) -> u32,
    // IModalWindow
    pub(crate) Show: unsafe extern "system" fn(*mut c_void, HWND) -> HRESULT,
    // IFileDialog
    pub(crate) SetFileTypes:
        unsafe extern "system" fn(*mut c_void, u32, *const COMDLG_FILTERSPEC) -> HRESULT,
    _type_idx: [usize; 4], // SetFileTypeIndex, GetFileTypeIndex, Advise, Unadvise
    pub(crate) SetOptions: unsafe extern "system" fn(*mut c_void, u32) -> HRESULT,
    _get_opts_def: [usize; 2], // GetOptions, SetDefaultFolder
    pub(crate) SetFolder: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
    _get_folder_sel: [usize; 2], // GetFolder, GetCurrentSelection
    _set_filename: usize,        // SetFileName
    _get_filename: usize,        // GetFileName
    pub(crate) SetTitle: unsafe extern "system" fn(*mut c_void, *const u16) -> HRESULT,
    _labels: [usize; 2], // SetOkButtonLabel, SetFileNameLabel
    pub(crate) GetResult: unsafe extern "system" fn(*mut c_void, *mut *mut IShellItem) -> HRESULT,
    _add_place: usize,      // AddPlace
    _def_ext: usize,        // SetDefaultExtension
    _close_etc: [usize; 4], // Close, SetClientGuid, ClearClientData, SetFilter
    // IFileOpenDialog
    pub(crate) GetResults:
        unsafe extern "system" fn(*mut c_void, *mut *mut IShellItemArray) -> HRESULT,
    // GetSelectedItems
}

// IFileSaveDialog
#[repr(C)]
pub(crate) struct IFileSaveDialog {
    pub(crate) vtbl: *const IFileSaveDialogVtbl,
}

impl IFileSaveDialog {
    pub(crate) unsafe fn Release(&self) -> u32 {
        unsafe { ((*self.vtbl).Release)(self as *const _ as *mut _) }
    }

    pub(crate) unsafe fn Show(&self, hwnd: HWND) -> HRESULT {
        unsafe { ((*self.vtbl).Show)(self as *const _ as *mut _, hwnd) }
    }

    pub(crate) unsafe fn SetFileTypes(
        &self,
        cFileTypes: u32,
        rgFilterSpec: *const COMDLG_FILTERSPEC,
    ) -> HRESULT {
        unsafe { ((*self.vtbl).SetFileTypes)(self as *const _ as *mut _, cFileTypes, rgFilterSpec) }
    }

    pub(crate) unsafe fn SetOptions(&self, fos: u32) -> HRESULT {
        unsafe { ((*self.vtbl).SetOptions)(self as *const _ as *mut _, fos) }
    }

    pub(crate) unsafe fn SetFolder(&self, psi: *mut IShellItem) -> HRESULT {
        unsafe { ((*self.vtbl).SetFolder)(self as *const _ as *mut _, psi as *mut c_void) }
    }

    pub(crate) unsafe fn SetFileName(&self, pszName: *const u16) -> HRESULT {
        unsafe { ((*self.vtbl).SetFileName)(self as *const _ as *mut _, pszName) }
    }

    pub(crate) unsafe fn SetTitle(&self, pszTitle: *const u16) -> HRESULT {
        unsafe { ((*self.vtbl).SetTitle)(self as *const _ as *mut _, pszTitle) }
    }

    pub(crate) unsafe fn SetDefaultExtension(&self, pszDefaultExtension: *const u16) -> HRESULT {
        unsafe {
            ((*self.vtbl).SetDefaultExtension)(self as *const _ as *mut _, pszDefaultExtension)
        }
    }

    pub(crate) unsafe fn GetResult(&self, ppsi: *mut *mut IShellItem) -> HRESULT {
        unsafe { ((*self.vtbl).GetResult)(self as *const _ as *mut _, ppsi) }
    }
}

#[repr(C)]
pub(crate) struct IFileSaveDialogVtbl {
    _unk: [usize; 2], // QueryInterface, AddRef
    pub(crate) Release: unsafe extern "system" fn(*mut c_void) -> u32,
    // IModalWindow
    pub(crate) Show: unsafe extern "system" fn(*mut c_void, HWND) -> HRESULT,
    // IFileDialog
    pub(crate) SetFileTypes:
        unsafe extern "system" fn(*mut c_void, u32, *const COMDLG_FILTERSPEC) -> HRESULT,
    _type_idx: [usize; 4], // SetFileTypeIndex, GetFileTypeIndex, Advise, Unadvise
    pub(crate) SetOptions: unsafe extern "system" fn(*mut c_void, u32) -> HRESULT,
    _get_opts_def: [usize; 2], // GetOptions, SetDefaultFolder
    pub(crate) SetFolder: unsafe extern "system" fn(*mut c_void, *mut c_void) -> HRESULT,
    _get_folder_sel: [usize; 2], // GetFolder, GetCurrentSelection
    pub(crate) SetFileName: unsafe extern "system" fn(*mut c_void, *const u16) -> HRESULT,
    _get_filename: usize, // GetFileName
    pub(crate) SetTitle: unsafe extern "system" fn(*mut c_void, *const u16) -> HRESULT,
    _labels: [usize; 2], // SetOkButtonLabel, SetFileNameLabel
    pub(crate) GetResult: unsafe extern "system" fn(*mut c_void, *mut *mut IShellItem) -> HRESULT,
    _add_place: usize, // AddPlace
    pub(crate) SetDefaultExtension: unsafe extern "system" fn(*mut c_void, *const u16) -> HRESULT,
    // Close, SetClientGuid, ClearClientData, SetFilter, then IFileSaveDialog methods
}

#[link(name = "ole32")]
unsafe extern "system" {
    pub(crate) fn CoCreateInstance(
        rclsid: *const GUID,
        pUnkOuter: *mut c_void,
        dwClsContext: u32,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
}

#[link(name = "shell32")]
unsafe extern "system" {
    pub(crate) fn SHCreateItemFromParsingName(
        pszPath: *const u16,
        pbc: *mut c_void,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
}
