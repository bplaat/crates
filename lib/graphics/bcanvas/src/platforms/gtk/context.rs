/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CString, c_void};

use super::headers::*;
use crate::{CanvasState, FontWeight, LineCap, LineJoin, TextAlign, TextBaseline};

pub(crate) struct PlatformCanvasContext {
    cr: *mut c_void,
    initial: CairoMatrix,
}
impl PlatformCanvasContext {
    pub(super) unsafe fn new(cr: *mut c_void) -> Self {
        let mut initial = CairoMatrix {
            xx: 1.0,
            yx: 0.0,
            xy: 0.0,
            yy: 1.0,
            x0: 0.0,
            y0: 0.0,
        };
        unsafe {
            cairo_get_matrix(cr, &mut initial);
            cairo_save(cr);
            cairo_new_path(cr);
        }
        Self { cr, initial }
    }
    pub(crate) fn save(&mut self) {
        unsafe { cairo_save(self.cr) }
    }
    pub(crate) fn restore(&mut self) {
        unsafe { cairo_restore(self.cr) }
    }
    pub(crate) fn clear_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let path = self.take_path();
        unsafe {
            cairo_save(self.cr);
            cairo_rectangle(self.cr, x.into(), y.into(), w.into(), h.into());
            cairo_set_operator(self.cr, 0);
            cairo_fill(self.cr);
            cairo_restore(self.cr)
        }
        self.restore_path(path);
    }
    pub(crate) fn fill_rect(&mut self, s: &CanvasState, x: f32, y: f32, w: f32, h: f32) {
        let path = self.take_path();
        self.fill_style(s);
        unsafe {
            cairo_rectangle(self.cr, x.into(), y.into(), w.into(), h.into());
            cairo_fill(self.cr)
        }
        self.restore_path(path);
    }
    pub(crate) fn stroke_rect(&mut self, s: &CanvasState, x: f32, y: f32, w: f32, h: f32) {
        let path = self.take_path();
        self.stroke_style(s);
        unsafe {
            cairo_rectangle(self.cr, x.into(), y.into(), w.into(), h.into());
            cairo_stroke(self.cr)
        }
        self.restore_path(path);
    }
    pub(crate) fn begin_path(&mut self) {
        unsafe { cairo_new_path(self.cr) }
    }
    pub(crate) fn close_path(&mut self) {
        unsafe { cairo_close_path(self.cr) }
    }
    pub(crate) fn move_to(&mut self, x: f32, y: f32) {
        unsafe { cairo_move_to(self.cr, x.into(), y.into()) }
    }
    pub(crate) fn line_to(&mut self, x: f32, y: f32) {
        unsafe { cairo_line_to(self.cr, x.into(), y.into()) }
    }
    pub(crate) fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        unsafe { cairo_rectangle(self.cr, x.into(), y.into(), w.into(), h.into()) }
    }
    pub(crate) fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32) {
        let r = r.max(0.0).min(w.abs() * 0.5).min(h.abs() * 0.5) as f64;
        let (x, y, w, h) = (x as f64, y as f64, w as f64, h as f64);
        unsafe {
            cairo_new_sub_path(self.cr);
            cairo_arc(
                self.cr,
                x + w - r,
                y + r,
                r,
                -std::f64::consts::FRAC_PI_2,
                0.0,
            );
            cairo_arc(
                self.cr,
                x + w - r,
                y + h - r,
                r,
                0.0,
                std::f64::consts::FRAC_PI_2,
            );
            cairo_arc(
                self.cr,
                x + r,
                y + h - r,
                r,
                std::f64::consts::FRAC_PI_2,
                std::f64::consts::PI,
            );
            cairo_arc(
                self.cr,
                x + r,
                y + r,
                r,
                std::f64::consts::PI,
                std::f64::consts::PI * 1.5,
            );
            cairo_close_path(self.cr);
        }
    }
    pub(crate) fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32) {
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }
        unsafe {
            cairo_new_sub_path(self.cr);
            cairo_save(self.cr);
            cairo_translate(self.cr, x.into(), y.into());
            cairo_scale(self.cr, rx.into(), ry.into());
            cairo_arc(self.cr, 0.0, 0.0, 1.0, 0.0, std::f64::consts::TAU);
            cairo_restore(self.cr);
        }
    }
    pub(crate) fn arc(&mut self, x: f32, y: f32, r: f32, a: f32, b: f32, ccw: bool) {
        unsafe {
            if ccw {
                cairo_arc_negative(self.cr, x.into(), y.into(), r.into(), a.into(), b.into())
            } else {
                cairo_arc(self.cr, x.into(), y.into(), r.into(), a.into(), b.into())
            }
        }
    }
    pub(crate) fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        let (mut x0, mut y0) = (0.0, 0.0);
        unsafe {
            if cairo_has_current_point(self.cr) == 0 {
                cairo_move_to(self.cr, cpx.into(), cpy.into());
            }
            cairo_get_current_point(self.cr, &mut x0, &mut y0);
            cairo_curve_to(
                self.cr,
                x0 + 2.0 / 3.0 * (f64::from(cpx) - x0),
                y0 + 2.0 / 3.0 * (f64::from(cpy) - y0),
                f64::from(x) + 2.0 / 3.0 * (f64::from(cpx) - f64::from(x)),
                f64::from(y) + 2.0 / 3.0 * (f64::from(cpy) - f64::from(y)),
                x.into(),
                y.into(),
            )
        }
    }
    pub(crate) fn bezier_curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
        unsafe {
            if cairo_has_current_point(self.cr) == 0 {
                cairo_move_to(self.cr, a.into(), b.into());
            }
            cairo_curve_to(
                self.cr,
                a.into(),
                b.into(),
                c.into(),
                d.into(),
                x.into(),
                y.into(),
            )
        }
    }
    pub(crate) fn fill(&mut self, s: &CanvasState) {
        self.fill_style(s);
        unsafe { cairo_fill_preserve(self.cr) }
    }
    pub(crate) fn stroke(&mut self, s: &CanvasState) {
        self.stroke_style(s);
        unsafe { cairo_stroke_preserve(self.cr) }
    }
    pub(crate) fn clip(&mut self) {
        let path = unsafe { cairo_copy_path(self.cr) };
        unsafe { cairo_clip(self.cr) };
        self.restore_path(path);
    }
    pub(crate) fn translate(&mut self, x: f32, y: f32) {
        self.transform([1.0, 0.0, 0.0, 1.0, x.into(), y.into()], false);
    }
    pub(crate) fn rotate(&mut self, a: f32) {
        let (sin, cos) = f64::from(a).sin_cos();
        self.transform([cos, sin, -sin, cos, 0.0, 0.0], false);
    }
    pub(crate) fn scale(&mut self, x: f32, y: f32) {
        self.transform([x.into(), 0.0, 0.0, y.into(), 0.0, 0.0], false);
    }
    pub(crate) fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        self.transform(
            [a.into(), b.into(), c.into(), d.into(), e.into(), f.into()],
            true,
        );
    }
    fn transform(&mut self, [a, b, c, d, e, f]: [f64; 6], replace: bool) {
        let m = CairoMatrix {
            xx: a,
            yx: b,
            xy: c,
            yy: d,
            x0: e,
            y0: f,
        };
        let mut base = self.initial;
        let mut candidate = self.initial;
        unsafe {
            if !replace {
                cairo_get_matrix(self.cr, &mut base);
            }
            // Cairo multiplication applies its first argument before the second.
            cairo_matrix_multiply(&mut candidate, &m, &base);
            let finite = |m: CairoMatrix| {
                [m.xx, m.yx, m.xy, m.yy, m.x0, m.y0]
                    .iter()
                    .all(|v| v.is_finite())
            };
            let mut inverse = candidate;
            // Context errors are permanent, unlike a failed matrix inversion.
            if finite(candidate) && cairo_matrix_invert(&mut inverse) == 0 && finite(inverse) {
                cairo_set_matrix(self.cr, &candidate);
            }
        }
    }
    pub(crate) fn reset_transform(&mut self) {
        unsafe { cairo_set_matrix(self.cr, &self.initial) }
    }
    pub(crate) fn fill_text(&mut self, s: &CanvasState, text: &str, x: f32, y: f32) {
        let Ok(text) = CString::new(text) else { return };
        let ext = self.text_extents(s, &text);
        let (mut x, mut y) = (f64::from(x), f64::from(y));
        align_text(s, &mut x, &mut y, &ext);
        let path = self.take_path();
        self.fill_style(s);
        unsafe {
            cairo_move_to(self.cr, x, y);
            cairo_show_text(self.cr, text.as_ptr())
        }
        self.restore_path(path);
    }
    pub(crate) fn measure_text(&mut self, s: &CanvasState, text: &str) -> f32 {
        let Ok(text) = CString::new(text) else {
            return 0.0;
        };
        self.text_extents(s, &text).x_advance as f32
    }
    fn fill_style(&self, s: &CanvasState) {
        let (r, g, b, a) = s.fill.components(s.alpha);
        unsafe { cairo_set_source_rgba(self.cr, r, g, b, a) }
    }
    fn stroke_style(&self, s: &CanvasState) {
        let (r, g, b, a) = s.stroke.components(s.alpha);
        unsafe {
            cairo_set_source_rgba(self.cr, r, g, b, a);
            cairo_set_line_width(self.cr, s.line_width.into());
            cairo_set_line_cap(
                self.cr,
                match s.line_cap {
                    LineCap::Butt => 0,
                    LineCap::Round => 1,
                    LineCap::Square => 2,
                },
            );
            cairo_set_line_join(
                self.cr,
                match s.line_join {
                    LineJoin::Miter => 0,
                    LineJoin::Round => 1,
                    LineJoin::Bevel => 2,
                },
            )
        }
    }
    fn text_extents(&self, s: &CanvasState, text: &CString) -> CairoTextExtents {
        let family = CString::new(s.font_family.as_ref())
            .unwrap_or_else(|_| CString::new("sans-serif").expect("literal"));
        let mut ext = CairoTextExtents::default();
        unsafe {
            cairo_select_font_face(
                self.cr,
                family.as_ptr(),
                0,
                match s.font_weight {
                    FontWeight::Normal => 0,
                    FontWeight::Bold => 1,
                },
            );
            cairo_set_font_size(self.cr, s.font_size.into());
            cairo_text_extents(self.cr, text.as_ptr(), &mut ext)
        }
        ext
    }
    fn take_path(&self) -> *mut CairoPath {
        unsafe {
            let path = cairo_copy_path(self.cr);
            cairo_new_path(self.cr);
            path
        }
    }
    fn restore_path(&self, path: *mut CairoPath) {
        if path.is_null() {
            return;
        }
        unsafe {
            cairo_new_path(self.cr);
            cairo_append_path(self.cr, path);
            cairo_path_destroy(path);
        }
    }
}
fn align_text(s: &CanvasState, x: &mut f64, y: &mut f64, e: &CairoTextExtents) {
    match s.text_align {
        TextAlign::Center => *x -= e.x_advance / 2.0,
        TextAlign::Right | TextAlign::End => *x -= e.x_advance,
        _ => {}
    }
    match s.text_baseline {
        TextBaseline::Top | TextBaseline::Hanging => *y -= e.y_bearing,
        TextBaseline::Middle => *y -= e.y_bearing + e.height / 2.0,
        TextBaseline::Bottom | TextBaseline::Ideographic => *y -= e.y_bearing + e.height,
        _ => {}
    }
}

