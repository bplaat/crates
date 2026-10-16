/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

pub use crate::transpiler::Transpiler;

pub(crate) mod transpiler;
mod types;
mod utils;
