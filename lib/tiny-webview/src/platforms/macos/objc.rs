/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! This file is a stripped down copy of /lib/objc/src/lib.rs because I want to
//! release the tiny-webview crate without releasing the quite bad objc crate.

use std::ffi::{CString, c_char, c_void};

/// Object type
pub(crate) type Object = c_void;
/// Class type
pub(crate) type Class = c_void;
/// Selector type
pub(crate) type Sel = c_void;

/// Block
#[repr(C)]
pub(crate) struct Block {
    /// Block isa
    pub isa: *const c_void,
    /// Block flags
    pub flags: i32,
    /// Block reserved
    pub reserved: i32,
    /// Block invoke
    pub invoke: *mut c_void,
    /// Block descriptor
    pub descriptor: *const c_void,
}

#[link(name = "objc", kind = "dylib")]
unsafe extern "C" {
    #![allow(missing_docs)]

    pub(crate) fn objc_getClass(name: *const c_char) -> *mut Class;
    pub(crate) fn sel_registerName(name: *const c_char) -> *mut Sel;
    pub(crate) fn objc_msgSend(receiver: *mut Object, sel: *const Sel, ...) -> *mut c_void;
    #[cfg(target_arch = "x86_64")]
    pub fn objc_msgSend_stret(ret: *mut c_void, receiver: *mut Object, sel: *const Sel, ...);

    pub(crate) fn object_getInstanceVariable(
        obj: *const Object,
        name: *const c_char,
        outValue: *mut *mut c_void,
    ) -> *const c_void;
    pub(crate) fn object_setInstanceVariable(
        obj: *mut Object,
        name: *const c_char,
        value: *const c_void,
    ) -> *const c_void;

    pub(crate) fn objc_allocateClassPair(
        superclass: *const Class,
        name: *const c_char,
        extraBytes: usize,
    ) -> *mut Class;
    pub(crate) fn class_addIvar(
        class: *mut Class,
        name: *const c_char,
        size: usize,
        alignment: u8,
        types: *const c_char,
    ) -> bool;
    pub(crate) fn class_addMethod(
        class: *mut Class,
        sel: *const Sel,
        imp: *const c_void,
        types: *const c_char,
    ) -> bool;
    pub(crate) fn objc_registerClassPair(class: *mut Class);
}

/// Get class by name
#[macro_export]
macro_rules! class {
    ($name:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let name = concat!(stringify!($name), '\0');
            $crate::platforms::macos::objc::objc_getClass(name.as_ptr() as *const std::ffi::c_char)
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
            $crate::platforms::macos::objc::sel_registerName(name.as_ptr() as *const std::ffi::c_char)
        }
    }};
    ($($name:ident :)+) => ({
        #[allow(unused_unsafe)]
        unsafe {
            let name = concat!($(stringify!($name), ':'),+, '\0');
            $crate::platforms::macos::objc::sel_registerName(name.as_ptr() as *const std::ffi::c_char)
        }
    });
}

/// Send message trait
pub(crate) trait MessageSend {
    /// Send message
    unsafe fn invoke<R>(obj: *mut Object, sel: *const Sel, args: Self) -> R;
}
macro_rules! message_send_impl {
    ($($a:ident : $t:ident),*) => (
        impl<$($t),*> MessageSend for ($($t,)*) {
            #[inline(always)]
            unsafe fn invoke<R>(obj: *mut Object, sel: *const Sel, ($($a,)*): Self) -> R {
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    if size_of::<R>() > 16 {
                        let mut ret = std::mem::zeroed();
                        let imp: unsafe extern "C" fn (*mut R, *mut Object, *const Sel, $($t,)*) =
                            std::mem::transmute(objc_msgSend_stret as *const c_void);
                        imp(&mut ret, obj, sel, $($a,)*);
                        ret
                    } else {
                        let imp: unsafe extern "C" fn (*mut Object, *const Sel, $($t,)*) -> R =
                            std::mem::transmute(objc_msgSend as *const c_void);
                        imp(obj, sel, $($a,)*)
                    }
                }
                #[cfg(not(target_arch = "x86_64"))]
                unsafe {
                    let imp: unsafe extern "C" fn (*mut Object, *const Sel, $($t,)*) -> R =
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
        $crate::platforms::macos::objc::MessageSend::invoke($receiver, $crate::sel!($sel), ())
    );
    ($receiver:expr, $($sel:ident : $arg:expr)+) => (
        $crate::platforms::macos::objc::MessageSend::invoke($receiver, $crate::sel!($($sel:)+), ($($arg,)+))
    );
}

/// Class declaration
#[repr(C)]
pub(crate) struct ClassDecl(*mut Class);

impl ClassDecl {
    /// Create a new class
    pub(crate) fn new(name: &str, superclass: *const Class) -> Option<Self> {
        #![allow(clippy::not_unsafe_ptr_arg_deref)]
        let name = CString::new(name).expect("Can't convert to CString");
        let class: *mut Class = unsafe { objc_allocateClassPair(superclass, name.as_ptr(), 0) };
        if class.is_null() {
            None
        } else {
            Some(Self(class))
        }
    }

    /// Add instance variable
    pub(crate) fn add_ivar<T>(&mut self, name: &str, types: &str) -> bool {
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
    pub(crate) fn add_method(&mut self, sel: *const Sel, imp: *const c_void, types: &str) -> bool {
        #![allow(clippy::not_unsafe_ptr_arg_deref)]
        let types = CString::new(types).expect("Can't convert to CString");
        unsafe { class_addMethod(self.0, sel, imp, types.as_ptr()) }
    }

    /// Register class
    pub(crate) fn register(&mut self) -> *mut Class {
        unsafe { objc_registerClassPair(self.0) }
        self.0
    }
}
