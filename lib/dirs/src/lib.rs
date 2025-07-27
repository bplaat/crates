/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [dirs](https://crates.io/crates/dirs) crate

use std::path::PathBuf;

#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "macos",
    windows
)))]
compile_error!("Unsupported platform");

#[cfg(windows)]
pub(crate) mod windows {
    #[repr(C)]
    pub(crate) struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
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
    #[link(name = "ole32")]
    unsafe extern "system" {
        fn CoTaskMemFree(pv: *mut std::ffi::c_void);
    }

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

    pub(crate) fn get_known_folder_path(folder_id: &Guid) -> String {
        let mut path_ptr: *mut u16 = std::ptr::null_mut();
        unsafe {
            SHGetKnownFolderPath(folder_id, KF_FLAG_DEFAULT, std::ptr::null(), &mut path_ptr)
        };
        let mut len = 0;
        unsafe {
            while *path_ptr.add(len) != 0 {
                len += 1;
            }
        }
        let path = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(path_ptr, len) });
        unsafe { CoTaskMemFree(path_ptr as *mut std::ffi::c_void) };
        path
    }
}

/// Get user's audio directory
pub fn config_dir() -> Option<PathBuf> {
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    {
        let path = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            format!("{}/.config", std::env::var("HOME").expect("$HOME not set"))
        });
        Some(PathBuf::from(path))
    }

    #[cfg(target_os = "macos")]
    {
        let path = std::env::var("HOME").unwrap_or_else(|_| panic!("$HOME not set"));
        Some(PathBuf::from(format!("{path}/Library/Application Support")))
    }

    #[cfg(windows)]
    {
        Some(PathBuf::from(windows::get_known_folder_path(
            &windows::FOLDERID_ROAMING_APPDATA,
        )))
    }
}

/// Get user's audio directory
pub fn audio_dir() -> Option<PathBuf> {
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    {
        let path = std::env::var("XDG_MUSIC_DIR")
            .unwrap_or_else(|_| format!("{}/Music", std::env::var("HOME").expect("$HOME not set")));
        Some(PathBuf::from(path))
    }

    #[cfg(target_os = "macos")]
    {
        let path = std::env::var("HOME").unwrap_or_else(|_| panic!("$HOME not set"));
        Some(PathBuf::from(format!("{path}/Music")))
    }

    #[cfg(windows)]
    {
        Some(PathBuf::from(windows::get_known_folder_path(
            &windows::FOLDERID_MUSIC,
        )))
    }
}
