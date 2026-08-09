/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::f32::consts::TAU;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::ptr::null_mut;

use windows::Win32::Foundation::HWND as WinHwnd;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_UNKNOWN;
use windows::core::HSTRING;
use windows_numerics::{Matrix3x2, Vector2};

use super::event_loop::send_event;
use super::win32::*;
use super::window::PlatformWindow;
use crate::{
    CanvasEvent, CanvasInterface, CanvasRenderingContext2d, CanvasState, Color, Event, FontWeight,
    TextAlign, TextBaseline,
};

pub(super) struct CanvasData {
    pub hwnd: HWND,
    factory: ID2D1Factory,
    target: ID2D1HwndRenderTarget,
    write: IDWriteFactory,
    brushes: Rc<RefCell<HashMap<(Color, u32), ID2D1SolidColorBrush>>>,
    text_formats: Rc<RefCell<HashMap<(String, u32, FontWeight), IDWriteTextFormat>>>,
}

pub(crate) struct PlatformCanvas {
    data: Box<CanvasData>,
    window_data: *mut super::window::WindowData,
}

impl PlatformCanvas {
    pub(crate) fn new(window: &PlatformWindow) -> Self {
        unsafe {
            let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
                .expect("Direct2D factory");
            let write: IDWriteFactory =
                DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED).expect("DirectWrite factory");
            let mut rect = RECT::default();
            GetClientRect(window.0.hwnd, &mut rect);
            let props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_UNKNOWN,
                    alphaMode: D2D1_ALPHA_MODE_UNKNOWN,
                },
                dpiX: 0.0,
                dpiY: 0.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };
            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd: WinHwnd(window.0.hwnd),
                pixelSize: D2D_SIZE_U {
                    width: (rect.right - rect.left) as u32,
                    height: (rect.bottom - rect.top) as u32,
                },
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };
            let target = factory
                .CreateHwndRenderTarget(&props, &hwnd_props)
                .expect("Direct2D render target");
            let mut data = Box::new(CanvasData {
                hwnd: window.0.hwnd,
                factory,
                target,
                write,
                brushes: Rc::new(RefCell::new(HashMap::new())),
                text_formats: Rc::new(RefCell::new(HashMap::new())),
            });
            let window_data =
                &*window.0 as *const super::window::WindowData as *mut super::window::WindowData;
            (*window_data).canvas_data = Some(data.as_mut() as *mut CanvasData);
            InvalidateRect(window.0.hwnd, null_mut(), FALSE);
            Self { data, window_data }
        }
    }
}
impl Drop for PlatformCanvas {
    fn drop(&mut self) {
        unsafe {
            if (*self.window_data).canvas_data == Some(self.data.as_mut() as *mut _) {
                (*self.window_data).canvas_data = None;
            }
        }
    }
}
impl CanvasInterface for PlatformCanvas {
    fn request_redraw(&mut self) {
        unsafe {
            InvalidateRect(self.data.hwnd, null_mut(), FALSE);
        }
    }
    fn request_animation_frame(&mut self) {
        unsafe {
            SetTimer(self.data.hwnd, CANVAS_ANIMATION_TIMER, 16, None);
        }
    }
}

pub(super) const CANVAS_ANIMATION_TIMER: usize = 0xBCA;

pub(super) fn draw(data: &mut CanvasData) {
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(data.hwnd, &mut rect);
        let _ = data.target.Resize(&D2D_SIZE_U {
            width: (rect.right - rect.left) as u32,
            height: (rect.bottom - rect.top) as u32,
        });
        data.target.BeginDraw();
        let dpi = GetDpiForWindow(data.hwnd);
        let scale_factor = if dpi == 0 {
            1.0
        } else {
            dpi as f32 / USER_DEFAULT_SCREEN_DPI as f32
        };
        let mut context = CanvasRenderingContext2d::new(
            PlatformCanvasContext {
                target: data.target.clone(),
                factory: data.factory.clone(),
                write: data.write.clone(),
                path: Path::None,
                last: None,
                initial: Matrix3x2::identity(),
                brushes: data.brushes.clone(),
                text_formats: data.text_formats.clone(),
            },
            (rect.right - rect.left) as f32 / scale_factor,
            (rect.bottom - rect.top) as f32 / scale_factor,
            scale_factor,
        );
        send_event(Event::Canvas(CanvasEvent::Draw(&mut context)));
        let _ = data.target.EndDraw(None, None);
    }
}

