/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::{Rect, Transform, VectorDecodeError};

pub(super) struct NumberParser<'a> {
    source: &'a [u8],
    position: usize,
    has_number: bool,
    has_separator: bool,
    compact: bool,
}
impl<'a> NumberParser<'a> {
    pub(super) const fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            position: 0,
            has_number: false,
            has_separator: false,
            compact: false,
        }
    }

    pub(super) const fn new_compact(source: &'a str) -> Self {
        Self {
            compact: true,
            ..Self::new(source)
        }
    }

    fn whitespace(&mut self) {
        let start = self.position;
        while self
            .source
            .get(self.position)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.position += 1;
        }
        self.has_separator |= self.has_number && self.position != start;
    }

    fn number_start(&mut self) -> Result<(), VectorDecodeError> {
        self.whitespace();
        if self.has_number && self.source.get(self.position) == Some(&b',') {
            self.position += 1;
            self.whitespace();
            self.has_separator = true;
        }
        if self.has_number && !self.compact && !self.has_separator {
            return Err(VectorDecodeError::InvalidData);
        }
        self.has_separator = false;
        Ok(())
    }

    pub(super) fn number(&mut self) -> Result<f64, VectorDecodeError> {
        self.number_start()?;
        let start = self.position;
        if self
            .source
            .get(self.position)
            .is_some_and(|b| matches!(b, b'+' | b'-'))
        {
            self.position += 1;
        }
        let mut digits = 0;
        while self
            .source
            .get(self.position)
            .is_some_and(u8::is_ascii_digit)
        {
            self.position += 1;
            digits += 1;
        }
        if self.source.get(self.position) == Some(&b'.') {
            self.position += 1;
            while self
                .source
                .get(self.position)
                .is_some_and(u8::is_ascii_digit)
            {
                self.position += 1;
                digits += 1;
            }
        }
        if digits == 0 {
            return Err(VectorDecodeError::InvalidData);
        }
        if self
            .source
            .get(self.position)
            .is_some_and(|b| matches!(b, b'e' | b'E'))
        {
            self.position += 1;
            if self
                .source
                .get(self.position)
                .is_some_and(|b| matches!(b, b'+' | b'-'))
            {
                self.position += 1;
            }
            let exponent = self.position;
            while self
                .source
                .get(self.position)
                .is_some_and(u8::is_ascii_digit)
            {
                self.position += 1;
            }
            if exponent == self.position {
                return Err(VectorDecodeError::InvalidData);
            }
        }
        let value = std::str::from_utf8(&self.source[start..self.position])
            .map_err(|_| VectorDecodeError::InvalidData)?
            .parse::<f64>()
            .map_err(|_| VectorDecodeError::InvalidData)?;
        if value.is_finite() {
            self.has_number = true;
            Ok(value)
        } else {
            Err(VectorDecodeError::InvalidData)
        }
    }

    pub(super) fn flag(&mut self) -> Result<bool, VectorDecodeError> {
        self.number_start()?;
        let value = match self.source.get(self.position) {
            Some(b'0') => {
                self.position += 1;
                false
            }
            Some(b'1') => {
                self.position += 1;
                true
            }
            _ => return Err(VectorDecodeError::InvalidData),
        };
        self.has_number = true;
        Ok(value)
    }

    pub(super) fn command(&mut self) -> Option<char> {
        self.whitespace();
        let byte = *self.source.get(self.position)?;
        if byte.is_ascii_alphabetic() {
            self.position += 1;
            self.has_number = false;
            self.has_separator = false;
            Some(char::from(byte))
        } else {
            None
        }
    }

    pub(super) fn done(&mut self) -> bool {
        self.whitespace();
        self.position == self.source.len()
    }
}

pub(super) fn number_list(source: &str) -> Result<Vec<f64>, VectorDecodeError> {
    let mut parser = NumberParser::new(source);
    let mut values = Vec::new();
    while !parser.done() {
        values.push(parser.number()?);
    }
    Ok(values)
}

pub(super) fn number(source: &str) -> Result<f64, VectorDecodeError> {
    let mut parser = NumberParser::new(source);
    let value = parser.number()?;
    if parser.done() {
        Ok(value)
    } else {
        Err(VectorDecodeError::InvalidData)
    }
}

pub(super) fn positive_number(source: &str) -> Result<f64, VectorDecodeError> {
    let value = number(source)?;
    if value >= 0.0 {
        Ok(value)
    } else {
        Err(VectorDecodeError::InvalidData)
    }
}

pub(super) fn unit_number(source: &str) -> Result<f64, VectorDecodeError> {
    let value = if let Some(value) = source.strip_suffix('%') {
        number(value)? / 100.0
    } else {
        number(source)?
    };
    Ok(value.clamp(0.0, 1.0))
}

pub(super) fn length(source: &str, percentage: f64) -> Result<f64, VectorDecodeError> {
    let units = ["px", "pt", "pc", "in", "cm", "mm", "Q", "%"];
    let unit = units.into_iter().find(|unit| source.ends_with(unit));
    let numeric = unit.map_or(source, |unit| &source[..source.len() - unit.len()]);
    let value = if unit.is_none() {
        match number(numeric) {
            Ok(value) => value,
            Err(error) => {
                let unit_start = numeric.bytes().position(|byte| byte.is_ascii_alphabetic());
                if unit_start.is_some_and(|index| {
                    index > 0
                        && numeric[index..]
                            .bytes()
                            .all(|byte| byte.is_ascii_alphabetic())
                        && number(&numeric[..index]).is_ok()
                }) {
                    return Err(VectorDecodeError::UnsupportedFeature("length unit"));
                }
                return Err(error);
            }
        }
    } else {
        number(numeric)?
    };
    Ok(value
        * match unit {
            None | Some("px" | "pt") => 1.0,
            Some("pc") => 12.0,
            Some("in") => 72.0,
            Some("cm") => 72.0 / 2.54,
            Some("mm") => 72.0 / 25.4,
            Some("Q") => 72.0 / 101.6,
            Some("%") => percentage / 100.0,
            _ => return Err(VectorDecodeError::UnsupportedFeature("length unit")),
        })
}

