/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ptr::null_mut;

use base64::prelude::*;
use macview_appkit::{Point, Rect, Size};
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send};

use crate::cocoa::ns_string;

/// The size used for an SVG that declares neither dimensions nor a view box.
const DEFAULT_SIZE: Size = Size {
    width: 512.0,
    height: 512.0,
};

/// The byte order mark that a document may start with.
const BYTE_ORDER_MARK: &[u8] = &[0xef, 0xbb, 0xbf];

/// An SVG document, ready to be displayed by WebKit.
pub(crate) struct Svg {
    /// The intrinsic size in display units.
    pub size: Size,
    /// A page that draws the document centered and scaled to fit.
    pub html: String,
}

/// Returns whether the bytes are an SVG document.
pub(crate) fn is_svg(bytes: &[u8]) -> bool {
    root_tag(bytes).is_some()
}

/// Reads the intrinsic size of an SVG document and wraps it in a page WebKit can display.
pub(crate) fn parse_svg(bytes: &[u8]) -> Svg {
    Svg {
        size: root_tag(bytes).and_then(tag_size).unwrap_or(DEFAULT_SIZE),
        // The document is embedded as an image so that any script or external reference it
        // contains stays inert.
        html: format!(
            "<!doctype html><meta charset=\"utf-8\"><style>\
             html,body{{margin:0;height:100%;overflow:hidden;background:transparent}}\
             img{{width:100%;height:100%;object-fit:contain}}\
             </style><img src=\"data:image/svg+xml;base64,{}\">",
            BASE64_STANDARD.encode(bytes)
        ),
    }
}

define_class!(
    #[unsafe(super(WKWebView))]
    #[name = "MacViewSvgView"]
    struct SvgView;

    impl SvgView {
        /// Lets every event through to the scroll view around it.
        ///
        /// A web view handles scrolling, magnifying and its own page menu itself, none of which
        /// belongs to an image. Taking the view out of hit testing hands all of it to the scroll
        /// view, which scrolls, pans and zooms the media the same way it does for the other
        /// formats, and leaves the page as a drawing only.
        #[unsafe(method(hitTest:))]
        const fn _hit_test(&self, _: Point) -> *mut Object {
            null_mut()
        }
    }
);

/// Creates an owned view that draws an SVG document with WebKit.
///
/// The caller owns the returned view and must send it `release`.
pub(crate) fn create_svg_view(frame: Rect, svg: &Svg) -> *mut Object {
    // SAFETY: All objects are valid WebKit instances and the configuration is released after the
    // web view retains its own copy. The page is loaded from memory without a base URL.
    unsafe {
        let configuration: *mut Object = msg_send![class!(WKWebViewConfiguration), new];
        let preferences: *mut Object = msg_send![configuration, defaultWebpagePreferences];
        let _: () = msg_send![preferences, setAllowsContentJavaScript: Bool::NO];
        let view: *mut Object = msg_send![SvgView::class(), alloc];
        let view: *mut Object = msg_send![view,
            initWithFrame: frame,
            configuration: configuration
        ];
        let _: () = msg_send![configuration, release];
        assert!(!view.is_null(), "failed to create SVG view");

        // A web view paints an opaque background of its own, which would hide the checkerboard.
        let opaque: *mut Object = msg_send![class!(NSNumber), numberWithBool: Bool::NO];
        let _: () = msg_send![view, setValue: opaque, forKey: ns_string("drawsBackground")];
        let _: *mut Object = msg_send![view,
            loadHTMLString: ns_string(&svg.html),
            baseURL: null_mut::<Object>()
        ];
        view
    }
}

/// Returns the attributes of the SVG root element, or `None` when the bytes are not an SVG
/// document.
///
/// Only a prolog may precede the root element, so bytes that hold anything else there are some
/// other format that happens to contain the tag, and a tag inside a comment is not a root either.
fn root_tag(bytes: &[u8]) -> Option<&str> {
    let mut rest = bytes.strip_prefix(BYTE_ORDER_MARK).unwrap_or(bytes);
    loop {
        rest = trim_start(rest);
        if let Some(attributes) = rest.strip_prefix(b"<svg") {
            let separator = attributes.first()?;
            if !separator.is_ascii_whitespace() && *separator != b'>' && *separator != b'/' {
                return None;
            }
            return std::str::from_utf8(&attributes[..tag_end(attributes)?]).ok();
        }
        rest = if rest.starts_with(b"<!--") {
            skip_past(rest, b"-->")?
        } else if rest.starts_with(b"<?") {
            skip_past(rest, b"?>")?
        } else if rest.starts_with(b"<!") {
            // A document type declaration, which the parser does not look inside.
            skip_past(rest, b">")?
        } else {
            return None;
        };
    }
}

/// Returns the bytes that follow the first occurrence of `terminator`.
fn skip_past<'a>(bytes: &'a [u8], terminator: &[u8]) -> Option<&'a [u8]> {
    bytes
        .windows(terminator.len())
        .position(|window| window == terminator)
        .map(|index| &bytes[index + terminator.len()..])
}

/// Returns the length of the attributes of an element, which end at the first unquoted `>`.
fn tag_end(attributes: &[u8]) -> Option<usize> {
    let mut quote = None;
    for (index, byte) in attributes.iter().enumerate() {
        match (quote, byte) {
            (None, b'"' | b'\'') => quote = Some(*byte),
            (Some(open), _) if *byte == open => quote = None,
            (None, b'>') => return Some(index),
            _ => {}
        }
    }
    None
}

