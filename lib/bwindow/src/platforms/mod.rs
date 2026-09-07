/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

cfg_select! {
    any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ) => {
        pub(crate) use gtk::*;
        pub(crate) mod gtk;
    }
    target_os = "macos" => {
        pub(crate) use macos::*;
        pub(crate) mod macos;
    }
    windows => {
        pub(crate) use windows::*;
        pub(crate) mod windows;
    }
    _ => {
        compile_error!("Unsupported platform");
    }
}
