/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::c_void;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::{NonNull, null_mut};
use std::rc::Rc;

#[repr(C)]
pub(crate) struct IUnknownVtbl {
    pub(crate) query_interface:
        unsafe extern "system" fn(*mut c_void, *const c_void, *mut *mut c_void) -> i32,
    pub(crate) add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    pub(crate) release: unsafe extern "system" fn(*mut c_void) -> u32,
}

/// One owned COM reference. Canvas COM objects stay on the window thread.
pub(crate) struct ComPtr<T> {
    pointer: NonNull<T>,
    _thread: PhantomData<Rc<()>>,
}

impl<T> ComPtr<T> {
    /// The callback must follow COM out-parameter ownership: any non-null result
    /// is an owned reference to T with an IUnknown-compatible vtable prefix.
    pub(crate) unsafe fn create(create: impl FnOnce(*mut *mut T) -> i32) -> Result<Self, i32> {
        let mut pointer = null_mut();
        let result = create(&mut pointer);
        let owned = NonNull::new(pointer).map(|pointer| Self {
            pointer,
            _thread: PhantomData,
        });
        check(result)?;
        owned.ok_or(0x80004003u32 as i32)
    }

    pub(crate) const fn as_ptr(&self) -> *mut T {
        self.pointer.as_ptr()
    }

    fn unknown(&self) -> &IUnknownVtbl {
        // Every COM interface begins with an IUnknown vtable prefix.
        unsafe { &**self.as_ptr().cast::<*const IUnknownVtbl>() }
    }
}

impl<T> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        unsafe { (self.unknown().add_ref)(self.as_ptr().cast()) };
        Self {
            pointer: self.pointer,
            _thread: PhantomData,
        }
    }
}

impl<T> Deref for ComPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.pointer.as_ref() }
    }
}

impl<T> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe { (self.unknown().release)(self.as_ptr().cast()) };
    }
}

pub(crate) const fn check(result: i32) -> Result<(), i32> {
    if result < 0 { Err(result) } else { Ok(()) }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[repr(C)]
    struct Object {
        vtable: *const IUnknownVtbl,
        refs: Cell<u32>,
        releases: Cell<u32>,
    }

    unsafe extern "system" fn query(_: *mut c_void, _: *const c_void, _: *mut *mut c_void) -> i32 {
        0x80004002u32 as i32
    }

    unsafe extern "system" fn add_ref(this: *mut c_void) -> u32 {
        let object = unsafe { &*this.cast::<Object>() };
        object.refs.set(object.refs.get() + 1);
        object.refs.get()
    }

    unsafe extern "system" fn release(this: *mut c_void) -> u32 {
        let object = unsafe { &*this.cast::<Object>() };
        object.refs.set(object.refs.get() - 1);
        object.releases.set(object.releases.get() + 1);
        object.refs.get()
    }

    const VTABLE: IUnknownVtbl = IUnknownVtbl {
        query_interface: query,
        add_ref,
        release,
    };

    #[test]
    fn owned_references_clone_and_release_once() {
        let mut object = Object {
            vtable: &VTABLE,
            refs: Cell::new(1),
            releases: Cell::new(0),
        };
        let owned = unsafe {
            ComPtr::create(|out| {
                *out = &mut object;
                0
            })
        }
        .expect("COM object");
        let cloned = owned.clone();
        assert_eq!(object.refs.get(), 2);
        drop(owned);
        assert_eq!(cloned.refs.get(), 1);
        drop(cloned);
        assert_eq!(object.refs.get(), 0);
        assert_eq!(object.releases.get(), 2);
    }

    #[test]
    fn failed_creation_releases_output_and_null_success_is_rejected() {
        let mut object = Object {
            vtable: &VTABLE,
            refs: Cell::new(1),
            releases: Cell::new(0),
        };
        assert!(
            unsafe {
                ComPtr::create(|out| {
                    *out = &mut object;
                    -1
                })
            }
            .is_err()
        );
        assert_eq!(object.refs.get(), 0);
        assert!(unsafe { ComPtr::<Object>::create(|_| 0) }.is_err());
        assert!(check(1).is_ok());
    }
}
