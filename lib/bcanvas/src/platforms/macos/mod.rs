/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::Cell;
use std::ptr::null_mut;
use std::rc::Rc;

use bwindow::ffi::*;
use bwindow::{NativeWindowHandle, WindowAttachment, WindowEventSender};
use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};

use self::cocoa::CGContextRef;
pub(crate) use self::context::PlatformCanvasContext;
use crate::CanvasRenderingContext2d;

mod cocoa;
mod context;

#[derive(Clone, Copy)]
struct Frame {
    context: CGContextRef,
    width: f32,
    height: f32,
    scale: f32,
}

struct ViewState {
    cursor: Cell<crate::CursorIcon>,
    sender: WindowEventSender,
    frame: Cell<Option<Frame>>,
    closed: Cell<bool>,
    painting: Cell<bool>,
    scheduled: Cell<bool>,
}

type ViewIvars = Rc<ViewState>;

define_class!(
    #[unsafe(super(NSView))]
    #[ivars = ViewIvars]
    struct CanvasView;

    impl CanvasView {
        #[unsafe(method(resetCursorRects))]
        fn reset_cursor_rects(&self) {
            let cursor: *mut Object = unsafe {
                match self.ivars().cursor.get() {
                    crate::CursorIcon::Pointer => msg_send![class!(NSCursor), pointingHandCursor],
                    crate::CursorIcon::Default | crate::CursorIcon::Progress => msg_send![class!(NSCursor), arrowCursor],
                }
            };
            let bounds: NSRect = unsafe { msg_send![self, bounds] };
            let _: () = unsafe { msg_send![self, addCursorRect:bounds, cursor:cursor] };
        }
        #[unsafe(method(mouseDown:))]
        fn _mouse_down(&self, event: *mut Object) {
            self.ivars().sender.send(unsafe { macos_pointer_event(self as *const Self as *mut Object, event, Some(bwindow::ButtonState::Pressed)) });
        }
        #[unsafe(method(mouseUp:))]
        fn _mouse_up(&self, event: *mut Object) {
            self.ivars().sender.send(unsafe { macos_pointer_event(self as *const Self as *mut Object, event, Some(bwindow::ButtonState::Released)) });
        }
        #[unsafe(method(rightMouseDown:))]
        fn _right_mouse_down(&self, event: *mut Object) {
            self.ivars().sender.send(unsafe { macos_pointer_event(self as *const Self as *mut Object, event, Some(bwindow::ButtonState::Pressed)) });
        }
        #[unsafe(method(rightMouseUp:))]
        fn _right_mouse_up(&self, event: *mut Object) {
            self.ivars().sender.send(unsafe { macos_pointer_event(self as *const Self as *mut Object, event, Some(bwindow::ButtonState::Released)) });
        }
        #[unsafe(method(otherMouseDown:))]
        fn _other_mouse_down(&self, event: *mut Object) {
            self.ivars().sender.send(unsafe { macos_pointer_event(self as *const Self as *mut Object, event, Some(bwindow::ButtonState::Pressed)) });
        }
        #[unsafe(method(otherMouseUp:))]
        fn _other_mouse_up(&self, event: *mut Object) {
            self.ivars().sender.send(unsafe { macos_pointer_event(self as *const Self as *mut Object, event, Some(bwindow::ButtonState::Released)) });
        }
        #[unsafe(method(mouseMoved:))]
        fn _mouse_moved(&self, event: *mut Object) {
            self.ivars().sender.send(unsafe { macos_pointer_event(self as *const Self as *mut Object, event, None) });
        }
        #[unsafe(method(mouseDragged:))]
        fn _mouse_dragged(&self, event: *mut Object) {
            self.ivars().sender.send(unsafe { macos_pointer_event(self as *const Self as *mut Object, event, None) });
        }
        #[unsafe(method(rightMouseDragged:))]
        fn _right_mouse_dragged(&self, event: *mut Object) {
            self.ivars().sender.send(unsafe { macos_pointer_event(self as *const Self as *mut Object, event, None) });
        }
        #[unsafe(method(otherMouseDragged:))]
        fn _other_mouse_dragged(&self, event: *mut Object) {
            self.ivars().sender.send(unsafe { macos_pointer_event(self as *const Self as *mut Object, event, None) });
        }
        #[unsafe(method(mouseExited:))]
        fn _mouse_exited(&self, _: *mut Object) {
            self.ivars().sender.send(bwindow::WindowEvent::MouseLeave);
        }

        #[unsafe(method(scrollWheel:))]
        fn _scroll_wheel(&self, event: *mut Object) {
            self.ivars().sender.send(unsafe { macos_scroll_event(event) });
        }

        #[unsafe(method(keyDown:))]
        fn _key_down(&self, event: *mut Object) {
            self.ivars().sender.send(bwindow::WindowEvent::KeyDown(unsafe {
                macos_key_event(event, bwindow::ButtonState::Pressed)
            }));
        }

        #[unsafe(method(keyUp:))]
        fn _key_up(&self, event: *mut Object) {
            self.ivars().sender.send(bwindow::WindowEvent::KeyUp(unsafe {
                macos_key_event(event, bwindow::ButtonState::Released)
            }));
        }

        #[cfg(feature = "file_drop")]
        #[unsafe(method(draggingEntered:))]
        const fn _dragging_entered(&self, _: *mut Object) -> u64 { NS_DRAG_OPERATION_COPY }

        #[cfg(feature = "file_drop")]
        #[unsafe(method(prepareForDragOperation:))]
        const fn _prepare_for_drag_operation(&self, _: *mut Object) -> Bool { Bool::YES }

        #[cfg(feature = "file_drop")]
        #[unsafe(method(performDragOperation:))]
        fn _perform_drag_operation(&self, sender: *mut Object) -> Bool {
            unsafe { perform_file_drop(sender) }
        }

        #[unsafe(method(isFlipped))]
        const fn _is_flipped(&self) -> Bool { Bool::YES }

        #[unsafe(method(acceptsFirstResponder))]
        const fn _accepts_first_responder(&self) -> Bool { Bool::YES }

        #[unsafe(method(drawRect:))]
        fn _draw_rect(&self, _: NSRect) { self.paint(); }

        #[unsafe(method(schedulePaint))]
        fn _schedule_paint(&self) {
            self.ivars().scheduled.set(false);
            if !self.ivars().closed.get() {
                let _: () = unsafe { msg_send![self, setNeedsDisplay:Bool::YES] };
            }
        }
    }
);

