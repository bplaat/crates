/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(dead_code)]

// Length of the session token in bytes (2048 bits)
pub(crate) const SESSION_TOKEN_LENGTH: usize = 256;

// Default session expiry duration in seconds (3 months)
pub(crate) const SESSION_EXPIRY_SECONDS: u64 = 90 * 24 * 60 * 60;

// Refresh threshold: extend expiry when less than 30 days remain
pub(crate) const SESSION_REFRESH_THRESHOLD_SECONDS: u64 = 30 * 24 * 60 * 60;

// Task runner interval in seconds (1 hour)
pub(crate) const TASK_RUNNER_INTERVAL_SECONDS: u64 = 60 * 60;

// Number of days after which trashed notes are permanently deleted
pub(crate) const TRASHED_NOTE_EXPIRY_DAYS: i64 = 30;

// PBKDF2 iterations: full strength in production, minimal in tests for speed
#[cfg(not(test))]
pub(crate) const PBKDF2_ITERATIONS: u32 = pbkdf2::DEFAULT_SAFE_ITERATIONS;
#[cfg(test)]
pub(crate) const PBKDF2_ITERATIONS: u32 = 1;

// Login rate limiting: max attempts per window
pub(crate) const LOGIN_RATE_LIMIT_MAX_ATTEMPTS: u32 = 10;

// Login rate limiting: window duration in seconds (15 minutes)
pub(crate) const LOGIN_RATE_LIMIT_WINDOW_SECONDS: u64 = 15 * 60;
