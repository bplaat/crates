/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A base64 encoder and decoder

use std::error::Error;
use std::fmt::{self, Display, Formatter};

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

// MARK: Encode
/// Encode bytes to base64
pub fn encode(input: &[u8], pad: bool) -> String {
    let mut output = String::new();
    let mut buffer = 0u32;
    let mut bits_collected = 0;

    for &byte in input {
        buffer = (buffer << 8) | byte as u32;
        bits_collected += 8;
        while bits_collected >= 6 {
            bits_collected -= 6;
            let index = (buffer >> bits_collected) & 0x3F;
            output.push(BASE64_CHARS[index as usize] as char);
        }
    }
    if bits_collected > 0 {
        buffer <<= 6 - bits_collected;
        let index = buffer & 0x3F;
        output.push(BASE64_CHARS[index as usize] as char);
    }

    if pad {
        while output.len() % 4 != 0 {
            output.push('=');
        }
    }

    output
}

// MARK: Decode
/// Decode base64 to bytes
pub fn decode(input: &str, check_padding: bool) -> Result<Vec<u8>, DecodeError> {
    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits_collected = 0;
    let mut padding = 0;

    for c in input.chars() {
        if c == '=' {
            padding += 1;
            continue;
        }

        let value = BASE64_CHARS.iter().position(|&x| x == c as u8);
        if let Some(index) = value {
            buffer = (buffer << 6) | index as u32;
            bits_collected += 6;
            if bits_collected >= 8 {
                bits_collected -= 8;
                output.push((buffer >> bits_collected) as u8);
            }
        } else {
            return Err(DecodeError);
        }
    }

    if check_padding && (padding > 2 || (input.len() % 4 != 0 && padding == 0)) {
        return Err(DecodeError);
    }

    Ok(output)
}

// MARK: DecodeError
/// Decode error
#[derive(Debug)]
pub struct DecodeError;

impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Base64 decode error")
    }
}

impl Error for DecodeError {}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(encode(b"hello", false), "aGVsbG8");
        assert_eq!(encode(b"hello", true), "aGVsbG8=");
        assert_eq!(encode(b"hello world", false), "aGVsbG8gd29ybGQ");
        assert_eq!(encode(b"hello world", true), "aGVsbG8gd29ybGQ=");
    }

    #[test]
    fn test_decode() {
        assert_eq!(decode("aGVsbG8=", true).unwrap(), b"hello");
        assert_eq!(decode("aGVsbG8gd29ybGQ=", true).unwrap(), b"hello world");
        assert_eq!(decode("aGVsbG8gd29ybGQ", false).unwrap(), b"hello world");
        assert!(decode("aGVsbG8", true).is_err());
        assert!(decode("aGVsbG8", false).is_ok());
        assert!(decode("aGVsbG8=", true).is_ok());
    }
}
