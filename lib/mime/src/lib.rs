/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [mime](https://crates.io/crates/mime) crate

#![allow(missing_docs)]

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// A MIME token borrowed from a [`Mime`] value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name<'a>(&'a str);

impl<'a> Name<'a> {
    /// Returns this token as a string slice.
    pub const fn as_str(&self) -> &'a str {
        self.0
    }
}

impl AsRef<str> for Name<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Display for Name<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl<'a> From<Name<'a>> for &'a str {
    fn from(name: Name<'a>) -> Self {
        name.0
    }
}

impl PartialEq<&str> for Name<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Name<'_>> for &str {
    fn eq(&self, other: &Name<'_>) -> bool {
        *self == other.0
    }
}

// MARK: Mime
/// A MIME type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mime {
    type_: &'static str,
    subtype: &'static str,
    suffix: Option<&'static str>,
}

impl Mime {
    /// Create a new `Mime` instance
    pub const fn new(
        type_: &'static str,
        subtype: &'static str,
        suffix: Option<&'static str>,
    ) -> Self {
        Self {
            type_,
            subtype,
            suffix,
        }
    }

    /// Type
    pub const fn type_(&self) -> Name<'_> {
        Name(self.type_)
    }

    /// Subtype
    pub const fn subtype(&self) -> Name<'_> {
        Name(self.subtype)
    }

    /// Suffix
    pub const fn suffix(&self) -> Option<Name<'_>> {
        match self.suffix {
            Some(suffix) => Some(Name(suffix)),
            None => None,
        }
    }
}

impl Display for Mime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.type_, self.subtype)?;
        if let Some(suffix) = self.suffix {
            write!(f, "+{suffix}")?;
        }
        Ok(())
    }
}

impl FromStr for Mime {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (type_, subtype) = value.split_once('/').ok_or("missing MIME type separator")?;
        let (subtype, suffix) = subtype
            .split_once('+')
            .map_or((subtype, None), |(subtype, suffix)| (subtype, Some(suffix)));
        if type_.is_empty() || subtype.is_empty() {
            return Err("empty MIME type component");
        }
        Ok(Self::new(
            Box::leak(type_.to_owned().into_boxed_str()),
            Box::leak(subtype.to_owned().into_boxed_str()),
            suffix.map(|suffix| Box::leak(suffix.to_owned().into_boxed_str()) as &'static str),
        ))
    }
}

// MARK: Common MIME types
pub const APPLICATION_GZIP: Mime = Mime::new("application", "gzip", None);
pub const APPLICATION_JAVASCRIPT: Mime = Mime::new("application", "javascript", None);
pub const APPLICATION_JSON: Mime = Mime::new("application", "json", None);
pub const APPLICATION_MANIFEST_JSON: Mime = Mime::new("application", "manifest", Some("json"));
pub const APPLICATION_OCTET_STREAM: Mime = Mime::new("application", "octet-stream", None);
pub const APPLICATION_PDF: Mime = Mime::new("application", "pdf", None);
pub const APPLICATION_WASM: Mime = Mime::new("application", "wasm", None);
pub const APPLICATION_X_TAR: Mime = Mime::new("application", "x-tar", None);
pub const APPLICATION_YAML: Mime = Mime::new("application", "yaml", None);
pub const APPLICATION_ZIP: Mime = Mime::new("application", "zip", None);

pub const AUDIO_AAC: Mime = Mime::new("audio", "aac", None);
pub const AUDIO_FLAC: Mime = Mime::new("audio", "flac", None);
pub const AUDIO_MPEG: Mime = Mime::new("audio", "mpeg", None);
pub const AUDIO_OGG: Mime = Mime::new("audio", "ogg", None);
pub const AUDIO_OPUS: Mime = Mime::new("audio", "opus", None);
pub const AUDIO_WAV: Mime = Mime::new("audio", "wav", None);

pub const FONT_OTF: Mime = Mime::new("font", "otf", None);
pub const FONT_TTF: Mime = Mime::new("font", "ttf", None);
pub const FONT_WOFF: Mime = Mime::new("font", "woff", None);
pub const FONT_WOFF2: Mime = Mime::new("font", "woff2", None);

pub const IMAGE_AVIF: Mime = Mime::new("image", "avif", None);
pub const IMAGE_BMP: Mime = Mime::new("image", "bmp", None);
pub const IMAGE_GIF: Mime = Mime::new("image", "gif", None);
pub const IMAGE_JPEG: Mime = Mime::new("image", "jpeg", None);
pub const IMAGE_PNG: Mime = Mime::new("image", "png", None);
pub const IMAGE_SVG: Mime = Mime::new("image", "svg", Some("xml"));
pub const IMAGE_TIFF: Mime = Mime::new("image", "tiff", None);
pub const IMAGE_WEBP: Mime = Mime::new("image", "webp", None);
pub const IMAGE_X_ICON: Mime = Mime::new("image", "x-icon", None);

pub const TEXT_CSV: Mime = Mime::new("text", "csv", None);
pub const TEXT_CSS: Mime = Mime::new("text", "css", None);
pub const TEXT_HTML: Mime = Mime::new("text", "html", None);
pub const TEXT_MARKDOWN: Mime = Mime::new("text", "markdown", None);
pub const TEXT_PLAIN: Mime = Mime::new("text", "plain", None);
pub const TEXT_XML: Mime = Mime::new("text", "xml", None);

pub const VIDEO_MP4: Mime = Mime::new("video", "mp4", None);
pub const VIDEO_OGG: Mime = Mime::new("video", "ogg", None);
pub const VIDEO_WEBM: Mime = Mime::new("video", "webm", None);

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_string() {
        let mime = Mime::new("text", "html", None);
        assert_eq!(mime.to_string(), "text/html");

        let mime = Mime::new("application", "vnd.api", Some("json"));
        assert_eq!(mime.to_string(), "application/vnd.api+json");

        assert_eq!(
            APPLICATION_OCTET_STREAM.to_string(),
            "application/octet-stream"
        );
        assert_eq!(IMAGE_SVG.to_string(), "image/svg+xml");
        assert_eq!(TEXT_XML.to_string(), "text/xml");
    }
}
