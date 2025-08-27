/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A cross-platform window library for Rust with minimal dependencies

/// Device-independent pixel structs
pub mod dpi;
/// Event
pub mod event;
/// EventLoop
pub mod event_loop;
/// Monitor
pub mod monitor;
pub(crate) mod platforms;
/// Window
pub mod window;
