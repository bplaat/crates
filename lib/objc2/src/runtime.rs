/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::{CStr, CString, c_void};
use std::ptr::NonNull;

use crate::encode::{Encode, Encoding};
use crate::ffi::*;

/// Class type (opaque).
#[repr(C)]
pub struct AnyClass([u8; 0]);

// SAFETY: Objective-C Class values are pointers with the `#` type encoding.
unsafe impl Encode for *const AnyClass {
    const ENCODING: Encoding = Encoding::Class;
}

// SAFETY: Objective-C Class values are pointers with the `#` type encoding.
unsafe impl Encode for *mut AnyClass {
    const ENCODING: Encoding = Encoding::Class;
}

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

/// Protocol type (opaque).
#[repr(C)]
pub struct AnyProtocol([u8; 0]);

impl AnyProtocol {
    /// Get a protocol by name, returning `None` if it is not registered.
    pub fn get(name: &CStr) -> Option<&'static Self> {
        // SAFETY: `name` is a valid null-terminated C string. The Objective-C runtime owns
        // registered protocol objects for the lifetime of the process.
        unsafe { objc_getProtocol(name.as_ptr()).as_ref() }
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
pub struct AnyObject([u8; 0]);

/// Marker trait for types represented by Objective-C object pointers.
///
/// # Safety
///
/// Implementors must be Objective-C object types with pointer-compatible representations.
pub unsafe trait Message {}

// SAFETY: AnyObject is the erased Objective-C object type.
unsafe impl Message for AnyObject {}

/// An Objective-C class defined in Rust.
///
/// # Safety
///
/// The hidden superclass must be the superclass used when registering this class.
pub unsafe trait ClassType: Message {
    #[doc(hidden)]
    fn __superclass() -> *const AnyClass;
}

/// A Rust-defined Objective-C class with explicitly initialized instance variables.
///
/// # Safety
///
/// The hidden offsets must identify storage registered for exactly `Ivars` and its initialization
/// flag on every instance of the class.
pub unsafe trait DefinedClass: ClassType {
    /// Rust instance-variable storage initialized by `Allocated::set_ivars`.
    type Ivars;

