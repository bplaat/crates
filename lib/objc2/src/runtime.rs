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
        // SAFETY: `name` is a valid null-terminated C string; `objc_getClass` returns either
        // a valid class pointer or null, both of which we check before use.
        let cls = unsafe { objc_getClass(name.as_ptr()) };
        if cls.is_null() {
            None
        } else {
            // SAFETY: `cls` is a non-null valid class pointer returned by the ObjC runtime;
            // casting to `*const AnyClass` (an opaque single-byte struct) is always valid.
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
// SAFETY: selectors are immutable global pointers managed by the ObjC runtime.
unsafe impl Send for Sel {}
// SAFETY: selectors are immutable global pointers managed by the ObjC runtime.
unsafe impl Sync for Sel {}

// SAFETY: a selector has ObjC encoding `:` which is how the runtime encodes `SEL`.
unsafe impl Encode for Sel {
    const ENCODING: Encoding = Encoding::Sel;
}

/// AnyObject type (opaque).
#[repr(C)]
pub struct AnyObject(u8);

// SAFETY: an ObjC object pointer has encoding `@` as defined by the Apple ABI.
unsafe impl Encode for *const AnyObject {
    const ENCODING: Encoding = Encoding::Object;
}
// SAFETY: an ObjC object pointer has encoding `@` as defined by the Apple ABI.
unsafe impl Encode for *mut AnyObject {
    const ENCODING: Encoding = Encoding::Object;
}

impl AnyObject {
    /// Get a reference to an instance variable of this object.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `T` exactly matches the declared type of the ivar named `name`.
    /// - `self` is a fully initialised Objective-C object (i.e., `init` has been called).
    /// - The ivar named `name` exists on `self`'s class (otherwise the function panics).
    /// - No `&mut` reference to the same ivar exists for the lifetime of the returned `&T`.
    #[deprecated]
    pub unsafe fn get_ivar<T: Encode>(&self, name: &str) -> &T {
        let name = CString::new(name).expect("Failed to convert to CString");
        // SAFETY: `self` is a valid ObjC object pointer.
        let cls = unsafe { object_getClass(self as *const AnyObject) };
        // SAFETY: `cls` comes from `object_getClass` on a valid object.
        let ivar = unsafe { class_getInstanceVariable(cls, name.as_ptr()) };
        assert!(
            !ivar.is_null(),
            "ivar '{}' not found",
            name.to_string_lossy()
        );
        // SAFETY: `ivar` is non-null (asserted above).
        let offset = unsafe { ivar_getOffset(ivar) } as usize;
        // SAFETY: caller guarantees `T` matches the declared ivar type and alignment.
        unsafe { &*((self as *const AnyObject as *const u8).add(offset) as *const T) }
    }

    /// Get a mutable reference to an instance variable of this object.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `T` exactly matches the declared type of the ivar named `name`.
    /// - `self` is a fully initialised Objective-C object (i.e., `init` has been called).
    /// - The ivar named `name` exists on `self`'s class (otherwise the function panics).
    /// - No other reference (shared or exclusive) to the same ivar exists for the lifetime
    ///   of the returned `&mut T`.
    #[deprecated]
    pub unsafe fn get_mut_ivar<T: Encode>(&mut self, name: &str) -> &mut T {
        let name = CString::new(name).expect("Failed to convert to CString");
        // SAFETY: `self` is a valid ObjC object pointer.
        let cls = unsafe { object_getClass(self as *const AnyObject) };
        // SAFETY: `cls` comes from `object_getClass` on a valid object.
        let ivar = unsafe { class_getInstanceVariable(cls, name.as_ptr()) };
        assert!(
            !ivar.is_null(),
            "ivar '{}' not found",
            name.to_string_lossy()
        );
        // SAFETY: `ivar` is non-null (asserted above).
        let offset = unsafe { ivar_getOffset(ivar) } as usize;
        // SAFETY: caller guarantees `T` matches the declared ivar type, holds `&mut self`
        // so exclusive access is guaranteed.
        unsafe { &mut *((self as *mut AnyObject as *mut u8).add(offset) as *mut T) }
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
// SAFETY: `Bool` is a transparent `u8`; ObjC encodes it as `B`.
unsafe impl Encode for Bool {
    const ENCODING: Encoding = Encoding::Bool;
}

/// Trait for `extern "C"` function pointers usable as ObjC method implementations.
///
/// Automatically derives the ObjC type encoding from the Rust function signature.
/// Implemented for `extern "C" fn(*mut AnyObject, Sel, ...) -> R` at all supported arities.
///
/// # Safety
///
/// The implementor must ensure that the function pointer returned by `imp_ptr()` has a
/// signature that exactly matches the encoding returned by `type_encoding()`. A mismatch
/// causes the ObjC runtime to call the function with incorrectly typed arguments, resulting
/// in undefined behavior.
pub unsafe trait MethodImpl: Copy {
    /// Returns the function pointer cast to `*const c_void`.
    fn imp_ptr(self) -> *const c_void;
    /// Builds the ObjC type encoding string for this function's full signature.
    fn type_encoding() -> CString;
}

macro_rules! impl_method_impl {
    ($($t:ident),*) => {
        // SAFETY: `type_encoding()` is derived mechanically from the same generic bounds
        // that constrain `imp_ptr()`, so the encoding always matches the function signature.
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
        // SAFETY: `name` is a valid null-terminated C string; `superclass` is a valid class
        // pointer (from `class!`). `objc_allocateClassPair` returns null on failure (e.g.,
        // duplicate class name), which we check before use.
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
        // SAFETY: `self.0` is a valid class pair not yet registered; `name` is null-terminated;
        // size and alignment are computed from `T`'s actual layout via `size_of`/`align_of`.
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
        // SAFETY: `self.0` is a valid class pair not yet registered; `sel.0` is a registered
        // selector; `imp_ptr` is a valid `extern "C"` function pointer whose signature matches
        // `encoding` by the `MethodImpl` safety contract.
        unsafe { class_addMethod(self.0, sel.0, imp_ptr, encoding.as_ptr()) }
    }

    /// Register the class and return it as a `*mut AnyObject`.
    ///
    /// Consumes the builder since ivars and methods cannot be added after registration.
    pub fn register(self) -> *mut AnyObject {
        // SAFETY: `self.0` is a valid, not-yet-registered class pair produced by
        // `objc_allocateClassPair`; registering it is safe exactly once (enforced by consuming self).
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

        // SAFETY: `class` is a freshly registered class; alloc returns a valid uninitialized object.
        let obj: *mut AnyObject = unsafe { msg_send![class, alloc] };
        // SAFETY: `obj` is a valid uninitialized object from alloc; init is always valid.
        let obj: *mut AnyObject = unsafe { msg_send![obj, init] };
        assert!(!obj.is_null());

        // SAFETY: `obj` is fully initialized; "value" ivar exists (added above) and T=i64 matches.
        #[allow(deprecated)]
        unsafe {
            *(*obj).get_mut_ivar::<i64>("value") = 12345;
            let read = *(*obj).get_ivar::<i64>("value");
            assert_eq!(read, 12345);
        }

        // SAFETY: `obj` is a valid ObjC object; release decrements the retain count.
        unsafe {
            let _: () = msg_send![obj, release];
        }
    }

    #[test]
    fn test_method_impl_type_encoding_zero_args() {
        type Method = extern "C" fn(*mut AnyObject, Sel) -> i32;
        assert_eq!(
            <Method as MethodImpl>::type_encoding()
                .to_str()
                .expect("valid encoding"),
            "i@:"
        );
    }

    #[test]
    fn test_method_impl_type_encoding_with_args() {
        type Method = extern "C" fn(*mut AnyObject, Sel, i32, Bool) -> ();
        assert_eq!(
            <Method as MethodImpl>::type_encoding()
                .to_str()
                .expect("valid encoding"),
            "v@:iB"
        );
    }
}
