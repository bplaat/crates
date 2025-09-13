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
    from_str(&fs::read_to_string(path.as_ref())?)
}

/// Read env from string and set environment variables
pub fn from_str(contents: impl AsRef<str>) -> io::Result<()> {
    let contents = contents.as_ref();
    for line in contents.lines() {
        // Remove inline comments
        let line = match line.find('#') {
            Some(idx) => &line[..idx],
            None => line,
        };

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let mut value = value.trim().to_string();

            // Expand variables in the value
            while let Some(start) = value.find('$') {
                if let Some(end) = value[start + 1..]
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .map(|e| start + 1 + e)
                    .or(Some(value.len()))
                {
                    let var_name = &value[start + 1..end];
                    if !var_name.is_empty() {
                        let var_value = env::var(var_name).unwrap_or_default();
                        value.replace_range(start..end, &var_value);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if !key.is_empty() && !value.is_empty() {
                unsafe { env::set_var(key, value) };
            }
        }
    }
    Ok(())
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parsing() {
        // Basic key-value parsing
        unsafe { env::remove_var("FOO") };
        from_str("FOO=bar").unwrap();
        assert_eq!(env::var("FOO").unwrap(), "bar");

        // Trimming whitespace
        unsafe { env::remove_var("BAR") };
        from_str("BAR = baz ").unwrap();
        assert_eq!(env::var("BAR").unwrap(), "baz");

        // Inline comment removal
        unsafe { env::remove_var("BAZ") };
        from_str("BAZ=qux # this is a comment").unwrap();
        assert_eq!(env::var("BAZ").unwrap(), "qux");

        // Variable expansion
        unsafe { env::set_var("USER", "alice") };
        unsafe { env::remove_var("GREETING") };
        from_str("GREETING=Hello $USER!").unwrap();
        assert_eq!(env::var("GREETING").unwrap(), "Hello alice!");

        // Multiple variables
        unsafe { env::remove_var("A") };
        unsafe { env::remove_var("B") };
        from_str("A=1\nB=2").unwrap();
        assert_eq!(env::var("A").unwrap(), "1");
        assert_eq!(env::var("B").unwrap(), "2");

        // Empty lines and comments
        unsafe { env::remove_var("EMPTY") };
        from_str("\n# Comment\nEMPTY=value\n").unwrap();
        assert_eq!(env::var("EMPTY").unwrap(), "value");

        // Variable expansion with missing variable
        unsafe { env::remove_var("MISSING") };
        unsafe { env::remove_var("EXPAND") };
        from_str("EXPAND=$MISSING").unwrap();
        assert_eq!(env::var("EXPAND"), Err(env::VarError::NotPresent));

        // No value or key
        assert!(from_str("NOVALUE=").is_ok());
        assert!(from_str("=NOKEY").is_ok());
    }
}