    #[doc(hidden)]
    fn __ivars_offset() -> usize;
    #[doc(hidden)]
    fn __ivars_initialized_offset() -> usize;
}

unsafe fn ivars_initialized_ptr<T: DefinedClass>(object: NonNull<T>) -> *mut u8 {
    // SAFETY: DefinedClass guarantees that the offset identifies the initialization flag.
    unsafe {
        object
            .cast::<u8>()
            .as_ptr()
            .add(T::__ivars_initialized_offset())
    }
}

unsafe fn ivars_ptr<T: DefinedClass>(object: NonNull<T>) -> *mut T::Ivars {
    // SAFETY: DefinedClass guarantees that the offset identifies aligned storage for T::Ivars.
    unsafe {
        object
            .cast::<u8>()
            .as_ptr()
            .add(T::__ivars_offset())
            .cast::<T::Ivars>()
    }
}

#[doc(hidden)]
pub unsafe fn initialize_ivars<T: DefinedClass>(object: NonNull<T>, ivars: T::Ivars) {
    // SAFETY: the caller supplies a valid newly allocated T.
    let initialized = unsafe { ivars_initialized_ptr(object) };
    // SAFETY: the flag is allocated as a u8 and Objective-C zero-initializes object storage.
    let state = unsafe { initialized.read() };
    assert_eq!(state, 0, "ivars were already initialized");
    // SAFETY: the caller guarantees exclusive initialization access to this object.
    unsafe { ivars_ptr(object).write(ivars) };
    // SAFETY: mark initialized only after the complete value was written.
    unsafe { initialized.write(1) };
}

#[doc(hidden)]
pub fn ivars<T: DefinedClass>(object: &T) -> &T::Ivars {
    let object = NonNull::from(object);
    // SAFETY: `object` came from a live shared reference to `T`.
    let initialized = unsafe { ivars_initialized_ptr(object) };
    // SAFETY: the flag remains allocated for the object's lifetime.
    let state = unsafe { initialized.read() };
    assert_eq!(state, 1, "ivars are not initialized");
    // SAFETY: the initialized flag proves set_ivars wrote a valid value. The returned reference
    // is tied to the input object's lifetime.
    unsafe { &*ivars_ptr(object) }
}

#[doc(hidden)]
pub unsafe fn destroy_ivars<T: DefinedClass>(object: NonNull<T>) {
    // SAFETY: the caller supplies the live object currently being deallocated.
    let initialized = unsafe { ivars_initialized_ptr(object) };
    // SAFETY: the flag remains allocated until the superclass dealloc runs.
    match unsafe { initialized.read() } {
        0 => {}
        1 => {
            // SAFETY: clear the flag first so unwinding cannot cause a second drop.
            unsafe { initialized.write(0) };
            // SAFETY: flag value 1 proves set_ivars fully initialized this value exactly once.
            unsafe { ivars_ptr(object).drop_in_place() };
        }
        _ => panic!("invalid ivar initialization flag"),
    }
}

// SAFETY: an ObjC object pointer has encoding `@` as defined by the Apple ABI.
unsafe impl<T: Message> Encode for *const T {
    const ENCODING: Encoding = Encoding::Object;
}
// SAFETY: an ObjC object pointer has encoding `@` as defined by the Apple ABI.
unsafe impl<T: Message> Encode for *mut T {
    const ENCODING: Encoding = Encoding::Object;
}
// SAFETY: a reference to a Message type is passed to Objective-C as an object pointer.
unsafe impl<T: Message> Encode for &T {
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bool {
    #[cfg(target_arch = "aarch64")]
    value: bool,
    #[cfg(not(target_arch = "aarch64"))]
    value: i8,
}
impl Bool {
    /// `YES`
    pub const YES: Self = Self {
        #[cfg(target_arch = "aarch64")]
        value: true,
        #[cfg(not(target_arch = "aarch64"))]
        value: 1,
    };
    /// `NO`
    pub const NO: Self = Self {
        #[cfg(target_arch = "aarch64")]
        value: false,
        #[cfg(not(target_arch = "aarch64"))]
        value: 0,
    };

    /// Convert the Objective-C boolean to a Rust boolean.
    pub const fn as_bool(&self) -> bool {
        #[cfg(target_arch = "aarch64")]
        {
            self.value
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            self.value != 0
        }
    }
}
// SAFETY: Apple's Objective-C BOOL is C _Bool on aarch64 and signed char elsewhere.
unsafe impl Encode for Bool {
    const ENCODING: Encoding = if cfg!(target_arch = "aarch64") {
        Encoding::Bool
    } else {
        Encoding::Char
    };
}

/// Trait for `extern "C-unwind"` function pointers usable as ObjC method implementations.
///
/// Automatically derives the ObjC type encoding from the Rust function signature.
/// Implemented for `extern "C-unwind" fn(*mut AnyObject, Sel, ...) -> R` at supported arities.
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
            for extern "C-unwind" fn(*mut AnyObject, Sel $(, $t)*) -> Ret
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
    ///
    /// # Safety
    ///
    /// `superclass` must be a valid Objective-C class pointer or null.
    pub unsafe fn new(name: &CStr, superclass: *mut AnyObject) -> Option<Self> {
        // SAFETY: The caller guarantees that superclass is null or a valid class pointer.
        let class =
            unsafe { objc_allocateClassPair(superclass as *const c_void, name.as_ptr(), 0) };
        if class.is_null() {
            None
        } else {
            Some(Self(class))
        }
    }

