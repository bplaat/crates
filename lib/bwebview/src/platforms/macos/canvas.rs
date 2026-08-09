/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::ffi::{c_char, c_void};
use std::ptr::{null, null_mut};

use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};

use super::cocoa::*;
use super::event_loop::send_event;
use super::window::PlatformWindow;
use crate::{
    CanvasEvent, CanvasInterface, CanvasRenderingContext2d, CanvasState, Color, Event, FontWeight,
    KeyboardEvent, LineCap, LineJoin, MouseEvent, TextAlign, TextBaseline, WindowEvent,
};

type CGContextRef = *mut c_void;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGContextSaveGState(context: CGContextRef);
    fn CGContextRestoreGState(context: CGContextRef);
    fn CGContextClearRect(context: CGContextRef, rect: CGRect);
    fn CGContextSetRGBFillColor(context: CGContextRef, r: f64, g: f64, b: f64, a: f64);
    fn CGContextSetRGBStrokeColor(context: CGContextRef, r: f64, g: f64, b: f64, a: f64);
    fn CGContextSetLineWidth(context: CGContextRef, width: f64);
    fn CGContextSetLineCap(context: CGContextRef, cap: i32);
    fn CGContextSetLineJoin(context: CGContextRef, join: i32);
    fn CGContextFillRect(context: CGContextRef, rect: CGRect);
    fn CGContextStrokeRect(context: CGContextRef, rect: CGRect);
    fn CGContextBeginPath(context: CGContextRef);
    fn CGContextClosePath(context: CGContextRef);
    fn CGContextMoveToPoint(context: CGContextRef, x: f64, y: f64);
    fn CGContextAddLineToPoint(context: CGContextRef, x: f64, y: f64);
    fn CGContextAddRect(context: CGContextRef, rect: CGRect);
    fn CGContextAddEllipseInRect(context: CGContextRef, rect: CGRect);
    fn CGPathCreateWithRoundedRect(
        rect: CGRect,
        corner_width: f64,
        corner_height: f64,
        transform: *const CGAffineTransform,
    ) -> *mut c_void;
    fn CGContextAddArc(
        context: CGContextRef,
        x: f64,
        y: f64,
        radius: f64,
        start: f64,
        end: f64,
        clockwise: bool,
    );
    fn CGContextAddQuadCurveToPoint(context: CGContextRef, cpx: f64, cpy: f64, x: f64, y: f64);
    fn CGContextAddCurveToPoint(
        context: CGContextRef,
        cp1x: f64,
        cp1y: f64,
        cp2x: f64,
        cp2y: f64,
        x: f64,
        y: f64,
    );
    fn CGContextFillPath(context: CGContextRef);
    fn CGContextStrokePath(context: CGContextRef);
    fn CGContextCopyPath(context: CGContextRef) -> *mut c_void;
    fn CGContextAddPath(context: CGContextRef, path: *mut c_void);
    fn CGPathRelease(path: *mut c_void);
    fn CGContextClip(context: CGContextRef);
    fn CGContextTranslateCTM(context: CGContextRef, x: f64, y: f64);
    fn CGContextRotateCTM(context: CGContextRef, angle: f64);
    fn CGContextScaleCTM(context: CGContextRef, x: f64, y: f64);
    fn CGContextConcatCTM(context: CGContextRef, transform: CGAffineTransform);
    fn CGContextGetCTM(context: CGContextRef) -> CGAffineTransform;
    fn CGContextSetCTM(context: CGContextRef, transform: CGAffineTransform);
}

#[derive(Clone, Copy)]
#[repr(C)]
struct CGAffineTransform {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    tx: f64,
    ty: f64,
}

