/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
