/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

pub use context::Context;
pub use value::{ArrayValue, ObjectValue, Value};

mod builtins;
mod context;
mod interpreter;
mod lexer;
mod parser;
mod value;
