/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [anyhow](https://crates.io/crates/anyhow) crate

/// A type alias for a Result with a boxed error.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
