/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::{
    Color, ColorSpace, Command, Document, Error, MAX_ELEMENTS, Path, PathNode, PathOperation,
    Point, Size, Style, Subpath,
};

pub(super) fn parse(data: &[u8]) -> Result<Document, Error> {
    Parser::new(data)?.parse()
}

#[derive(Clone, Copy)]
enum CoordinateRange {
    Reduced,
    Default,
    Enhanced,
}

#[derive(Clone, Copy)]
enum StyleKind {
    Flat,
    Linear,
    Radial,
}

struct Parser<'a> {
    data: &'a [u8],
    position: usize,
    scale: f64,
    coordinate_range: CoordinateRange,
    size: Size,
    colors: Vec<Color>,
    commands: Vec<Command>,
}

impl<'a> Parser<'a> {
    fn new(data: &'a [u8]) -> Result<Self, Error> {
        let mut parser = Self {
            data,
            position: 0,
            scale: 1.0,
            coordinate_range: CoordinateRange::Default,
            size: Size {
                width: 0.0,
                height: 0.0,
            },
            colors: Vec::new(),
            commands: Vec::new(),
        };
        if parser.read_exact(2)? != [0x72, 0x56] {
            return Err(Error::InvalidMagic);
        }
        let version = parser.read_u8()?;
        if version != 1 {
            return Err(Error::UnsupportedVersion(version));
        }
        let flags = parser.read_u8()?;
        parser.scale = f64::from(1u32 << (flags & 0x0f));
        let color_encoding = (flags >> 4) & 0x03;
        parser.coordinate_range = match flags >> 6 {
            0 => CoordinateRange::Default,
            1 => CoordinateRange::Reduced,
            2 => CoordinateRange::Enhanced,
            _ => return Err(Error::InvalidData("reserved coordinate range")),
        };
        parser.size = Size {
            width: parser.read_dimension()?,
            height: parser.read_dimension()?,
        };
        let color_count = parser.read_count(false)?;
        parser.colors = parser.read_colors(color_encoding, color_count)?;
        Ok(parser)
    }

    fn parse(mut self) -> Result<Document, Error> {
        loop {
            let command_byte = self.read_u8()?;
            if command_byte == 0 {
                break;
            }
            let command_index = command_byte & 0x3f;
            let primary_style = Self::style_kind(command_byte >> 6)?;
            match command_index {
                1 => self.fill_polygon(primary_style)?,
                2 => self.fill_rectangles(primary_style)?,
                3 => self.fill_path(primary_style)?,
                4 => self.draw_lines(primary_style)?,
                5 => self.draw_line_segments(primary_style, true)?,
                6 => self.draw_line_segments(primary_style, false)?,
                7 => self.draw_path(primary_style)?,
                8 => self.outline_fill_polygon(primary_style)?,
                9 => self.outline_fill_rectangles(primary_style)?,
                10 => self.outline_fill_path(primary_style)?,
                _ => return Err(Error::InvalidData("unknown command")),
            }
            if self.commands.len() > MAX_ELEMENTS {
                return Err(Error::TooManyElements);
            }
        }
        Ok(Document {
            size: self.size,
            commands: self.commands,
        })
    }

    fn fill_polygon(&mut self, style_kind: StyleKind) -> Result<(), Error> {
        let count = self.read_count(true)?;
        if count < 3 {
            return Err(Error::InvalidData("polygon has fewer than three points"));
        }
        let style = self.read_style(style_kind)?;
        let path = self.read_polygon(count, true)?;
        self.commands.push(Command::Fill { path, style });
        Ok(())
    }

    fn fill_rectangles(&mut self, style_kind: StyleKind) -> Result<(), Error> {
        let count = self.read_count(true)?;
        let style = self.read_style(style_kind)?;
        for _ in 0..count {
            let path = self.read_rectangle()?;
            self.commands.push(Command::Fill {
                path,
                style: style.clone(),
            });
        }
        Ok(())
    }

    fn fill_path(&mut self, style_kind: StyleKind) -> Result<(), Error> {
        let count = self.read_count(true)?;
        let style = self.read_style(style_kind)?;
        let path = self.read_path(count)?;
        self.commands.push(Command::Fill { path, style });
        Ok(())
    }