pub(super) fn compatible_length(
    source: Option<&str>,
    percentage: f64,
) -> Result<Option<f64>, VectorDecodeError> {
    match source.map(|source| length(source, percentage)).transpose() {
        Err(VectorDecodeError::UnsupportedFeature(_)) => Ok(None),
        result => result,
    }
}

pub(super) fn parse_view_box(source: &str) -> Result<Rect, VectorDecodeError> {
    let values = number_list(source)?;
    if values.len() != 4 || values[2] <= 0.0 || values[3] <= 0.0 {
        return Err(VectorDecodeError::InvalidData);
    }
    Ok(Rect {
        x: values[0],
        y: values[1],
        width: values[2],
        height: values[3],
    })
}

pub(super) fn view_box_transform(
    view: Rect,
    width: f64,
    height: f64,
    aspect: Option<&str>,
) -> Result<Transform, VectorDecodeError> {
    let aspect = aspect.unwrap_or("xMidYMid meet").trim();
    if aspect == "none" {
        return Ok(Transform {
            a: width / view.width,
            d: height / view.height,
            e: -view.x * width / view.width,
            f: -view.y * height / view.height,
            b: 0.0,
            c: 0.0,
        });
    }
    let mut parts = aspect.split_whitespace();
    let align = parts.next().ok_or(VectorDecodeError::InvalidData)?;
    let mode = parts.next().unwrap_or("meet");
    if parts.next().is_some() {
        return Err(VectorDecodeError::InvalidData);
    }
    let sx = width / view.width;
    let sy = height / view.height;
    let scale = match mode {
        "meet" => sx.min(sy),
        "slice" => sx.max(sy),
        _ => return Err(VectorDecodeError::InvalidData),
    };
    let extra_x = width - view.width * scale;
    let extra_y = height - view.height * scale;
    let ax = if align.starts_with("xMin") {
        0.0
    } else if align.starts_with("xMid") {
        0.5
    } else if align.starts_with("xMax") {
        1.0
    } else {
        return Err(VectorDecodeError::InvalidData);
    };
    let ay = if align.ends_with("YMin") {
        0.0
    } else if align.ends_with("YMid") {
        0.5
    } else if align.ends_with("YMax") {
        1.0
    } else {
        return Err(VectorDecodeError::InvalidData);
    };
    Ok(Transform {
        a: scale,
        b: 0.0,
        c: 0.0,
        d: scale,
        e: extra_x * ax - view.x * scale,
        f: extra_y * ay - view.y * scale,
    })
}

pub(super) fn parse_transform(source: &str) -> Result<Transform, VectorDecodeError> {
    if source.trim() == "none" {
        return Ok(Transform::IDENTITY);
    }
    let mut rest = source.trim();
    let mut result = Transform::IDENTITY;
    while !rest.is_empty() {
        let open = rest.find('(').ok_or(VectorDecodeError::InvalidData)?;
        let close = rest[open + 1..]
            .find(')')
            .map(|v| v + open + 1)
            .ok_or(VectorDecodeError::InvalidData)?;
        let name = rest[..open].trim();
        let values = number_list(&rest[open + 1..close])?;
        let transform = match (name, values.as_slice()) {
            ("matrix", [a, b, c, d, e, f]) => Transform {
                a: *a,
                b: *b,
                c: *c,
                d: *d,
                e: *e,
                f: *f,
            },
            ("translate", [x]) => Transform {
                e: *x,
                ..Transform::IDENTITY
            },
            ("translate", [x, y]) => Transform {
                e: *x,
                f: *y,
                ..Transform::IDENTITY
            },
            ("scale", [x]) => Transform {
                a: *x,
                d: *x,
                ..Transform::IDENTITY
            },
            ("scale", [x, y]) => Transform {
                a: *x,
                d: *y,
                ..Transform::IDENTITY
            },
            ("rotate", [angle]) => rotation(*angle),
            ("rotate", [angle, cx, cy]) => Transform {
                e: -*cx,
                f: -*cy,
                ..Transform::IDENTITY
            }
            .then(rotation(*angle))
            .then(Transform {
                e: *cx,
                f: *cy,
                ..Transform::IDENTITY
            }),
            ("skewX", [angle]) => Transform {
                c: angle.to_radians().tan(),
                ..Transform::IDENTITY
            },
            ("skewY", [angle]) => Transform {
                b: angle.to_radians().tan(),
                ..Transform::IDENTITY
            },
            _ => return Err(VectorDecodeError::InvalidData),
        };
        result = result.then(transform);
        let tail = &rest[close + 1..];
        let mut next = tail.trim_start_matches(|character: char| character.is_ascii_whitespace());
        let had_whitespace = next.len() != tail.len();
        if let Some(after_comma) = next.strip_prefix(',') {
            next =
                after_comma.trim_start_matches(|character: char| character.is_ascii_whitespace());
            if next.is_empty() || next.starts_with(',') {
                return Err(VectorDecodeError::InvalidData);
            }
        } else if !next.is_empty() && !had_whitespace {
            return Err(VectorDecodeError::InvalidData);
        }
        rest = next;
    }
    Ok(result)
}

pub(super) fn rotation(angle: f64) -> Transform {
    let (sin, cos) = angle.to_radians().sin_cos();
    Transform {
        a: cos,
        b: sin,
        c: -sin,
        d: cos,
        e: 0.0,
        f: 0.0,
    }
}
