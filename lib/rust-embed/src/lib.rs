/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [rust-embed](https://crates.io/crates/rust-embed) crate

use std::borrow::Cow;

pub use rust_embed_impl::Embed;

/// A trait that allows you to access embedded files
pub trait RustEmbed {
    /// Get an embedded file by its path.
    fn get(file_path: &str) -> Option<EmbeddedFile>;

    /// Iterate over the relative paths of all embedded files.
    fn iter() -> impl Iterator<Item = Cow<'static, str>>;
}

/// A file embedded in the binary
pub struct EmbeddedFile {
    /// The content of the file
    pub data: Cow<'static, [u8]>,
}

/// Returns the bundle Resources directory when running from a macOS .app bundle.
///
/// Standard layout: `{bundle}.app/Contents/MacOS/<exe>` -> `{bundle}.app/Contents/Resources/`
///
/// Override with `RUST_EMBED_RESOURCES_DIR` env var (useful for tests).
#[cfg(feature = "macos-bundle")]
pub fn bundle_resources_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("RUST_EMBED_RESOURCES_DIR") {
        return std::path::PathBuf::from(dir);
    }
    std::env::current_exe()
        .expect("current_exe failed")
        .parent()
        .expect("exe has no parent dir")
        .parent()
        .expect("MacOS/ has no parent dir")
        .join("Resources")
}
