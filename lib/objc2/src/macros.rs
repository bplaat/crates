/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
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
                as *mut $crate::runtime::AnyObject
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
            $crate::runtime::Sel(
                $crate::ffi::sel_registerName(name.as_ptr() as *const std::ffi::c_char)
                    as *const std::ffi::c_void,
            )
        }
    }};
    ($($name:ident :)+) => ({
        #[allow(unused_unsafe)]
        unsafe {
            let name = concat!($(stringify!($name), ':'),+, '\0');
            $crate::runtime::Sel(
                $crate::ffi::sel_registerName(name.as_ptr() as *const std::ffi::c_char)
                    as *const std::ffi::c_void,
            )
        }
    });
}

/// Internal trait used by `msg_send!`. Not part of the public API.
#[doc(hidden)]
pub trait MessageSend {
    /// # Safety
    /// Caller must ensure `obj` is a valid ObjC object and `sel` is a valid selector.
    unsafe fn invoke<R: crate::Encode>(obj: *mut AnyObject, sel: Sel, args: Self) -> R;
}
macro_rules! message_send_impl {
    ($($a:ident : $t:ident),*) => (
        impl<$($t: crate::Encode),*> MessageSend for ($($t,)*) {
            #[inline(always)]
            unsafe fn invoke<R: crate::Encode>(obj: *mut AnyObject, sel: Sel, ($($a,)*): Self) -> R {
                #[cfg(debug_assertions)]
                crate::verify::verify_send(obj, sel, &[$($t::ENCODING),*], &R::ENCODING);
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    if const { size_of::<R>() > 16 } {
                        let mut ret = std::mem::zeroed();
                        let imp: unsafe extern "C" fn (*mut R, *mut AnyObject, *const c_void, $($t,)*) =
                            std::mem::transmute(crate::ffi::objc_msgSend_stret as *const c_void);
                        imp(&mut ret, obj, sel.0, $($a,)*);
                        ret
                    } else {
                        let imp: unsafe extern "C" fn (*mut AnyObject, *const c_void, $($t,)*) -> R =
                            std::mem::transmute(objc_msgSend as *const c_void);
                        imp(obj, sel.0, $($a,)*)
                    }
                }
                #[cfg(not(target_arch = "x86_64"))]
                unsafe {
                    let imp: unsafe extern "C" fn (*mut AnyObject, *const c_void, $($t,)*) -> R =
                        std::mem::transmute(objc_msgSend as *const c_void);
                    imp(obj, sel.0, $($a,)*)
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

// MARK: Tests
#[cfg(test)]
mod test {
    use std::ffi::{CStr, c_void};

    use crate::ffi::sel_getName;
    use crate::runtime::AnyObject;

    #[link(name = "Foundation", kind = "framework")]
    unsafe extern "C" {}

    #[test]
    fn test_message_send_no_args() {
        let ns: *mut AnyObject = unsafe { msg_send![class!(NSString), alloc] };
        assert!(!ns.is_null());
        let ns: *mut AnyObject = unsafe { msg_send![ns, init] };
        assert!(!ns.is_null());
        let length: u64 = unsafe { msg_send![ns, length] };
        assert_eq!(length, 0);
        unsafe {
            let _: () = msg_send![ns, release];
        }
    }

    #[test]
    fn test_message_send_alloc_init() {
        unsafe {
            let obj: *mut AnyObject = msg_send![class!(NSObject), alloc];
            assert!(!obj.is_null());
            let obj: *mut AnyObject = msg_send![obj, init];
            assert!(!obj.is_null());
            let _: () = msg_send![obj, release];
        }
    }

    #[test]
    fn test_message_send_three_args() {
        unsafe {
            let hello = b"hello";
            let ns: *mut AnyObject = msg_send![class!(NSString), alloc];
            let ns: *mut AnyObject = msg_send![ns,
                initWithBytes: hello.as_ptr() as *const c_void,
                length: 5u64,
                encoding: 4u64
            ];
            assert!(!ns.is_null());
            let len: u64 = msg_send![ns, length];
            assert_eq!(len, 5);
            let _: () = msg_send![ns, release];
        }
    }

    #[test]
    fn test_sel_macro_single_name() {
        let name = unsafe { CStr::from_ptr(sel_getName(sel!(length).0)) };
        assert_eq!(name.to_bytes(), b"length");
    }

    #[test]
    fn test_sel_macro_multi_name() {
        let name = unsafe { CStr::from_ptr(sel_getName(sel!(setObject: forKey:).0)) };
        assert_eq!(name.to_bytes(), b"setObject:forKey:");
    }
}
