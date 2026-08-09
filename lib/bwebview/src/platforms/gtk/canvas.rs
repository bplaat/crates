/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::Cell;
use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr::{null, null_mut};

use super::event_loop::send_event;
use super::headers::*;
use super::window::PlatformWindow;
use crate::{
    CanvasEvent, CanvasInterface, CanvasRenderingContext2d, CanvasState, Event, FontWeight,
    KeyboardEvent, LineCap, LineJoin, MouseEvent, TextAlign, TextBaseline, WindowEvent,
};

#[repr(C)]
#[derive(Clone, Copy)]
struct CairoMatrix {
    xx: f64,
    yx: f64,
    xy: f64,
    yy: f64,
    x0: f64,
    y0: f64,
}
#[repr(C)]
#[derive(Default)]
struct CairoTextExtents {
    x_bearing: f64,
    y_bearing: f64,
    width: f64,
    height: f64,
    x_advance: f64,
    y_advance: f64,
}
#[repr(C)]
struct CairoPath([u8; 0]);

unsafe extern "C" {
    fn gtk_widget_get_allocated_width(widget: *mut GtkWidget) -> i32;
    fn gtk_widget_get_allocated_height(widget: *mut GtkWidget) -> i32;
    fn gtk_widget_get_scale_factor(widget: *mut GtkWidget) -> i32;
    fn cairo_save(cr: *mut c_void);
    fn cairo_restore(cr: *mut c_void);
    fn cairo_set_source_rgba(cr: *mut c_void, r: f64, g: f64, b: f64, a: f64);
    fn cairo_set_operator(cr: *mut c_void, op: i32);
    fn cairo_paint(cr: *mut c_void);
    fn cairo_rectangle(cr: *mut c_void, x: f64, y: f64, w: f64, h: f64);
    fn cairo_fill(cr: *mut c_void);
    fn cairo_fill_preserve(cr: *mut c_void);
    fn cairo_stroke(cr: *mut c_void);
    fn cairo_stroke_preserve(cr: *mut c_void);
    fn cairo_new_path(cr: *mut c_void);
    fn cairo_new_sub_path(cr: *mut c_void);
    fn cairo_copy_path(cr: *mut c_void) -> *mut CairoPath;
    fn cairo_append_path(cr: *mut c_void, path: *const CairoPath);
    fn cairo_path_destroy(path: *mut CairoPath);
    fn cairo_close_path(cr: *mut c_void);
    fn cairo_move_to(cr: *mut c_void, x: f64, y: f64);
    fn cairo_line_to(cr: *mut c_void, x: f64, y: f64);
    fn cairo_arc(cr: *mut c_void, x: f64, y: f64, r: f64, a: f64, b: f64);
    fn cairo_arc_negative(cr: *mut c_void, x: f64, y: f64, r: f64, a: f64, b: f64);
    fn cairo_curve_to(cr: *mut c_void, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64);
    fn cairo_get_current_point(cr: *mut c_void, x: *mut f64, y: *mut f64);
    fn cairo_clip(cr: *mut c_void);
    fn cairo_translate(cr: *mut c_void, x: f64, y: f64);
    fn cairo_rotate(cr: *mut c_void, a: f64);
    fn cairo_scale(cr: *mut c_void, x: f64, y: f64);
    fn cairo_get_matrix(cr: *mut c_void, m: *mut CairoMatrix);
    fn cairo_set_matrix(cr: *mut c_void, m: *const CairoMatrix);
    fn cairo_transform(cr: *mut c_void, m: *const CairoMatrix);
    fn cairo_set_line_width(cr: *mut c_void, w: f64);
    fn cairo_set_line_cap(cr: *mut c_void, c: i32);
    fn cairo_set_line_join(cr: *mut c_void, j: i32);
    fn cairo_select_font_face(cr: *mut c_void, family: *const c_char, slant: i32, weight: i32);
    fn cairo_set_font_size(cr: *mut c_void, size: f64);
    fn cairo_text_extents(cr: *mut c_void, text: *const c_char, extents: *mut CairoTextExtents);
    fn cairo_show_text(cr: *mut c_void, text: *const c_char);
}

pub(crate) struct PlatformCanvas {
    widget: *mut GtkWidget,
}

