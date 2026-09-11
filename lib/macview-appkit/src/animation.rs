/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::{Cell, RefCell};
use std::ptr::null_mut;
use std::time::Duration;

use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyObject as Object, Bool};
use objc2::{class, define_class, msg_send, sel};

use crate::headers::*;
use crate::ns_string;

type TargetIvars = Cell<*mut Object>;

#[derive(Clone)]
pub(super) struct NativeFrame {
    pub(super) image: Retained<Object>,
    pub(super) delay: Duration,
}

#[derive(Clone)]
pub(super) struct Animation {
    pub(super) frames: Vec<NativeFrame>,
    pub(super) loop_count: u32,
}

#[derive(Default)]
struct Playback {
    index: usize,
    completed: u32,
    stopped: bool,
}

impl Playback {
    const fn advance(&mut self, count: usize, plays: u32) -> bool {
        if self.stopped {
            return false;
        }
        if self.index + 1 == count {
            self.completed = self.completed.saturating_add(1);
            if plays != 0 && self.completed >= plays {
                self.stopped = true;
                return false;
            }
            self.index = 0;
        } else {
            self.index += 1;
        }
        true
    }
}

struct ViewIvars {
    animation: Animation,
    playback: RefCell<Playback>,
    timer: RefCell<Option<Retained<Object>>>,
    target: Retained<TimerTarget>,
}

impl ViewIvars {
    fn cancel(&self) {
        if let Some(timer) = self.timer.borrow_mut().take() {
            // SAFETY: The view owns this NSTimer and cancellation runs on the main thread.
            unsafe {
                let _: () = msg_send![&*timer, invalidate];
            }
        }
    }
}

impl Drop for ViewIvars {
    fn drop(&mut self) {
        self.target.ivars().set(null_mut());
        self.cancel();
    }
}

// NSTimer retains this proxy, which borrows the view only until the view's ivars
// are dropped. This avoids a timer -> view -> timer retain cycle.
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "MacViewAnimationTimerTarget"]
    #[ivars = TargetIvars]
    struct TimerTarget;

    impl TimerTarget {
        #[unsafe(method(tick:))]
        fn tick(&self, _: *mut Object) {
            let view = self.ivars().get();
            if !view.is_null() {
                // SAFETY: Main-thread cancellation clears the pointer before view destruction.
                unsafe { let _: () = msg_send![view, advanceFrame]; }
            }
        }
    }
);

define_class!(
    #[unsafe(super(NSImageView))]
    #[name = "MacViewAnimationView"]
    #[ivars = ViewIvars]
    struct AnimationView;

    impl AnimationView {
        #[unsafe(method(viewDidMoveToWindow))]
        fn moved(&self) {
            // SAFETY: This is a live NSImageView on the main thread.
            unsafe { let _: () = msg_send![super(self), viewDidMoveToWindow]; }
            self.ivars().cancel();
            self.schedule();
        }

        #[unsafe(method(advanceFrame))]
        fn advance(&self) {
            self.ivars().cancel();
            let animation = &self.ivars().animation;
            let next = {
                let mut playback = self.ivars().playback.borrow_mut();
                if !playback.advance(animation.frames.len(), animation.loop_count) { return; }
                playback.index
            };
            // SAFETY: The animation owns the frame and NSImageView retains it.
            unsafe {
                let this = self as *const Self as *mut Object;
                let _: () = msg_send![this, setImage: animation.frames[next].image.as_ptr()];
            }
            self.schedule();
        }
    }
);

impl AnimationView {
    fn schedule(&self) {
        let ivars = self.ivars();
        let playback = ivars.playback.borrow();
        if playback.stopped || ivars.timer.borrow().is_some() {
            return;
        }
        let interval = ivars.animation.frames[playback.index]
            .delay
            .max(Duration::from_millis(10))
            .as_secs_f64();
        // SAFETY: This runs on the main thread. The timer owns its proxy target;
        // the view invalidates it on detachment and destruction.
        unsafe {
            let this = self as *const Self as *mut Object;
            let window: *mut Object = msg_send![this, window];
            if window.is_null() {
                return;
            }
            let timer: *mut Object = msg_send![class!(NSTimer),
                timerWithTimeInterval: interval,
                target: ivars.target.as_ptr(),
                selector: sel!(tick:),
                userInfo: null_mut::<Object>(),
                repeats: Bool::NO
            ];
            let timer = Retained::retain(timer).expect("NSTimer returned null");
            let run_loop: *mut Object = msg_send![class!(NSRunLoop), mainRunLoop];
            let _: () = msg_send![run_loop, addTimer: timer.as_ptr(), forMode: ns_string!("kCFRunLoopCommonModes")];
            ivars.timer.replace(Some(timer));
        }
    }
}

pub(super) fn create_view(frame: Rect, animation: Animation) -> Retained<Object> {
    // SAFETY: Both classes are registered and initialized with their Rust ivars.
    // The caller creates views on the main thread and animation has multiple frames.
    unsafe {
        let target: Allocated<TimerTarget> = msg_send![TimerTarget::class(), alloc];
        let target: Retained<TimerTarget> =
            msg_send![super(target.set_ivars(Cell::new(null_mut()))), init];
        let first = animation.frames[0].image.clone();
        let view: Allocated<AnimationView> = msg_send![AnimationView::class(), alloc];
        let view: Retained<AnimationView> = msg_send![super(view.set_ivars(ViewIvars {
            animation, playback: RefCell::new(Playback::default()), timer: RefCell::new(None), target: target.clone(),
        })), initWithFrame: frame];
        target.ivars().set(view.as_ptr().cast());
        let _: () = msg_send![&*view, setImage: first.as_ptr()];
        let _: () = msg_send![&*view, setImageScaling: NS_IMAGE_SCALE_PROPORTIONALLY_UP_OR_DOWN];
        Retained::into_any(view)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_playback_stops_on_last_frame() {
        let mut state = Playback::default();
        for expected in [1, 0, 1] {
            assert!(state.advance(2, 2));
            assert_eq!(state.index, expected);
        }
        assert!(!state.advance(2, 2));
        assert_eq!(state.index, 1);
        assert!(!state.advance(2, 2));
    }

    #[test]
    fn infinite_playback_keeps_wrapping() {
        let mut state = Playback::default();
        for _ in 0..1000 {
            assert!(state.advance(2, 0));
        }
        assert_eq!(state.index, 0);
        assert!(!state.stopped);
    }
}