impl CanvasView {
    fn request_redraw(&self) {
        let state = self.ivars();
        if state.closed.get() {
            return;
        }
        if !state.painting.get() {
            unsafe {
                if state.scheduled.replace(false) {
                    let _: () =
                        msg_send![class!(NSObject), cancelPreviousPerformRequestsWithTarget:self];
                }
                let _: () = msg_send![self, setNeedsDisplay:Bool::YES];
            }
            return;
        }
        if state.scheduled.replace(true) {
            return;
        }
        // A display invalidation inside drawRect can be absorbed by that paint.
        // Queue and coalesce a later invalidation instead.
        let _: () = unsafe {
            msg_send![self, performSelector:sel!(schedulePaint),
                withObject:null_mut::<Object>(), afterDelay:1.0f64 / 60.0]
        };
    }

    fn close(&self) {
        if self.ivars().closed.replace(true) {
            return;
        }
        self.ivars().frame.set(None);
        unsafe {
            let _: () = msg_send![class!(NSObject), cancelPreviousPerformRequestsWithTarget:self];
            let _: () = msg_send![self, removeFromSuperview];
        }
    }

    fn paint(&self) {
        if self.ivars().closed.get() {
            return;
        }
        // Application code can drop its canvas or close the parent during paint.
        let retained: Retained<Self> = unsafe { msg_send![self, retain] };
        let state = retained.ivars();
        if state.painting.get() {
            self.request_redraw();
            return;
        }
        unsafe {
            let graphics: *mut Object = msg_send![class!(NSGraphicsContext), currentContext];
            let context: CGContextRef = msg_send![graphics, CGContext];
            if context.is_null() {
                return;
            }
            let bounds: NSRect = msg_send![self, bounds];
            let window: *mut Object = msg_send![self, window];
            let scale: f64 = msg_send![window, backingScaleFactor];
            state.painting.set(true);
            state.frame.set(Some(Frame {
                context,
                width: bounds.size.width as f32,
                height: bounds.size.height as f32,
                scale: scale as f32,
            }));
            if !state.sender.redraw(|| state.frame.set(None)) {
                state.frame.set(None);
                self.request_redraw();
            }
            state.painting.set(false);
        }
    }
}