impl PlatformCanvas {
    pub(crate) fn new(window: &PlatformWindow) -> Self {
        unsafe {
            let widget = gtk_drawing_area_new();
            gtk_widget_set_can_focus(widget, true);
            gtk_widget_add_events(
                widget,
                0x000004
                    | 0x000008
                    | 0x000010
                    | 0x000020
                    | 0x000100
                    | 0x000200
                    | 0x000400
                    | 0x000800
                    | 0x001000
                    | 0x002000,
            );
            gtk_container_add(window.0.window as *mut GtkWidget, widget);
            g_signal_connect_data(
                widget as *mut GObject,
                c"draw".as_ptr(),
                canvas_draw as *const c_void,
                null(),
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                widget as *mut GObject,
                c"button-press-event".as_ptr(),
                mouse_down as *const c_void,
                null(),
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                widget as *mut GObject,
                c"button-release-event".as_ptr(),
                mouse_up as *const c_void,
                null(),
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                widget as *mut GObject,
                c"motion-notify-event".as_ptr(),
                mouse_move as *const c_void,
                null(),
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window.0.window as *mut GObject,
                c"key-press-event".as_ptr(),
                key_down as *const c_void,
                null(),
                null(),
                G_CONNECT_DEFAULT,
            );
            g_signal_connect_data(
                window.0.window as *mut GObject,
                c"key-release-event".as_ptr(),
                key_up as *const c_void,
                null(),
                null(),
                G_CONNECT_DEFAULT,
            );
            gtk_widget_show_all(window.0.window as *mut GtkWidget);
            gtk_widget_grab_focus(widget);
            gtk_widget_queue_draw(widget);
            Self { widget }
        }
    }
}

thread_local! { static LAST_MOUSE:Cell<(f32,f32)>=const{Cell::new((0.0,0.0))}; }
fn modifiers(state: u32) -> (bool, bool, bool, bool) {
    (
        state & 8 != 0,
        state & 4 != 0,
        state & (1 << 26) != 0,
        state & 1 != 0,
    )
}
fn buttons(state: u32) -> u16 {
    ((state >> 8) & 1) as u16 | (((state >> 9) & 1) as u16) << 2 | (((state >> 10) & 1) as u16) << 1
}
fn button_event(e: &GdkEventButton) -> MouseEvent {
    let (alt, ctrl, meta, shift) = modifiers(e.state);
    let (last_x, last_y) = LAST_MOUSE.with(Cell::get);
    LAST_MOUSE.with(|p| p.set((e.x as f32, e.y as f32)));
    MouseEvent {
        client_x: e.x as f32,
        client_y: e.y as f32,
        screen_x: e.x_root as f32,
        screen_y: e.y_root as f32,
        movement_x: e.x as f32 - last_x,
        movement_y: e.y as f32 - last_y,
        button: match e.button {
            1 => 0,
            2 => 1,
            3 => 2,
            n => n as i16,
        },
        buttons: buttons(e.state),
        detail: 1,
        alt_key: alt,
        ctrl_key: ctrl,
        meta_key: meta,
        shift_key: shift,
    }
}
extern "C" fn mouse_down(_: *mut GtkWidget, e: &GdkEventButton, _: *mut c_void) -> bool {
    send_event(Event::Window(WindowEvent::MouseDown(button_event(e))));
    false
}
extern "C" fn mouse_up(_: *mut GtkWidget, e: &GdkEventButton, _: *mut c_void) -> bool {
    let event = button_event(e);
    send_event(Event::Window(WindowEvent::MouseUp(event.clone())));
    send_event(Event::Window(WindowEvent::Click(event)));
    false
}
extern "C" fn mouse_move(_: *mut GtkWidget, e: &GdkEventMotion, _: *mut c_void) -> bool {
    let (alt, ctrl, meta, shift) = modifiers(e.state);
    let (last_x, last_y) = LAST_MOUSE.with(Cell::get);
    LAST_MOUSE.with(|p| p.set((e.x as f32, e.y as f32)));
    send_event(Event::Window(WindowEvent::MouseMove(MouseEvent {
        client_x: e.x as f32,
        client_y: e.y as f32,
        screen_x: e.x_root as f32,
        screen_y: e.y_root as f32,
        movement_x: e.x as f32 - last_x,
        movement_y: e.y as f32 - last_y,
        button: -1,
        buttons: buttons(e.state),
        detail: 0,
        alt_key: alt,
        ctrl_key: ctrl,
        meta_key: meta,
        shift_key: shift,
    })));
    false
}
extern "C" fn key_down(_: *mut GtkWidget, e: &GdkEventKey, _: *mut c_void) -> bool {
    send_event(Event::Window(WindowEvent::KeyDown(key_event(e))));
    false
}
extern "C" fn key_up(_: *mut GtkWidget, e: &GdkEventKey, _: *mut c_void) -> bool {
    send_event(Event::Window(WindowEvent::KeyUp(key_event(e))));
    false
}
fn key_event(e: &GdkEventKey) -> KeyboardEvent {
    let (alt, ctrl, meta, shift) = modifiers(e.state);
    let unicode = unsafe { gdk_keyval_to_unicode(e.keyval) };
    let name = unsafe {
        let p = gdk_keyval_name(e.keyval);
        if p.is_null() {
            "Unidentified".into()
        } else {
            CStr::from_ptr(p).to_string_lossy().into_owned()
        }
    };
    let key = char::from_u32(unicode).map_or_else(
        || match name.as_str() {
            "Return" => "Enter".into(),
            "Left" => "ArrowLeft".into(),
            "Right" => "ArrowRight".into(),
            "Up" => "ArrowUp".into(),
            "Down" => "ArrowDown".into(),
            other => other.into(),
        },
        |c| c.to_string(),
    );
    let code = if key.len() == 1 && key.chars().next().is_some_and(char::is_alphabetic) {
        format!("Key{}", key.to_uppercase())
    } else {
        name
    };
    KeyboardEvent {
        key,
        code,
        location: 0,
        repeat: false,
        is_composing: false,
        alt_key: alt,
        ctrl_key: ctrl,
        meta_key: meta,
        shift_key: shift,
    }
}
impl CanvasInterface for PlatformCanvas {
    fn request_redraw(&mut self) {
        unsafe { gtk_widget_queue_draw(self.widget) }
    }
}

