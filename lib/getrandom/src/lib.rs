/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal crypto random bytes library

use std::io::{Error, ErrorKind};

// MARK: getrandom
#[cfg(windows)]
mod win32 {
    pub(crate) const PROV_RSA_FULL: u32 = 1;
    pub(crate) const CRYPT_VERIFYCONTEXT: u32 = 0xF0000000;
    extern "C" {
        pub(crate) fn CryptAcquireContextW(
            phProv: *mut usize,
            szContainer: *const u16,
            szProvider: *const u16,
            dwProvType: u32,
            dwFlags: u32,
        ) -> i32;
        pub(crate) fn CryptGenRandom(hProv: usize, dwLen: u32, pbBuffer: *mut u8) -> i32;
        pub(crate) fn CryptReleaseContext(hProv: usize, dwFlags: u32) -> i32;
    }
}

/// Get crypto random bytes
pub fn getrandom(buf: &mut [u8]) -> Result<(), Error> {
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
        let mut h_prov = 0;
        if unsafe {
            win32::CryptAcquireContextW(
                &mut h_prov,
                std::ptr::null(),
                std::ptr::null(),
                win32::PROV_RSA_FULL,
                win32::CRYPT_VERIFYCONTEXT,
            )
        } == 0
        {
            return Err(Error::new(ErrorKind::Other, "CryptAcquireContextW failed"));
        }
        if unsafe { win32::CryptGenRandom(h_prov, buf.len() as u32, buf.as_mut_ptr()) } == 0 {
            unsafe { win32::CryptReleaseContext(h_prov, 0) };
            return Err(Error::new(ErrorKind::Other, "CryptGenRandom failed"));
        }
        unsafe { win32::CryptReleaseContext(h_prov, 0) };
    }
    Ok(())
}

// MARK: Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn randomness() {
        let mut buf1 = [0u8; 32];
        getrandom(&mut buf1).unwrap();

        let mut buf2 = [0u8; 32];
        getrandom(&mut buf2).unwrap();

        assert_ne!(buf1, buf2);
    }
}
