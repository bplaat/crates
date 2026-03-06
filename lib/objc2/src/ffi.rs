/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(missing_docs, non_camel_case_types)]

use std::ffi::{c_char, c_void};

use crate::runtime::{AnyClass, AnyObject as Object};

#[repr(C)]
pub struct objc_super {
    pub receiver: *mut Object,
    pub super_class: *const AnyClass,
}

#[link(name = "objc", kind = "dylib")]
unsafe extern "C" {
    pub fn objc_getClass(name: *const c_char) -> *mut c_void;
    pub fn sel_registerName(name: *const c_char) -> *mut c_void;
    pub fn objc_msgSend(receiver: *mut Object, sel: *const c_void, ...) -> *mut c_void;
    #[cfg(target_arch = "x86_64")]
    pub fn objc_msgSend_stret(ret: *mut c_void, receiver: *mut Object, sel: *const c_void, ...);
    pub fn objc_msgSendSuper(receiver: *const objc_super, sel: *const c_void, ...) -> *mut c_void;

    pub fn object_getClass(obj: *const Object) -> *mut c_void;
    pub fn class_getInstanceVariable(cls: *const c_void, name: *const c_char) -> *const c_void;
    pub fn ivar_getOffset(ivar: *const c_void) -> isize;
    pub fn class_getInstanceMethod(cls: *const c_void, sel: *const c_void) -> *const c_void;
    pub fn method_getTypeEncoding(method: *const c_void) -> *const c_char;
    pub fn class_getName(cls: *const c_void) -> *const c_char;
    pub fn sel_getName(sel: *const c_void) -> *const c_char;

    pub fn objc_allocateClassPair(
        superclass: *const c_void,
        name: *const c_char,
        extraBytes: usize,
    ) -> *mut c_void;
    pub fn class_addIvar(
        class: *mut c_void,
        name: *const c_char,
        size: usize,
        alignment: u8,
        types: *const c_char,
    ) -> bool;
    pub fn class_addMethod(
        class: *mut c_void,
        sel: *const c_void,
        imp: *const c_void,
        types: *const c_char,
    ) -> bool;
    pub fn objc_registerClassPair(class: *mut c_void);

    pub fn objc_autoreleasePoolPush() -> *mut c_void;
    pub fn objc_autoreleasePoolPop(pool: *mut c_void);
}