    fn draw_lines(&mut self, style_kind: StyleKind) -> Result<(), Error> {
        let count = self.read_count(true)?;
        let style = self.read_style(style_kind)?;
        let line_width = self.read_line_width()?;
        let mut subpaths = Vec::with_capacity(count);
        for _ in 0..count {
            let start = self.read_point()?;
            let end = self.read_point()?;
            subpaths.push(Subpath {
                start,
                nodes: vec![PathNode {
                    operation: PathOperation::LineTo(end),
                    line_width: None,
                }],
            });
        }
        self.commands.push(Command::Stroke {
            path: Path { subpaths },
            style,
            line_width,
        });
        Ok(())
    }

    fn draw_line_segments(&mut self, style_kind: StyleKind, close: bool) -> Result<(), Error> {
        let count = self.read_count(true)?;
        let style = self.read_style(style_kind)?;
        let line_width = self.read_line_width()?;
        let path = self.read_polygon(count, close)?;
        self.commands.push(Command::Stroke {
            path,
            style,
            line_width,
        });
        Ok(())
    }

    fn draw_path(&mut self, style_kind: StyleKind) -> Result<(), Error> {
        let count = self.read_count(true)?;
        let style = self.read_style(style_kind)?;
        let line_width = self.read_line_width()?;
        let path = self.read_path(count)?;
        self.commands.push(Command::Stroke {
            path,
            style,
            line_width,
        });
        Ok(())
    }

    fn outline_header(
        &mut self,
        primary_style: StyleKind,
    ) -> Result<(usize, Style, Style, f64), Error> {
        let count_and_style = self.read_u8()?;
        let count = usize::from(count_and_style & 0x3f) + 1;
        let secondary_style = Self::style_kind(count_and_style >> 6)?;
        let fill_style = self.read_style(primary_style)?;
        let line_style = self.read_style(secondary_style)?;
        let line_width = self.read_line_width()?;
        Ok((count, fill_style, line_style, line_width))
    }

    fn outline_fill_polygon(&mut self, primary_style: StyleKind) -> Result<(), Error> {
        let (count, fill_style, line_style, line_width) = self.outline_header(primary_style)?;
        if count < 3 {
            return Err(Error::InvalidData("polygon has fewer than three points"));
        }
        let path = self.read_polygon(count, true)?;
        self.commands.push(Command::Fill {
            path: path.clone(),
            style: fill_style,
        });
        self.commands.push(Command::Stroke {
            path,
            style: line_style,
            line_width,
        });
        Ok(())
    }

    fn outline_fill_rectangles(&mut self, primary_style: StyleKind) -> Result<(), Error> {
        let (count, fill_style, line_style, line_width) = self.outline_header(primary_style)?;
        for _ in 0..count {
            let path = self.read_rectangle()?;
            self.commands.push(Command::Fill {
                path: path.clone(),
                style: fill_style.clone(),
            });
            self.commands.push(Command::Stroke {
                path,
                style: line_style.clone(),
                line_width,
            });
        }
        Ok(())
    }

    fn outline_fill_path(&mut self, primary_style: StyleKind) -> Result<(), Error> {
        let (count, fill_style, line_style, line_width) = self.outline_header(primary_style)?;
        let path = self.read_path(count)?;
        self.commands.push(Command::Fill {
            path: path.clone(),
            style: fill_style,
        });
        self.commands.push(Command::Stroke {
            path,
            style: line_style,
            line_width,
        });
        Ok(())
    }

    fn read_polygon(&mut self, count: usize, close: bool) -> Result<Path, Error> {
        let start = self.read_point()?;
        let mut nodes = Vec::with_capacity(count);
        for _ in 1..count {
            nodes.push(PathNode {
                operation: PathOperation::LineTo(self.read_point()?),
                line_width: None,
            });
        }
        if close {
            nodes.push(PathNode {
                operation: PathOperation::Close,
                line_width: None,
            });
        }
        Ok(Path {
            subpaths: vec![Subpath { start, nodes }],
        })
    }

    fn read_rectangle(&mut self) -> Result<Path, Error> {
        let origin = self.read_point()?;
        let width = self.read_unit()?;
        let height = self.read_unit()?;
        if width <= 0.0 || height <= 0.0 {
            return Err(Error::InvalidData("rectangle has a non-positive size"));
        }
        let right = origin.x + width;
        let bottom = origin.y + height;
        Ok(Path {
            subpaths: vec![Subpath {
                start: origin,
                nodes: vec![
                    PathNode {
                        operation: PathOperation::LineTo(Point {
                            x: right,
                            y: origin.y,
                        }),
                        line_width: None,
                    },
                    PathNode {
                        operation: PathOperation::LineTo(Point {
                            x: right,
                            y: bottom,
                        }),
                        line_width: None,
                    },
                    PathNode {
                        operation: PathOperation::LineTo(Point {
                            x: origin.x,
                            y: bottom,
                        }),
                        line_width: None,
                    },
                    PathNode {
                        operation: PathOperation::Close,
                        line_width: None,
                    },
                ],
            }],
        })
    }

