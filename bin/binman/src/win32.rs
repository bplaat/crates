/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unsafe_code)]

use std::ffi::c_void;

pub(super) const TOKEN_QUERY: u32 = 0x0008;
pub(super) const TOKEN_ELEVATION_CLASS: u32 = 20;
pub(super) const SEE_MASK_NOCLOSEPROCESS: u32 = 0x0000_0040;
pub(super) const SW_SHOWNORMAL: i32 = 1;
pub(super) const WAIT_OBJECT_0: u32 = 0;

#[repr(C)]
pub(super) struct TokenElevation {
    pub(super) token_is_elevated: u32,
}

#[repr(C)]
pub(super) struct ShellExecuteInfoW {
    pub(super) size: u32,
    pub(super) mask: u32,
    pub(super) hwnd: *mut c_void,
    pub(super) verb: *const u16,
    pub(super) file: *const u16,
    pub(super) parameters: *const u16,
    pub(super) directory: *const u16,
    pub(super) show: i32,
    pub(super) instance: *mut c_void,
    pub(super) id_list: *mut c_void,
    pub(super) class: *const u16,
    pub(super) class_key: *mut c_void,
    pub(super) hot_key: u32,
    pub(super) icon_or_monitor: *mut c_void,
    pub(super) process: *mut c_void,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub(super) fn CloseHandle(object: *mut c_void) -> i32;
    pub(super) fn GetCurrentProcess() -> *mut c_void;
    pub(super) fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    pub(super) fn WaitForSingleObject(handle: *mut c_void, milliseconds: u32) -> u32;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    pub(super) fn GetTokenInformation(
        token: *mut c_void,
        information_class: u32,
        information: *mut c_void,
        information_length: u32,
        return_length: *mut u32,
    ) -> i32;
    pub(super) fn OpenProcessToken(
        process: *mut c_void,
        access: u32,
        token: *mut *mut c_void,
    ) -> i32;
}

#[link(name = "shell32")]
unsafe extern "system" {
    pub(super) fn ShellExecuteExW(info: *mut ShellExecuteInfoW) -> i32;
}

#[link(name = "user32")]
unsafe extern "system" {
    pub(super) fn WaitForInputIdle(process: *mut c_void, milliseconds: u32) -> u32;
}
