/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(target_os = "macos")]
pub(crate) use macos::*;
#[cfg(not(target_os = "macos"))]
pub(crate) use stub::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod stub;
