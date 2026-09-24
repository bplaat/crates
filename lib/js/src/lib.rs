/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

pub use crate::context::Context;
pub use crate::value::Value;

mod buildins;
mod context;
mod interpreter;
mod lexer;
mod parser;
mod value;
