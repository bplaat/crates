/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A basic Objective-C ffi library

use std::ffi::{c_char, c_void, CString};

/// Class type
pub type Class = *const c_void;
/// Selector type
pub type Sel = *const c_void;
/// Object type
pub type Object = *const c_void;
/// Super class
#[repr(C)]
pub struct Super {
    /// Receiver
    pub receiver: Object,
    /// Super class
    pub superclass: Class,
}

#[link(name = "objc", kind = "dylib")]
extern "C" {
    #![allow(missing_docs)]

    pub fn objc_getClass(name: *const c_char) -> Class;
    pub fn sel_registerName(name: *const c_char) -> Sel;
    pub fn objc_msgSend(receiver: Object, sel: Sel, ...) -> *const c_void;
    #[cfg(target_arch = "x86_64")]
    pub fn objc_msgSend_stret(ret: *mut c_void, receiver: Object, sel: Sel, ...);
    pub fn objc_msgSendSuper(receiver: &Super, sel: Sel, ...) -> *const c_void;

    pub fn object_getInstanceVariable(
        obj: Object,
        name: *const c_char,
        outValue: *mut *const c_void,
    ) -> *const c_void;
    pub fn object_setInstanceVariable(
        obj: Object,
        name: *const c_char,
        value: *const c_void,
    ) -> *const c_void;

    pub fn objc_allocateClassPair(
        superclass: Class,
        name: *const c_char,
        extraBytes: usize,
    ) -> Class;
    pub fn class_addIvar(
        class: Class,
        name: *const c_char,
        size: usize,
        alignment: u8,
        types: *const c_char,
    ) -> bool;
    pub fn class_addMethod(
        class: Class,
        sel: Sel,
        imp: *const c_void,
        types: *const c_char,
    ) -> bool;
    pub fn objc_registerClassPair(class: Class);
}

/// Get class by name
#[macro_export]
macro_rules! class {
    ($name:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let name = concat!(stringify!($name), '\0');
            $crate::objc_getClass(name.as_ptr() as *const std::ffi::c_char)
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
            $crate::sel_registerName(name.as_ptr() as *const std::ffi::c_char)
        }
    }};
    ($($name:ident :)+) => ({
        #[allow(unused_unsafe)]
        unsafe {
            let name = concat!($(stringify!($name), ':'),+, '\0');
            $crate::sel_registerName(name.as_ptr() as *const std::ffi::c_char)
        }
    });
}

/// Send message trait
pub trait MessageSend {
    /// Send message
    /// # Safety
    /// This function is unsafe because it calls objc_msgSend
    unsafe fn invoke<R>(obj: Object, sel: Sel, args: Self) -> R;
}
macro_rules! message_send_impl {
    ($($a:ident : $t:ident),*) => (
        impl<$($t),*> MessageSend for ($($t,)*) {
            #[inline(always)]
            unsafe fn invoke<R>(obj: Object, sel: Sel, ($($a,)*): Self) -> R {
                #[cfg(target_arch = "x86_64")]
                {
                    if size_of::<R>() > 16 {
                        let mut ret = std::mem::zeroed();
                        let imp: unsafe extern fn (*mut R, Object, Sel, $($t,)*) =
                            std::mem::transmute(objc_msgSend_stret as *const c_void);
                        imp(&mut ret, obj, sel, $($a,)*);
                        ret
                    } else {
                        let imp: unsafe extern fn (Object, Sel, $($t,)*) -> R =
                            std::mem::transmute(objc_msgSend as *const c_void);
                        imp(obj, sel, $($a,)*)
                    }
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    let imp: unsafe extern fn (Object, Sel, $($t,)*) -> R =
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
        $crate::MessageSend::invoke($receiver, $crate::sel!($sel), ())
    );
    ($receiver:expr, $($sel:ident : $arg:expr)+) => (
        $crate::MessageSend::invoke($receiver, $crate::sel!($($sel:)+), ($($arg,)+))
    );
}

/// Class declaration
#[repr(C)]
pub struct ClassDecl(Class);

impl ClassDecl {
    /// Create a new class
    pub fn new(name: &str, superclass: Class) -> Option<Self> {
        #![allow(clippy::not_unsafe_ptr_arg_deref)]
        let name = CString::new(name).expect("Can't convert to CString");
        let class: Class = unsafe { objc_allocateClassPair(superclass, name.as_ptr(), 0) };
        if class.is_null() {
            None
        } else {
            Some(Self(class))
        }
    }

    /// Add instance variable
    pub fn add_ivar<T>(&mut self, name: &str, types: &str) -> bool {
        let name = CString::new(name).expect("Can't convert to CString");
        let types = CString::new(types).expect("Can't convert to CString");
        unsafe {
            class_addIvar(
                self.0,
                name.as_ptr(),
                size_of::<T>(),
                align_of::<T>().trailing_zeros() as u8,
                types.as_ptr(),
            )
        }
    }

    /// Add method
    pub fn add_method(&mut self, sel: Sel, imp: *const c_void, types: &str) -> bool {
        #![allow(clippy::not_unsafe_ptr_arg_deref)]
        let types = CString::new(types).expect("Can't convert to CString");
        unsafe { class_addMethod(self.0, sel, imp, types.as_ptr()) }
    }

    /// Register class
    pub fn register(&mut self) -> Class {
        unsafe { objc_registerClassPair(self.0) }
        self.0
    }
}
