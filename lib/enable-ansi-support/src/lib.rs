/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [enable-ansi-support](https://crates.io/crates/enable-ansi-support) crate

/// Enables ANSI escape code support for terminal output on Windows.
pub fn enable_ansi_support() -> Result<(), std::io::Error> {
    #[cfg(unix)]
    {
        Ok(())
    }

    #[cfg(windows)]
    unsafe {
        const STD_OUTPUT_HANDLE: i32 = -11;
        const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;
        #[link(name = "kernel32")]
        unsafe extern "C" {
            unsafe fn GetStdHandle(nStdHandle: i32) -> *mut std::ffi::c_void;
            unsafe fn GetConsoleMode(
                hConsoleHandle: *mut std::ffi::c_void,
                lpMode: *mut u32,
            ) -> i32;
            unsafe fn SetConsoleMode(hConsoleHandle: *mut std::ffi::c_void, dwMode: u32) -> i32;
        }
        let h_stdout = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut mode: u32 = 0;
        if GetConsoleMode(h_stdout, &mut mode) == 0 {
            return Err(std::io::Error::last_os_error());
        }
        mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        if SetConsoleMode(h_stdout, mode) == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    compile_error!("Unsupported platform");
}
