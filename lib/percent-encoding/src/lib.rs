/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [percent-encoding](https://crates.io/crates/percent-encoding) crate

use std::borrow::Cow;
use std::{fmt, str};

// MARK: AsciiSet
/// A set of ASCII bytes that should be percent-encoded.
///
/// Bytes outside ASCII (>= 128) are always encoded.
pub struct AsciiSet(
    // 256-bit mask: bit `i` set means byte `i` should be encoded.
    [u64; 4],
);

impl AsciiSet {
    fn contains(&self, byte: u8) -> bool {
        (self.0[(byte / 64) as usize] >> (byte % 64)) & 1 != 0
    }
}

/// Encodes every byte that is not an ASCII alphanumeric character.
pub const NON_ALPHANUMERIC: &AsciiSet = &AsciiSet([
    // bytes   0- 63: all encoded except digits 48-57
    0xFC00_FFFF_FFFF_FFFF,
    // bytes  64-127: '@', '['-'`', '{'-DEL encoded; 'A'-'Z' and 'a'-'z' not encoded
    0xF800_0001_F800_0001,
    // bytes 128-191: always encoded
    0xFFFF_FFFF_FFFF_FFFF,
    // bytes 192-255: always encoded
    0xFFFF_FFFF_FFFF_FFFF,
]);

// MARK: PercentEncode
/// The result of [`utf8_percent_encode`], which implements [`Display`](fmt::Display).
pub struct PercentEncode<'a> {
    input: &'a str,
    set: &'static AsciiSet,
}

impl fmt::Display for PercentEncode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.input.bytes() {
            if self.set.contains(byte) {
                write!(f, "%{byte:02X}")?;
            } else {
                write!(f, "{}", byte as char)?;
            }
        }
        Ok(())
    }
}

/// Percent-encode every byte of the UTF-8 encoding of `input` that is in `set`.
pub fn utf8_percent_encode<'a>(input: &'a str, set: &'static AsciiSet) -> PercentEncode<'a> {
    PercentEncode { input, set }
}

// MARK: percent_encode_byte
const fn build_percent_table() -> [u8; 1024] {
    let hex = b"0123456789ABCDEF";
    let mut buf = [0u8; 1024];
    let mut i = 0usize;
    while i < 256 {
        buf[i * 4] = b'%';
        buf[i * 4 + 1] = hex[i >> 4];
        buf[i * 4 + 2] = hex[i & 0xF];
        i += 1;
    }
    buf
}
static PERCENT_TABLE: [u8; 1024] = build_percent_table();

/// Return the percent-encoding of the given byte as a `&'static str` of the form `%XX`.
pub fn percent_encode_byte(byte: u8) -> &'static str {
    let idx = (byte as usize) * 4;
    unsafe { str::from_utf8_unchecked(&PERCENT_TABLE[idx..idx + 3]) }
}

// MARK: percent_decode
/// Decode a percent-encoded byte string.
pub fn percent_decode(input: &[u8]) -> Cow<'_, [u8]> {
    if !input.contains(&b'%') {
        return Cow::Borrowed(input);
    }
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'%'
            && i + 2 < input.len()
            && let (Some(hi), Some(lo)) = (from_hex(input[i + 1]), from_hex(input[i + 2]))
        {
            out.push((hi << 4) | lo);
            i += 3;
            continue;
        }
        out.push(input[i]);
        i += 1;
    }
    Cow::Owned(out)
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_non_alphanumeric_plain() {
        assert_eq!(
            utf8_percent_encode("abc123", NON_ALPHANUMERIC).to_string(),
            "abc123"
        );
    }

    #[test]
    fn test_non_alphanumeric_spaces_and_special() {
        assert_eq!(
            utf8_percent_encode("hello world!", NON_ALPHANUMERIC).to_string(),
            "hello%20world%21"
        );
    }

    #[test]
    fn test_non_alphanumeric_query() {
        assert_eq!(
            utf8_percent_encode("foo=bar&baz=1", NON_ALPHANUMERIC).to_string(),
            "foo%3Dbar%26baz%3D1"
        );
    }

    #[test]
    fn test_non_alphanumeric_unicode() {
        assert_eq!(
            utf8_percent_encode("\u{00e9}", NON_ALPHANUMERIC).to_string(),
            "%C3%A9"
        );
    }

    #[test]
    fn test_percent_encode_byte() {
        assert_eq!(percent_encode_byte(b' '), "%20");
        assert_eq!(percent_encode_byte(b'A'), "%41");
        assert_eq!(percent_encode_byte(0xFF), "%FF");
    }

    #[test]
    fn test_percent_decode() {
        assert_eq!(percent_decode(b"hello%20world").as_ref(), b"hello world");
        assert_eq!(percent_decode(b"no encoding").as_ref(), b"no encoding");
        assert_eq!(percent_decode(b"%C3%A9").as_ref(), "\u{00e9}".as_bytes());
    }
}