pub(crate) struct PlatformCanvas {
    view: Retained<CanvasView>,
    attachment: WindowAttachment,
}

impl PlatformCanvas {
    pub(crate) fn new(attachment: WindowAttachment) -> Self {
        let Some(NativeWindowHandle::AppKit(window)) = (unsafe { attachment.native_handle() })
        else {
            panic!("expected an open AppKit window");
        };
        unsafe {
            let window = window.cast::<Object>();
            let content: *mut Object = msg_send![window, contentView];
            let bounds: NSRect = msg_send![content, bounds];
            let view: Allocated<CanvasView> = msg_send![CanvasView::class(), alloc];
            let view: Retained<CanvasView> = msg_send![
                super(view.set_ivars(Rc::new(ViewState {
                    cursor: Cell::new(crate::CursorIcon::Default),
                    sender: attachment.event_sender(),
                    frame: Cell::new(None), closed: Cell::new(false),
                    painting: Cell::new(false),
                    scheduled: Cell::new(false),
                }))),
                initWithFrame:bounds
            ];
            let _: () = msg_send![&*view, setAutoresizingMask:NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE];
            let _: () = msg_send![content, addSubview:&*view];
            let tracking: Allocated<Object> = msg_send![class!(NSTrackingArea), alloc];
            let tracking: Retained<Object> = msg_send![tracking,
                initWithRect:bounds, options:((1u64 << 0) | (1 << 1) | (1 << 5) | (1 << 9) | (1 << 10)),
                owner:&*view, userInfo:null_mut::<Object>()];
            let _: () = msg_send![&*view, addTrackingArea:&*tracking];
            #[cfg(feature = "file_drop")]
            if attachment.allow_file_drop() {
                register_dragged_types(&*view as *const CanvasView as *mut Object);
            }
            let _: Bool = msg_send![window, makeFirstResponder:&*view];
            let _: () = msg_send![window, setAcceptsMouseMovedEvents:Bool::YES];
            let redraw = view.clone();
            attachment.on_redraw(move || redraw.request_redraw());
            let close = view.clone();
            attachment.on_close(move || close.close());
            view.request_redraw();
            Self { view, attachment }
        }
    }

    pub(crate) fn request_redraw(&self) {
        self.view.request_redraw();
    }

    pub(crate) fn set_cursor(&mut self, cursor: crate::CursorIcon) {
        if self.attachment.is_closed() || self.view.ivars().cursor.replace(cursor) == cursor {
            return;
        }
        unsafe {
            let window: *mut Object = msg_send![&*self.view, window];
            let _: () = msg_send![window, invalidateCursorRectsForView:&*self.view];
        }
    }

    pub(crate) fn draw(
        &mut self,
        draw: impl for<'frame> FnOnce(&mut CanvasRenderingContext2d<'frame>),
    ) -> bool {
        if self.attachment.is_closed() {
            return false;
        }
        let Some(frame) = self.view.ivars().frame.take() else {
            return false;
        };
        let mut context = CanvasRenderingContext2d::new(
            unsafe { PlatformCanvasContext::new(frame.context) },
            frame.width,
            frame.height,
            frame.scale,
        );
        draw(&mut context);
        true
    }
}

impl Drop for PlatformCanvas {
    fn drop(&mut self) {
        self.attachment.disconnect();
        self.view.close();
    }
}
