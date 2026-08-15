/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A parser for the TinyVG binary vector image format.

use std::{error, fmt};

const MAX_ELEMENTS: usize = 1_000_000;

mod binary;
mod text;

/// A parsed TinyVG document.
#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    /// The intrinsic image size in display units.
    pub size: Size,
    /// Absolute drawing commands in file order.
    pub commands: Vec<Command>,
}

/// A two-dimensional size.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Size {
    /// The horizontal extent.
    pub width: f64,
    /// The vertical extent.
    pub height: f64,
}

/// A point in TinyVG's absolute, top-left-origin coordinate system.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Point {
    /// The horizontal coordinate.
    pub x: f64,
    /// The vertical coordinate.
    pub y: f64,
}

/// A normalized drawing command.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// Fill a path using the even-odd rule.
    Fill {
        /// The geometry to fill.
        path: Path,
        /// The fill paint.
        style: Style,
    },
    /// Stroke a path with round caps and joins.
    Stroke {
        /// The geometry to stroke.
        path: Path,
        /// The stroke paint.
        style: Style,
        /// The initial stroke width in display units.
        line_width: f64,
    },
}

/// A path made from one or more subpaths.
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    /// The path's independently started subpaths.
    pub subpaths: Vec<Subpath>,
}

/// A path segment with an absolute starting point.
#[derive(Clone, Debug, PartialEq)]
pub struct Subpath {
    /// The absolute starting point.
    pub start: Point,
    /// Path nodes in drawing order.
    pub nodes: Vec<PathNode>,
}

/// An absolute path node.
#[derive(Clone, Debug, PartialEq)]
pub struct PathNode {
    /// The geometric operation.
    pub operation: PathOperation,
    /// An optional new stroke width starting at this node.
    pub line_width: Option<f64>,
}

/// An absolute path operation.
#[derive(Clone, Debug, PartialEq)]
pub enum PathOperation {
    /// Draw a straight line to an absolute point.
    LineTo(Point),
    /// Draw a cubic Bezier curve to an absolute point.
    CubicTo {
        /// The first absolute control point.
        control_0: Point,
        /// The second absolute control point.
        control_1: Point,
        /// The absolute endpoint.
        to: Point,
    },
    /// Draw a quadratic Bezier curve to an absolute point.
    QuadraticTo {
        /// The absolute control point.
        control: Point,
        /// The absolute endpoint.
        to: Point,
    },
    /// Draw an elliptical arc to an absolute point.
    ArcTo {
        /// The horizontal radius.
        radius_x: f64,
        /// The vertical radius.
        radius_y: f64,
        /// The clockwise ellipse rotation in degrees.
        rotation: f64,
        /// Whether to use the arc larger than 180 degrees.
        large_arc: bool,
        /// Whether the arc bends left in TinyVG's coordinate system.
        sweep: bool,
        /// The absolute endpoint.
        to: Point,
    },
    /// Close the current subpath.
    Close,
}

/// A color's source color space.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorSpace {
    /// Standard RGB.
    Srgb,
    /// Linear extended sRGB (scRGB).
    LinearSrgb,
}

/// A straight-alpha RGBA color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    /// Red component.
    pub red: f64,
    /// Green component.
    pub green: f64,
    /// Blue component.
    pub blue: f64,
    /// Alpha component.
    pub alpha: f64,
    /// The interpretation of the RGB components.
    pub color_space: ColorSpace,
}

/// A shape paint style with resolved colors.
#[derive(Clone, Debug, PartialEq)]
pub enum Style {
    /// A uniform color.
    Solid(Color),
    /// A two-point linear gradient.
    LinearGradient {
        /// The gradient start point.
        start: Point,
        /// The gradient end point.
        end: Point,
        /// The color at the start point.
        start_color: Color,
        /// The color at the end point.
        end_color: Color,
    },
    /// A two-point radial gradient centered at `center`.
    RadialGradient {
        /// The center of the gradient.
        center: Point,
        /// A point on the outer circle.
        edge: Point,
        /// The color at the center.
        center_color: Color,
        /// The color at and beyond the edge.
        edge_color: Color,
    },
}

/// A TinyVG parsing error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// The input ended before a complete document was decoded.
    UnexpectedEnd,
    /// The file signature is not TinyVG.
    InvalidMagic,
    /// The TinyVG version is unsupported.
    UnsupportedVersion(u8),
    /// The custom color encoding requires out-of-band information.
    UnsupportedColorEncoding,
    /// An encoded value or command violates the specification.
    InvalidData(&'static str),
    /// An encoded collection exceeds the parser's safety limit.
    TooManyElements,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd => formatter.write_str("unexpected end of TinyVG data"),
            Self::InvalidMagic => formatter.write_str("not a TinyVG image"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported TinyVG version {version}")
            }
            Self::UnsupportedColorEncoding => {
                formatter.write_str("unsupported custom TinyVG color encoding")
            }
            Self::InvalidData(description) => {
                write!(formatter, "invalid TinyVG data: {description}")
            }
            Self::TooManyElements => formatter.write_str("TinyVG image contains too many elements"),
        }
    }
}

impl error::Error for Error {}

/// Parses a TinyVG binary document into absolute drawing commands.
pub fn parse(data: &[u8]) -> Result<Document, Error> {
    binary::parse(data)
}

/// Parses a TinyVG textual document into absolute drawing commands.
pub fn parse_text(source: &str) -> Result<Document, Error> {
    text::parse(source)
}

/// Parses either the binary or textual TinyVG representation.
pub fn parse_auto(data: &[u8]) -> Result<Document, Error> {
    if data.starts_with(&[0x72, 0x56]) {
        parse(data)
    } else {
        let source = std::str::from_utf8(data)
            .map_err(|_| Error::InvalidData("TinyVG text is not UTF-8"))?;
        parse_text(source.strip_prefix('\u{feff}').unwrap_or(source))
    }
}

/// Returns whether bytes start with a binary or textual TinyVG signature.
pub fn is_tinyvg(data: &[u8]) -> bool {
    data.starts_with(&[0x72, 0x56]) || text::is_text(data)
}
