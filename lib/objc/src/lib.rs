/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! An Objective-C ffi library

#![cfg(target_os = "macos")]

use std::ffi::{CString, c_char, c_void};

/// Object type
pub type Object = c_void;
/// Class type
pub type Class = c_void;
/// Selector type
pub type Sel = c_void;

/// Super class
// FIXME: Make a msg_send_super macro instead of exposing this struct
#[repr(C)]
pub struct Super {
    /// Receiver
    pub receiver: *mut Object,
    /// Super class
    pub superclass: *const Class,
}

/// Block
#[repr(C)]
pub struct Block {
    /// Block isa
    pub isa: *const c_void,
    /// Block flags
    pub flags: i32,
    /// Block reserved
    pub reserved: i32,
    /// Block invoke
    pub invoke: extern "C" fn(),
    /// Block descriptor
    pub descriptor: *const c_void,
}

#[link(name = "objc", kind = "dylib")]
unsafe extern "C" {
    #![allow(missing_docs)]

    pub fn objc_getClass(name: *const c_char) -> *mut Class;
    pub fn sel_registerName(name: *const c_char) -> *mut Sel;
    pub fn objc_msgSend(receiver: *mut Object, sel: *const Sel, ...) -> *mut c_void;
    #[cfg(target_arch = "x86_64")]
    pub fn objc_msgSend_stret(ret: *mut c_void, receiver: *mut Object, sel: *const Sel, ...);
    pub fn objc_msgSendSuper(receiver: &Super, sel: *const Sel, ...) -> *mut c_void;

    pub fn object_getInstanceVariable(
        obj: *const Object,
        name: *const c_char,
        outValue: *mut *mut c_void,
    ) -> *const c_void;
    pub fn object_setInstanceVariable(
        obj: *mut Object,
        name: *const c_char,
        value: *const c_void,
    ) -> *const c_void;

    pub fn objc_allocateClassPair(
        superclass: *const Class,
        name: *const c_char,
        extraBytes: usize,
    ) -> *mut Class;
    pub fn class_addIvar(
        class: *mut Class,
        name: *const c_char,
        size: usize,
        alignment: u8,
        types: *const c_char,
    ) -> bool;
    pub fn class_addMethod(
        class: *mut Class,
        sel: *const Sel,
        imp: *const c_void,
        types: *const c_char,
    ) -> bool;
    pub fn objc_registerClassPair(class: *mut Class);
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
        $crate::MessageSend::invoke($receiver, $crate::sel!($sel), ())
    );
    ($receiver:expr, $($sel:ident : $arg:expr)+) => (
        $crate::MessageSend::invoke($receiver, $crate::sel!($($sel:)+), ($($arg,)+))
    );
}

/// Class declaration
#[repr(C)]
pub struct ClassDecl(*mut Class);

impl ClassDecl {
    /// Create a new class
    pub fn new(name: &str, superclass: *const Class) -> Option<Self> {
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
    pub fn add_method(&mut self, sel: *const Sel, imp: *const c_void, types: &str) -> bool {
        #![allow(clippy::not_unsafe_ptr_arg_deref)]
        let types = CString::new(types).expect("Can't convert to CString");
        unsafe { class_addMethod(self.0, sel, imp, types.as_ptr()) }
    }

    /// Register class
    pub fn register(&mut self) -> *mut Class {
        unsafe { objc_registerClassPair(self.0) }
        self.0
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_message_send() {
        let string: *mut Object = unsafe { msg_send![class!(NSString), string] };
        let length: u32 = unsafe { msg_send![string, length] };
        assert_eq!(length, 0);
    }

    #[test]
    fn test_class_declaration() {
        extern "C" fn test_method(_self: *mut Object, _cmd: *const Sel) {}

        let mut class_decl =
            ClassDecl::new("TestClass", class!(NSObject)).expect("Failed to create class");
        assert!(class_decl.add_ivar::<i32>("test_ivar", "i32"));
        assert!(class_decl.add_method(sel!(testMethod), test_method as *const c_void, "v@:"));
        let class: *mut Class = class_decl.register();
        assert!(!class.is_null());
    }
}
