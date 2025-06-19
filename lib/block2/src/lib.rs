/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [block2](https://crates.io/crates/block2) crate

#![cfg(target_os = "macos")]

use std::ffi::c_void;

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

impl Block {
    /// Call a block
    pub fn call<T>(&self, _arg: T) {
        // unsafe {
        //     (self.invoke)();
        // }
    }
}
