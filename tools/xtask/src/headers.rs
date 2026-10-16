/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unsafe_code)]

use std::ffi::{c_int, c_long};

#[cfg(target_os = "linux")]
pub(crate) const SC_ARG_MAX: c_int = 0;
#[cfg(target_os = "macos")]
pub(crate) const SC_ARG_MAX: c_int = 1;

unsafe extern "C" {
    pub(crate) fn sysconf(name: c_int) -> c_long;
}
