/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [enable-ansi-support](https://crates.io/crates/enable-ansi-support) crate

/// Enables ANSI escape code support for terminal output on Windows.
#[allow(unsafe_code)]
pub fn enable_ansi_support() -> Result<(), std::io::Error> {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            Ok(())
        } else if #[cfg(windows)] {
            const STD_OUTPUT_HANDLE: i32 = -11;
            const INVALID_HANDLE_VALUE: *mut std::ffi::c_void = (-1isize) as *mut std::ffi::c_void;
            const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;
            #[link(name = "kernel32")]
            unsafe extern "system" {
                fn GetStdHandle(nStdHandle: i32) -> *mut std::ffi::c_void;
                fn GetConsoleMode(hConsoleHandle: *mut std::ffi::c_void, lpMode: *mut u32) -> i32;
                fn SetConsoleMode(hConsoleHandle: *mut std::ffi::c_void, dwMode: u32) -> i32;
            }

            // SAFETY: STD_OUTPUT_HANDLE is a valid standard handle constant for GetStdHandle.
            let stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
            if stdout == INVALID_HANDLE_VALUE {
                return Err(std::io::Error::last_os_error());
            }
            let mut mode: u32 = 0;
            // SAFETY: stdout is a valid console handle (checked above); mode is a valid out-pointer.
            if unsafe { GetConsoleMode(stdout, &mut mode) } == 0 {
                return Err(std::io::Error::last_os_error());
            }
            mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
            // SAFETY: stdout is a valid console handle and mode contains the original flags plus ENABLE_VIRTUAL_TERMINAL_PROCESSING.
            if unsafe { SetConsoleMode(stdout, mode) } == 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        } else {
            compile_error!("Unsupported platform")
        }
    }
}
