/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

pub use canvas::*;
mod canvas;
#[cfg(any(windows, test))]
#[cfg_attr(not(windows), allow(dead_code))]
mod path;
mod platforms;
