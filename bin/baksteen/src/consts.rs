/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// NOTE: The bed and head joints sizes are switched because otherwise the example wouldn't fit
pub(crate) const BRICK_BED_JOINT: f64 = 10.0; // Padding between bricks
pub(crate) const BRICK_HEAD_JOINT: f64 = 12.5; // Padding between rows

pub(crate) const BRICK_HALF_WIDTH: f64 = 100.0;
pub(crate) const BRICK_WIDTH: f64 = BRICK_HALF_WIDTH * 2.0 + BRICK_BED_JOINT;
pub(crate) const BRICK_TWO_THIRDS_WIDTH: f64 = BRICK_HALF_WIDTH * 1.5 + BRICK_BED_JOINT / 2.0;
pub(crate) const BRICK_HEIGHT: f64 = 50.0;

pub(crate) const STRIDE_WIDTH: f64 = 800.0; // Robot's horizontal reach
pub(crate) const STRIDE_HEIGHT: f64 = 1300.0; // Robot's vertical reach
