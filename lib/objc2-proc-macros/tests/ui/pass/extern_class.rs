/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

extern crate self as objc2;

use objc2_proc_macros::extern_class;

pub mod runtime {
    pub struct AnyObject;
    pub unsafe trait Message {}
}

#[macro_export]
macro_rules! class {
    ($name:ident) => {
        ::std::ptr::null_mut::<$crate::runtime::AnyObject>()
    };
}

extern_class!(
    #[unsafe(super(NSObject))]
    #[name = "RenamedObject"]
    pub struct Object;
);

fn main() {}
