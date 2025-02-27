/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A unsecure PBKDF2-HMAC-SHA256 password hashing library

#![forbid(unsafe_code)]

pub use crate::sha256::Sha256;
pub use crate::utils::{PasswordHashDecodeError, password_hash, password_verify};

mod sha256;
mod utils;

/// PBKDF2-HMAC-SHA256 key derivation function
pub fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], iterations: u32, dklen: usize) -> Vec<u8> {
    fn f(password: &[u8], salt: &[u8], iterations: u32, block_index: u32) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(password);
        hasher.update(salt);
        hasher.update(&block_index.to_be_bytes());
        let mut u = hasher.finalize_reset().to_vec();
        let mut t = u.clone();
        for _ in 1..iterations {
            hasher.update(&u);
            u = hasher.finalize_reset().to_vec();
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