impl Drop for PlatformCanvasContext {
    fn drop(&mut self) {
        unsafe {
            cairo_new_path(self.cr);
            cairo_restore(self.cr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    struct Surface(*mut c_void, *mut c_void);

    impl Surface {
        fn new() -> Self {
            unsafe {
                let surface = cairo_image_surface_create(0, 4, 1);
                Self(surface, cairo_create(surface))
            }
        }

        fn pixels(&self) -> [u32; 4] {
            unsafe {
                cairo_surface_flush(self.0);
                std::ptr::read_unaligned(cairo_image_surface_get_data(self.0).cast())
            }
        }
    }

    impl Drop for Surface {
        fn drop(&mut self) {
            unsafe {
                cairo_destroy(self.1);
                cairo_surface_destroy(self.0);
            }
        }
    }

    #[test]
    fn invalid_transforms_preserve_drawing_reset_and_restore() {
        let surface = Surface::new();
        let mut canvas = unsafe { PlatformCanvasContext::new(surface.1) };
        let state = CanvasState {
            fill: Color::rgb(255, 0, 0),
            ..CanvasState::default()
        };
        canvas.translate(1.0, 0.0);
        canvas.save();
        canvas.scale(0.0, 1.0);
        canvas.set_transform(1.0, 2.0, 2.0, 4.0, 0.0, 0.0);
        canvas.translate(f32::INFINITY, 0.0);
        canvas.rotate(f32::NAN);
        canvas.scale(f32::INFINITY, 1.0);
        canvas.set_transform(1.0, 0.0, 0.0, 1.0, f32::NAN, 0.0);
        assert_eq!(unsafe { cairo_status(surface.1) }, 0);
        canvas.fill_rect(&state, 0.0, 0.0, 1.0, 1.0);
        canvas.reset_transform();
        canvas.fill_rect(&state, 0.0, 0.0, 1.0, 1.0);
        canvas.restore();
        canvas.fill_rect(&state, 1.0, 0.0, 1.0, 1.0);
        drop(canvas);
        assert_eq!(unsafe { cairo_status(surface.1) }, 0);
        assert_eq!(surface.pixels(), [0xffff0000, 0xffff0000, 0xffff0000, 0]);
    }

    #[test]
    fn transforms_preserve_composition_and_initial_device_scale() {
        let surface = Surface::new();
        unsafe { cairo_scale(surface.1, 2.0, 2.0) };
        let mut canvas = unsafe { PlatformCanvasContext::new(surface.1) };
        canvas.translate(1.0, 2.0);
        canvas.scale(3.0, -4.0);
        let mut m = canvas.initial;
        unsafe { cairo_get_matrix(surface.1, &mut m) };
        assert_eq!([m.xx, m.yy, m.x0, m.y0], [6.0, -8.0, 2.0, 4.0]);
        canvas.set_transform(1.0, 0.0, 0.0, 1.0, 3.0, 4.0);
        unsafe { cairo_get_matrix(surface.1, &mut m) };
        assert_eq!([m.xx, m.yy, m.x0, m.y0], [2.0, 2.0, 6.0, 8.0]);
        canvas.scale(0.0001, 0.0001);
        unsafe { cairo_get_matrix(surface.1, &mut m) };
        assert!(m.xx > 0.0 && m.xx < 0.001);
        assert_eq!(unsafe { cairo_status(surface.1) }, 0);
    }
}
