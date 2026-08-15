/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::{
    Color, ColorSpace, Command, Document, Error, MAX_ELEMENTS, Path, PathNode, PathOperation,
    Point, Size, Style, Subpath,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Token<'a> {
    Open,
    Close,
    Atom(&'a str),
}

struct Lexer<'a> {
    source: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    const fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    fn next(&mut self) -> Option<Token<'a>> {
        let bytes = self.source.as_bytes();
        while self.position < bytes.len() && bytes[self.position].is_ascii_whitespace() {
            self.position += 1;
        }
        let first = *bytes.get(self.position)?;
        self.position += 1;
        match first {
            b'(' => Some(Token::Open),
            b')' => Some(Token::Close),
            _ => {
                let start = self.position - 1;
                while self.position < bytes.len()
                    && !bytes[self.position].is_ascii_whitespace()
                    && !matches!(bytes[self.position], b'(' | b')')
                {
                    self.position += 1;
                }
                Some(Token::Atom(&self.source[start..self.position]))
            }
        }
    }
}

#[derive(Clone, Copy)]
enum TextColorEncoding {
    Srgb,
    LinearSrgb,
}

pub(super) fn parse(source: &str) -> Result<Document, Error> {
    TextParser::new(source).parse()
}

pub(super) fn is_text(source: &[u8]) -> bool {
    let source = source.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(source);
    source
        .iter()
        .copied()
        .skip_while(u8::is_ascii_whitespace)
        .take(4)
        .eq(b"(tvg".iter().copied())
}

struct TextParser<'a> {
    lexer: Lexer<'a>,
    lookahead: Option<Token<'a>>,
    colors: Vec<Color>,
    commands: Vec<Command>,
}

