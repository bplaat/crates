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
        // SAFETY: `name` is a compile-time null-terminated string literal. The returned
        // pointer is either a valid class pointer or null (caller is responsible for checking).
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
        // SAFETY: `name` is a compile-time null-terminated string literal.
        // `sel_registerName` always returns a valid, globally registered selector pointer.
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
        // SAFETY: `name` is a compile-time null-terminated string literal.
        // `sel_registerName` always returns a valid, globally registered selector pointer.
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
                // SAFETY: `objc_msgSend`/`objc_msgSend_stret` are C variadics that accept any
                // argument list. The transmute gives them the concrete Rust types we verified via
                // `verify_send` (debug) or statically via `Encode` bounds (release). The call is
                // sound because the caller (`msg_send!`) must ensure `obj` is a valid ObjC object
                // and `sel` is a registered selector, as documented on `MessageSend::invoke`.
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
                // SAFETY: see the x86_64 branch above for the full justification.
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
        // SAFETY: NSString is a valid Foundation class; alloc returns a valid uninitialized object.
        let ns: *mut AnyObject = unsafe { msg_send![class!(NSString), alloc] };
        assert!(!ns.is_null());
        // SAFETY: `ns` is a valid uninitialized NSString from alloc; init always succeeds.
        let ns: *mut AnyObject = unsafe { msg_send![ns, init] };
        assert!(!ns.is_null());
        // SAFETY: `ns` is a fully initialized NSString; length is a valid method returning NSUInteger.
        let length: u64 = unsafe { msg_send![ns, length] };
        assert_eq!(length, 0);
        // SAFETY: `ns` is a valid ObjC object; release decrements the retain count.
        unsafe {
            let _: () = msg_send![ns, release];
        }
    }

    #[test]
    fn test_message_send_alloc_init() {
        // SAFETY: NSObject is a valid Foundation class; alloc/init/release are standard ObjC methods.
        unsafe {
            let obj: *mut AnyObject = msg_send![class!(NSObject), alloc];
            assert!(!obj.is_null());
            let obj: *mut AnyObject = msg_send![obj, init];
            assert!(!obj.is_null());
            let _: () = msg_send![obj, release];
        }
    }

    #[test]
    fn test_message_send_two_args() {
        // SAFETY: all classes are valid Foundation types; all selectors match their declared signatures.
        unsafe {
            let dict: *mut AnyObject = msg_send![class!(NSMutableDictionary), new];
            assert!(!dict.is_null());
            let key: *mut AnyObject = msg_send![class!(NSString), new];
            let val: *mut AnyObject = msg_send![class!(NSObject), new];
            let _: () = msg_send![dict, setObject: val, forKey: key];
            let count: u64 = msg_send![dict, count];
            assert_eq!(count, 1);
            let _: () = msg_send![key, release];
            let _: () = msg_send![val, release];
            let _: () = msg_send![dict, release];
        }
    }

    #[test]
    fn test_message_send_three_args() {
        // SAFETY: all classes are valid Foundation types; all selectors match their declared signatures.
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

    #[repr(C)]
    struct NSRange {
        location: u64,
        length: u64,
    }
    // SAFETY: NSRange is typedef'd from `struct _NSRange { u64 location; u64 length; }`;
    // the ObjC runtime uses the underlying struct name `_NSRange` in type encodings.
    unsafe impl crate::Encode for NSRange {
        const ENCODING: crate::Encoding = crate::Encoding::Struct(
            "_NSRange",
            &[crate::Encoding::ULongLong, crate::Encoding::ULongLong],
        );
    }

    unsafe fn make_nsstring(bytes: &[u8]) -> *mut AnyObject {
        // SAFETY: NSString is a valid Foundation class; initWithBytes:length:encoding: is a standard initializer.
        unsafe {
            let ns: *mut AnyObject = msg_send![class!(NSString), alloc];
            msg_send![ns,
                initWithBytes: bytes.as_ptr() as *const c_void,
                length: bytes.len() as u64,
                encoding: 4u64
            ]
        }
    }

    #[test]
    fn test_message_send_four_args() {
        // SAFETY: all classes are valid Foundation types; all selectors match their declared signatures.
        unsafe {
            let src = make_nsstring(b"hello world");
            let from = make_nsstring(b"world");
            let to = make_nsstring(b"rust");
            let result: *mut AnyObject = msg_send![src,
                stringByReplacingOccurrencesOfString: from,
                withString: to,
                options: 0u64,
                range: NSRange { location: 0, length: 11 }
            ];
            assert!(!result.is_null());
            let len: u64 = msg_send![result, length];
            assert_eq!(len, 10); // "hello rust" = 10 chars
            let _: () = msg_send![src, release];
            let _: () = msg_send![from, release];
            let _: () = msg_send![to, release];
        }
    }

    #[test]
    fn test_sel_macro_single_name() {
        // SAFETY: `sel!(length)` is a registered selector; sel_getName returns a valid null-terminated C string.
        let name = unsafe { CStr::from_ptr(sel_getName(sel!(length).0)) };
        assert_eq!(name.to_bytes(), b"length");
    }

    #[test]
    fn test_sel_macro_multi_name() {
        // SAFETY: `sel!(setObject: forKey:)` is a registered selector; sel_getName returns a valid null-terminated C string.
        let name = unsafe { CStr::from_ptr(sel_getName(sel!(setObject: forKey:).0)) };
        assert_eq!(name.to_bytes(), b"setObject:forKey:");
    }
}
