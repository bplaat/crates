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
        mod gtk;
    }
    target_os = "macos" => {
        pub(crate) use macos::*;
        mod macos;
    }
    windows => {
        pub(crate) use windows::*;
        mod windows;
    }
    _ => {
        compile_error!("Unsupported platform");
    }
}
