/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [getrandom](https://crates.io/crates/getrandom) crate

use std::io::Error;

/// Fill buffer with crypto random bytes
pub fn fill(buf: &mut [u8]) -> Result<(), Error> {
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "openbsd"))))]
    {
        unsafe extern "C" {
            fn getrandom(buf: *mut u8, size: usize, flags: u32) -> usize;
        }
        if unsafe { getrandom(buf.as_mut_ptr(), buf.len(), 0) } != buf.len() {
            return Err(Error::other("getrandom failed"));
        }
    }

    #[cfg(any(target_os = "macos", target_os = "openbsd"))]
    {
        unsafe extern "C" {
            fn getentropy(buf: *mut u8, buflen: usize) -> i32;
        }
        for chunk in buf.chunks_mut(256) {
            if unsafe { getentropy(chunk.as_mut_ptr(), chunk.len()) } != 0 {
                return Err(Error::other("getentropy failed"));
            }
        }
    }

    #[cfg(windows)]
    {
        #[link(name = "bcryptprimitives", kind = "raw-dylib")]
        unsafe extern "system" {
            fn ProcessPrng(pbData: *mut u8, cbData: usize) -> bool;
        }
        unsafe { ProcessPrng(buf.as_mut_ptr(), buf.len()) };
    }

    #[cfg(not(any(unix, windows)))]
    {
        compile_error!("Unsupported platform");
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