    fn read_path(&mut self, count: usize) -> Result<Path, Error> {
        Self::check_count(count)?;
        let mut lengths = Vec::with_capacity(count);
        let mut total_nodes = 0usize;
        for _ in 0..count {
            let length = self.read_count(true)?;
            total_nodes = total_nodes
                .checked_add(length)
                .ok_or(Error::TooManyElements)?;
            Self::check_count(total_nodes)?;
            lengths.push(length);
        }
        let mut subpaths = Vec::with_capacity(count);
        for length in lengths {
            let start = self.read_point()?;
            let mut current = start;
            let mut nodes = Vec::with_capacity(length);
            for _ in 0..length {
                let tag = self.read_u8()?;
                if tag & 0xe8 != 0 {
                    return Err(Error::InvalidData("invalid path node tag"));
                }
                let line_width = if tag & 0x10 != 0 {
                    Some(self.read_line_width()?)
                } else {
                    None
                };
                let operation = match tag & 0x07 {
                    0 => PathOperation::LineTo(self.read_point()?),
                    1 => PathOperation::LineTo(Point {
                        x: self.read_unit()?,
                        y: current.y,
                    }),
                    2 => PathOperation::LineTo(Point {
                        x: current.x,
                        y: self.read_unit()?,
                    }),
                    3 => PathOperation::CubicTo {
                        control_0: self.read_point()?,
                        control_1: self.read_point()?,
                        to: self.read_point()?,
                    },
                    4 => self.read_arc(true)?,
                    5 => self.read_arc(false)?,
                    6 => PathOperation::Close,
                    7 => PathOperation::QuadraticTo {
                        control: self.read_point()?,
                        to: self.read_point()?,
                    },
                    _ => unreachable!(),
                };
                current = match operation {
                    PathOperation::LineTo(point)
                    | PathOperation::CubicTo { to: point, .. }
                    | PathOperation::QuadraticTo { to: point, .. }
                    | PathOperation::ArcTo { to: point, .. } => point,
                    PathOperation::Close => start,
                };
                nodes.push(PathNode {
                    operation,
                    line_width,
                });
            }
            subpaths.push(Subpath { start, nodes });
        }
        Ok(Path { subpaths })
    }

    fn read_arc(&mut self, circle: bool) -> Result<PathOperation, Error> {
        let flags = self.read_u8()?;
        if flags & !0x03 != 0 {
            return Err(Error::InvalidData("invalid arc flags"));
        }
        let radius_x = self.read_unit()?;
        let (radius_y, rotation) = if circle {
            (radius_x, 0.0)
        } else {
            (self.read_unit()?, self.read_unit()?)
        };
        if radius_x < 0.0 || radius_y < 0.0 {
            return Err(Error::InvalidData("negative arc radius"));
        }
        Ok(PathOperation::ArcTo {
            radius_x,
            radius_y,
            rotation,
            large_arc: flags & 1 != 0,
            sweep: flags & 2 != 0,
            to: self.read_point()?,
        })
    }

    fn read_style(&mut self, kind: StyleKind) -> Result<Style, Error> {
        match kind {
            StyleKind::Flat => {
                let index = self.read_color_index()?;
                Ok(Style::Solid(self.colors[index]))
            }
            StyleKind::Linear | StyleKind::Radial => {
                let start = self.read_point()?;
                let end = self.read_point()?;
                let start_color_index = self.read_color_index()?;
                let end_color_index = self.read_color_index()?;
                let start_color = self.colors[start_color_index];
                let end_color = self.colors[end_color_index];
                match kind {
                    StyleKind::Linear => Ok(Style::LinearGradient {
                        start,
                        end,
                        start_color,
                        end_color,
                    }),
                    StyleKind::Radial => Ok(Style::RadialGradient {
                        center: start,
                        edge: end,
                        center_color: start_color,
                        edge_color: end_color,
                    }),
                    StyleKind::Flat => unreachable!(),
                }
            }
        }
    }

