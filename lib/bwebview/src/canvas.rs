/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::marker::PhantomData;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::Window;
use crate::platforms::{PlatformCanvas, PlatformCanvasContext};

/// A portable RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    /// Red component.
    pub red: u8,
    /// Green component.
    pub green: u8,
    /// Blue component.
    pub blue: u8,
    /// Alpha component.
    pub alpha: u8,
}

impl Color {
    /// Create an opaque color from a packed `0xRRGGBB` value.
    pub const fn from_rgb(rgb: u32) -> Self {
        Self::rgb(
            ((rgb >> 16) & 0xff) as u8,
            ((rgb >> 8) & 0xff) as u8,
            (rgb & 0xff) as u8,
        )
    }

    /// Create an opaque RGB color.
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 255,
        }
    }

    /// Create an RGBA color.
    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub(crate) fn components(self, global_alpha: f32) -> (f64, f64, f64, f64) {
        (
            f64::from(self.red) / 255.0,
            f64::from(self.green) / 255.0,
            f64::from(self.blue) / 255.0,
            f64::from(self.alpha) / 255.0 * f64::from(global_alpha.clamp(0.0, 1.0)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn color_from_packed_rgb_uses_rrggbb_order() {
        assert_eq!(Color::from_rgb(0x12_34_56), Color::rgb(0x12, 0x34, 0x56));
        assert_eq!(Color::from_rgb(0), Color::rgba(0, 0, 0, 255));
    }
}

/// Canvas line cap style.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineCap {
    /// Flat line ends.
    #[default]
    Butt,
    /// Round line ends.
    Round,
    /// Square line ends.
    Square,
}

/// Canvas line join style.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LineJoin {
    /// Mitered joins.
    #[default]
    Miter,
    /// Rounded joins.
    Round,
    /// Beveled joins.
    Bevel,
}

/// Horizontal text alignment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextAlign {
    /// Align at the text start.
    Start,
    /// Align at the text end.
    End,
    /// Align left.
    #[default]
    Left,
    /// Align right.
    Right,
    /// Center text.
    Center,
}

/// Vertical text alignment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextBaseline {
    /// Top edge.
    Top,
    /// Hanging baseline.
    Hanging,
    /// Middle.
    Middle,
    /// Alphabetic baseline.
    #[default]
    Alphabetic,
    /// Ideographic baseline.
    Ideographic,
    /// Bottom edge.
    Bottom,
}

/// Canvas font weight.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FontWeight {
    /// Normal font weight.
    #[default]
    Normal,
    /// Bold font weight.
    Bold,
}

#[derive(Clone)]
pub(crate) struct CanvasState {
    pub fill: Color,
    pub stroke: Color,
    pub alpha: f32,
    pub line_width: f32,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub text_align: TextAlign,
    pub text_baseline: TextBaseline,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            fill: Color::rgb(0, 0, 0),
            stroke: Color::rgb(0, 0, 0),
            alpha: 1.0,
            line_width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            font_family: "sans-serif".into(),
            font_size: 10.0,
            font_weight: FontWeight::Normal,
            text_align: TextAlign::Left,
            text_baseline: TextBaseline::Alphabetic,
        }
    }
}

/// A canvas paint event.
pub enum CanvasEvent<'a> {
    /// Draw the current canvas frame.
    Draw(&'a mut CanvasRenderingContext2d<'a>),
}

/// Builder for a native 2D canvas view.
pub struct CanvasBuilder<'a> {
    window: &'a Window,
}

impl<'a> CanvasBuilder<'a> {
    /// Create a canvas builder attached to `window`.
    pub const fn new(window: &'a Window) -> Self {
        Self { window }
    }

    /// Build the canvas view.
    pub fn build(self) -> Canvas {
        Canvas {
            platform: PlatformCanvas::new(&self.window.platform),
        }
    }
}

pub(crate) trait CanvasInterface {
    fn request_redraw(&mut self);

    fn request_animation_frame(&mut self) {
        self.request_redraw();
    }
}

/// A native 2D canvas view.
pub struct Canvas {
    pub(crate) platform: PlatformCanvas,
}

impl Canvas {
    /// Schedule a canvas draw event.
    pub fn request_redraw(&mut self) {
        self.platform.request_redraw();
    }

    /// Schedule drawing of the next animation frame.
    ///
    /// Native backends coalesce repeated requests with their paint loop, making
    /// this suitable for game animation without a polling thread.
    pub fn request_animation_frame(&mut self) {
        self.platform.request_animation_frame();
    }
}

/// A frame-scoped HTML5 Canvas 2D-like drawing context.
pub struct CanvasRenderingContext2d<'a> {
    pub(crate) platform: PlatformCanvasContext,
    pub(crate) state: CanvasState,
    pub(crate) stack: Vec<CanvasState>,
    width: f32,
    height: f32,
    scale_factor: f32,
    timestamp: Duration,
    _lifetime: PhantomData<&'a mut ()>,
}

