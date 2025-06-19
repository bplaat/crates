/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString, c_void};
use std::ptr::null_mut;

use crate::ffi::*;
use crate::{Encode, Encoding};

/// Class type
pub type AnyClass = c_void;
/// Selector type
pub type Sel = c_void;

/// AnyObject type
#[repr(C)]
pub struct AnyObject(u8);

impl AnyObject {
    /// Get instance variable of object
    #[deprecated]
    pub unsafe fn get_ivar<T: Encode>(&self, name: &str) -> &T {
        let name = CString::new(name).expect("Failed to convert to CString");
        unsafe {
            let mut out = null_mut();
            object_getInstanceVariable(
                self as *const AnyObject as *mut AnyObject,
                name.as_ptr(),
                &mut out,
            );
            &*(out as *mut T)
        }
    }

    /// Get mut ptr to instance variable of object
    #[deprecated]
    pub unsafe fn get_mut_ivar<T: Encode>(&mut self, name: &str) -> &mut T {
        let name = CString::new(name).expect("Failed to convert to CString");
        unsafe {
            let mut out = null_mut();
            object_getInstanceVariable(
                self as *const AnyObject as *mut AnyObject,
                name.as_ptr(),
                &mut out,
            );
            &mut *(out as *mut T)
        }
    }
}

/// Bool
#[repr(transparent)]
pub struct Bool {
    value: u8,
}
impl Bool {
    /// Yes value
    pub const YES: Self = Self { value: 1 };
    /// No value
    pub const NO: Self = Self { value: 0 };
}
unsafe impl Encode for Bool {
    const ENCODING: Encoding = Encoding::Bool;
}

/// Class declaration
#[repr(C)]
pub struct ClassBuilder(*mut AnyClass);

impl ClassBuilder {
    /// Create a new class
    pub fn new(name: &CStr, superclass: *const AnyClass) -> Option<Self> {
        #![allow(clippy::not_unsafe_ptr_arg_deref)]
        let class: *mut AnyClass = unsafe { objc_allocateClassPair(superclass, name.as_ptr(), 0) };
        if class.is_null() {
            None
        } else {
            Some(Self(class))
        }
    }

    /// Add instance variable
    pub fn add_ivar<T: Encode>(&mut self, name: &CStr) -> bool {
        let types = CString::new(T::ENCODING.to_string()).expect("Can't convert to CString");
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
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn add_method<T: Encode>(&mut self, sel: *const Sel, _imp: T) -> bool {
        let types = CString::new(T::ENCODING.to_string()).expect("Can't convert to CString");
        unsafe { class_addMethod(self.0, sel, null_mut(), types.as_ptr()) }
    }

    /// Register class
    pub fn register(&mut self) -> *mut AnyClass {
        unsafe { objc_registerClassPair(self.0) }
        self.0
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::*;

    #[test]
    fn test_message_send() {
        let string: *mut AnyObject = unsafe { msg_send![class!(NSString), string] };
        let length: u32 = unsafe { msg_send![string, length] };
        assert_eq!(length, 0);
    }

    #[test]
    fn test_class_declaration() {
        extern "C" fn test_method(_self: *mut AnyObject, _cmd: *const Sel) {}

        let mut class_decl =
            ClassBuilder::new(c"TestClass", class!(NSObject)).expect("Failed to create class");
        assert!(class_decl.add_ivar::<i32>(c"test_ivar"));
        assert!(class_decl.add_method(sel!(testMethod), test_method as *const c_void));
        let class: *mut AnyClass = class_decl.register();
        assert!(!class.is_null());
    }
}