    fn read_colors(&mut self, encoding: u8, count: usize) -> Result<Vec<Color>, Error> {
        Self::check_count(count)?;
        let mut colors = Vec::with_capacity(count);
        for _ in 0..count {
            let color = match encoding {
                0 => Color {
                    red: f64::from(self.read_u8()?) / 255.0,
                    green: f64::from(self.read_u8()?) / 255.0,
                    blue: f64::from(self.read_u8()?) / 255.0,
                    alpha: f64::from(self.read_u8()?) / 255.0,
                    color_space: ColorSpace::Srgb,
                },
                1 => {
                    let value = self.read_u16()?;
                    Color {
                        red: f64::from(value & 0x1f) / 31.0,
                        green: f64::from((value >> 5) & 0x3f) / 63.0,
                        blue: f64::from((value >> 11) & 0x1f) / 31.0,
                        alpha: 1.0,
                        color_space: ColorSpace::Srgb,
                    }
                }
                2 => {
                    let color = Color {
                        red: f64::from(self.read_f32()?),
                        green: f64::from(self.read_f32()?),
                        blue: f64::from(self.read_f32()?),
                        alpha: f64::from(self.read_f32()?),
                        color_space: ColorSpace::LinearSrgb,
                    };
                    if !color.red.is_finite()
                        || !color.green.is_finite()
                        || !color.blue.is_finite()
                        || !color.alpha.is_finite()
                        || !(0.0..=1.0).contains(&color.alpha)
                    {
                        return Err(Error::InvalidData("invalid floating-point color"));
                    }
                    color
                }
                3 => return Err(Error::UnsupportedColorEncoding),
                _ => unreachable!(),
            };
            colors.push(color);
        }
        Ok(colors)
    }

    const fn style_kind(value: u8) -> Result<StyleKind, Error> {
        match value {
            0 => Ok(StyleKind::Flat),
            1 => Ok(StyleKind::Linear),
            2 => Ok(StyleKind::Radial),
            _ => Err(Error::InvalidData("reserved style kind")),
        }
    }

    fn read_color_index(&mut self) -> Result<usize, Error> {
        let index = usize::try_from(self.read_varuint()?)
            .map_err(|_| Error::InvalidData("color index is too large"))?;
        if index >= self.colors.len() {
            return Err(Error::InvalidData("color index is out of range"));
        }
        Ok(index)
    }

    fn read_line_width(&mut self) -> Result<f64, Error> {
        let width = self.read_unit()?;
        if width < 0.0 {
            return Err(Error::InvalidData("negative line width"));
        }
        Ok(width)
    }

    fn read_point(&mut self) -> Result<Point, Error> {
        Ok(Point {
            x: self.read_unit()?,
            y: self.read_unit()?,
        })
    }

    fn read_dimension(&mut self) -> Result<f64, Error> {
        let value = match self.coordinate_range {
            CoordinateRange::Reduced => match self.read_u8()? {
                0 => 256,
                value => u64::from(value),
            },
            CoordinateRange::Default => match self.read_u16()? {
                0 => 65_536,
                value => u64::from(value),
            },
            CoordinateRange::Enhanced => match self.read_u32()? {
                0 => 4_294_967_296,
                value => u64::from(value),
            },
        };
        Ok(value as f64)
    }

    fn read_unit(&mut self) -> Result<f64, Error> {
        let value = match self.coordinate_range {
            CoordinateRange::Reduced => i64::from(self.read_i8()?),
            CoordinateRange::Default => i64::from(self.read_i16()?),
            CoordinateRange::Enhanced => i64::from(self.read_i32()?),
        };
        Ok(value as f64 / self.scale)
    }

    fn read_count(&mut self, offset: bool) -> Result<usize, Error> {
        let value = usize::try_from(self.read_varuint()?).map_err(|_| Error::TooManyElements)?;
        let count = if offset {
            value.checked_add(1).ok_or(Error::TooManyElements)?
        } else {
            value
        };
        Self::check_count(count)?;
        Ok(count)
    }

    const fn check_count(count: usize) -> Result<(), Error> {
        if count > MAX_ELEMENTS {
            Err(Error::TooManyElements)
        } else {
            Ok(())
        }
    }

