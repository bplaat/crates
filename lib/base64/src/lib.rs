/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A base64 encoder and decoder

use std::error::Error;
use std::fmt::{self, Display, Formatter};

// MARK: Lookup tables
const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const BASE64_CHARS_REVERSE: [i8; 256] = {
    let mut lookup = [-1; 256];
    let mut i = 0;
    while i < BASE64_CHARS.len() {
        lookup[BASE64_CHARS[i] as usize] = i as i8;
        i += 1;
    }
    lookup
};

// MARK: Encode
/// Encode bytes to base64
pub fn encode(input: &[u8], omit_padding: bool) -> String {
    let mut output = String::with_capacity(input.len() * 4 / 3 + 3);
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
    if !omit_padding {
        while output.len() % 4 != 0 {
            output.push('=');
        }
    }
    output
}

// MARK: Decode
/// Decode base64 to bytes
pub fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buffer = 0u32;
    let mut bits_collected = 0;
    for c in input.bytes() {
        if c == b'=' {
            continue;
        }
        let index = BASE64_CHARS_REVERSE[c as usize];
        if index == -1 {
            return Err(DecodeError);
        }
        buffer = (buffer << 6) | index as u32;
        bits_collected += 6;
        if bits_collected >= 8 {
            bits_collected -= 8;
            output.push((buffer >> bits_collected) as u8);
        }
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
        assert_eq!(encode(b"hello", true), "aGVsbG8");
        assert_eq!(encode(b"hello", false), "aGVsbG8=");
        assert_eq!(encode(b"hello world", true), "aGVsbG8gd29ybGQ");
        assert_eq!(encode(b"hello world", false), "aGVsbG8gd29ybGQ=");
    }

    #[test]
    fn test_decode() {
        assert_eq!(decode("aGVsbG8").unwrap(), b"hello");
        assert_eq!(decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(decode("aGVsbG8==").unwrap(), b"hello");
        assert_eq!(decode("aGVsbG8===").unwrap(), b"hello");
        assert_eq!(decode("aGVsbG8gd29ybGQ").unwrap(), b"hello world");
        assert_eq!(decode("aGVsbG8gd29ybGQ=").unwrap(), b"hello world");
        assert_eq!(decode("aGVsbG8gd29ybGQ==").unwrap(), b"hello world");
        assert_eq!(decode("aGVsbG8gd29ybGQ===").unwrap(), b"hello world");
    }
}