enum Path {
    None,
    Rect(D2D_RECT_F),
    RoundRect(D2D1_ROUNDED_RECT),
    Ellipse(D2D1_ELLIPSE),
}
pub(crate) struct PlatformCanvasContext {
    target: ID2D1HwndRenderTarget,
    factory: ID2D1Factory,
    write: IDWriteFactory,
    path: Path,
    last: Option<Vector2>,
    initial: Matrix3x2,
    brushes: Rc<RefCell<HashMap<(Color, u32), ID2D1SolidColorBrush>>>,
    text_formats: Rc<RefCell<HashMap<(String, u32, FontWeight), IDWriteTextFormat>>>,
}
impl PlatformCanvasContext {
    pub(crate) fn save(&mut self) {}
    pub(crate) fn restore(&mut self) {}
    pub(crate) fn clear_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        unsafe {
            if x <= 0.0 && y <= 0.0 {
                self.target.Clear(Some(&D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }));
            } else {
                let brush = self.brush(Color::rgba(0, 0, 0, 0), 1.0);
                self.target.FillRectangle(&rect(x, y, w, h), &brush);
            }
        }
    }
    pub(crate) fn fill_rect(&mut self, s: &CanvasState, x: f32, y: f32, w: f32, h: f32) {
        unsafe {
            self.target
                .FillRectangle(&rect(x, y, w, h), &self.brush(s.fill, s.alpha))
        }
    }
    pub(crate) fn stroke_rect(&mut self, s: &CanvasState, x: f32, y: f32, w: f32, h: f32) {
        unsafe {
            self.target.DrawRectangle(
                &rect(x, y, w, h),
                &self.brush(s.stroke, s.alpha),
                s.line_width,
                None,
            )
        }
    }
    pub(crate) fn begin_path(&mut self) {
        self.path = Path::None;
        self.last = None
    }
    pub(crate) fn close_path(&mut self) {}
    pub(crate) fn move_to(&mut self, x: f32, y: f32) {
        self.last = Some(Vector2 { X: x, Y: y })
    }
    pub(crate) fn line_to(&mut self, x: f32, y: f32) {
        if let Some(from) = self.last {
            unsafe {
                self.target.DrawLine(
                    from,
                    Vector2 { X: x, Y: y },
                    &self.brush(Color::rgb(0, 0, 0), 1.0),
                    1.0,
                    None,
                )
            }
        }
        self.last = Some(Vector2 { X: x, Y: y })
    }
    pub(crate) fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.path = Path::Rect(rect(x, y, w, h))
    }
    pub(crate) fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32) {
        let r = r.max(0.0).min(w.abs() * 0.5).min(h.abs() * 0.5);
        self.path = Path::RoundRect(D2D1_ROUNDED_RECT {
            rect: rect(x, y, w, h),
            radiusX: r,
            radiusY: r,
        });
    }
    pub(crate) fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32) {
        self.path = Path::Ellipse(D2D1_ELLIPSE {
            point: Vector2 { X: x, Y: y },
            radiusX: rx,
            radiusY: ry,
        });
    }
    pub(crate) fn arc(&mut self, x: f32, y: f32, r: f32, start: f32, end: f32, _ccw: bool) {
        if (end - start).abs() >= TAU - 0.001 {
            self.path = Path::Ellipse(D2D1_ELLIPSE {
                point: Vector2 { X: x, Y: y },
                radiusX: r,
                radiusY: r,
            })
        }
    }
    pub(crate) fn quadratic_curve_to(&mut self, _a: f32, _b: f32, x: f32, y: f32) {
        self.last = Some(Vector2 { X: x, Y: y })
    }
    pub(crate) fn bezier_curve_to(&mut self, _a: f32, _b: f32, _c: f32, _d: f32, x: f32, y: f32) {
        self.last = Some(Vector2 { X: x, Y: y })
    }
    pub(crate) fn fill(&mut self, s: &CanvasState) {
        unsafe {
            let brush = self.brush(s.fill, s.alpha);
            match self.path {
                Path::Rect(r) => self.target.FillRectangle(&r, &brush),
                Path::RoundRect(r) => self.target.FillRoundedRectangle(&r, &brush),
                Path::Ellipse(e) => self.target.FillEllipse(&e, &brush),
                Path::None => {}
            }
        }
    }
    pub(crate) fn stroke(&mut self, s: &CanvasState) {
        unsafe {
            let brush = self.brush(s.stroke, s.alpha);
            match self.path {
                Path::Rect(r) => self.target.DrawRectangle(&r, &brush, s.line_width, None),
                Path::RoundRect(r) => self.target.DrawRoundedRectangle(&r, &brush, s.line_width, None),
                Path::Ellipse(e) => self.target.DrawEllipse(&e, &brush, s.line_width, None),
                Path::None => {}
            }
        }
    }
    pub(crate) fn clip(&mut self) {}
    pub(crate) fn translate(&mut self, x: f32, y: f32) {
        self.concat(Matrix3x2::translation(x, y))
    }
    pub(crate) fn rotate(&mut self, a: f32) {
        self.concat(Matrix3x2::rotation(a))
    }
    pub(crate) fn scale(&mut self, x: f32, y: f32) {
        self.concat(Matrix3x2::scale(x, y))
    }
    pub(crate) fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        unsafe {
            self.target.SetTransform(&Matrix3x2 {
                M11: a,
                M12: b,
                M21: c,
                M22: d,
                M31: e,
                M32: f,
            })
        }
    }
    pub(crate) fn reset_transform(&mut self) {
        unsafe { self.target.SetTransform(&self.initial) }
    }
    pub(crate) fn fill_text(&mut self, s: &CanvasState, text: &str, x: f32, y: f32) {
        unsafe {
            let wide: Vec<u16> = text.encode_utf16().collect();
            let format = self.format(s);
            let Ok(text_layout) = self
                .write
                .CreateTextLayout(&wide, &format, 100000.0, 10000.0)
            else {
                return;
            };
            let mut metrics = DWRITE_TEXT_METRICS::default();
            if text_layout.GetMetrics(&mut metrics).is_err() {
                return;
            }
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
            self.target.DrawText(
                &wide,
                &format,
                &layout,
                &self.brush(s.fill, s.alpha),
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            )
        }
    }
    pub(crate) fn measure_text(&mut self, s: &CanvasState, text: &str) -> f32 {
        unsafe {
            let wide: Vec<u16> = text.encode_utf16().collect();
            let Ok(layout) = self
                .write
                .CreateTextLayout(&wide, &self.format(s), 100000.0, 10000.0)
            else {
                return 0.0;
            };
            let mut metrics = DWRITE_TEXT_METRICS::default();
            if layout.GetMetrics(&mut metrics).is_ok() {
                metrics.widthIncludingTrailingWhitespace
            } else {
                0.0
            }
        }
    }
    fn brush(&self, color: Color, alpha: f32) -> ID2D1SolidColorBrush {
        let key = (color, alpha.to_bits());
        if let Some(brush) = self.brushes.borrow().get(&key) {
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
                    None,
                )
                .expect("Direct2D brush")
        };
        self.brushes.borrow_mut().insert(key, brush.clone());
        brush
    }
    fn format(&self, s: &CanvasState) -> IDWriteTextFormat {
        let key = (s.font_family.clone(), s.font_size.to_bits(), s.font_weight);
        if let Some(format) = self.text_formats.borrow().get(&key) {
            return format.clone();
        }
        let family = HSTRING::from(&s.font_family);
        let locale = HSTRING::from("en-us");
        let format = unsafe {
            self.write
                .CreateTextFormat(
                    &family,
                    None,
                    match s.font_weight {
                        FontWeight::Normal => DWRITE_FONT_WEIGHT_NORMAL,
                        FontWeight::Bold => DWRITE_FONT_WEIGHT_BOLD,
                    },
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    s.font_size,
                    &locale,
                )
                .expect("DirectWrite text format")
        };
        unsafe {
            let _ = format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_NEAR);
        }
        self.text_formats.borrow_mut().insert(key, format.clone());
        format
    }
    fn concat(&mut self, m: Matrix3x2) {
        let mut old = Matrix3x2::identity();
        unsafe {
            self.target.GetTransform(&mut old);
            self.target.SetTransform(&(m * old));
        }
    }
}
fn rect(x: f32, y: f32, w: f32, h: f32) -> D2D_RECT_F {
    D2D_RECT_F {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    }
}
