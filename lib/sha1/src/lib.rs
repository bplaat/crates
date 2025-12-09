/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [sha1](https://crates.io/crates/sha1) crate

#![forbid(unsafe_code)]

/// Digest
pub trait Digest {
    /// New instance of the digest
    fn new() -> Self;
    /// Update the digest with new data
    fn update(&mut self, data: impl AsRef<[u8]>);
    /// Finalize the digest and return the hash
    fn finalize(self) -> Vec<u8>;
}

/// A simple SHA1 implementation
pub struct Sha1 {
    buffer: Vec<u8>,
}

impl Sha1 {
    /// Compute the SHA1 digest of the given data
    pub fn digest(data: impl AsRef<[u8]>) -> Vec<u8> {
        let mut hasher = Sha1::new();
        hasher.update(data);
        hasher.finalize()
    }
}

impl Digest for Sha1 {
    fn new() -> Self {
        Sha1 { buffer: Vec::new() }
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        self.buffer.extend_from_slice(data.as_ref());
    }

    #[allow(unused_mut)]
    fn finalize(mut self) -> Vec<u8> {
        let hash = sha1(&self.buffer);
        hash.to_vec()
    }
}

fn sha1(input: &[u8]) -> [u8; 20] {
    let mut h0 = 0x67452301u32;
    let mut h1 = 0xEFCDAB89u32;
    let mut h2 = 0x98BADCFEu32;
    let mut h3 = 0x10325476u32;
    let mut h4 = 0xC3D2E1F0u32;

    // Pad the input
    let mut padded = Vec::with_capacity(input.len() + 9 + (64 - (input.len() + 9) % 64));
    padded.extend_from_slice(input);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    let bit_len = (input.len() as u64) * 8;
    padded.extend_from_slice(&bit_len.to_be_bytes());

    // Process each 512-bit chunk
    for chunk in padded.chunks(64) {
        let mut w = [0u32; 80];
        // Copy chunk into first 16 words
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        // Extend the 16 words into 80
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        // Main loop
        for (i, w_i) in w.iter_mut().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                60..=79 => (b ^ c ^ d, 0xCA62C1D6),
                _ => unreachable!(),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*w_i);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    // Produce the final hash
    let mut result = [0u8; 20];
    for (i, h) in [h0, h1, h2, h3, h4].iter().enumerate() {
        result[i * 4..i * 4 + 4].copy_from_slice(&h.to_be_bytes());
    }
    result
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    fn to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn test_sha1() {
        let test_cases = [
            ("", "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
            ("abc", "a9993e364706816aba3e25717850c26c9cd0d89d"),
            ("hello world", "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"),
        ];

        for (input, expected) in test_cases {
            let hash = Sha1::digest(input.as_bytes());
            assert_eq!(to_hex(&hash), expected, "Failed for input: {input}");
        }
    }
}
