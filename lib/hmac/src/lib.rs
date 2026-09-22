/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [hmac](https://crates.io/crates/hmac) crate

use crypto_common::BlockSizeUser;
pub use crypto_common::KeyInit;
use digest::{Digest, FixedOutputReset, Output, Reset};
use subtle::ConstantTimeEq;

/// An error returned when an HMAC key has an invalid length.
#[derive(Debug)]
pub struct InvalidLength;

/// An error returned when an HMAC tag does not match.
#[derive(Debug)]
pub struct MacError;

/// A streaming HMAC computation.
pub struct Hmac<D> {
    key: Vec<u8>,
    message: Vec<u8>,
    digest: std::marker::PhantomData<D>,
}

impl<D> Hmac<D> {
    /// Creates an HMAC computation from a key of any length.
    pub fn new_from_slice(key: &[u8]) -> Result<Self, InvalidLength> {
        Ok(Self {
            key: key.to_vec(),
            message: Vec::new(),
            digest: std::marker::PhantomData,
        })
    }
}

/// Common operations for message authentication codes.
pub trait Mac: Sized {
    /// Adds message bytes to this computation.
    fn update(&mut self, data: &[u8]);

    /// Verifies a complete tag in constant time.
    fn verify_slice(self, tag: &[u8]) -> Result<(), MacError>;
}

impl<D> Mac for Hmac<D>
where
    D: Digest + BlockSizeUser + FixedOutputReset + Reset,
{
    fn update(&mut self, data: &[u8]) {
        self.message.extend_from_slice(data);
    }

    fn verify_slice(self, tag: &[u8]) -> Result<(), MacError> {
        verify::<D>(&self.key, &self.message, tag)
            .then_some(())
            .ok_or(MacError)
    }
}

// MARK: hmac
/// Computes HMAC over `message` with `key` and returns the raw digest bytes.
pub fn hmac<D>(key: &[u8], message: &[u8]) -> Output<D>
where
    D: Digest + BlockSizeUser + FixedOutputReset + Reset,
{
    let block_size = D::block_size();
    let mut key_block = vec![0u8; block_size];
    if key.len() > block_size {
        let hashed = D::digest(key);
        let hashed: &[u8] = hashed.as_ref();
        key_block[..hashed.len()].copy_from_slice(hashed);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut ikey = vec![0u8; block_size];
    let mut okey = vec![0u8; block_size];
    for (ik, &kb) in ikey.iter_mut().zip(key_block.iter()) {
        *ik = kb ^ 0x36;
    }
    for (ok, &kb) in okey.iter_mut().zip(key_block.iter()) {
        *ok = kb ^ 0x5c;
    }

    let mut h = D::new();
    Digest::update(&mut h, &ikey);
    Digest::update(&mut h, message);
    let inner = h.finalize_reset();

    Digest::update(&mut h, &okey);
    let inner: &[u8] = inner.as_ref();
    Digest::update(&mut h, inner);
    h.finalize_reset()
}

/// Verifies that `tag` is the HMAC of `message` under `key` in constant time.
pub fn verify<D>(key: &[u8], message: &[u8], tag: &[u8]) -> bool
where
    D: Digest + BlockSizeUser + FixedOutputReset + Reset,
{
    let actual = hmac::<D>(key, message);
    let actual: &[u8] = actual.as_ref();
    if actual.len() != tag.len() {
        return false;
    }
    actual.ct_eq(tag).into()
}

// MARK: Tests
#[cfg(test)]
mod test {
    use sha2::Sha256;

    use super::*;

    #[test]
    fn test_hmac_sha256_rfc4231_tc1() {
        // RFC 4231 Test Case 1
        let expected = [
            0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b,
            0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7, 0x26, 0xe9, 0x37, 0x6c,
            0x2e, 0x32, 0xcf, 0xf7,
        ];
        assert_eq!(
            hmac::<Sha256>(
                b"\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b",
                b"Hi There"
            ),
            expected
        );
    }

    #[test]
    fn test_verify_hmac_sha256() {
        let tag = hmac::<Sha256>(b"secret", b"payload");
        assert!(verify::<Sha256>(b"secret", b"payload", &tag));
        assert!(!verify::<Sha256>(b"secret", b"changed", &tag));
        assert!(!verify::<Sha256>(b"secret", b"payload", &tag[..31]));
    }
}
