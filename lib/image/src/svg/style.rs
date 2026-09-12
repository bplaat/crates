/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::values::{length, number, number_list, positive_number, unit_number};
use super::xml::Element;
use crate::{Color, FillRule, LineCap, LineJoin, StrokeStyle, VectorColorSpace, VectorDecodeError};

#[derive(Clone)]
pub(super) struct Style {
    pub(super) color: Color,
    pub(super) fill: PaintValue,
    pub(super) stroke: PaintValue,
    pub(super) fill_opacity: f64,
    pub(super) stroke_opacity: f64,
    pub(super) opacity: f64,
    pub(super) fill_rule: FillRule,
    pub(super) visible: bool,
    pub(super) displayed: bool,
    pub(super) stroke_style: StrokeStyle,
    pub(super) clip_path: Option<String>,
    pub(super) mask: Option<String>,
}

#[derive(Clone)]
pub(super) enum PaintValue {
    None,
    Color(Color),
    Reference(String),
}

impl Default for Style {
    fn default() -> Self {
        let black = Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
            color_space: VectorColorSpace::Srgb,
        };
        Self {
            color: black,
            fill: PaintValue::Color(black),
            stroke: PaintValue::None,
            fill_opacity: 1.0,
            stroke_opacity: 1.0,
            opacity: 1.0,
            fill_rule: FillRule::NonZero,
            visible: true,
            displayed: true,
            stroke_style: StrokeStyle {
                width: 1.0,
                line_cap: LineCap::Butt,
                line_join: LineJoin::Miter,
                miter_limit: 4.0,
                dash_array: Vec::new(),
                dash_offset: 0.0,
            },
            clip_path: None,
            mask: None,
        }
    }
}

impl Element<'_> {
    pub(super) fn attr(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|&&(prefix, local, _)| prefix.is_empty() && local == name)
            .map(|&(_, _, value)| value)
    }

    pub(super) fn attr_prefixed(&self, wanted_prefix: &str, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|&&(prefix, local, _)| prefix == wanted_prefix && local == name)
            .map(|&(_, _, value)| value)
    }
}

pub(super) fn skips_unsupported_subtree(name: &str) -> bool {
    matches!(
        name,
        "filter"
            | "pattern"
            | "text"
            | "tspan"
            | "textPath"
            | "image"
            | "marker"
            | "style"
            | "script"
            | "foreignObject"
            | "animate"
            | "animateTransform"
            | "set"
    )
}

pub(super) fn local_reference(value: &str) -> Result<&str, VectorDecodeError> {
    value
        .strip_prefix('#')
        .filter(|id| !id.is_empty())
        .ok_or(VectorDecodeError::UnsupportedFeature("external URL"))
}

