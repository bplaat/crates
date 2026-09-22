/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [getrandom](https://crates.io/crates/getrandom) crate

use std::fmt::{self, Display, Formatter};

/// Raw operating-system error type.
pub type RawOsError = i32;

/// Error returned when the operating system cannot provide random bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error(std::num::NonZeroI32);

impl Error {
    const INTERNAL_START: RawOsError = 1 << 16;
    const CUSTOM_START: RawOsError = 1 << 17;

    /// This target or platform is not supported.
    pub const UNSUPPORTED: Self = Self::new_internal(0);
    /// The platform-specific error number was not positive.
    pub const ERRNO_NOT_POSITIVE: Self = Self::new_internal(1);
    /// An unexpected situation was encountered.
    pub const UNEXPECTED: Self = Self::new_internal(2);

    /// Creates an error from an application-defined error code.
    pub const fn new_custom(code: u16) -> Self {
        Self::from_nonzero(Self::CUSTOM_START + code as RawOsError)
    }

    const fn new_internal(code: u16) -> Self {
        Self::from_nonzero(Self::INTERNAL_START + code as RawOsError)
    }

    #[allow(unsafe_code)]
    const fn from_nonzero(code: RawOsError) -> Self {
        // SAFETY: Internal and custom error ranges are strictly positive.
        Self(unsafe { std::num::NonZeroI32::new_unchecked(code) })
    }

    fn from_io_error(error: std::io::Error) -> Self {
        let Some(code) = error.raw_os_error() else {
            return Self::UNEXPECTED;
        };
        let Some(code) = code.checked_neg().and_then(std::num::NonZeroI32::new) else {
            return Self::ERRNO_NOT_POSITIVE;
        };
        Self(code)
    }

    /// Returns the raw operating-system error code, if this is an OS error.
    pub fn raw_os_error(self) -> Option<RawOsError> {
        let code = self.0.get();
        (code < 0).then(|| -code)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.raw_os_error() {
            return std::io::Error::from_raw_os_error(code).fmt(f);
        }
        match *self {
            Self::UNSUPPORTED => f.write_str("getrandom: this target is not supported"),
            Self::ERRNO_NOT_POSITIVE => f.write_str("errno: did not return a positive value"),
            Self::UNEXPECTED => f.write_str("unexpected situation"),
            _ => write!(f, "Unknown Error: {}", self.0),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for std::io::Error {
    fn from(error: Error) -> Self {
        match error.raw_os_error() {
            Some(code) => Self::from_raw_os_error(code),
            None => Self::other(error),
        }
    }
}

/// Fill buffer with crypto random bytes
#[allow(unsafe_code)]
pub fn fill(buf: &mut [u8]) -> Result<(), Error> {
    cfg_select! {
        any(target_os = "macos", target_os = "openbsd") => {
            unsafe extern "C" {
                fn getentropy(buf: *mut u8, buflen: usize) -> i32;
            }
            for chunk in buf.chunks_mut(256) {
                // SAFETY: chunk is a valid mutable byte slice with length <= 256, satisfying getentropy's requirements.
                if unsafe { getentropy(chunk.as_mut_ptr(), chunk.len()) } != 0 {
                    return Err(Error::from_io_error(std::io::Error::last_os_error()));
                }
            }
        }
        unix => {
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
                // SAFETY: RTLD_DEFAULT and the c-string literal are valid arguments to dlsym.
                let ptr = unsafe { dlsym(RTLD_DEFAULT, c"getrandom".as_ptr()) };
                if ptr.is_null() {
                    None
                } else {
                    // SAFETY: dlsym returned a non-null pointer for "getrandom", which has the GetrandomFn signature.
                    Some(unsafe { std::mem::transmute::<*mut std::ffi::c_void, GetrandomFn>(ptr) })
                }
            }
            static GETRANDOM: std::sync::LazyLock<Option<GetrandomFn>> =
                std::sync::LazyLock::new(resolve_getrandom);
            if let Some(getrandom) = *GETRANDOM {
                // SAFETY: buf is a valid mutable byte slice; getrandom was resolved and has the correct signature.
                let n = unsafe { getrandom(buf.as_mut_ptr(), buf.len(), 0) };
                if n >= 0 && n as usize == buf.len() {
                    return Ok(());
                }
                // Fall through to /dev/urandom if getrandom fails or returns ENOSYS
            }

            use std::io::Read;
            let mut file = std::fs::File::open("/dev/urandom").map_err(Error::from_io_error)?;
            file.read_exact(buf).map_err(Error::from_io_error)?;
        }
        windows => {
            #[cfg_attr(
                target_arch = "x86",
                link(
                    name = "bcryptprimitives",
                    kind = "raw-dylib",
                    import_name_type = "undecorated"
                )
            )]
            #[cfg_attr(
                not(target_arch = "x86"),
                link(name = "bcryptprimitives", kind = "raw-dylib")
            )]
            unsafe extern "system" {
                fn ProcessPrng(pbData: *mut u8, cbData: usize) -> i32;
            }
            // SAFETY: buf is a valid mutable byte slice; ProcessPrng is a documented Windows API that fills it.
            if unsafe { ProcessPrng(buf.as_mut_ptr(), buf.len()) } == 0 {
                return Err(Error::UNEXPECTED);
            }
        }
        _ => compile_error!("Unsupported platform"),
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
