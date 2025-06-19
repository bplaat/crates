/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(missing_docs, non_camel_case_types)]

use std::ffi::{c_char, c_void};

use crate::runtime::{AnyClass as Class, AnyObject as Object, Sel};

#[repr(C)]
pub struct objc_super {
    pub receiver: *mut Object,
    pub super_class: *const Class,
}

#[link(name = "objc", kind = "dylib")]
unsafe extern "C" {
    pub fn objc_getClass(name: *const c_char) -> *mut Class;
    pub fn sel_registerName(name: *const c_char) -> *mut Sel;
    pub fn objc_msgSend(receiver: *mut Object, sel: *const Sel, ...) -> *mut c_void;
    #[cfg(target_arch = "x86_64")]
    pub fn objc_msgSend_stret(ret: *mut c_void, receiver: *mut Object, sel: *const Sel, ...);
    pub fn objc_msgSendSuper(receiver: *const objc_super, sel: *const Sel, ...) -> *mut c_void;

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
