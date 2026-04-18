/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [terminal_size](https://crates.io/crates/terminal_size) crate

#![allow(non_camel_case_types, non_snake_case, clippy::upper_case_acronyms)]

use std::mem::MaybeUninit;

/// Represents the width of a terminal in characters.
pub struct Width(pub u16);

/// Represents the height of a terminal in characters.
pub struct Height(pub u16);

/// Returns the terminal size as a tuple.
#[allow(unsafe_code)]
pub fn terminal_size() -> Option<(Width, Height)> {
    cfg_select! {
        unix => {
            #[repr(C)]
            struct winsize {
                ws_row: u16,
                ws_col: u16,
                ws_xpixel: u16,
                ws_ypixel: u16,
            }
            const STDOUT_FILENO: i32 = 1;
            const TIOCGWINSZ: std::ffi::c_ulong = if cfg!(any(
                target_os = "macos",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd"
            )) {
                0x40087468
            } else {
                0x5413
            };
            unsafe extern "C" {
                fn ioctl(fd: i32, op: std::ffi::c_ulong, ...) -> i32;
            }

            let mut size = MaybeUninit::<winsize>::uninit();
            // SAFETY: size is a valid MaybeUninit<winsize> pointer; ioctl with TIOCGWINSZ fills it on success.
            if unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, size.as_mut_ptr()) } == -1 {
                return None;
            }
            // SAFETY: ioctl returned success, so size was fully initialized by the kernel.
            let size = unsafe { size.assume_init() };
            Some((Width(size.ws_col), Height(size.ws_row)))
        }
        windows => {
            #[repr(C)]
            struct COORD {
                X: i16,
                Y: i16,
            }
            #[repr(C)]
            struct SMALL_RECT {
                Left: i16,
                Top: i16,
                Right: i16,
                Bottom: i16,
            }
            #[repr(C)]
            struct CONSOLE_SCREEN_BUFFER_INFO {
                dwSize: COORD,
                dwCursorPosition: COORD,
                wAttributes: u16,
                srWindow: SMALL_RECT,
                dwMaximumWindowSize: COORD,
            }
            const STD_OUTPUT_HANDLE: i32 = -11;
            const INVALID_HANDLE_VALUE: *mut std::ffi::c_void = (-1isize) as *mut std::ffi::c_void;
            #[link(name = "kernel32")]
            unsafe extern "system" {
                fn GetStdHandle(nStdHandle: i32) -> *mut std::ffi::c_void;
                fn GetConsoleScreenBufferInfo(
                    hConsoleOutput: *mut std::ffi::c_void,
                    lpConsoleScreenBufferInfo: *mut CONSOLE_SCREEN_BUFFER_INFO,
                ) -> i32;
            }

            // SAFETY: STD_OUTPUT_HANDLE is a valid standard handle constant for GetStdHandle.
            let stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
            if stdout == INVALID_HANDLE_VALUE {
                return None;
            }
            let mut csbi = MaybeUninit::<CONSOLE_SCREEN_BUFFER_INFO>::uninit();
            // SAFETY: stdout is a valid console handle (checked above); csbi.as_mut_ptr() is a valid output pointer.
            if unsafe { GetConsoleScreenBufferInfo(stdout, csbi.as_mut_ptr()) } == 0 {
                return None;
            }
            // SAFETY: GetConsoleScreenBufferInfo returned success, so csbi was fully initialized by the OS.
            let csbi = unsafe { csbi.assume_init() };
            Some((Width(csbi.dwSize.X as u16), Height(csbi.dwSize.Y as u16)))
        }
        _ => {
            compile_error!("Unsupported platform")
        }
    }
}