    /// Add opaque storage that `Allocated::set_ivars` initializes before it is read.
    ///
    /// Stores `T` as a single opaque ivar with size/alignment derived from its layout. Used
    /// internally by `define_class!`-generated code; prefer `add_ivar` for typed ivars.
    #[doc(hidden)]
    pub fn add_ivar_raw<T>(&mut self, name: &CStr) -> bool {
        // SAFETY: `self.0` is a valid not-yet-registered class pair; `name` is null-terminated;
        // `c"?"` is the ObjC "unknown" encoding, valid for any opaque Rust type.
        unsafe {
            class_addIvar(
                self.0,
                name.as_ptr(),
                size_of::<T>().max(1),
                align_of::<T>().trailing_zeros() as u8,
                c"?".as_ptr(),
            )
            .as_bool()
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
            .as_bool()
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
        unsafe { class_addMethod(self.0, sel.0, imp_ptr, encoding.as_ptr()).as_bool() }
    }

    /// Make the class conform to the given protocol.
    pub fn add_protocol(&mut self, protocol: &AnyProtocol) -> bool {
        // SAFETY: `self.0` is a valid not-yet-registered class pair and `protocol` is a
        // process-lifetime protocol object returned by the Objective-C runtime.
        unsafe { class_addProtocol(self.0.cast(), protocol).as_bool() }
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
    fn test_anyprotocol_get_known() {
        let protocol = AnyProtocol::get(c"NSCopying");
        assert!(protocol.is_some(), "NSCopying should always exist");
    }

    #[test]
    fn test_class_add_protocol() {
        let protocol = AnyProtocol::get(c"NSCopying").expect("NSCopying should exist");
        // SAFETY: NSObject is a valid Objective-C class returned by the runtime.
        let mut builder = unsafe { ClassBuilder::new(c"TestProtocolClass", class!(NSObject)) }
            .expect("create class");
        assert!(builder.add_protocol(protocol));
        builder.register();
    }

    #[test]
    fn test_class_declaration() {
        extern "C-unwind" fn test_method(_self: *mut AnyObject, _cmd: Sel) {}

        // SAFETY: NSObject is a valid Objective-C class returned by the runtime.
        let mut builder = unsafe { ClassBuilder::new(c"TestClass2", class!(NSObject)) }
            .expect("Failed to create class");
        assert!(builder.add_ivar::<i32>(c"test_ivar"));
        assert!(builder.add_method(sel!(testMethod), test_method as extern "C-unwind" fn(_, _)));
        let class = builder.register();
        assert!(!class.is_null());
    }

    #[test]
    fn test_ivar_read_write() {
        // SAFETY: NSObject is a valid Objective-C class returned by the runtime.
        let mut builder =
            unsafe { ClassBuilder::new(c"TestIvarClass", class!(NSObject)) }.expect("create class");
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
        type Method = extern "C-unwind" fn(*mut AnyObject, Sel) -> i32;
        assert_eq!(
            <Method as MethodImpl>::type_encoding()
                .to_str()
                .expect("valid encoding"),
            "i@:"
        );
    }

    #[test]
    fn test_method_impl_type_encoding_with_args() {
        type Method = extern "C-unwind" fn(*mut AnyObject, Sel, i32, Bool) -> ();
        let expected = format!("v@:i{}", Bool::ENCODING);
        assert_eq!(
            <Method as MethodImpl>::type_encoding()
                .to_str()
                .expect("valid encoding"),
            expected
        );
    }

    #[test]
    fn test_classbuilder_duplicate_name_returns_none() {
        extern "C-unwind" fn noop(_this: *mut AnyObject, _cmd: Sel) {}
        // SAFETY: NSObject is a valid Objective-C class returned by the runtime.
        let mut builder = unsafe { ClassBuilder::new(c"DupTestClass", class!(NSObject)) }
            .expect("first registration ok");
        assert!(builder.add_method(sel!(noop), noop as extern "C-unwind" fn(_, _)));
        builder.register();
        assert!(
            // SAFETY: NSObject is a valid Objective-C class returned by the runtime.
            unsafe { ClassBuilder::new(c"DupTestClass", class!(NSObject)) }.is_none(),
            "re-registering an existing class name must return None"
        );
    }
}
