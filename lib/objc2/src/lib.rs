/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [objc2](https://crates.io/crates/objc2) crate

#![cfg(target_vendor = "apple")]
#![allow(unsafe_code)]

pub use encode::{Encode, Encoding};

/// Encode
pub mod encode;
/// Raw FFI bindings
pub mod ffi;
#[doc(hidden)]
pub mod macros;
/// Reference counting utilities
pub mod rc;
/// Runtime
pub mod runtime;
#[cfg(debug_assertions)]
pub(crate) mod verify;
