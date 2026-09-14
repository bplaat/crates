/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::MAX_ITEMS;
use super::style::{GeometryStyle, GeometryValue};
use super::values::NumberParser;
use super::xml::Element;
use crate::vector::append_arc;
use crate::{PathSegment, Point, VectorDecodeError};

impl GeometryStyle {
    pub(super) fn path(
        &self,
        element: &Element<'_>,
    ) -> Result<Option<Vec<PathSegment>>, VectorDecodeError> {
        let value = |value: Option<GeometryValue>| value.map_or(0.0, |value| value.length_or(0.0));
        Ok(match element.name {
            "path" => Some(parse_path(element.attr("d").unwrap_or(""))?),
            "line" => Some(vec![
                PathSegment::MoveTo(Point {
                    x: value(self.x1),
                    y: value(self.y1),
                }),
                PathSegment::LineTo(Point {
                    x: value(self.x2),
                    y: value(self.y2),
                }),
            ]),
            "polyline" | "polygon" => {
                let points = point_list(element.attr("points").unwrap_or(""))?;
                if points.is_empty() {
                    Some(Vec::new())
                } else {
                    let mut path = vec![PathSegment::MoveTo(points[0])];
                    path.extend(points[1..].iter().copied().map(PathSegment::LineTo));
                    if element.name == "polygon" {
                        path.push(PathSegment::Close);
                    }
                    Some(path)
                }
            }
            "rect" => {
                let px = value(self.x);
                let py = value(self.y);
                let width = value(self.width);
                let height = value(self.height);
                if width <= 0.0 || height <= 0.0 {
                    return Ok(Some(Vec::new()));
                }
                let (rx, ry) = Self::radius_pair(self.rx, self.ry);
                Some(rect_path(px, py, width, height, rx, ry))
            }
            "circle" => {
                let cx = value(self.cx);
                let cy = value(self.cy);
                let r = value(self.r);
                if r <= 0.0 {
                    return Ok(Some(Vec::new()));
                }
                Some(ellipse_path(cx, cy, r, r))
            }
            "ellipse" => {
                let cx = value(self.cx);
                let cy = value(self.cy);
                let (rx, ry) = Self::radius_pair(self.rx, self.ry);
                if rx <= 0.0 || ry <= 0.0 {
                    return Ok(Some(Vec::new()));
                }
                Some(ellipse_path(cx, cy, rx, ry))
            }
            _ => None,
        })
    }

    fn radius_pair(first: Option<GeometryValue>, second: Option<GeometryValue>) -> (f64, f64) {
        let first = first.and_then(|value| match value {
            GeometryValue::Auto => None,
            GeometryValue::Length(value) => (value >= 0.0).then_some(value),
        });
        let second = second.and_then(|value| match value {
            GeometryValue::Auto => None,
            GeometryValue::Length(value) => (value >= 0.0).then_some(value),
        });
        (
            first.or(second).unwrap_or(0.0),
            second.or(first).unwrap_or(0.0),
        )
    }
}

