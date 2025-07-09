/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Build and test examples test

#![cfg(not(windows))]

include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