define_class!(
    #[unsafe(super(NSView))]
    struct CanvasView;

    impl CanvasView {
        #[unsafe(method(isFlipped))]
        const fn _is_flipped(&self) -> Bool { Bool::YES }

        #[unsafe(method(acceptsFirstResponder))]
        const fn _accepts_first_responder(&self) -> Bool { Bool::YES }

        #[unsafe(method(drawRect:))]
        fn _draw_rect(&self, _: NSRect) { self.draw(); }

        #[unsafe(method(requestAnimationFrame))]
        fn _request_animation_frame(&self) {
            let view = self as *const CanvasView as *mut Object;
            let _: () = unsafe { msg_send![view, setNeedsDisplay:Bool::YES] };
        }

        #[unsafe(method(mouseDown:))]
        fn _mouse_down(&self, event:*mut Object) { send_event(Event::Window(WindowEvent::MouseDown(mouse_event(self,event)))); }
        #[unsafe(method(mouseUp:))]
        fn _mouse_up(&self, event:*mut Object) { let event=mouse_event(self,event); send_event(Event::Window(WindowEvent::MouseUp(event.clone()))); send_event(Event::Window(WindowEvent::Click(event))); }
        #[unsafe(method(mouseMoved:))]
        fn _mouse_moved(&self, event:*mut Object) { send_event(Event::Window(WindowEvent::MouseMove(mouse_event(self,event)))); }
        #[unsafe(method(mouseDragged:))]
        fn _mouse_dragged(&self, event:*mut Object) { send_event(Event::Window(WindowEvent::MouseMove(mouse_event(self,event)))); }
        #[unsafe(method(rightMouseDown:))]
        fn _right_mouse_down(&self, event:*mut Object) { send_event(Event::Window(WindowEvent::MouseDown(mouse_event(self,event)))); }
        #[unsafe(method(rightMouseUp:))]
        fn _right_mouse_up(&self, event:*mut Object) { send_event(Event::Window(WindowEvent::MouseUp(mouse_event(self,event)))); }
        #[unsafe(method(keyDown:))]
        fn _key_down(&self,event:*mut Object){send_event(Event::Window(WindowEvent::KeyDown(keyboard_event(event))));}
        #[unsafe(method(keyUp:))]
        fn _key_up(&self,event:*mut Object){send_event(Event::Window(WindowEvent::KeyUp(keyboard_event(event))));}
    }
);

impl CanvasView {
    fn draw(&self) {
        unsafe {
            let graphics_context: *mut Object =
                msg_send![class!(NSGraphicsContext), currentContext];
            let context: CGContextRef = msg_send![graphics_context, CGContext];
            if context.is_null() {
                return;
            }
            let view: *mut Object = self as *const CanvasView as *mut Object;
            let bounds: NSRect = msg_send![view, bounds];
            let window: *mut Object = msg_send![view, window];
            let scale_factor: f64 = msg_send![window, backingScaleFactor];
            let mut context = CanvasRenderingContext2d::new(
                PlatformCanvasContext {
                    context,
                    initial_transform: CGContextGetCTM(context),
                    text_attributes: HashMap::new(),
                },
                bounds.size.width as f32,
                bounds.size.height as f32,
                scale_factor as f32,
            );
            send_event(Event::Canvas(CanvasEvent::Draw(&mut context)));
        }
    }
}

pub(crate) struct PlatformCanvas {
    view: *mut Object,
}

impl PlatformCanvas {
    pub(crate) fn new(window: &PlatformWindow) -> Self {
        unsafe {
            let content: *mut Object = msg_send![window.0.window, contentView];
            let bounds: NSRect = msg_send![content, bounds];
            let view: *mut Object = msg_send![CanvasView::class(), alloc];
            let view: *mut Object = msg_send![view, initWithFrame:bounds];
            let _: () =
                msg_send![view, setAutoresizingMask:NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE];
            let _: () = msg_send![content, addSubview:view, positioned:NS_WINDOW_BELOW, relativeTo:null_mut::<Object>()];
            let _: Bool = msg_send![window.0.window, makeFirstResponder:view];
            let _: () = msg_send![window.0.window, setAcceptsMouseMovedEvents:Bool::YES];
            let _: () = msg_send![view, setNeedsDisplay:Bool::YES];
            Self { view }
        }
    }
}