    fn read_varuint(&mut self) -> Result<u32, Error> {
        let mut result = 0u32;
        for index in 0..5 {
            let byte = self.read_u8()?;
            if index == 4 && byte & 0xf0 != 0 {
                return Err(Error::InvalidData("VarUInt is out of range"));
            }
            result |= u32::from(byte & 0x7f) << (index * 7);
            if byte & 0x80 == 0 {
                return Ok(result);
            }
        }
        Err(Error::InvalidData("VarUInt is too long"))
    }

    fn read_exact(&mut self, length: usize) -> Result<&'a [u8], Error> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(Error::UnexpectedEnd)?;
        let bytes = self
            .data
            .get(self.position..end)
            .ok_or(Error::UnexpectedEnd)?;
        self.position = end;
        Ok(bytes)
    }

    fn read_u8(&mut self) -> Result<u8, Error> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_i8(&mut self) -> Result<i8, Error> {
        Ok(i8::from_le_bytes([self.read_u8()?]))
    }

    fn read_u16(&mut self) -> Result<u16, Error> {
        let bytes: [u8; 2] = self
            .read_exact(2)?
            .try_into()
            .expect("slice has exact length");
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_i16(&mut self) -> Result<i16, Error> {
        let bytes: [u8; 2] = self
            .read_exact(2)?
            .try_into()
            .expect("slice has exact length");
        Ok(i16::from_le_bytes(bytes))
    }

    fn read_u32(&mut self) -> Result<u32, Error> {
        let bytes: [u8; 4] = self
            .read_exact(4)?
            .try_into()
            .expect("slice has exact length");
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_i32(&mut self) -> Result<i32, Error> {
        let bytes: [u8; 4] = self
            .read_exact(4)?
            .try_into()
            .expect("slice has exact length");
        Ok(i32::from_le_bytes(bytes))
    }

    fn read_f32(&mut self) -> Result<f32, Error> {
        Ok(f32::from_bits(self.read_u32()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(width: u8, height: u8, colors: &[[u8; 4]]) -> Vec<u8> {
        let mut data = vec![0x72, 0x56, 1, 0x40, width, height, colors.len() as u8];
        for color in colors {
            data.extend_from_slice(color);
        }
        data
    }

    #[test]
    fn parses_a_filled_polygon_into_absolute_commands() {
        let mut data = header(10, 20, &[[255, 0, 128, 255]]);
        data.extend_from_slice(&[1, 2, 0, 1, 2, 8, 2, 4, 9, 0]);

        let document = parse(&data).expect("valid TinyVG should parse");
        assert_eq!(document.size.width, 10.0);
        assert_eq!(document.size.height, 20.0);
        assert_eq!(document.commands.len(), 1);
        let Command::Fill { path, style } = &document.commands[0] else {
            panic!("expected fill command");
        };
        assert_eq!(path.subpaths[0].start, Point { x: 1.0, y: 2.0 });
        assert_eq!(path.subpaths[0].nodes.len(), 3);
        assert!(matches!(
            path.subpaths[0].nodes[2].operation,
            PathOperation::Close
        ));
        assert!(matches!(style, Style::Solid(color) if color.red == 1.0));
    }

    #[test]
    fn normalizes_horizontal_and_vertical_path_nodes() {
        let mut data = header(10, 10, &[[0, 0, 0, 255]]);
        data.extend_from_slice(&[3, 0, 0, 2, 1, 2, 1, 5, 2, 6, 0, 7, 8, 0]);

        let document = parse(&data).expect("valid TinyVG should parse");
        let Command::Fill { path, .. } = &document.commands[0] else {
            panic!("expected fill command");
        };
        assert_eq!(
            path.subpaths[0]
                .nodes
                .iter()
                .map(|node| &node.operation)
                .collect::<Vec<_>>(),
            vec![
                &PathOperation::LineTo(Point { x: 5.0, y: 2.0 }),
                &PathOperation::LineTo(Point { x: 5.0, y: 6.0 }),
                &PathOperation::LineTo(Point { x: 7.0, y: 8.0 }),
            ]
        );
    }

    #[test]
    fn rejects_an_out_of_range_color() {
        let mut data = header(10, 10, &[]);
        data.extend_from_slice(&[1, 2, 0]);
        assert_eq!(
            parse(&data),
            Err(Error::InvalidData("color index is out of range"))
        );
    }

    #[test]
    fn rejects_a_nonzero_end_command_style() {
        let mut data = header(10, 10, &[]);
        data.push(0x40);
        assert_eq!(parse(&data), Err(Error::InvalidData("unknown command")));
    }
}
