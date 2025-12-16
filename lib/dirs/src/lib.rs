/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [dirs](https://crates.io/crates/dirs) crate

use std::path::PathBuf;

#[cfg(not(any(unix, windows)))]
compile_error!("Unsupported platform");

#[cfg(windows)]
pub(crate) mod windows {
    use super::*;

    #[repr(C)]
    pub(crate) struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    #[link(name = "ole32")]
    unsafe extern "system" {
        fn CoTaskMemFree(pv: *mut std::ffi::c_void);
    }

    #[allow(clippy::upper_case_acronyms)]
    pub(crate) struct LPWSTR(*mut u16);
    impl Default for LPWSTR {
        fn default() -> Self {
            Self(std::ptr::null_mut())
        }
    }
    impl LPWSTR {
        pub(crate) fn as_mut_ptr(&mut self) -> *mut *mut u16 {
            &mut self.0
        }
    }
    impl std::fmt::Display for LPWSTR {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.0.is_null() {
                return Ok(());
            }
            let mut len = 0;
            unsafe {
                while *self.0.add(len) != 0 {
                    len += 1;
                }
            }
            let str = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(self.0, len) });
            write!(f, "{str}")
        }
    }
    impl Drop for LPWSTR {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { CoTaskMemFree(self.0 as *mut std::ffi::c_void) };
            }
        }
    }

    const KF_FLAG_DEFAULT: u32 = 0x00000000;
    #[link(name = "shell32")]
    unsafe extern "system" {
        fn SHGetKnownFolderPath(
            rfid: *const Guid,
            dwFlags: u32,
            hToken: *const std::ffi::c_void,
            ppszPath: *mut *mut u16,
        ) -> i32;
    }

    pub(crate) const FOLDERID_LOCAL_APPDATA: Guid = Guid {
        data1: 0xF1B32785,
        data2: 0x6FBA,
        data3: 0x4FCF,
        data4: [0x9D, 0x55, 0x7B, 0x8E, 0x7F, 0x15, 0x70, 0x91],
    };
    pub(crate) const FOLDERID_ROAMING_APPDATA: Guid = Guid {
        data1: 0x3EB685DB,
        data2: 0x65F9,
        data3: 0x4CF6,
        data4: [0xA0, 0x3A, 0xE3, 0xEF, 0x65, 0x72, 0x9F, 0x3D],
    };
    pub(crate) const FOLDERID_MUSIC: Guid = Guid {
        data1: 0x4BD8D571,
        data2: 0x6D19,
        data3: 0x48D3,
        data4: [0xBE, 0x97, 0x42, 0x22, 0x20, 0x08, 0x0E, 0x43],
    };

    pub(crate) fn get_known_folder_path(folder_id: &Guid) -> PathBuf {
        let mut path = LPWSTR::default();
        unsafe {
            SHGetKnownFolderPath(
                folder_id,
                KF_FLAG_DEFAULT,
                std::ptr::null(),
                path.as_mut_ptr(),
            )
        };
        PathBuf::from(path.to_string())
    }
}

/// Get user's cache directory
pub fn cache_dir() -> Option<PathBuf> {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let xdg_cache = std::env::var("XDG_CACHE_HOME").map(PathBuf::from);
        Some(xdg_cache.unwrap_or_else(|_| {
            std::env::home_dir()
                .expect("Can't find home dir")
                .join(".cache")
        }))
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::home_dir().expect("Can't find home dir");
        Some(home.join("Library").join("Caches"))
    }
    #[cfg(windows)]
    {
        Some(windows::get_known_folder_path(
            &windows::FOLDERID_LOCAL_APPDATA,
        ))
    }
}

/// Get user's audio directory
pub fn config_dir() -> Option<PathBuf> {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let xdg_config = std::env::var("XDG_CONFIG_HOME").map(PathBuf::from);
        Some(xdg_config.unwrap_or_else(|_| {
            std::env::home_dir()
                .expect("Can't find home dir")
                .join(".config")
        }))
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::home_dir().expect("Can't find home dir");
        Some(home.join("Library").join("Application Support"))
    }
    #[cfg(windows)]
    {
        Some(windows::get_known_folder_path(
            &windows::FOLDERID_ROAMING_APPDATA,
        ))
    }
}

/// Get user's audio directory
pub fn audio_dir() -> Option<PathBuf> {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let xdg_music = std::env::var("XDG_MUSIC_DIR").map(PathBuf::from);
        Some(xdg_music.unwrap_or_else(|_| {
            std::env::home_dir()
                .expect("Can't find home dir")
                .join("Music")
        }))
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::home_dir().expect("Can't find home dir");
        Some(home.join("Music"))
    }
    #[cfg(windows)]
    {
        Some(windows::get_known_folder_path(&windows::FOLDERID_MUSIC))
    }
}