impl<'a> TextParser<'a> {
    const fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source),
            lookahead: None,
            colors: Vec::new(),
            commands: Vec::new(),
        }
    }

    fn parse(mut self) -> Result<Document, Error> {
        self.expect_open()?;
        self.expect_name("tvg")?;
        let version = self.parse_integer::<u8>()?;
        if version != 1 {
            return Err(Error::UnsupportedVersion(version));
        }
        let (size, color_encoding) = self.parse_header()?;
        self.colors = self.parse_colors(color_encoding)?;
        self.parse_commands()?;
        self.expect_close()?;
        if self.next_token().is_some() {
            return Err(Self::syntax_error());
        }
        Ok(Document {
            size,
            commands: self.commands,
        })
    }

    fn parse_header(&mut self) -> Result<(Size, TextColorEncoding), Error> {
        self.expect_open()?;
        let width = self.parse_integer::<u64>()?;
        let height = self.parse_integer::<u64>()?;
        if width == 0 || height == 0 {
            return Err(Error::InvalidData("text document has a zero size"));
        }
        match self.expect_atom()? {
            "1/1" | "1/2" | "1/4" | "1/8" | "1/16" | "1/32" | "1/64" | "1/128" | "1/256"
            | "1/512" | "1/1024" | "1/2048" | "1/4096" | "1/8192" | "1/16384" | "1/32768" => {}
            _ => return Err(Error::InvalidData("unknown text scale")),
        }
        let color_encoding = match self.expect_atom()? {
            "u8888" | "u565" => TextColorEncoding::Srgb,
            "f32" => TextColorEncoding::LinearSrgb,
            "custom" => return Err(Error::UnsupportedColorEncoding),
            _ => return Err(Error::InvalidData("unknown text color encoding")),
        };
        match self.expect_atom()? {
            "reduced" | "default" | "enhanced" => {}
            _ => return Err(Error::InvalidData("unknown text coordinate range")),
        }
        self.expect_close()?;
        Ok((
            Size {
                width: width as f64,
                height: height as f64,
            },
            color_encoding,
        ))
    }

    fn parse_colors(&mut self, encoding: TextColorEncoding) -> Result<Vec<Color>, Error> {
        self.expect_open()?;
        let mut colors = Vec::new();
        while !self.at_close()? {
            self.expect_open()?;
            let red = self.parse_number()?;
            let green = self.parse_number()?;
            let blue = self.parse_number()?;
            let alpha = if self.at_close()? {
                1.0
            } else {
                self.parse_number()?
            };
            self.expect_close()?;
            if !(0.0..=1.0).contains(&alpha) {
                return Err(Error::InvalidData("text color alpha is out of range"));
            }
            colors.push(Color {
                red,
                green,
                blue,
                alpha,
                color_space: match encoding {
                    TextColorEncoding::Srgb => ColorSpace::Srgb,
                    TextColorEncoding::LinearSrgb => ColorSpace::LinearSrgb,
                },
            });
            Self::check_count(colors.len())?;
        }
        self.expect_close()?;
        Ok(colors)
    }

    fn parse_commands(&mut self) -> Result<(), Error> {
        self.expect_open()?;
        while !self.at_close()? {
            self.expect_open()?;
            let command = self.expect_atom()?;
            match command {
                "fill_polygon" => {
                    let style = self.parse_style()?;
                    let points = self.parse_points()?;
                    if points.len() < 3 {
                        return Err(Error::InvalidData("polygon has fewer than three points"));
                    }
                    self.push(Command::Fill {
                        path: Self::polygon_path(points, true)?,
                        style,
                    })?;
                }
                "fill_rectangles" => {
                    let style = self.parse_style()?;
                    for path in self.parse_rectangles()? {
                        self.push(Command::Fill {
                            path,
                            style: style.clone(),
                        })?;
                    }
                }
                "fill_path" => {
                    let style = self.parse_style()?;
                    let path = self.parse_path()?;
                    self.push(Command::Fill { path, style })?;
                }
                "draw_lines" => {
                    let style = self.parse_style()?;
                    let line_width = self.parse_line_width()?;
                    let path = self.parse_lines()?;
                    self.push(Command::Stroke {
                        path,
                        style,
                        line_width,
                    })?;
                }
                "draw_line_loop" | "draw_line_strip" => {
                    let style = self.parse_style()?;
                    let line_width = self.parse_line_width()?;
                    let points = self.parse_points()?;
                    let path = Self::polygon_path(points, command == "draw_line_loop")?;
                    self.push(Command::Stroke {
                        path,
                        style,
                        line_width,
                    })?;
                }
                "draw_line_path" => {
                    let style = self.parse_style()?;
                    let line_width = self.parse_line_width()?;
                    let path = self.parse_path()?;
                    self.push(Command::Stroke {
                        path,
                        style,
                        line_width,
                    })?;
                }
                "outline_fill_polygon" => {
                    let fill_style = self.parse_style()?;
                    let line_style = self.parse_style()?;
                    let line_width = self.parse_line_width()?;
                    let points = self.parse_points()?;
                    if points.len() < 3 {
                        return Err(Error::InvalidData("polygon has fewer than three points"));
                    }
                    let path = Self::polygon_path(points, true)?;
                    self.push(Command::Fill {
                        path: path.clone(),
                        style: fill_style,
                    })?;
                    self.push(Command::Stroke {
                        path,
                        style: line_style,
                        line_width,
                    })?;
                }
                "outline_fill_rectangles" => {
                    let fill_style = self.parse_style()?;
                    let line_style = self.parse_style()?;
                    let line_width = self.parse_line_width()?;
                    for path in self.parse_rectangles()? {
                        self.push(Command::Fill {
                            path: path.clone(),
                            style: fill_style.clone(),
                        })?;
                        self.push(Command::Stroke {
                            path,
                            style: line_style.clone(),
                            line_width,
                        })?;
                    }
                }
                "outline_fill_path" => {
                    let fill_style = self.parse_style()?;
                    let line_style = self.parse_style()?;
                    let line_width = self.parse_line_width()?;
                    let path = self.parse_path()?;
                    self.push(Command::Fill {
                        path: path.clone(),
                        style: fill_style,
                    })?;
                    self.push(Command::Stroke {
                        path,
                        style: line_style,
                        line_width,
                    })?;
                }
                _ => return Err(Error::InvalidData("unknown text command")),
            }
            self.expect_close()?;
        }
        self.expect_close()
    }

    fn parse_style(&mut self) -> Result<Style, Error> {
        self.expect_open()?;
        let style_name = self.expect_atom()?;
        let style = match style_name {
            "flat" => Style::Solid(self.parse_color()?),
            "linear" | "radial" => {
                let start = self.parse_point()?;
                let end = self.parse_point()?;
                let start_color = self.parse_color()?;
                let end_color = self.parse_color()?;
                if style_name == "linear" {
                    Style::LinearGradient {
                        start,
                        end,
                        start_color,
                        end_color,
                    }
                } else {
                    Style::RadialGradient {
                        center: start,
                        edge: end,
                        center_color: start_color,
                        edge_color: end_color,
                    }
                }
            }
            _ => return Err(Error::InvalidData("unknown text style")),
        };
        self.expect_close()?;
        Ok(style)
    }

    fn parse_color(&mut self) -> Result<Color, Error> {
        let index = self.parse_integer::<usize>()?;
        self.colors
            .get(index)
            .copied()
            .ok_or(Error::InvalidData("color index is out of range"))
    }

    fn parse_points(&mut self) -> Result<Vec<Point>, Error> {
        self.expect_open()?;
        let mut points = Vec::new();
        while !self.at_close()? {
            points.push(self.parse_point()?);
            Self::check_count(points.len())?;
        }
        self.expect_close()?;
        Ok(points)
    }

    fn parse_rectangles(&mut self) -> Result<Vec<Path>, Error> {
        self.expect_open()?;
        let mut paths = Vec::new();
        while !self.at_close()? {
            self.expect_open()?;
            let origin = Point {
                x: self.parse_number()?,
                y: self.parse_number()?,
            };
            let width = self.parse_number()?;
            let height = self.parse_number()?;
            self.expect_close()?;
            paths.push(Self::rectangle_path(origin, width, height)?);
            Self::check_count(paths.len())?;
        }
        self.expect_close()?;
        Ok(paths)
    }

    fn parse_lines(&mut self) -> Result<Path, Error> {
        self.expect_open()?;
        let mut subpaths = Vec::new();
        while !self.at_close()? {
            self.expect_open()?;
            let start = self.parse_point()?;
            let end = self.parse_point()?;
            self.expect_close()?;
            subpaths.push(Subpath {
                start,
                nodes: vec![PathNode {
                    operation: PathOperation::LineTo(end),
                    line_width: None,
                }],
            });
            Self::check_count(subpaths.len())?;
        }
        self.expect_close()?;
        Ok(Path { subpaths })
    }

    fn parse_path(&mut self) -> Result<Path, Error> {
        self.expect_open()?;
        let mut subpaths = Vec::new();
        while !self.at_close()? {
            let start = self.parse_point()?;
            let mut current = start;
            let mut nodes = Vec::new();
            self.expect_open()?;
            while !self.at_close()? {
                self.expect_open()?;
                let operation_name = self.expect_atom()?;
                let line_width = self.parse_optional_line_width()?;
                let operation = match operation_name {
                    "line" => PathOperation::LineTo(self.parse_inline_point()?),
                    "horiz" => PathOperation::LineTo(Point {
                        x: self.parse_number()?,
                        y: current.y,
                    }),
                    "vert" => PathOperation::LineTo(Point {
                        x: current.x,
                        y: self.parse_number()?,
                    }),
                    "bezier" => PathOperation::CubicTo {
                        control_0: self.parse_point()?,
                        control_1: self.parse_point()?,
                        to: self.parse_point()?,
                    },
                    "quadratic_bezier" => PathOperation::QuadraticTo {
                        control: self.parse_point()?,
                        to: self.parse_point()?,
                    },
                    "arc_circle" => {
                        let radius = self.parse_number()?;
                        Self::validate_radius(radius)?;
                        PathOperation::ArcTo {
                            radius_x: radius,
                            radius_y: radius,
                            rotation: 0.0,
                            large_arc: self.parse_boolean()?,
                            sweep: self.parse_boolean()?,
                            to: self.parse_point()?,
                        }
                    }
                    "arc_ellipse" => {
                        let radius_x = self.parse_number()?;
                        let radius_y = self.parse_number()?;
                        Self::validate_radius(radius_x)?;
                        Self::validate_radius(radius_y)?;
                        PathOperation::ArcTo {
                            radius_x,
                            radius_y,
                            rotation: self.parse_number()?,
                            large_arc: self.parse_boolean()?,
                            sweep: self.parse_boolean()?,
                            to: self.parse_point()?,
                        }
                    }
                    "close" => PathOperation::Close,
                    _ => return Err(Error::InvalidData("unknown text path operation")),
                };
                self.expect_close()?;
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
                Self::check_count(nodes.len())?;
            }
            self.expect_close()?;
            subpaths.push(Subpath { start, nodes });
            Self::check_count(subpaths.len())?;
        }
        self.expect_close()?;
        Ok(Path { subpaths })
    }

    fn parse_point(&mut self) -> Result<Point, Error> {
        self.expect_open()?;
        let point = self.parse_inline_point()?;
        self.expect_close()?;
        Ok(point)
    }

    fn parse_inline_point(&mut self) -> Result<Point, Error> {
        Ok(Point {
            x: self.parse_number()?,
            y: self.parse_number()?,
        })
    }

    fn parse_optional_line_width(&mut self) -> Result<Option<f64>, Error> {
        let atom = self.expect_atom()?;
        if atom == "-" {
            return Ok(None);
        }
        let width = Self::number(atom)?;
        if width < 0.0 {
            return Err(Error::InvalidData("negative line width"));
        }
        Ok(Some(width))
    }

    fn parse_line_width(&mut self) -> Result<f64, Error> {
        let width = self.parse_number()?;
        if width < 0.0 {
            return Err(Error::InvalidData("negative line width"));
        }
        Ok(width)
    }

    fn parse_boolean(&mut self) -> Result<bool, Error> {
        match self.expect_atom()? {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(Error::InvalidData("invalid text boolean")),
        }
    }

    fn parse_number(&mut self) -> Result<f64, Error> {
        Self::number(self.expect_atom()?)
    }

    fn number(atom: &str) -> Result<f64, Error> {
        let value = atom
            .parse::<f64>()
            .map_err(|_| Error::InvalidData("invalid text number"))?;
        if !value.is_finite() {
            return Err(Error::InvalidData("non-finite text number"));
        }
        Ok(value)
    }

    fn parse_integer<T>(&mut self) -> Result<T, Error>
    where
        T: std::str::FromStr,
    {
        self.expect_atom()?
            .parse::<T>()
            .map_err(|_| Error::InvalidData("invalid text integer"))
    }

    fn polygon_path(points: Vec<Point>, close: bool) -> Result<Path, Error> {
        let mut points = points.into_iter();
        let start = points
            .next()
            .ok_or(Error::InvalidData("point list is empty"))?;
        let mut nodes = points
            .map(|point| PathNode {
                operation: PathOperation::LineTo(point),
                line_width: None,
            })
            .collect::<Vec<_>>();
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

    fn rectangle_path(origin: Point, width: f64, height: f64) -> Result<Path, Error> {
        if width <= 0.0 || height <= 0.0 {
            return Err(Error::InvalidData("rectangle has a non-positive size"));
        }
        let right = origin.x + width;
        let bottom = origin.y + height;
        Self::polygon_path(
            vec![
                origin,
                Point {
                    x: right,
                    y: origin.y,
                },
                Point {
                    x: right,
                    y: bottom,
                },
                Point {
                    x: origin.x,
                    y: bottom,
                },
            ],
            true,
        )
    }

    const fn validate_radius(radius: f64) -> Result<(), Error> {
        if radius < 0.0 {
            Err(Error::InvalidData("negative arc radius"))
        } else {
            Ok(())
        }
    }

    fn push(&mut self, command: Command) -> Result<(), Error> {
        self.commands.push(command);
        Self::check_count(self.commands.len())
    }

    const fn check_count(count: usize) -> Result<(), Error> {
        if count > MAX_ELEMENTS {
            Err(Error::TooManyElements)
        } else {
            Ok(())
        }
    }

    fn expect_name(&mut self, expected: &str) -> Result<(), Error> {
        if self.expect_atom()? == expected {
            Ok(())
        } else {
            Err(Self::syntax_error())
        }
    }

    fn expect_open(&mut self) -> Result<(), Error> {
        if self.next_token() == Some(Token::Open) {
            Ok(())
        } else {
            Err(Self::syntax_error())
        }
    }

    fn expect_close(&mut self) -> Result<(), Error> {
        if self.next_token() == Some(Token::Close) {
            Ok(())
        } else {
            Err(Self::syntax_error())
        }
    }

    fn expect_atom(&mut self) -> Result<&'a str, Error> {
        if let Some(Token::Atom(atom)) = self.next_token() {
            Ok(atom)
        } else {
            Err(Self::syntax_error())
        }
    }

    fn at_close(&mut self) -> Result<bool, Error> {
        if self.lookahead.is_none() {
            self.lookahead = self.lexer.next();
        }
        match self.lookahead {
            Some(Token::Close) => Ok(true),
            Some(_) => Ok(false),
            None => Err(Error::UnexpectedEnd),
        }
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        self.lookahead.take().or_else(|| self.lexer.next())
    }

    const fn syntax_error() -> Error {
        Error::InvalidData("invalid TinyVG text syntax")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_paths_into_absolute_commands() {
        let source = r#"
            (tvg 1
              (24 24 1/2048 u8888 enhanced)
              ((0.161 0.678 1.0) (1.0 0.945 0.91))
              ((fill_path (flat 0) (
                (12 1) ((line - 3 5) (vert - 11))
                (12 5) ((bezier - (13.5 5) (15 6.2) (15 8)))
              )))
            )
        "#;
        let document = parse(source).expect("valid TinyVG text should parse");
        assert_eq!(
            document.size,
            Size {
                width: 24.0,
                height: 24.0
            }
        );
        assert_eq!(document.commands.len(), 1);
        let Command::Fill { path, .. } = &document.commands[0] else {
            panic!("expected fill command");
        };
        assert_eq!(path.subpaths.len(), 2);
        assert_eq!(
            path.subpaths[0].nodes[1].operation,
            PathOperation::LineTo(Point { x: 3.0, y: 11.0 })
        );
    }

    #[test]
    fn rejects_trailing_text() {
        assert!(parse("(tvg 1 (1 1 1/1 u8888 reduced) () ()) trailing").is_err());
    }
}
