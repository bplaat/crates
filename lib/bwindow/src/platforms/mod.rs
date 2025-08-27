/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(target_os = "macos")]
pub(crate) use self::macos::*;

mod macos;

#[cfg(not(target_os = "macos"))]
compile_error!("Unsupported platform");
