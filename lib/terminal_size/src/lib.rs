/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [terminal_size](https://crates.io/crates/terminal_size) crate

#![allow(non_camel_case_types, non_snake_case, clippy::upper_case_acronyms)]

/// Represents the width of a terminal in characters.
pub struct Width(pub u16);

/// Represents the height of a terminal in characters.
pub struct Height(pub u16);

/// Returns the terminal size as a tuple.
pub fn terminal_size() -> Option<(Width, Height)> {
    #[cfg(unix)]
    {
        #[repr(C)]
        struct winsize {
            ws_row: u16,
            ws_col: u16,
            ws_xpixel: u16,
            ws_ypixel: u16,
        }
        const STDOUT_FILENO: i32 = 1;
        const TIOCGWINSZ: i32 = if cfg!(any(
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
            fn ioctl(fd: i32, op: i32, ...) -> i32;
        }

        let mut size: winsize = unsafe { std::mem::zeroed() };
        if unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) } == -1 {
            return None;
        }
        Some((Width(size.ws_col), Height(size.ws_row)))
    }

    #[cfg(windows)]
    {
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
        #[link(name = "kernel32")]
        unsafe extern "C" {
            fn GetStdHandle(nStdHandle: i32) -> *mut std::ffi::c_void;
            fn GetConsoleScreenBufferInfo(
                hConsoleOutput: *mut std::ffi::c_void,
                lpConsoleScreenBufferInfo: *mut CONSOLE_SCREEN_BUFFER_INFO,
            ) -> i32;
        }

        let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
        _ = unsafe { GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &mut csbi) };
        Some((Width(csbi.dwSize.X as u16), Height(csbi.dwSize.Y as u16)))
    }

    #[cfg(not(any(unix, windows)))]
    compile_error!("Unsupported platform");
}
