/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

mod executor;
mod log;

pub use executor::{Executor, ExecutorBuilder, Task, TaskAction};
pub use log::{Log, LogEntry};
