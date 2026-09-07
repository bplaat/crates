/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::ptr::{null, null_mut};
use std::rc::Rc;

use bwindow::ffi::ToWideString;

use super::direct2d::*;
use crate::path::{Command, Path, Point, Transform};
use crate::platforms::com::ComPtr;
use crate::{CanvasState, Color, FontWeight, LineCap, LineJoin, TextAlign, TextBaseline};

pub(crate) struct PlatformCanvasContext {
    target: ComPtr<ID2D1HwndRenderTarget>,
    factory: ComPtr<ID2D1Factory>,
    write: ComPtr<IDWriteFactory>,
    path: Path,
    transform: Transform,
    stack: Vec<(Transform, usize)>,
    layers: usize,
    brushes: HashMap<(Color, u32), ComPtr<ID2D1SolidColorBrush>>,
    text_formats: HashMap<(Rc<str>, u32, FontWeight), ComPtr<IDWriteTextFormat>>,
}

struct PreparedText {
    text: Vec<u16>,
    format: ComPtr<IDWriteTextFormat>,
    metrics: DWRITE_TEXT_METRICS,
}

impl PlatformCanvasContext {
    pub(super) fn new(
        target: ComPtr<ID2D1HwndRenderTarget>,
        factory: ComPtr<ID2D1Factory>,
        write: ComPtr<IDWriteFactory>,
    ) -> Self {
        unsafe { target.SetTransform(&D2D_MATRIX_3X2_F::identity()) };
        Self {
            target,
            factory,
            write,
            path: Path::default(),
            transform: Transform::IDENTITY,
            stack: Vec::new(),
            layers: 0,
            brushes: HashMap::new(),
            text_formats: HashMap::new(),
        }
    }

    pub(crate) fn save(&mut self) {
        self.stack.push((self.transform, self.layers));
    }

    pub(crate) fn restore(&mut self) {
        if let Some((transform, layers)) = self.stack.pop() {
            self.pop_layers(layers);
            self.transform = transform;
            self.apply_transform();
        }
    }

    fn pop_layers(&mut self, depth: usize) {
        while self.layers > depth {
            unsafe { self.target.PopLayer() };
            self.layers -= 1;
        }
    }

