/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Minimal clap-compatible argument parsing library
//!
//! See [README.md](../README.md) for usage.

#![forbid(unsafe_code)]

pub use clap_derive::{Parser, Subcommand};

/// Private module for generated code use only.
#[doc(hidden)]
pub mod __private {
    /// Internal trait implemented by `#[derive(Subcommand)]` enums.
    pub trait Subcommand: Sized {
        /// Try to parse a subcommand from `name`.  `pos` points at the subcommand
        /// token in `args`; the implementation may advance it to consume additional
        /// positional arguments belonging to the variant.
        fn try_parse(name: &str, args: &[String], pos: &mut usize) -> Option<Self>;
    }
}
