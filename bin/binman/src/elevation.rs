/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unsafe_code)]

use std::ffi::{OsStr, c_void};
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null;

use crate::win32::*;

pub(crate) fn is_process_elevated() -> bool {
    let mut token = null::<c_void>().cast_mut();
    // SAFETY: `token` points to writable storage for the returned process-token
    // handle, and GetCurrentProcess returns a valid pseudo-handle.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return false;
    }

    let mut elevation = TokenElevation {
        token_is_elevated: 0,
    };
    let mut returned = 0;
    // SAFETY: `token` was opened successfully, and `elevation` is writable storage
    // with the exact size passed to GetTokenInformation.
    let success = unsafe {
        GetTokenInformation(
            token,
            TOKEN_ELEVATION_CLASS,
            (&raw mut elevation).cast(),
            size_of::<TokenElevation>() as u32,
            &mut returned,
        )
    } != 0;
    // SAFETY: `token` is an owned handle returned by OpenProcessToken and is closed once.
    _ = unsafe { CloseHandle(token) };
    success && elevation.token_is_elevated != 0
}

pub(crate) fn restart() -> io::Result<()> {
    let executable = std::env::current_exe()?;
    let operation: Vec<u16> = OsStr::new("runas").encode_wide().chain(Some(0)).collect();
    let executable: Vec<u16> = executable
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();

    let mut info = ShellExecuteInfoW {
        size: size_of::<ShellExecuteInfoW>() as u32,
        mask: SEE_MASK_NOCLOSEPROCESS,
        hwnd: null::<c_void>().cast_mut(),
        verb: operation.as_ptr(),
        file: executable.as_ptr(),
        parameters: null(),
        directory: null(),
        show: SW_SHOWNORMAL,
        instance: null::<c_void>().cast_mut(),
        id_list: null::<c_void>().cast_mut(),
        class: null(),
        class_key: null::<c_void>().cast_mut(),
        hot_key: 0,
        icon_or_monitor: null::<c_void>().cast_mut(),
        process: null::<c_void>().cast_mut(),
    };

    // SAFETY: `info` has the documented size and layout, and its UTF-16 buffers
    // are null-terminated and remain alive for the duration of the call.
    if unsafe { ShellExecuteExW(&raw mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if info.process.is_null() {
        return Err(io::Error::other(
            "Windows did not return an elevated process handle",
        ));
    }

    // SAFETY: `process` is an owned process handle returned by ShellExecuteExW.
    _ = unsafe { WaitForInputIdle(info.process, 5_000) };
    // SAFETY: A zero-timeout wait only checks whether the owned process has exited.
    let exited = unsafe { WaitForSingleObject(info.process, 0) } == WAIT_OBJECT_0;
    // SAFETY: The owned process handle is closed exactly once after the checks.
    _ = unsafe { CloseHandle(info.process) };

    if exited {
        Err(io::Error::other(
            "The elevated Binman process exited before opening its window",
        ))
    } else {
        Ok(())
    }
}