fn mouse_event(view: &CanvasView, event: *mut Object) -> MouseEvent {
    unsafe {
        let view = view as *const CanvasView as *mut Object;
        let point: NSPoint = msg_send![event, locationInWindow];
        let point: NSPoint = msg_send![view,convertPoint:point,fromView:null_mut::<Object>()];
        let window: *mut Object = msg_send![view, window];
        let screen: NSPoint = msg_send![window,convertPointToScreen:point];
        let button_number: i64 = msg_send![event, buttonNumber];
        let click_count: i64 = msg_send![event, clickCount];
        let delta_x: f64 = msg_send![event, deltaX];
        let delta_y: f64 = msg_send![event, deltaY];
        let flags: u64 = msg_send![event, modifierFlags];
        let pressed: u64 = msg_send![class!(NSEvent), pressedMouseButtons];
        MouseEvent {
            client_x: point.x as f32,
            client_y: point.y as f32,
            screen_x: screen.x as f32,
            screen_y: screen.y as f32,
            movement_x: delta_x as f32,
            movement_y: delta_y as f32,
            button: match button_number {
                0 => 0,
                1 => 2,
                2 => 1,
                n => n as i16,
            },
            buttons: ((pressed & 1) | ((pressed & 2) << 1) | ((pressed & 4) >> 1)) as u16,
            detail: click_count.min(i64::from(u16::MAX)) as u16,
            alt_key: flags & NS_EVENT_MODIFIER_FLAG_OPTION != 0,
            ctrl_key: flags & NS_EVENT_MODIFIER_FLAG_CONTROL != 0,
            meta_key: flags & NS_EVENT_MODIFIER_FLAG_COMMAND != 0,
            shift_key: flags & NS_EVENT_MODIFIER_FLAG_SHIFT != 0,
        }
    }
}

fn keyboard_event(event: *mut Object) -> KeyboardEvent {
    unsafe {
        let chars: NSString = msg_send![event, charactersIgnoringModifiers];
        let mut key = chars.to_string();
        let key_code: u16 = msg_send![event, keyCode];
        if let Some(named) = mac_named_key(key_code) {
            key = named.into();
        }
        let flags: u64 = msg_send![event, modifierFlags];
        let repeat: Bool = msg_send![event, isARepeat];
        KeyboardEvent {
            key,
            code: mac_code(key_code).into(),
            location: 0,
            repeat: repeat == Bool::YES,
            is_composing: false,
            alt_key: flags & NS_EVENT_MODIFIER_FLAG_OPTION != 0,
            ctrl_key: flags & NS_EVENT_MODIFIER_FLAG_CONTROL != 0,
            meta_key: flags & NS_EVENT_MODIFIER_FLAG_COMMAND != 0,
            shift_key: flags & NS_EVENT_MODIFIER_FLAG_SHIFT != 0,
        }
    }
}
const fn mac_named_key(code: u16) -> Option<&'static str> {
    Some(match code {
        36 => "Enter",
        48 => "Tab",
        49 => " ",
        51 => "Backspace",
        53 => "Escape",
        123 => "ArrowLeft",
        124 => "ArrowRight",
        125 => "ArrowDown",
        126 => "ArrowUp",
        _ => return None,
    })
}
const fn mac_code(code: u16) -> &'static str {
    match code {
        0 => "KeyA",
        1 => "KeyS",
        2 => "KeyD",
        3 => "KeyF",
        4 => "KeyH",
        5 => "KeyG",
        6 => "KeyZ",
        7 => "KeyX",
        8 => "KeyC",
        9 => "KeyV",
        11 => "KeyB",
        12 => "KeyQ",
        13 => "KeyW",
        14 => "KeyE",
        15 => "KeyR",
        16 => "KeyY",
        17 => "KeyT",
        31 => "KeyO",
        32 => "KeyU",
        34 => "KeyI",
        35 => "KeyP",
        37 => "KeyL",
        38 => "KeyJ",
        40 => "KeyK",
        45 => "KeyN",
        46 => "KeyM",
        36 => "Enter",
        48 => "Tab",
        49 => "Space",
        51 => "Backspace",
        53 => "Escape",
        123 => "ArrowLeft",
        124 => "ArrowRight",
        125 => "ArrowDown",
        126 => "ArrowUp",
        _ => "Unidentified",
    }
}

impl CanvasInterface for PlatformCanvas {
    fn request_redraw(&mut self) {
        let _: () = unsafe { msg_send![self.view, setNeedsDisplay:Bool::YES] };
    }

    fn request_animation_frame(&mut self) {
        // A redraw requested from inside drawRect can be absorbed by the paint
        // currently being completed. Queue it onto the next run-loop frame so
        // animation callbacks continue reliably.
        let selector = sel!(requestAnimationFrame);
        let _: () = unsafe {
            msg_send![self.view,
                performSelector:selector,
                withObject:null_mut::<Object>(),
                afterDelay:1.0f64 / 60.0
            ]
        };
    }
}

