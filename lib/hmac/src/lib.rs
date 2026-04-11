/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [hmac](https://crates.io/crates/hmac) crate

use digest::Digest;

// MARK: hmac
/// Compute HMAC over `message` with `key` using hash function `D`.
///
/// Returns the raw MAC bytes as `D::Output`.
pub fn hmac<D: Digest>(key: &[u8], message: &[u8]) -> D::Output {
    let mut key_block = vec![0u8; D::BLOCK_SIZE];
    if key.len() > D::BLOCK_SIZE {
        let hashed = D::digest(key);
        key_block[..hashed.as_ref().len()].copy_from_slice(hashed.as_ref());
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut ikey = vec![0u8; D::BLOCK_SIZE];
    let mut okey = vec![0u8; D::BLOCK_SIZE];
    for (ik, &kb) in ikey.iter_mut().zip(key_block.iter()) {
        *ik = kb ^ 0x36;
    }
    for (ok, &kb) in okey.iter_mut().zip(key_block.iter()) {
        *ok = kb ^ 0x5c;
    }

    let mut h = D::default();
    h.update(&ikey);
    h.update(message);
    let inner = h.finalize_reset();

    h.update(&okey);
    h.update(inner.as_ref());
    h.finalize_reset()
}

// MARK: Tests
#[cfg(test)]
mod test {
    use sha2::Sha256;

    use super::*;

    #[test]
    fn test_hmac_sha256_rfc4231_tc1() {
        // RFC 4231 Test Case 1
        let key =
            b"\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b";
        let data = b"Hi There";
        let expected = [
            0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b,
            0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7, 0x26, 0xe9, 0x37, 0x6c,
            0x2e, 0x32, 0xcf, 0xf7,
        ];
        assert_eq!(hmac::<Sha256>(key, data), expected);
    }
}