pub(super) fn parse_style(
    element: &Element<'_>,
    inherited: &Style,
) -> Result<Style, VectorDecodeError> {
    let mut style = inherited.clone();
    style.opacity = 1.0;
    style.clip_path = None;
    style.mask = None;
    style.displayed = true;
    let mut declarations = Vec::new();
    for &(prefix, name, value) in &element.attributes {
        if prefix.is_empty() {
            declarations.push((name, value, false));
        }
    }
    if let Some(inline) = element.attr("style") {
        for declaration in inline.split(';') {
            let Some((name, value)) = declaration.split_once(':') else {
                if !declaration.trim().is_empty() {
                    return Err(VectorDecodeError::InvalidData);
                }
                continue;
            };
            declarations.push((name.trim(), value.trim(), true));
        }
    }
    for &(name, value, fallback) in &declarations {
        if name == "color" && value != "inherit" {
            match parse_color(value, style.color) {
                Ok(color) => style.color = color,
                Err(error) if !fallback => return Err(error),
                Err(_) => {}
            }
        }
    }
    for (name, value, fallback) in declarations {
        let result = (|| -> Result<(), VectorDecodeError> {
            match name {
                "color" | "font-size" if value == "inherit" => {}
                "color" => {}
                "fill" | "stroke" | "fill-opacity" | "stroke-opacity" | "fill-rule"
                | "clip-rule" | "visibility" | "stroke-width" | "stroke-linecap"
                | "stroke-linejoin" | "stroke-miterlimit" | "stroke-dasharray"
                | "stroke-dashoffset"
                    if value == "inherit" => {}
                "fill" => style.fill = parse_paint(value, style.color)?,
                "stroke" => style.stroke = parse_paint(value, style.color)?,
                "fill-opacity" => style.fill_opacity = unit_number(value)?,
                "stroke-opacity" => style.stroke_opacity = unit_number(value)?,
                "opacity" => style.opacity = unit_number(value)?,
                "fill-rule" | "clip-rule" => {
                    style.fill_rule = match value {
                        "nonzero" => FillRule::NonZero,
                        "evenodd" => FillRule::EvenOdd,
                        _ => return Err(VectorDecodeError::InvalidData),
                    }
                }
                "visibility" => {
                    style.visible = match value {
                        "visible" => true,
                        "hidden" | "collapse" => false,
                        _ => return Err(VectorDecodeError::InvalidData),
                    }
                }
                "display" => {
                    style.displayed = match value {
                        "none" => false,
                        "inline" | "block" => true,
                        _ => return Err(VectorDecodeError::UnsupportedFeature("display value")),
                    }
                }
                "stroke-width" => {
                    let width = length(value, 1.0)?;
                    if width < 0.0 {
                        return Err(VectorDecodeError::InvalidData);
                    }
                    style.stroke_style.width = width;
                }
                "stroke-linecap" => {
                    style.stroke_style.line_cap = match value {
                        "butt" => LineCap::Butt,
                        "round" => LineCap::Round,
                        "square" => LineCap::Square,
                        _ => return Err(VectorDecodeError::InvalidData),
                    }
                }
                "stroke-linejoin" => {
                    style.stroke_style.line_join = match value {
                        "miter" | "miter-clip" => LineJoin::Miter,
                        "round" => LineJoin::Round,
                        "bevel" | "arcs" => LineJoin::Bevel,
                        _ => return Err(VectorDecodeError::InvalidData),
                    }
                }
                "stroke-miterlimit" => style.stroke_style.miter_limit = positive_number(value)?,
                "stroke-dasharray" => {
                    let mut values = if value == "none" {
                        Vec::new()
                    } else {
                        number_list(value)?
                    };
                    if values.iter().any(|value| *value < 0.0) {
                        return Err(VectorDecodeError::InvalidData);
                    }
                    if values.len() % 2 == 1 {
                        values.extend(values.clone());
                    }
                    style.stroke_style.dash_array = values;
                }
                "stroke-dashoffset" => style.stroke_style.dash_offset = number(value)?,
                "clip-path" => {
                    style.clip_path = if value == "none" {
                        None
                    } else {
                        Some(
                            value
                                .strip_prefix("url(")
                                .and_then(|value| value.strip_suffix(')'))
                                .ok_or(VectorDecodeError::InvalidData)?
                                .trim()
                                .to_owned(),
                        )
                    }
                }
                "mask" => {
                    style.mask = if value == "none" {
                        None
                    } else {
                        Some(
                            value
                                .strip_prefix("url(")
                                .and_then(|value| value.strip_suffix(')'))
                                .ok_or(VectorDecodeError::InvalidData)?
                                .trim()
                                .to_owned(),
                        )
                    }
                }
                "transform"
                | "d"
                | "x"
                | "y"
                | "x1"
                | "y1"
                | "x2"
                | "y2"
                | "cx"
                | "cy"
                | "r"
                | "rx"
                | "ry"
                | "width"
                | "height"
                | "points"
                | "viewBox"
                | "preserveAspectRatio"
                | "id"
                | "xmlns"
                | "href" => {}
                "maskUnits" | "maskContentUnits" | "mask-type" => {}
                _ if name.starts_with("xmlns") => {}
                _ => {}
            }
            Ok(())
        })();
        if let Err(error) = result {
            if matches!(error, VectorDecodeError::UnsupportedFeature(_)) {
                continue;
            }
            if !fallback {
                return Err(error);
            }
        }
    }
    Ok(style)
}