pub(crate) struct PlatformCanvasContext {
    context: CGContextRef,
    initial_transform: CGAffineTransform,
    text_attributes: HashMap<TextStyleKey, *mut Object>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct TextStyleKey {
    family: String,
    size: u32,
    weight: FontWeight,
    color: Color,
    alpha: u32,
}

impl Drop for PlatformCanvasContext {
    fn drop(&mut self) {
        for attrs in self.text_attributes.values() {
            let _: () = unsafe { msg_send![*attrs, release] };
        }
    }
}

impl PlatformCanvasContext {
    pub(crate) fn save(&mut self) {
        unsafe { CGContextSaveGState(self.context) }
    }
    pub(crate) fn restore(&mut self) {
        unsafe { CGContextRestoreGState(self.context) }
    }
    pub(crate) fn clear_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        unsafe { CGContextClearRect(self.context, rect(x, y, w, h)) }
    }
    pub(crate) fn fill_rect(&mut self, s: &CanvasState, x: f32, y: f32, w: f32, h: f32) {
        self.fill_style(s);
        unsafe { CGContextFillRect(self.context, rect(x, y, w, h)) }
    }
    pub(crate) fn stroke_rect(&mut self, s: &CanvasState, x: f32, y: f32, w: f32, h: f32) {
        self.stroke_style(s);
        unsafe { CGContextStrokeRect(self.context, rect(x, y, w, h)) }
    }
    pub(crate) fn begin_path(&mut self) {
        unsafe { CGContextBeginPath(self.context) }
    }
    pub(crate) fn close_path(&mut self) {
        unsafe { CGContextClosePath(self.context) }
    }
    pub(crate) fn move_to(&mut self, x: f32, y: f32) {
        unsafe { CGContextMoveToPoint(self.context, x.into(), y.into()) }
    }
    pub(crate) fn line_to(&mut self, x: f32, y: f32) {
        unsafe { CGContextAddLineToPoint(self.context, x.into(), y.into()) }
    }
    pub(crate) fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        unsafe { CGContextAddRect(self.context, rect(x, y, w, h)) }
    }
    pub(crate) fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32) {
        let radius = r.max(0.0).min(w.abs() * 0.5).min(h.abs() * 0.5);
        unsafe {
            let path = CGPathCreateWithRoundedRect(
                rect(x, y, w, h),
                radius.into(),
                radius.into(),
                null(),
            );
            CGContextAddPath(self.context, path);
            CGPathRelease(path);
        }
    }
    pub(crate) fn ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32) {
        unsafe {
            CGContextAddEllipseInRect(self.context, rect(x - rx, y - ry, rx * 2.0, ry * 2.0))
        }
    }
    pub(crate) fn arc(&mut self, x: f32, y: f32, r: f32, start: f32, end: f32, ccw: bool) {
        unsafe {
            CGContextAddArc(
                self.context,
                x.into(),
                y.into(),
                r.into(),
                start.into(),
                end.into(),
                !ccw,
            )
        }
    }
    pub(crate) fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        unsafe {
            CGContextAddQuadCurveToPoint(self.context, cpx.into(), cpy.into(), x.into(), y.into())
        }
    }
    pub(crate) fn bezier_curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
        unsafe {
            CGContextAddCurveToPoint(
                self.context,
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
        unsafe {
            let path = CGContextCopyPath(self.context);
            CGContextFillPath(self.context);
            if !path.is_null() {
                CGContextAddPath(self.context, path);
                CGPathRelease(path);
            }
        }
    }
    pub(crate) fn stroke(&mut self, s: &CanvasState) {
        self.stroke_style(s);
        unsafe {
            let path = CGContextCopyPath(self.context);
            CGContextStrokePath(self.context);
            if !path.is_null() {
                CGContextAddPath(self.context, path);
                CGPathRelease(path);
            }
        }
    }
    pub(crate) fn clip(&mut self) {
        unsafe { CGContextClip(self.context) }
    }
    pub(crate) fn translate(&mut self, x: f32, y: f32) {
        unsafe { CGContextTranslateCTM(self.context, x.into(), y.into()) }
    }
    pub(crate) fn rotate(&mut self, a: f32) {
        unsafe { CGContextRotateCTM(self.context, a.into()) }
    }
    pub(crate) fn scale(&mut self, x: f32, y: f32) {
        unsafe { CGContextScaleCTM(self.context, x.into(), y.into()) }
    }
    pub(crate) fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        unsafe {
            CGContextSetCTM(self.context, self.initial_transform);
            CGContextConcatCTM(
                self.context,
                CGAffineTransform {
                    a: a.into(),
                    b: b.into(),
                    c: c.into(),
                    d: d.into(),
                    tx: e.into(),
                    ty: f.into(),
                },
            )
        }
    }
    pub(crate) fn reset_transform(&mut self) {
        unsafe { CGContextSetCTM(self.context, self.initial_transform) }
    }
    pub(crate) fn fill_text(&mut self, s: &CanvasState, text: &str, x: f32, y: f32) {
        unsafe {
            let attrs = self.text_attributes(s);
            let string = NSString::from_str(text);
            let measured: NSSize = msg_send![string.0, sizeWithAttributes:attrs];
            let (x, y) =
                aligned_text_position(s, x, y, measured.width as f32, measured.height as f32);
            let _: () = msg_send![string.0, drawAtPoint:NSPoint::new(x.into(),y.into()), withAttributes:attrs];
        }
    }
    pub(crate) fn measure_text(&mut self, s: &CanvasState, text: &str) -> f32 {
        unsafe {
            let attrs = self.text_attributes(s);
            let string = NSString::from_str(text);
            let size: NSSize = msg_send![string.0,sizeWithAttributes:attrs];
            size.width as f32
        }
    }
    fn text_attributes(&mut self, s: &CanvasState) -> *mut Object {
        let key = TextStyleKey {
            family: s.font_family.clone(),
            size: s.font_size.to_bits(),
            weight: s.font_weight,
            color: s.fill,
            alpha: s.alpha.to_bits(),
        };
        if let Some(attrs) = self.text_attributes.get(&key) {
            return *attrs;
        }
        let attrs = unsafe { create_text_attributes(s) };
        self.text_attributes.insert(key, attrs);
        attrs
    }
    fn fill_style(&self, s: &CanvasState) {
        let (r, g, b, a) = s.fill.components(s.alpha);
        unsafe { CGContextSetRGBFillColor(self.context, r, g, b, a) }
    }
    fn stroke_style(&self, s: &CanvasState) {
        let (r, g, b, a) = s.stroke.components(s.alpha);
        unsafe {
            CGContextSetRGBStrokeColor(self.context, r, g, b, a);
            CGContextSetLineWidth(self.context, s.line_width.into());
            CGContextSetLineCap(
                self.context,
                match s.line_cap {
                    LineCap::Butt => 0,
                    LineCap::Round => 1,
                    LineCap::Square => 2,
                },
            );
            CGContextSetLineJoin(
                self.context,
                match s.line_join {
                    LineJoin::Miter => 0,
                    LineJoin::Round => 1,
                    LineJoin::Bevel => 2,
                },
            );
        }
    }
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> CGRect {
    CGRect::new(
        CGPoint::new(x.into(), y.into()),
        CGSize::new(w.into(), h.into()),
    )
}

