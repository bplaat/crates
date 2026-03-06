/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CStr, CString, c_void};

use crate::encode::{Encode, Encoding};
use crate::ffi::*;

/// Class type (opaque).
#[repr(C)]
pub struct AnyClass(u8);

impl AnyClass {
    /// Get a class by name, returning `None` if not found.
    pub fn get(name: &CStr) -> Option<&'static Self> {
        let cls = unsafe { objc_getClass(name.as_ptr()) };
        if cls.is_null() {
            None
        } else {
            Some(unsafe { &*(cls as *const Self) })
        }
    }
}

/// An Objective-C selector (pointer-sized, equivalent to C's `SEL`).
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Sel(pub *const c_void);

// Selectors are globally registered by `sel_registerName` and never freed; they are safe
// to share across threads.
unsafe impl Send for Sel {}
unsafe impl Sync for Sel {}

unsafe impl Encode for Sel {
    const ENCODING: Encoding = Encoding::Sel;
}

/// AnyObject type (opaque).
#[repr(C)]
pub struct AnyObject(u8);

unsafe impl Encode for *const AnyObject {
    const ENCODING: Encoding = Encoding::Object;
}
unsafe impl Encode for *mut AnyObject {
    const ENCODING: Encoding = Encoding::Object;
}

impl AnyObject {
    /// Get a reference to an instance variable of this object.
    #[allow(clippy::missing_safety_doc)]
    #[deprecated]
    pub unsafe fn get_ivar<T: Encode>(&self, name: &str) -> &T {
        let name = CString::new(name).expect("Failed to convert to CString");
        unsafe {
            let cls = object_getClass(self as *const AnyObject);
            let ivar = class_getInstanceVariable(cls, name.as_ptr());
            assert!(
                !ivar.is_null(),
                "ivar '{}' not found",
                name.to_string_lossy()
            );
            let offset = ivar_getOffset(ivar) as usize;
            &*((self as *const AnyObject as *const u8).add(offset) as *const T)
        }
    }

    /// Get a mutable reference to an instance variable of this object.
    #[allow(clippy::missing_safety_doc)]
    #[deprecated]
    pub unsafe fn get_mut_ivar<T: Encode>(&mut self, name: &str) -> &mut T {
        let name = CString::new(name).expect("Failed to convert to CString");
        unsafe {
            let cls = object_getClass(self as *const AnyObject);
            let ivar = class_getInstanceVariable(cls, name.as_ptr());
            assert!(
                !ivar.is_null(),
                "ivar '{}' not found",
                name.to_string_lossy()
            );
            let offset = ivar_getOffset(ivar) as usize;
            &mut *((self as *mut AnyObject as *mut u8).add(offset) as *mut T)
        }
    }
}

/// Objective-C boolean type.
#[repr(transparent)]
pub struct Bool {
    value: u8,
}
impl Bool {
    /// `YES`
    pub const YES: Self = Self { value: 1 };
    /// `NO`
    pub const NO: Self = Self { value: 0 };
}
unsafe impl Encode for Bool {
    const ENCODING: Encoding = Encoding::Bool;
}

/// Trait for `extern "C"` function pointers usable as ObjC method implementations.
///
/// Automatically derives the ObjC type encoding from the Rust function signature.
/// Implemented for `extern "C" fn(*mut AnyObject, Sel, ...) -> R` at all supported arities.
#[allow(clippy::missing_safety_doc)]
pub unsafe trait MethodImpl: Copy {
    /// Returns the function pointer cast to `*const c_void`.
    fn imp_ptr(self) -> *const c_void;
    /// Builds the ObjC type encoding string for this function's full signature.
    fn type_encoding() -> CString;
}