fn parse_paint(value: &str, current: Color) -> Result<PaintValue, VectorDecodeError> {
    if value == "none" {
        Ok(PaintValue::None)
    } else if value == "currentColor" {
        Ok(PaintValue::Color(current))
    } else if value.starts_with("url(") && value.ends_with(')') {
        Ok(PaintValue::Reference(
            value[4..value.len() - 1].trim().to_owned(),
        ))
    } else {
        Ok(PaintValue::Color(parse_color(value, current)?))
    }
}

pub(super) fn parse_color(value: &str, current: Color) -> Result<Color, VectorDecodeError> {
    if value == "currentColor" {
        return Ok(current);
    }
    let rgba = if value == "transparent" {
        [0, 0, 0, 0]
    } else if let Some(hex) = value.strip_prefix('#') {
        parse_hex(hex)?
    } else if value.starts_with("rgb(") || value.starts_with("rgba(") {
        parse_rgb(value)?
    } else {
        named_color(value).ok_or(VectorDecodeError::InvalidData)?
    };
    Ok(Color {
        red: f64::from(rgba[0]) / 255.0,
        green: f64::from(rgba[1]) / 255.0,
        blue: f64::from(rgba[2]) / 255.0,
        alpha: f64::from(rgba[3]) / 255.0,
        color_space: VectorColorSpace::Srgb,
    })
}

fn parse_hex(value: &str) -> Result<[u8; 4], VectorDecodeError> {
    let digit = |byte: u8| {
        char::from(byte)
            .to_digit(16)
            .map(|v| v as u8)
            .ok_or(VectorDecodeError::InvalidData)
    };
    match value.as_bytes() {
        [r, g, b] => Ok([digit(*r)? * 17, digit(*g)? * 17, digit(*b)? * 17, 255]),
        [r, g, b, a] => Ok([
            digit(*r)? * 17,
            digit(*g)? * 17,
            digit(*b)? * 17,
            digit(*a)? * 17,
        ]),
        [r0, r1, g0, g1, b0, b1] => Ok([
            digit(*r0)? * 16 + digit(*r1)?,
            digit(*g0)? * 16 + digit(*g1)?,
            digit(*b0)? * 16 + digit(*b1)?,
            255,
        ]),
        [r0, r1, g0, g1, b0, b1, a0, a1] => Ok([
            digit(*r0)? * 16 + digit(*r1)?,
            digit(*g0)? * 16 + digit(*g1)?,
            digit(*b0)? * 16 + digit(*b1)?,
            digit(*a0)? * 16 + digit(*a1)?,
        ]),
        _ => Err(VectorDecodeError::InvalidData),
    }
}

