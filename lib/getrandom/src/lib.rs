/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [getrandom](https://crates.io/crates/getrandom) crate

use std::io::{Error, ErrorKind};

// MARK: getrandom
#[cfg(windows)]
mod windows {
    pub(crate) const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x00000002;
    #[link(name = "bcrypt")]
    extern "C" {
        pub(crate) fn BCryptGenRandom(
            h_alg: *mut std::ffi::c_void,
            pb_output: *mut u8,
            cb_output: u32,
            dw_flags: u32,
        ) -> i32;
    }
}

/// Fill buffer with crypto random bytes
pub fn fill(buf: &mut [u8]) -> Result<(), Error> {
    #[cfg(unix)]
    {
        use std::io::Read;
        let mut file = std::fs::File::open("/dev/urandom")
            .map_err(|_| Error::new(ErrorKind::Other, "Can't open /dev/urandom"))?;
        file.read_exact(buf)
            .map_err(|_| Error::new(ErrorKind::Other, "Can't read from /dev/urandom"))?;
    }

    #[cfg(windows)]
    {
        if unsafe {
            windows::BCryptGenRandom(
                std::ptr::null_mut(),
                buf.as_mut_ptr(),
                buf.len() as u32,
                windows::BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        } != 0
        {
            return Err(Error::new(ErrorKind::Other, "BCryptGenRandom failed"));
        }
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
