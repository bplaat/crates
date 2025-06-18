/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [mime_guess](https://crates.io/crates/mime_guess) crate

#![deny(unsafe_code)]

use std::path::Path;

use mime::Mime;

/// Create a new `MimeGuess` from a file path
pub fn from_path(path: impl AsRef<Path>) -> MimeGuess {
    MimeGuess::from_path(path.as_ref())
}

/// MimeGuess
pub struct MimeGuess {
    extension: String,
}

impl MimeGuess {
    /// Create a new `MimeGuess` from a file path
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let extension = path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map_or_else(String::new, |ext| ext.to_string());
        Self { extension }
    }

    /// Guess MIME type or return `application/octet-stream` if unknown
    pub fn first_or_octet_stream(&self) -> Mime {
        match self.extension.as_str() {
            "html" | "htm" => mime::TEXT_HTML,
            "css" => mime::TEXT_CSS,
            "js" => mime::APPLICATION_JAVASCRIPT,
            "json" => mime::APPLICATION_JSON,
            "xml" => mime::TEXT_XML,
            "png" => mime::IMAGE_PNG,
            "jpg" | "jpeg" => mime::IMAGE_JPEG,
            "gif" => mime::IMAGE_GIF,
            "svg" => mime::IMAGE_SVG,
            "txt" => mime::TEXT_PLAIN,
            _ => mime::APPLICATION_OCTET_STREAM,
        }
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_guess() {
        let guess = MimeGuess::from_path("example.html");
        assert_eq!(guess.first_or_octet_stream(), mime::TEXT_HTML);

        let guess = MimeGuess::from_path("style.css");
        assert_eq!(guess.first_or_octet_stream(), mime::TEXT_CSS);

        let guess = MimeGuess::from_path("script.js");
        assert_eq!(guess.first_or_octet_stream(), mime::APPLICATION_JAVASCRIPT);

        let guess = MimeGuess::from_path("data.json");
        assert_eq!(guess.first_or_octet_stream(), mime::APPLICATION_JSON);

        let guess = MimeGuess::from_path("image.png");
        assert_eq!(guess.first_or_octet_stream(), mime::IMAGE_PNG);

        let guess = MimeGuess::from_path("unknown.xyz");
        assert_eq!(
            guess.first_or_octet_stream(),
            mime::APPLICATION_OCTET_STREAM
        );
    }
}