impl<'a> CanvasRenderingContext2d<'a> {
    pub(crate) fn new(
        platform: PlatformCanvasContext,
        width: f32,
        height: f32,
        scale_factor: f32,
    ) -> Self {
        static START: OnceLock<Instant> = OnceLock::new();
        Self {
            platform,
            state: CanvasState::default(),
            stack: Vec::new(),
            width,
            height,
            scale_factor,
            timestamp: START.get_or_init(Instant::now).elapsed(),
            _lifetime: PhantomData,
        }
    }
    /// Width of this frame in logical pixels.
    pub const fn width(&self) -> f32 {
        self.width
    }
    /// Height of this frame in logical pixels.
    pub const fn height(&self) -> f32 {
        self.height
    }
    /// Native pixels per logical pixel for this frame.
    pub const fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
    /// Monotonic time since the first canvas frame in this process.
    pub const fn timestamp(&self) -> Duration {
        self.timestamp
    }
    /// Save the drawing state.
    pub fn save(&mut self) {
        self.platform.save();
        self.stack.push(self.state.clone());
    }
    /// Restore the most recently saved drawing state.
    pub fn restore(&mut self) {
        if let Some(state) = self.stack.pop() {
            self.platform.restore();
            self.state = state;
        }
    }
    /// Set the fill color.
    pub const fn set_fill_style(&mut self, color: Color) {
        self.state.fill = color;
    }
    /// Set the stroke color.
    pub const fn set_stroke_style(&mut self, color: Color) {
        self.state.stroke = color;
    }
    /// Set global alpha.
    pub const fn set_global_alpha(&mut self, alpha: f32) {
        self.state.alpha = alpha.clamp(0.0, 1.0);
    }
    /// Set line width.
    pub const fn set_line_width(&mut self, width: f32) {
        self.state.line_width = width.max(0.0);
    }
    /// Set line cap.
    pub const fn set_line_cap(&mut self, cap: LineCap) {
        self.state.line_cap = cap;
    }
    /// Set line join.
    pub const fn set_line_join(&mut self, join: LineJoin) {
        self.state.line_join = join;
    }
    /// Set text font family and size.
    pub fn set_font(&mut self, family: impl Into<String>, size: f32) {
        self.state.font_family = family.into();
        self.state.font_size = size.max(1.0);
    }
    /// Set text font weight.
    pub const fn set_font_weight(&mut self, weight: FontWeight) {
        self.state.font_weight = weight;
    }
    /// Set horizontal text alignment.
    pub const fn set_text_align(&mut self, align: TextAlign) {
        self.state.text_align = align;
    }
    /// Set vertical text baseline.
    pub const fn set_text_baseline(&mut self, baseline: TextBaseline) {
        self.state.text_baseline = baseline;
    }
    /// Clear a rectangle.
    pub fn clear_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.platform.clear_rect(x, y, width, height);
    }
    /// Fill a rectangle.
    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.platform.fill_rect(&self.state, x, y, width, height);
    }
    /// Stroke a rectangle.
    pub fn stroke_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.platform.stroke_rect(&self.state, x, y, width, height);
    }
    /// Begin a new path.
    pub fn begin_path(&mut self) {
        self.platform.begin_path();
    }
    /// Close the current path.
    pub fn close_path(&mut self) {
        self.platform.close_path();
    }
    /// Move the current point.
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.platform.move_to(x, y);
    }
    /// Add a line segment.
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.platform.line_to(x, y);
    }
    /// Add a rectangle to the path.
    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.platform.rect(x, y, width, height);
    }
    /// Add a rounded rectangle to the current path.
    pub fn round_rect(&mut self, x: f32, y: f32, width: f32, height: f32, radius: f32) {
        self.platform.round_rect(x, y, width, height, radius);
    }
    /// Add an ellipse to the current path.
    pub fn ellipse(&mut self, x: f32, y: f32, radius_x: f32, radius_y: f32) {
        self.platform.ellipse(x, y, radius_x, radius_y);
    }
    /// Add a circular arc, with angles in radians.
    pub fn arc(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        start: f32,
        end: f32,
        counterclockwise: bool,
    ) {
        self.platform
            .arc(x, y, radius, start, end, counterclockwise);
    }
    /// Add a quadratic Bézier segment.
    pub fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        self.platform.quadratic_curve_to(cpx, cpy, x, y);
    }
    /// Add a cubic Bézier segment.
    pub fn bezier_curve_to(&mut self, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32) {
        self.platform.bezier_curve_to(cp1x, cp1y, cp2x, cp2y, x, y);
    }
    /// Fill the current path.
    pub fn fill(&mut self) {
        self.platform.fill(&self.state);
    }
    /// Stroke the current path.
    pub fn stroke(&mut self) {
        self.platform.stroke(&self.state);
    }
    /// Clip future drawing to the current path.
    pub fn clip(&mut self) {
        self.platform.clip();
    }
    /// Translate the current transform.
    pub fn translate(&mut self, x: f32, y: f32) {
        self.platform.translate(x, y);
    }
    /// Rotate the current transform by radians.
    pub fn rotate(&mut self, angle: f32) {
        self.platform.rotate(angle);
    }
    /// Scale the current transform.
    pub fn scale(&mut self, x: f32, y: f32) {
        self.platform.scale(x, y);
    }
    /// Replace the current affine transform.
    pub fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        self.platform.set_transform(a, b, c, d, e, f);
    }
    /// Reset the current transform.
    pub fn reset_transform(&mut self) {
        self.platform.reset_transform();
    }
    /// Draw filled text.
    pub fn fill_text(&mut self, text: impl AsRef<str>, x: f32, y: f32) {
        self.platform.fill_text(&self.state, text.as_ref(), x, y);
    }
    /// Measure text width in logical pixels.
    pub fn measure_text(&mut self, text: impl AsRef<str>) -> f32 {
        self.platform.measure_text(&self.state, text.as_ref())
    }
}