pub(super) fn rect_path(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    rx: f64,
    ry: f64,
) -> Vec<PathSegment> {
    let rx = rx.max(0.0).min(width / 2.0);
    let ry = ry.max(0.0).min(height / 2.0);
    if rx == 0.0 || ry == 0.0 {
        return vec![
            PathSegment::MoveTo(Point { x, y }),
            PathSegment::LineTo(Point { x: x + width, y }),
            PathSegment::LineTo(Point {
                x: x + width,
                y: y + height,
            }),
            PathSegment::LineTo(Point { x, y: y + height }),
            PathSegment::Close,
        ];
    }
    let k = 0.552_284_749_830_793_6;
    vec![
        PathSegment::MoveTo(Point { x: x + rx, y }),
        PathSegment::LineTo(Point {
            x: x + width - rx,
            y,
        }),
        PathSegment::CubicTo {
            control_0: Point {
                x: x + width - rx + rx * k,
                y,
            },
            control_1: Point {
                x: x + width,
                y: y + ry - ry * k,
            },
            to: Point {
                x: x + width,
                y: y + ry,
            },
        },
        PathSegment::LineTo(Point {
            x: x + width,
            y: y + height - ry,
        }),
        PathSegment::CubicTo {
            control_0: Point {
                x: x + width,
                y: y + height - ry + ry * k,
            },
            control_1: Point {
                x: x + width - rx + rx * k,
                y: y + height,
            },
            to: Point {
                x: x + width - rx,
                y: y + height,
            },
        },
        PathSegment::LineTo(Point {
            x: x + rx,
            y: y + height,
        }),
        PathSegment::CubicTo {
            control_0: Point {
                x: x + rx - rx * k,
                y: y + height,
            },
            control_1: Point {
                x,
                y: y + height - ry + ry * k,
            },
            to: Point {
                x,
                y: y + height - ry,
            },
        },
        PathSegment::LineTo(Point { x, y: y + ry }),
        PathSegment::CubicTo {
            control_0: Point {
                x,
                y: y + ry - ry * k,
            },
            control_1: Point {
                x: x + rx - rx * k,
                y,
            },
            to: Point { x: x + rx, y },
        },
        PathSegment::Close,
    ]
}

pub(super) fn ellipse_path(cx: f64, cy: f64, rx: f64, ry: f64) -> Vec<PathSegment> {
    if rx < 0.0 || ry < 0.0 {
        return Vec::new();
    }
    let k = 0.552_284_749_830_793_6;
    vec![
        PathSegment::MoveTo(Point { x: cx + rx, y: cy }),
        PathSegment::CubicTo {
            control_0: Point {
                x: cx + rx,
                y: cy + ry * k,
            },
            control_1: Point {
                x: cx + rx * k,
                y: cy + ry,
            },
            to: Point { x: cx, y: cy + ry },
        },
        PathSegment::CubicTo {
            control_0: Point {
                x: cx - rx * k,
                y: cy + ry,
            },
            control_1: Point {
                x: cx - rx,
                y: cy + ry * k,
            },
            to: Point { x: cx - rx, y: cy },
        },
        PathSegment::CubicTo {
            control_0: Point {
                x: cx - rx,
                y: cy - ry * k,
            },
            control_1: Point {
                x: cx - rx * k,
                y: cy - ry,
            },
            to: Point { x: cx, y: cy - ry },
        },
        PathSegment::CubicTo {
            control_0: Point {
                x: cx + rx * k,
                y: cy - ry,
            },
            control_1: Point {
                x: cx + rx,
                y: cy - ry * k,
            },
            to: Point { x: cx + rx, y: cy },
        },
        PathSegment::Close,
    ]
}

pub(super) fn parse_path(source: &str) -> Result<Vec<PathSegment>, VectorDecodeError> {
    if source.len() > 1024 * 1024 {
        return Err(VectorDecodeError::ResourceLimit);
    }
    if matches!(source.trim(), "" | "none") || source.trim_start().starts_with('#') {
        return Ok(Vec::new());
    }
    match parse_path_strict(source) {
        Ok(path) => Ok(path),
        Err(VectorDecodeError::InvalidData) => {
            for (index, _) in source.char_indices().rev().take(256) {
                if index == 0 {
                    continue;
                }
                match parse_path_strict(&source[..index]) {
                    Ok(path) if !path.is_empty() => return Ok(path),
                    _ => {}
                }
            }
            Err(VectorDecodeError::InvalidData)
        }
        Err(error) => Err(error),
    }
}