    pub(crate) fn clear_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        unsafe {
            self.target
                .PushAxisAlignedClip(&rect(x, y, w, h), D2D1_ANTIALIAS_MODE_ALIASED);
            self.target.Clear(&D2D1_COLOR_F::default());
            self.target.PopAxisAlignedClip();
        }
    }

    pub(crate) fn fill_rect(&mut self, state: &CanvasState, x: f32, y: f32, w: f32, h: f32) {
        let brush = self.brush(state.fill, state.alpha);
        unsafe { self.target.FillRectangle(&rect(x, y, w, h), brush.as_ptr()) };
    }

    pub(crate) fn stroke_rect(&mut self, state: &CanvasState, x: f32, y: f32, w: f32, h: f32) {
        let brush = self.brush(state.stroke, state.alpha);
        unsafe {
            self.target.DrawRectangle(
                &rect(x, y, w, h),
                brush.as_ptr(),
                state.line_width,
                self.stroke_style(state).as_ptr(),
            )
        };
    }

    pub(crate) fn begin_path(&mut self) {
        self.path = Path::default();
    }
    pub(crate) fn close_path(&mut self) {
        self.path.close();
    }
    pub(crate) fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(self.transform, [x, y]);
    }
    pub(crate) fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(self.transform, [x, y]);
    }
    pub(crate) fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.path.rect(self.transform, x, y, w, h);
    }
    pub(crate) fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32) {
        self.path.round_rect(self.transform, x, y, w, h, r);
    }
    pub(crate) fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32) {
        self.path.ellipse(self.transform, [x, y], [rx, ry]);
    }
    pub(crate) fn arc(&mut self, x: f32, y: f32, r: f32, start: f32, end: f32, ccw: bool) {
        self.path
            .arc(self.transform, [x, y], [r, r], start, end, ccw);
    }
    pub(crate) fn quadratic_curve_to(&mut self, a: f32, b: f32, x: f32, y: f32) {
        self.path.quadratic(self.transform, [a, b], [x, y]);
    }
    pub(crate) fn bezier_curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
        self.path.cubic(self.transform, [a, b], [c, d], [x, y]);
    }

    fn geometry(&self, transform: Transform) -> ComPtr<ID2D1PathGeometry> {
        let geometry = unsafe { self.factory.CreatePathGeometry() }.expect("Direct2D path");
        let sink = unsafe { geometry.Open() }.expect("Direct2D geometry sink");
        let mut open = false;
        let mut start = [0.0; 2];
        unsafe {
            sink.SetFillMode(D2D1_FILL_MODE_WINDING);
            for command in &self.path.commands {
                if let Command::Move(point) = *command {
                    if open {
                        sink.EndFigure(D2D1_FIGURE_END_OPEN);
                    }
                    start = point;
                    sink.BeginFigure(vector(transform.point(point)), D2D1_FIGURE_BEGIN_FILLED);
                    open = true;
                    continue;
                }
                if !open {
                    sink.BeginFigure(vector(transform.point(start)), D2D1_FIGURE_BEGIN_FILLED);
                    open = true;
                }
                match *command {
                    Command::Move(_) => unreachable!(),
                    Command::Line(point) => sink.AddLine(vector(transform.point(point))),
                    Command::Cubic(a, b, end) => sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                        point1: vector(transform.point(a)),
                        point2: vector(transform.point(b)),
                        point3: vector(transform.point(end)),
                    }),
                    Command::Quadratic(control, end) => {
                        sink.AddQuadraticBezier(&D2D1_QUADRATIC_BEZIER_SEGMENT {
                            point1: vector(transform.point(control)),
                            point2: vector(transform.point(end)),
                        })
                    }
                    Command::Close => {
                        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                        open = false;
                    }
                }
            }
            if open {
                sink.EndFigure(D2D1_FIGURE_END_OPEN);
            }
            sink.Close().expect("finish Direct2D geometry");
        }
        geometry
    }

    pub(crate) fn fill(&mut self, state: &CanvasState) {
        let geometry = self.geometry(Transform::IDENTITY);
        let brush = self.brush(state.fill, state.alpha);
        unsafe {
            self.target.SetTransform(&D2D_MATRIX_3X2_F::identity());
            self.target
                .FillGeometry(geometry.as_ptr(), brush.as_ptr(), null_mut());
        }
        self.apply_transform();
    }

    pub(crate) fn stroke(&mut self, state: &CanvasState) {
        // Paths capture their construction transform, while strokes use the current
        // transform for width/caps. Undo it on the geometry before native drawing.
        let Some(inverse) = self.transform.inverse() else {
            return;
        };
        let geometry = self.geometry(inverse);
        let brush = self.brush(state.stroke, state.alpha);
        unsafe {
            self.target.DrawGeometry(
                geometry.as_ptr(),
                brush.as_ptr(),
                state.line_width,
                self.stroke_style(state).as_ptr(),
            )
        };
    }

    pub(crate) fn clip(&mut self) {
        let geometry = self.geometry(Transform::IDENTITY);
        let params = D2D1_LAYER_PARAMETERS {
            contentBounds: rect(-1.0e10, -1.0e10, 2.0e10, 2.0e10),
            geometricMask: geometry.as_ptr(),
            maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
            maskTransform: D2D_MATRIX_3X2_F::identity(),
            opacity: 1.0,
            ..Default::default()
        };
        unsafe {
            self.target.SetTransform(&D2D_MATRIX_3X2_F::identity());
            self.target.PushLayer(&params, null_mut());
        }
        self.layers += 1;
        self.apply_transform();
    }

    fn stroke_style(&self, state: &CanvasState) -> ComPtr<ID2D1StrokeStyle> {
        let cap = match state.line_cap {
            LineCap::Butt => D2D1_CAP_STYLE_FLAT,
            LineCap::Round => D2D1_CAP_STYLE_ROUND,
            LineCap::Square => D2D1_CAP_STYLE_SQUARE,
        };
        unsafe {
            self.factory.CreateStrokeStyle(
                &D2D1_STROKE_STYLE_PROPERTIES {
                    startCap: cap,
                    endCap: cap,
                    dashCap: cap,
                    lineJoin: match state.line_join {
                        LineJoin::Miter => D2D1_LINE_JOIN_MITER,
                        LineJoin::Round => D2D1_LINE_JOIN_ROUND,
                        LineJoin::Bevel => D2D1_LINE_JOIN_BEVEL,
                    },
                    miterLimit: 10.0,
                    ..Default::default()
                },
                null(),
                0,
            )
        }
        .expect("Direct2D stroke style")
    }

    pub(crate) fn translate(&mut self, x: f32, y: f32) {
        self.concat(Transform([1.0, 0.0, 0.0, 1.0, x, y]));
    }
    pub(crate) fn rotate(&mut self, angle: f32) {
        let (s, c) = angle.sin_cos();
        self.concat(Transform([c, s, -s, c, 0.0, 0.0]));
    }
    pub(crate) fn scale(&mut self, x: f32, y: f32) {
        self.concat(Transform([x, 0.0, 0.0, y, 0.0, 0.0]));
    }
    pub(crate) fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        self.transform = Transform([a, b, c, d, e, f]);
        self.apply_transform();
    }
    pub(crate) fn reset_transform(&mut self) {
        self.transform = Transform::IDENTITY;
        self.apply_transform();
    }
    fn concat(&mut self, transform: Transform) {
        self.transform = self.transform.concat(transform);
        self.apply_transform();
    }
    fn apply_transform(&self) {
        let [a, b, c, d, e, f] = self.transform.0;
        unsafe {
            self.target.SetTransform(&D2D_MATRIX_3X2_F {
                m11: a,
                m12: b,
                m21: c,
                m22: d,
                dx: e,
                dy: f,
            })
        };
    }

    pub(crate) fn fill_text(&mut self, s: &CanvasState, text: &str, x: f32, y: f32) {
        let Some(PreparedText {
            text,
            format,
            metrics,
        }) = self.prepare_text(s, text)
        else {
            return;
        };
        unsafe {
            let mut left = x;
            let mut top = y;
            match s.text_align {
                TextAlign::Center => left -= metrics.widthIncludingTrailingWhitespace / 2.0,
                TextAlign::Right | TextAlign::End => {
                    left -= metrics.widthIncludingTrailingWhitespace;
                }
                _ => {}
            }
            match s.text_baseline {
                TextBaseline::Middle => top -= metrics.height / 2.0,
                TextBaseline::Bottom | TextBaseline::Ideographic => {
                    top -= metrics.height;
                }
                TextBaseline::Alphabetic => top -= s.font_size * 0.8,
                _ => {}
            }
            let layout = D2D_RECT_F {
                left,
                top,
                right: left + metrics.widthIncludingTrailingWhitespace + s.font_size,
                bottom: top + metrics.height + s.font_size,
            };
            let brush = self.brush(s.fill, s.alpha);
            self.target.DrawText(
                text.as_ptr(),
                text.len() as u32,
                format.as_ptr(),
                &layout,
                brush.as_ptr(),
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            )
        }
    }
    pub(crate) fn measure_text(&mut self, s: &CanvasState, text: &str) -> f32 {
        self.prepare_text(s, text).map_or(0.0, |prepared| {
            prepared.metrics.widthIncludingTrailingWhitespace
        })
    }

    fn prepare_text(&mut self, s: &CanvasState, text: &str) -> Option<PreparedText> {
        let text: Vec<u16> = text.encode_utf16().collect();
        let format = self.format(s);
        unsafe {
            let layout = self
                .write
                .CreateTextLayout(
                    text.as_ptr(),
                    text.len() as u32,
                    format.as_ptr(),
                    100000.0,
                    10000.0,
                )
                .ok()?;
            let mut metrics = DWRITE_TEXT_METRICS::default();
            layout.GetMetrics(&mut metrics).ok()?;
            Some(PreparedText {
                text,
                format,
                metrics,
            })
        }
    }
    fn brush(&mut self, color: Color, alpha: f32) -> ComPtr<ID2D1SolidColorBrush> {
        let key = (color, alpha.to_bits());
        if let Some(brush) = self.brushes.get(&key) {
            return brush.clone();
        }
        let (r, g, b, a) = color.components(alpha);
        let brush = unsafe {
            self.target
                .CreateSolidColorBrush(
                    &D2D1_COLOR_F {
                        r: r as f32,
                        g: g as f32,
                        b: b as f32,
                        a: a as f32,
                    },
                    null(),
                )
                .expect("Direct2D brush")
        };
        self.brushes.insert(key, brush.clone());
        brush
    }
    fn format(&mut self, s: &CanvasState) -> ComPtr<IDWriteTextFormat> {
        let key = (s.font_family.clone(), s.font_size.to_bits(), s.font_weight);
        if let Some(format) = self.text_formats.get(&key) {
            return format.clone();
        }
        let family = (match s.font_family.as_ref() {
            "sans-serif" => "Segoe UI",
            "serif" => "Times New Roman",
            "monospace" => "Consolas",
            name => name,
        })
        .to_wide_string();
        let locale = "en-us".to_wide_string();
        let format = unsafe {
            self.write
                .CreateTextFormat(
                    family.as_ptr(),
                    null_mut(),
                    match s.font_weight {
                        FontWeight::Normal => DWRITE_FONT_WEIGHT_NORMAL,
                        FontWeight::Bold => DWRITE_FONT_WEIGHT_BOLD,
                    },
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    s.font_size,
                    locale.as_ptr(),
                )
                .expect("DirectWrite text format")
        };
        unsafe {
            let _ = format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_NEAR);
        }
        self.text_formats.insert(key, format.clone());
        format
    }
}
impl Drop for PlatformCanvasContext {
    fn drop(&mut self) {
        self.pop_layers(0);
    }
}
const fn vector([x, y]: Point) -> D2D_POINT_2F {
    D2D_POINT_2F { x, y }
}
fn rect(x: f32, y: f32, w: f32, h: f32) -> D2D_RECT_F {
    D2D_RECT_F {
        left: x.min(x + w),
        top: y.min(y + h),
        right: x.max(x + w),
        bottom: y.max(y + h),
    }
}
