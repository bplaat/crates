/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A PBKDF2-HMAC-SHA256 password hashing library

#![forbid(unsafe_code)]

use sha2::Sha256;

pub use crate::utils::{PasswordHashDecodeError, password_hash, password_verify};

mod utils;

/// PBKDF2-HMAC-SHA256 key derivation function
pub fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], iterations: u32, dklen: usize) -> Vec<u8> {
    fn f(password: &[u8], salt: &[u8], iterations: u32, block_index: u32) -> [u8; 32] {
        let mut u_input = Vec::with_capacity(salt.len() + 4);
        u_input.extend_from_slice(salt);
        u_input.extend_from_slice(&block_index.to_be_bytes());
        let mut u = hmac_sha256(password, &u_input);
        let mut t = u;
        for _ in 1..iterations {
            u = hmac_sha256(password, &u);
            for (ti, ui) in t.iter_mut().zip(u.iter()) {
                *ti ^= ui;
            }
        }
        t
    }

    let mut derived_key = Vec::with_capacity(dklen);
    for i in 1..=dklen.div_ceil(32) {
        derived_key.extend(f(password, salt, iterations, i as u32));
    }
    derived_key.truncate(dklen);
    derived_key
}

// HMAC-SHA256 implementation
fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut key_block = [0u8; 64];
    if key.len() > 64 {
        key_block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut ikey = [0u8; 64];
    let mut okey = [0u8; 64];
    for i in 0..64 {
        ikey[i] = key_block[i] ^ 0x36;
        okey[i] = key_block[i] ^ 0x5c;
    }

    let mut h = Sha256::new();
    h.update(ikey);
    h.update(data);
    let inner = h.finalize_reset();

    h.update(okey);
    h.update(inner);
    h.finalize_reset()
}
