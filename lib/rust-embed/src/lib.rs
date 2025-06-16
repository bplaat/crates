/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [rust-embed](https://crates.io/crates/rust-embed) crate

#![forbid(unsafe_code)]

use std::borrow::Cow;

pub use rust_embed_impl::Embed;

/// A trait that allows you to access embedded files
pub trait RustEmbed {
    /// Get an embedded file by its path.
    fn get(file_path: &str) -> Option<EmbeddedFile>;
}

/// A file embedded in the binary
pub struct EmbeddedFile {
    /// The content of the file
    pub data: Cow<'static, [u8]>,
}
