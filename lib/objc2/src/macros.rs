/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;

use crate::ffi::objc_msgSend;
use crate::runtime::{AnyObject, Sel};

/// Get class by name
#[macro_export]
macro_rules! class {
    ($name:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let name = concat!(stringify!($name), '\0');
            $crate::ffi::objc_getClass(name.as_ptr() as *const std::ffi::c_char)
        }
    }};
}

/// Get selector by name
#[macro_export]
macro_rules! sel {
    ($name:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let name = concat!(stringify!($name), '\0');
            $crate::ffi::sel_registerName(name.as_ptr() as *const std::ffi::c_char)
        }
    }};
    ($($name:ident :)+) => ({
        #[allow(unused_unsafe)]
        unsafe {
            let name = concat!($(stringify!($name), ':'),+, '\0');
            $crate::ffi::sel_registerName(name.as_ptr() as *const std::ffi::c_char)
        }
    });
}

/// Send message trait
pub trait MessageSend {
    /// Send message
    /// # Safety
    /// This function is unsafe because it calls objc_msgSend
    unsafe fn invoke<R>(obj: *mut AnyObject, sel: *const Sel, args: Self) -> R;
}
macro_rules! message_send_impl {
    ($($a:ident : $t:ident),*) => (
        impl<$($t),*> MessageSend for ($($t,)*) {
            #[inline(always)]
            unsafe fn invoke<R>(obj: *mut AnyObject, sel: *const Sel, ($($a,)*): Self) -> R {
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    if size_of::<R>() > 16 {
                        let mut ret = std::mem::zeroed();
                        let imp: unsafe extern "C" fn (*mut R, *mut AnyObject, *const Sel, $($t,)*) =
                            std::mem::transmute(crate::ffi::objc_msgSend_stret as *const c_void);
                        imp(&mut ret, obj, sel, $($a,)*);
                        ret
                    } else {
                        let imp: unsafe extern "C" fn (*mut AnyObject, *const Sel, $($t,)*) -> R =
                            std::mem::transmute(objc_msgSend as *const c_void);
                        imp(obj, sel, $($a,)*)
                    }
                }
                #[cfg(not(target_arch = "x86_64"))]
                unsafe {
                    let imp: unsafe extern "C" fn (*mut AnyObject, *const Sel, $($t,)*) -> R =
                        std::mem::transmute(objc_msgSend as *const c_void);
                    imp(obj, sel, $($a,)*)
                }
            }
        }
    );
}
message_send_impl!();
message_send_impl!(a: A);
message_send_impl!(a: A, b: B);
message_send_impl!(a: A, b: B, c: C);
message_send_impl!(a: A, b: B, c: C, d: D);
message_send_impl!(a: A, b: B, c: C, d: D, e: E);
message_send_impl!(a: A, b: B, c: C, d: D, e: E, f: F);
message_send_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G);
message_send_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H);

/// Send message to object
#[macro_export]
macro_rules! msg_send {
    ($receiver:expr, $sel:ident) => (
        $crate::macros::MessageSend::invoke($receiver, $crate::sel!($sel), ())
    );
    ($receiver:expr $(,$sel:ident : $arg:expr)+) => (
        $crate::macros::MessageSend::invoke($receiver, $crate::sel!($($sel:)+), ($($arg,)+))
    );
}
