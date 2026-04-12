/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Constants

/// Session token length in bytes
pub(crate) const SESSION_TOKEN_LENGTH: usize = 32;
/// Session expiry in seconds (30 days)
pub(crate) const SESSION_EXPIRY_SECONDS: u64 = 30 * 24 * 60 * 60;
/// Session refresh threshold in seconds (7 days)
pub(crate) const SESSION_REFRESH_THRESHOLD_SECONDS: u64 = 7 * 24 * 60 * 60;
/// Max login attempts per window
pub(crate) const LOGIN_RATE_LIMIT_MAX_ATTEMPTS: u32 = 10;
/// Login rate limit window in seconds
pub(crate) const LOGIN_RATE_LIMIT_WINDOW_SECONDS: u64 = 60;
/// Default pagination limit
pub(crate) const DEFAULT_LIMIT: i64 = 20;