fn parse_rgb(value: &str) -> Result<[u8; 4], VectorDecodeError> {
    let rgba = value.starts_with("rgba(");
    let inside = value
        .split_once('(')
        .and_then(|(_, rest)| rest.strip_suffix(')'))
        .ok_or(VectorDecodeError::InvalidData)?;
    let component = |part: &str| -> Result<u8, VectorDecodeError> {
        if let Some(value) = part.strip_suffix('%') {
            Ok((number(value)?.clamp(0.0, 100.0) * 2.55).round() as u8)
        } else {
            Ok(number(part)?.clamp(0.0, 255.0).round() as u8)
        }
    };
    let alpha = |part: &str| -> Result<u8, VectorDecodeError> {
        if let Some(value) = part.strip_suffix('%') {
            Ok((number(value)?.clamp(0.0, 100.0) * 2.55).round() as u8)
        } else {
            Ok((number(part)?.clamp(0.0, 1.0) * 255.0).round() as u8)
        }
    };
    if inside.contains(',') {
        if inside.contains('/') {
            return Err(VectorDecodeError::InvalidData);
        }
        let parts = inside.split(',').map(str::trim).collect::<Vec<_>>();
        let expected = if rgba { 4 } else { 3 };
        if parts.len() != expected || parts.iter().any(|part| part.is_empty()) {
            return Err(VectorDecodeError::InvalidData);
        }
        Ok([
            component(parts[0])?,
            component(parts[1])?,
            component(parts[2])?,
            if rgba { alpha(parts[3])? } else { 255 },
        ])
    } else {
        let mut sides = inside.split('/');
        let colors = sides
            .next()
            .ok_or(VectorDecodeError::InvalidData)?
            .split_whitespace()
            .collect::<Vec<_>>();
        if colors.len() != 3 {
            return Err(VectorDecodeError::InvalidData);
        }
        let alpha = if let Some(value) = sides.next() {
            let values = value.split_whitespace().collect::<Vec<_>>();
            if values.len() != 1 || sides.next().is_some() {
                return Err(VectorDecodeError::InvalidData);
            }
            alpha(values[0])?
        } else {
            255
        };
        Ok([
            component(colors[0])?,
            component(colors[1])?,
            component(colors[2])?,
            alpha,
        ])
    }
}

