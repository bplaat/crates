/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [dotenv](https://crates.io/crates/dotenv) crate

use std::path::Path;
use std::{env, fs, io};

/// Read .env file from current directory and set environment variables
pub fn dotenv() -> io::Result<()> {
    from_path(".env")
}

/// Read env file and set environment variables
pub fn from_path(path: impl AsRef<Path>) -> io::Result<()> {
    let contents = fs::read_to_string(path)?;
    for line in contents.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            if !key.is_empty() && !value.is_empty() {
                unsafe { env::set_var(key, value) };
            }
        }
    }
    Ok(())
}
