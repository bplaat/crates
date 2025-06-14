/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::process::exit;

use anyhow::Result;

use crate::{Args, MetadataService};

pub(crate) fn get_album_ids(metadata_service: &MetadataService, args: &Args) -> Result<Vec<i64>> {
    Ok(if args.is_artist {
        let artist_id = if args.is_id {
            args.query.parse()?
        } else {
            let artists = metadata_service.seach_artist(&args.query)?;
            if artists.is_empty() {
                eprintln!("No artist found");
                exit(1);
            }
            artists[0].id
        };

        let albums = metadata_service.get_artist_albums(artist_id)?;
        if args.with_singles {
            albums.iter().map(|album| album.id).collect()
        } else {
            albums
                .iter()
                .filter(|album| {
                    (album.r#type == "album" || album.r#type == "ep")
                        && album.record_type != "single"
                })
                .map(|album| album.id)
                .collect()
        }
    } else if args.is_id {
        vec![args.query.parse()?]
    } else {
        let albums = metadata_service.search_album(&args.query)?;
        if albums.is_empty() {
            eprintln!("No album found");
            exit(1);
        }
        vec![albums[0].id]
    })
}

pub(crate) fn user_music_dir() -> String {
    #[cfg(unix)]
    {
        std::env::var("XDG_MUSIC_DIR").unwrap_or_else(|_| {
            format!(
                "{}/Music",
                std::env::var("HOME").expect("Can't read #HOME env variable")
            )
        })
    }

    #[cfg(windows)]
    {
        const KF_FLAG_DEFAULT: u32 = 0x00000000;
        #[repr(C)]
        struct Guid {
            data1: u32,
            data2: u16,
            data3: u16,
            data4: [u8; 8],
        }
        #[link(name = "shell32")]
        unsafe extern "C" {
            fn SHGetKnownFolderPath(
                rfid: *const Guid,
                dwFlags: u32,
                hToken: *const std::ffi::c_void,
                ppszPath: *mut *mut u16,
            ) -> i32;
        }
        #[link(name = "ole32")]
        unsafe extern "C" {
            fn CoTaskMemFree(pv: *mut std::ffi::c_void);
        }

        const FOLDERID_MUSIC: Guid = Guid {
            data1: 0x4BD8D571,
            data2: 0x6D19,
            data3: 0x48D3,
            data4: [0xBE, 0x97, 0x42, 0x22, 0x20, 0x08, 0x0E, 0x43],
        };
        let mut path_ptr: *mut u16 = std::ptr::null_mut();
        let result = unsafe {
            SHGetKnownFolderPath(
                &FOLDERID_MUSIC,
                KF_FLAG_DEFAULT,
                std::ptr::null(),
                &mut path_ptr,
            )
        };
        if result != 0 {
            panic!("Failed to get known folder path");
        }

        let len = (0..)
            .take_while(|&i| unsafe { *path_ptr.add(i) } != 0)
            .count();
        let path = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(path_ptr, len) });
        unsafe { CoTaskMemFree(path_ptr as *mut std::ffi::c_void) };
        path
    }

    #[cfg(not(any(unix, windows)))]
    compile_error!("Unsupported platform");
}

pub(crate) fn escape_path(path: &str) -> String {
    path.chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect()
}
