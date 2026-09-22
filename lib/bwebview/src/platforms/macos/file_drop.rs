/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use objc2::runtime::{AnyClass, AnyObject as Object, Bool};
use objc2::{ClassType as _, class, define_class};

use super::headers::*;
define_class!(
    #[unsafe(super(WKWebView))]
    #[name = "BWebviewDroppableWebview"]
    struct DroppableWebview;

    impl DroppableWebview {
        #[unsafe(method(draggingEntered:))]
        #[allow(clippy::missing_const_for_fn)]
        fn _dragging_entered(&self, _: *mut Object) -> u64 {
            NS_DRAG_OPERATION_COPY
        }

        #[unsafe(method(draggingUpdated:))]
        #[allow(clippy::missing_const_for_fn)]
        fn _dragging_updated(&self, _: *mut Object) -> u64 {
            NS_DRAG_OPERATION_COPY
        }

        #[unsafe(method(prepareForDragOperation:))]
        #[allow(clippy::missing_const_for_fn)]
        fn _prepare_for_drag_operation(&self, _: *mut Object) -> Bool {
            Bool::YES
        }

        #[unsafe(method(performDragOperation:))]
        fn _perform_drag_operation(&self, sender: *mut Object) -> Bool {
            unsafe { perform_file_drop(sender) }
        }
    }
);

/// A WKWebView subclass that reports native file drags instead of navigating to them
pub(super) fn droppable_webview_class() -> &'static AnyClass {
    DroppableWebview::class()
}