fn parse_path_strict(source: &str) -> Result<Vec<PathSegment>, VectorDecodeError> {
    let mut parser = NumberParser::new_compact(source);
    let mut path = Vec::new();
    let mut command = None;
    let mut current = Point { x: 0.0, y: 0.0 };
    let mut start = current;
    let mut last_cubic = None;
    let mut last_quad = None;
    while !parser.done() {
        if let Some(next) = parser.command() {
            command = Some(next);
        }
        let cmd = command.ok_or(VectorDecodeError::InvalidData)?;
        let relative = cmd.is_ascii_lowercase();
        let upper = cmd.to_ascii_uppercase();
        let map = |x: f64, y: f64, current: Point| {
            if relative {
                Point {
                    x: current.x + x,
                    y: current.y + y,
                }
            } else {
                Point { x, y }
            }
        };
        match upper {
            'M' => {
                let to = map(parser.number()?, parser.number()?, current);
                path.push(PathSegment::MoveTo(to));
                current = to;
                start = to;
                command = Some(if relative { 'l' } else { 'L' });
            }
            'L' => {
                let to = map(parser.number()?, parser.number()?, current);
                path.push(PathSegment::LineTo(to));
                current = to;
            }
            'H' => {
                let value = parser.number()?;
                current.x = if relative { current.x + value } else { value };
                path.push(PathSegment::LineTo(current));
            }
            'V' => {
                let value = parser.number()?;
                current.y = if relative { current.y + value } else { value };
                path.push(PathSegment::LineTo(current));
            }
            'C' => {
                let c0 = map(parser.number()?, parser.number()?, current);
                let c1 = map(parser.number()?, parser.number()?, current);
                let to = map(parser.number()?, parser.number()?, current);
                path.push(PathSegment::CubicTo {
                    control_0: c0,
                    control_1: c1,
                    to,
                });
                current = to;
                last_cubic = Some(c1);
                last_quad = None;
                continue;
            }
            'S' => {
                let c0 = last_cubic.map_or(current, |p| Point {
                    x: current.x * 2.0 - p.x,
                    y: current.y * 2.0 - p.y,
                });
                let c1 = map(parser.number()?, parser.number()?, current);
                let to = map(parser.number()?, parser.number()?, current);
                path.push(PathSegment::CubicTo {
                    control_0: c0,
                    control_1: c1,
                    to,
                });
                current = to;
                last_cubic = Some(c1);
                last_quad = None;
                continue;
            }
            'Q' => {
                let control = map(parser.number()?, parser.number()?, current);
                let to = map(parser.number()?, parser.number()?, current);
                path.push(PathSegment::QuadraticTo { control, to });
                current = to;
                last_quad = Some(control);
                last_cubic = None;
                continue;
            }
            'T' => {
                let control = last_quad.map_or(current, |p| Point {
                    x: current.x * 2.0 - p.x,
                    y: current.y * 2.0 - p.y,
                });
                let to = map(parser.number()?, parser.number()?, current);
                path.push(PathSegment::QuadraticTo { control, to });
                current = to;
                last_quad = Some(control);
                last_cubic = None;
                continue;
            }
            'A' => {
                let rx = parser.number()?;
                let ry = parser.number()?;
                if rx < 0.0 || ry < 0.0 {
                    return Err(VectorDecodeError::InvalidData);
                }
                let rotation = parser.number()?;
                let large = parser.flag()?;
                let sweep = parser.flag()?;
                let to = map(parser.number()?, parser.number()?, current);
                append_arc(&mut path, current, to, rx, ry, rotation, large, sweep);
                current = to;
            }
            'Z' => {
                path.push(PathSegment::Close);
                current = start;
                command = None;
            }
            _ => return Err(VectorDecodeError::InvalidData),
        }
        last_cubic = None;
        last_quad = None;
        if path.len() > MAX_ITEMS {
            return Err(VectorDecodeError::ResourceLimit);
        }
    }
    Ok(path)
}

pub(super) fn point_list(source: &str) -> Result<Vec<Point>, VectorDecodeError> {
    let mut parser = NumberParser::new(source);
    let mut points = Vec::new();
    while !parser.done() {
        points.push(Point {
            x: parser.number()?,
            y: parser.number()?,
        });
        if points.len() > MAX_ITEMS {
            return Err(VectorDecodeError::ResourceLimit);
        }
    }
    Ok(points)
}
