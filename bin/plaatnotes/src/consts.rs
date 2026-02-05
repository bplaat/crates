/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

// Length of the session token in bytes (2048 bits)
pub(crate) const SESSION_TOKEN_LENGTH: usize = 256;

// Default session expiry duration in seconds (1 year)
pub(crate) const SESSION_EXPIRY_SECONDS: u64 = 365 * 24 * 60 * 60;