fn named_color(value: &str) -> Option<[u8; 4]> {
    let rgb = match value.to_ascii_lowercase().as_str() {
        "black" => 0x000000,
        "white" => 0xffffff,
        "red" => 0xff0000,
        "green" => 0x008000,
        "blue" => 0x0000ff,
        "yellow" => 0xffff00,
        "gray" | "grey" => 0x808080,
        "silver" => 0xc0c0c0,
        "maroon" => 0x800000,
        "purple" => 0x800080,
        "fuchsia" => 0xff00ff,
        "lime" => 0x00ff00,
        "olive" => 0x808000,
        "navy" => 0x000080,
        "teal" => 0x008080,
        "aqua" | "cyan" => 0x00ffff,
        "orange" => 0xffa500,
        "magenta" => 0xff00ff,
        "aliceblue" => 0xf0f8ff,
        "antiquewhite" => 0xfaebd7,
        "aquamarine" => 0x7fffd4,
        "azure" => 0xf0ffff,
        "beige" => 0xf5f5dc,
        "bisque" => 0xffe4c4,
        "blanchedalmond" => 0xffebcd,
        "blueviolet" => 0x8a2be2,
        "brown" => 0xa52a2a,
        "burlywood" => 0xdeb887,
        "cadetblue" => 0x5f9ea0,
        "chartreuse" => 0x7fff00,
        "chocolate" => 0xd2691e,
        "coral" => 0xff7f50,
        "cornflowerblue" => 0x6495ed,
        "cornsilk" => 0xfff8dc,
        "crimson" => 0xdc143c,
        "darkblue" => 0x00008b,
        "darkcyan" => 0x008b8b,
        "darkgoldenrod" => 0xb8860b,
        "darkgray" | "darkgrey" => 0xa9a9a9,
        "darkgreen" => 0x006400,
        "darkkhaki" => 0xbdb76b,
        "darkmagenta" => 0x8b008b,
        "darkolivegreen" => 0x556b2f,
        "darkorange" => 0xff8c00,
        "darkorchid" => 0x9932cc,
        "darkred" => 0x8b0000,
        "darksalmon" => 0xe9967a,
        "darkseagreen" => 0x8fbc8f,
        "darkslateblue" => 0x483d8b,
        "darkslategray" | "darkslategrey" => 0x2f4f4f,
        "darkturquoise" => 0x00ced1,
        "darkviolet" => 0x9400d3,
        "deeppink" => 0xff1493,
        "deepskyblue" => 0x00bfff,
        "dimgray" | "dimgrey" => 0x696969,
        "dodgerblue" => 0x1e90ff,
        "firebrick" => 0xb22222,
        "floralwhite" => 0xfffaf0,
        "forestgreen" => 0x228b22,
        "gainsboro" => 0xdcdcdc,
        "ghostwhite" => 0xf8f8ff,
        "gold" => 0xffd700,
        "goldenrod" => 0xdaa520,
        "greenyellow" => 0xadff2f,
        "honeydew" => 0xf0fff0,
        "hotpink" => 0xff69b4,
        "indianred" => 0xcd5c5c,
        "indigo" => 0x4b0082,
        "ivory" => 0xfffff0,
        "khaki" => 0xf0e68c,
        "lavender" => 0xe6e6fa,
        "lavenderblush" => 0xfff0f5,
        "lawngreen" => 0x7cfc00,
        "lemonchiffon" => 0xfffacd,
        "lightblue" => 0xadd8e6,
        "lightcoral" => 0xf08080,
        "lightcyan" => 0xe0ffff,
        "lightgoldenrodyellow" => 0xfafad2,
        "lightgray" | "lightgrey" => 0xd3d3d3,
        "lightgreen" => 0x90ee90,
        "lightpink" => 0xffb6c1,
        "lightsalmon" => 0xffa07a,
        "lightseagreen" => 0x20b2aa,
        "lightskyblue" => 0x87cefa,
        "lightslategray" | "lightslategrey" => 0x778899,
        "lightsteelblue" => 0xb0c4de,
        "lightyellow" => 0xffffe0,
        "limegreen" => 0x32cd32,
        "linen" => 0xfaf0e6,
        "mediumaquamarine" => 0x66cdaa,
        "mediumblue" => 0x0000cd,
        "mediumorchid" => 0xba55d3,
        "mediumpurple" => 0x9370db,
        "mediumseagreen" => 0x3cb371,
        "mediumslateblue" => 0x7b68ee,
        "mediumspringgreen" => 0x00fa9a,
        "mediumturquoise" => 0x48d1cc,
        "mediumvioletred" => 0xc71585,
        "midnightblue" => 0x191970,
        "mintcream" => 0xf5fffa,
        "mistyrose" => 0xffe4e1,
        "moccasin" => 0xffe4b5,
        "navajowhite" => 0xffdead,
        "oldlace" => 0xfdf5e6,
        "olivedrab" => 0x6b8e23,
        "orangered" => 0xff4500,
        "orchid" => 0xda70d6,
        "palegoldenrod" => 0xeee8aa,
        "palegreen" => 0x98fb98,
        "paleturquoise" => 0xafeeee,
        "palevioletred" => 0xdb7093,
        "papayawhip" => 0xffefd5,
        "peachpuff" => 0xffdab9,
        "peru" => 0xcd853f,
        "pink" => 0xffc0cb,
        "plum" => 0xdda0dd,
        "powderblue" => 0xb0e0e6,
        "rebeccapurple" => 0x663399,
        "rosybrown" => 0xbc8f8f,
        "royalblue" => 0x4169e1,
        "saddlebrown" => 0x8b4513,
        "salmon" => 0xfa8072,
        "sandybrown" => 0xf4a460,
        "seagreen" => 0x2e8b57,
        "seashell" => 0xfff5ee,
        "sienna" => 0xa0522d,
        "skyblue" => 0x87ceeb,
        "slateblue" => 0x6a5acd,
        "slategray" | "slategrey" => 0x708090,
        "snow" => 0xfffafa,
        "springgreen" => 0x00ff7f,
        "steelblue" => 0x4682b4,
        "tan" => 0xd2b48c,
        "thistle" => 0xd8bfd8,
        "tomato" => 0xff6347,
        "turquoise" => 0x40e0d0,
        "violet" => 0xee82ee,
        "wheat" => 0xf5deb3,
        "whitesmoke" => 0xf5f5f5,
        "yellowgreen" => 0x9acd32,
        _ => return None,
    };
    Some([(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255])
}