extern "C" fn canvas_draw(widget: *mut GtkWidget, cr: *mut c_void, _data: *mut c_void) -> bool {
    let mut initial = CairoMatrix {
        xx: 1.0,
        yx: 0.0,
        xy: 0.0,
        yy: 1.0,
        x0: 0.0,
        y0: 0.0,
    };
    unsafe { cairo_get_matrix(cr, &mut initial) };
    let mut context = CanvasRenderingContext2d::new(
        PlatformCanvasContext { cr, initial },
        unsafe { gtk_widget_get_allocated_width(widget) as f32 },
        unsafe { gtk_widget_get_allocated_height(widget) as f32 },
        unsafe { gtk_widget_get_scale_factor(widget) as f32 },
    );
    send_event(Event::Canvas(CanvasEvent::Draw(&mut context)));
    false
}

pub(crate) struct PlatformCanvasContext {
    cr: *mut c_void,
    initial: CairoMatrix,
}
impl PlatformCanvasContext {
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
            cairo_arc(self.cr, x + w - r, y + r, r, -std::f64::consts::FRAC_PI_2, 0.0);
            cairo_arc(self.cr, x + w - r, y + h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
            cairo_arc(self.cr, x + r, y + h - r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
            cairo_arc(self.cr, x + r, y + r, r, std::f64::consts::PI, std::f64::consts::PI * 1.5);
            cairo_close_path(self.cr);
        }
    }
    pub(crate) fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32) {
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }
        unsafe {
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
        unsafe { cairo_translate(self.cr, x.into(), y.into()) }
    }
    pub(crate) fn rotate(&mut self, a: f32) {
        unsafe { cairo_rotate(self.cr, a.into()) }
    }
    pub(crate) fn scale(&mut self, x: f32, y: f32) {
        unsafe { cairo_scale(self.cr, x.into(), y.into()) }
    }
    pub(crate) fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        let m = CairoMatrix {
            xx: a.into(),
            yx: b.into(),
            xy: c.into(),
            yy: d.into(),
            x0: e.into(),
            y0: f.into(),
        };
        unsafe {
            cairo_set_matrix(self.cr, &self.initial);
            cairo_transform(self.cr, &m)
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
        let family = CString::new(s.font_family.as_str())
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
        TextBaseline::Bottom | TextBaseline::Ideographic => {
            *y -= e.y_bearing + e.height
        }
        _ => {}
    }
}