macro_rules! impl_method_impl {
    ($($t:ident),*) => {
        unsafe impl<Ret: Encode, $($t: Encode,)*> MethodImpl
            for extern "C" fn(*mut AnyObject, Sel $(, $t)*) -> Ret
        {
            fn imp_ptr(self) -> *const c_void {
                self as *const c_void
            }
            fn type_encoding() -> CString {
                let mut enc = Ret::ENCODING.to_string();
                enc.push('@');
                enc.push(':');
                $(enc.push_str(&$t::ENCODING.to_string());)*
                CString::new(enc).expect("ObjC type encoding contains no null bytes")
            }
        }
    };
}
impl_method_impl!();
impl_method_impl!(A);
impl_method_impl!(A, B);
impl_method_impl!(A, B, C);
impl_method_impl!(A, B, C, D);
impl_method_impl!(A, B, C, D, E);
impl_method_impl!(A, B, C, D, E, F);
impl_method_impl!(A, B, C, D, E, F, G);
impl_method_impl!(A, B, C, D, E, F, G, H);

/// Class declaration builder.
pub struct ClassBuilder(*mut c_void);

impl ClassBuilder {
    /// Create a new class with the given name and superclass.
    /// Note: unlike the real `objc2`, `superclass` here is `*mut AnyObject` (as returned by `class!`).
    pub fn new(name: &CStr, superclass: *mut AnyObject) -> Option<Self> {
        let class =
            unsafe { objc_allocateClassPair(superclass as *const c_void, name.as_ptr(), 0) };
        if class.is_null() {
            None
        } else {
            Some(Self(class))
        }
    }

    /// Add an instance variable of type `T`.
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

    /// Add a method to the class.
    ///
    /// The ObjC type encoding is derived automatically from `T`'s function pointer type.
    pub fn add_method<T: MethodImpl>(&mut self, sel: Sel, imp: T) -> bool {
        let encoding = T::type_encoding();
        let imp_ptr = imp.imp_ptr();
        unsafe { class_addMethod(self.0, sel.0, imp_ptr, encoding.as_ptr()) }
    }

    /// Register the class and return it as a `*mut AnyObject`.
    ///
    /// Consumes the builder since ivars and methods cannot be added after registration.
    pub fn register(self) -> *mut AnyObject {
        unsafe { objc_registerClassPair(self.0) };
        self.0 as *mut AnyObject
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::*;

    #[link(name = "Foundation", kind = "framework")]
    unsafe extern "C" {}

    #[test]
    fn test_anyclass_get_known() {
        let cls = AnyClass::get(c"NSObject");
        assert!(cls.is_some(), "NSObject should always exist");
    }

    #[test]
    fn test_anyclass_get_unknown() {
        let cls = AnyClass::get(c"NoSuchClassXyzAbc123");
        assert!(cls.is_none(), "unknown class should return None");
    }

    #[test]
    fn test_class_declaration() {
        extern "C" fn test_method(_self: *mut AnyObject, _cmd: Sel) {}

        let mut builder =
            ClassBuilder::new(c"TestClass2", class!(NSObject)).expect("Failed to create class");
        assert!(builder.add_ivar::<i32>(c"test_ivar"));
        assert!(builder.add_method(sel!(testMethod), test_method as extern "C" fn(_, _)));
        let class = builder.register();
        assert!(!class.is_null());
    }

    #[test]
    fn test_ivar_read_write() {
        let mut builder =
            ClassBuilder::new(c"TestIvarClass", class!(NSObject)).expect("create class");
        assert!(builder.add_ivar::<i64>(c"value"));
        let class = builder.register();

        let obj: *mut AnyObject = unsafe { msg_send![class, alloc] };
        let obj: *mut AnyObject = unsafe { msg_send![obj, init] };
        assert!(!obj.is_null());

        #[allow(deprecated)]
        unsafe {
            *(*obj).get_mut_ivar::<i64>("value") = 12345;
            let read = *(*obj).get_ivar::<i64>("value");
            assert_eq!(read, 12345);
        }

        unsafe {
            let _: () = msg_send![obj, release];
        }
    }
}
