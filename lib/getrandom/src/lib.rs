/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [getrandom](https://crates.io/crates/getrandom) crate

use std::io::Error;

/// Fill buffer with crypto random bytes
pub fn fill(buf: &mut [u8]) -> Result<(), Error> {
    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "macos", target_os = "openbsd"))] {
            unsafe extern "C" {
                fn getentropy(buf: *mut u8, buflen: usize) -> i32;
            }
            for chunk in buf.chunks_mut(256) {
                if unsafe { getentropy(chunk.as_mut_ptr(), chunk.len()) } != 0 {
                    return Err(Error::other("getentropy failed"));
                }
            }
        }

        else if #[cfg(unix)] {
            type GetrandomFn = unsafe extern "C" fn(*mut u8, usize, u32) -> isize;
            fn resolve_getrandom() -> Option<GetrandomFn> {
                const RTLD_DEFAULT: *mut std::ffi::c_void = std::ptr::null_mut();
                #[cfg_attr(target_os = "linux", link(name = "dl"))]
                unsafe extern "C" {
                    fn dlsym(
                        handle: *mut std::ffi::c_void,
                        symbol: *const std::ffi::c_char,
                    ) -> *mut std::ffi::c_void;
                }
                let ptr = unsafe { dlsym(RTLD_DEFAULT, c"getrandom".as_ptr()) };
                if ptr.is_null() {
                    None
                } else {
                    Some(unsafe { std::mem::transmute::<*mut std::ffi::c_void, GetrandomFn>(ptr) })
                }
            }
            static GETRANDOM: std::sync::LazyLock<Option<GetrandomFn>> = std::sync::LazyLock::new(resolve_getrandom);
            if let Some(getrandom) = *GETRANDOM {
                let n = unsafe { getrandom(buf.as_mut_ptr(), buf.len(), 0) };
                if n < 0 || n as usize != buf.len() {
                    return Err(Error::other("getrandom failed"));
                }
            }

            use std::io::Read;
            let mut file = std::fs::File::open("/dev/urandom")
                .map_err(|_| Error::other("failed to open /dev/urandom"))?;
            file.read_exact(buf)
                .map_err(|_| Error::other("failed to read /dev/urandom"))?;
        }

        else if #[cfg(windows)] {
            #[cfg(not(target_arch = "x86"))]
            #[link(name = "bcryptprimitives", kind = "raw-dylib")]
            unsafe extern "system" {
                fn ProcessPrng(pbData: *mut u8, cbData: usize) -> bool;
            }
            #[cfg(target_arch = "x86")]
            #[link(
                name = "bcryptprimitives",
                kind = "raw-dylib",
                import_name_type = "undecorated"
            )]
            unsafe extern "system" {
                fn ProcessPrng(pbData: *mut u8, cbData: usize) -> bool;
            }
            unsafe { ProcessPrng(buf.as_mut_ptr(), buf.len()) };
        }

        else {
            compile_error!("Unsupported platform");
        }
    }
    Ok(())
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_randomness() {
        let mut buf1 = [0u8; 32];
        fill(&mut buf1).unwrap();

        let mut buf2 = [0u8; 32];
        fill(&mut buf2).unwrap();

        assert_ne!(buf1, buf2);
    }
}
