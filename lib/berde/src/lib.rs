/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! The Bassie Serialize Deserialize library.

/// JSON
#[cfg(feature = "json")]
pub mod json;

/// Serialize
pub mod ser;

/// URL encoded
#[cfg(feature = "urlencoded")]
pub mod urlencoded;

/// YAML
#[cfg(feature = "yaml")]
pub mod yaml;

/// TOML
#[cfg(feature = "toml")]
pub mod toml;
