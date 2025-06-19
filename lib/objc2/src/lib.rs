/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [objc2](https://crates.io/crates/objc2) crate

#![cfg(target_os = "macos")]

pub use encode::{Encode, Encoding};

/// Encode
pub mod encode;
/// Raw FFI bindings
pub mod ffi;
/// Macros
pub mod macros;
/// Runtime
pub mod runtime;
