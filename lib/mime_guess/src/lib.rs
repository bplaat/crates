/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [mime_guess](https://crates.io/crates/mime_guess) crate

use std::path::Path;
use std::sync::{Mutex, OnceLock};

use mime::Mime;

fn parse_mime(value: &'static str) -> Mime {
    static CACHE: OnceLock<Mutex<std::collections::HashMap<&'static str, Mime>>> = OnceLock::new();
    CACHE
        .get_or_init(|| Mutex::new(std::collections::HashMap::new()))
        .lock()
        .expect("MIME cache mutex poisoned")
        .entry(value)
        .or_insert_with(|| value.parse().expect("static MIME type must be valid"))
        .clone()
}

/// Create a new `MimeGuess` from a file path
pub fn from_path<P: AsRef<Path>>(path: P) -> MimeGuess {
    MimeGuess::from_path(path.as_ref())
}

/// MimeGuess
pub struct MimeGuess {
    extension: String,
}

impl MimeGuess {
    /// Create a new `MimeGuess` from a file path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
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
            // Text
            "html" | "htm" => mime::TEXT_HTML,
            "css" => mime::TEXT_CSS,
            "js" | "mjs" => mime::APPLICATION_JAVASCRIPT,
            "json" => mime::APPLICATION_JSON,
            "xml" => mime::TEXT_XML,
            "txt" => mime::TEXT_PLAIN,
            "csv" => mime::TEXT_CSV,
            "md" | "markdown" => parse_mime("text/markdown"),
            "yaml" | "yml" => parse_mime("application/yaml"),
            // Web
            "wasm" => parse_mime("application/wasm"),
            "webmanifest" => parse_mime("application/manifest+json"),
            // Images
            "png" => mime::IMAGE_PNG,
            "jpg" | "jpeg" => mime::IMAGE_JPEG,
            "gif" => mime::IMAGE_GIF,
            "svg" => mime::IMAGE_SVG,
            "webp" => parse_mime("image/webp"),
            "ico" => parse_mime("image/x-icon"),
            "avif" => parse_mime("image/avif"),
            "bmp" => mime::IMAGE_BMP,
            "tiff" | "tif" => parse_mime("image/tiff"),
            // Fonts
            "woff" => mime::FONT_WOFF,
            "woff2" => mime::FONT_WOFF2,
            "ttf" => parse_mime("font/ttf"),
            "otf" => parse_mime("font/otf"),
            // Audio
            "mp3" => parse_mime("audio/mpeg"),
            "wav" => parse_mime("audio/wav"),
            "ogg" => parse_mime("audio/ogg"),
            "opus" => parse_mime("audio/opus"),
            "flac" => parse_mime("audio/flac"),
            "m4a" | "aac" => parse_mime("audio/aac"),
            // Video
            "mp4" => parse_mime("video/mp4"),
            "webm" => parse_mime("video/webm"),
            "ogv" => parse_mime("video/ogg"),
            // Documents & archives
            "pdf" => mime::APPLICATION_PDF,
            "zip" => parse_mime("application/zip"),
            "gz" => parse_mime("application/gzip"),
            "tar" => parse_mime("application/x-tar"),
            _ => mime::APPLICATION_OCTET_STREAM,
        }
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_guess() {
        let cases = [
            ("index.html", "text/html"),
            ("index.htm", "text/html"),
            ("style.css", "text/css"),
            ("script.js", "application/javascript"),
            ("module.mjs", "application/javascript"),
            ("data.json", "application/json"),
            ("feed.xml", "text/xml"),
            ("readme.txt", "text/plain"),
            ("data.csv", "text/csv"),
            ("docs.md", "text/markdown"),
            ("docs.markdown", "text/markdown"),
            ("config.yaml", "application/yaml"),
            ("config.yml", "application/yaml"),
            ("app.wasm", "application/wasm"),
            ("app.webmanifest", "application/manifest+json"),
            ("image.png", "image/png"),
            ("photo.jpg", "image/jpeg"),
            ("photo.jpeg", "image/jpeg"),
            ("anim.gif", "image/gif"),
            ("icon.svg", "image/svg+xml"),
            ("image.webp", "image/webp"),
            ("favicon.ico", "image/x-icon"),
            ("image.avif", "image/avif"),
            ("image.bmp", "image/bmp"),
            ("image.tiff", "image/tiff"),
            ("image.tif", "image/tiff"),
            ("font.woff", "font/woff"),
            ("font.woff2", "font/woff2"),
            ("font.ttf", "font/ttf"),
            ("font.otf", "font/otf"),
            ("audio.mp3", "audio/mpeg"),
            ("audio.wav", "audio/wav"),
            ("audio.ogg", "audio/ogg"),
            ("audio.opus", "audio/opus"),
            ("audio.flac", "audio/flac"),
            ("audio.aac", "audio/aac"),
            ("video.mp4", "video/mp4"),
            ("video.webm", "video/webm"),
            ("video.ogv", "video/ogg"),
            ("doc.pdf", "application/pdf"),
            ("archive.zip", "application/zip"),
            ("archive.gz", "application/gzip"),
            ("archive.tar", "application/x-tar"),
            ("unknown.xyz", "application/octet-stream"),
        ];

        for (path, expected) in cases {
            assert_eq!(MimeGuess::from_path(path).first_or_octet_stream().to_string(), expected);
        }
    }
}