/// Returns the bytes without their leading whitespace.
fn trim_start(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    &bytes[start..]
}

/// Returns the size of a root element, preferring its dimensions over its view box.
fn tag_size(tag: &str) -> Option<Size> {
    let width = attribute(tag, "width").and_then(length);
    let height = attribute(tag, "height").and_then(length);
    if let (Some(width), Some(height)) = (width, height) {
        return Some(Size { width, height });
    }
    let view_box = attribute(tag, "viewBox")?;
    let mut sizes = view_box
        .split([' ', '\t', '\n', '\r', ','])
        .filter(|part| !part.is_empty())
        .skip(2)
        .filter_map(length);
    Some(Size {
        width: sizes.next()?,
        height: sizes.next()?,
    })
}

/// Returns the value of an attribute of an element.
fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let mut rest = tag;
    while let Some(index) = rest.find(name) {
        let starts_name = index == 0 || rest[..index].ends_with(char::is_whitespace);
        rest = &rest[index + name.len()..];
        if !starts_name {
            continue;
        }
        let Some(value) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        let value = value.trim_start();
        let mut characters = value.chars();
        let Some(quote @ ('"' | '\'')) = characters.next() else {
            continue;
        };
        return value[1..].split(quote).next();
    }
    None
}

/// Converts a positive SVG length to points the way AppKit does, or `None` for a relative or
/// unknown length.
///
/// User units and pixels count as one point each, while the physical units are converted at 72
/// dots per inch. Reporting the same size as AppKit keeps the title of the window equal to the
/// size the Quick Look extensions show for the same file.
fn length(value: &str) -> Option<f64> {
    let value = value.trim();
    let split = value
        .find(|character: char| !character.is_ascii_digit() && !"+-.".contains(character))
        .unwrap_or(value.len());
    let (number, unit) = value.split_at(split);
    let factor = match unit.trim() {
        "" | "px" | "pt" => 1.0,
        "pc" => 12.0,
        "in" => 72.0,
        "cm" => 72.0 / 2.54,
        "mm" => 72.0 / 25.4,
        // Percentages and font relative units say nothing about the intrinsic size.
        _ => return None,
    };
    let length = number.parse::<f64>().ok()? * factor;
    (length > 0.0).then_some(length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_svg_documents() {
        assert!(is_svg(b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"));
        assert!(is_svg(
            b"\xef\xbb\xbf<?xml version=\"1.0\"?>\n<!DOCTYPE svg>\n<!-- a comment -->\n<svg/>"
        ));
        assert!(!is_svg(b"\x89PNG\r\n\x1a\n"));
        assert!(!is_svg(
            b"<html><body><svg viewBox=\"0 0 1 1\"/></body></html>"
        ));
        assert!(!is_svg(b"<svgfoo/>"));
        // A root element that never closes is not a document either.
        assert!(!is_svg(b"<svg width=\"1\""));
    }

    #[test]
    fn a_prolog_of_any_length_is_skipped() {
        // The size comes from the root element, not from a tag quoted in the comment before it.
        let mut source = b"<!-- <svg width='1' height='1'/> ".to_vec();
        source.extend(std::iter::repeat_n(b'.', 4096));
        source.extend_from_slice(b" -->\n<svg width='120' height='60'/>");
        let svg = parse_svg(&source);
        assert_eq!(svg.size.width, 120.0);
        assert_eq!(svg.size.height, 60.0);
    }

    #[test]
    fn physical_lengths_are_converted_to_points() {
        // AppKit resolves an A4 drawing to the same size, so both report 595 by 842 points.
        let svg = parse_svg(b"<svg width=\"210mm\" height=\"297mm\"/>");
        assert_eq!(svg.size.width.round(), 595.0);
        assert_eq!(svg.size.height.round(), 842.0);
        assert_eq!(length("1in"), Some(72.0));
        assert_eq!(length("2pc"), Some(24.0));
        assert_eq!(length("96px"), Some(96.0));
        assert_eq!(length("3em"), None);
        assert_eq!(length("0"), None);
    }

    #[test]
    fn reads_the_size_from_the_dimensions() {
        let svg = parse_svg(b"<svg width=\"600px\" height=\"400\" viewBox=\"0 0 1 1\"/>");
        assert_eq!(svg.size.width, 600.0);
        assert_eq!(svg.size.height, 400.0);
    }

    #[test]
    fn falls_back_to_the_view_box_and_to_a_default() {
        // Relative dimensions say nothing about the intrinsic size.
        let svg = parse_svg(b"<svg width='100%' height='100%' viewBox='0 0 24 48'/>");
        assert_eq!(svg.size.width, 24.0);
        assert_eq!(svg.size.height, 48.0);

        let svg = parse_svg(b"<svg stroke-width=\"2\"/>");
        assert_eq!(svg.size.width, DEFAULT_SIZE.width);
        assert_eq!(svg.size.height, DEFAULT_SIZE.height);
    }

    #[test]
    fn embeds_the_document_as_an_image() {
        let svg = parse_svg(b"<svg/>");
        assert!(svg.html.contains("data:image/svg+xml;base64,PHN2Zy8+"));
    }
}