unsafe fn create_text_attributes(s: &CanvasState) -> *mut Object {
    let family = NSString::from_str(&s.font_family);
    let mut font: *mut Object = match s.font_weight {
        FontWeight::Normal => unsafe {
            msg_send![class!(NSFont),fontWithName:family,size:s.font_size as f64]
        },
        FontWeight::Bold => unsafe {
            msg_send![class!(NSFont),boldSystemFontOfSize:s.font_size as f64]
        },
    };
    if font.is_null() {
        font = unsafe { msg_send![class!(NSFont),systemFontOfSize:s.font_size as f64] };
    }
    let (r, g, b, a) = s.fill.components(s.alpha);
    let color: *mut Object =
        unsafe { msg_send![class!(NSColor),colorWithRed:r,green:g,blue:b,alpha:a] };
    let attrs: *mut Object = unsafe { msg_send![class!(NSMutableDictionary), dictionary] };
    let _: () = unsafe { msg_send![attrs, setObject:font, forKey:ns_string!("NSFont")] };
    let _: () = unsafe { msg_send![attrs, setObject:color, forKey:ns_string!("NSColor")] };
    let attrs: *mut Object = unsafe { msg_send![attrs, retain] };
    attrs
}

fn aligned_text_position(s: &CanvasState, mut x: f32, mut y: f32, w: f32, h: f32) -> (f32, f32) {
    match s.text_align {
        TextAlign::Center => x -= w / 2.0,
        TextAlign::Right | TextAlign::End => x -= w,
        _ => {}
    }
    match s.text_baseline {
        TextBaseline::Middle => y -= h / 2.0,
        TextBaseline::Bottom | TextBaseline::Ideographic => y -= h,
        TextBaseline::Alphabetic => y -= h * 0.8,
        _ => {}
    }
    (x, y)
}
