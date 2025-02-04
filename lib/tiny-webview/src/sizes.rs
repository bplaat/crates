/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

/// A point in device-independent pixels
#[derive(Debug, Clone, Copy)]
pub struct LogicalPoint {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

impl LogicalPoint {
    /// Create new logical point
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A size in device-independent pixels
#[derive(Debug, Clone, Copy)]
pub struct LogicalSize {
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl LogicalSize {
    /// Create new logical size
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}
