/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [mime](https://crates.io/crates/mime) crate

#![allow(missing_docs)]
#![forbid(unsafe_code)]

use std::fmt::{self, Display, Formatter};

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
    pub fn type_(&self) -> &str {
        self.type_
    }

    /// Subtype
    pub fn subtype(&self) -> &str {
        self.subtype
    }

    /// Suffix
    pub fn suffix(&self) -> Option<&str> {
        self.suffix
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

// MARK: Common MIME types
pub const APPLICATION_OCTET_STREAM: Mime = Mime::new("application", "octet-stream", None);
pub const APPLICATION_JAVASCRIPT: Mime = Mime::new("application", "javascript", None);
pub const APPLICATION_JSON: Mime = Mime::new("application", "json", None);

pub const IMAGE_PNG: Mime = Mime::new("image", "png", None);
pub const IMAGE_JPEG: Mime = Mime::new("image", "jpeg", None);
pub const IMAGE_GIF: Mime = Mime::new("image", "gif", None);
pub const IMAGE_SVG: Mime = Mime::new("image", "svg", Some("xml"));

pub const TEXT_PLAIN: Mime = Mime::new("text", "plain", None);
pub const TEXT_HTML: Mime = Mime::new("text", "html", None);
pub const TEXT_CSS: Mime = Mime::new("text", "css", None);
pub const TEXT_XML: Mime = Mime::new("text", "xml", None);

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
