/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

cfg_if::cfg_if! {
    if #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ))] {
        pub(crate) use gtk::*;
        mod gtk;
    } else if #[cfg(target_os = "macos")] {
        pub(crate) use macos::*;
        mod macos;
    } else if #[cfg(windows)] {
        pub(crate) use windows::*;
        mod windows;
    } else {
        compile_error!("Unsupported platform");
    }
}
