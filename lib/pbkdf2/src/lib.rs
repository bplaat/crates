/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A PBKDF2-HMAC-SHA256 password hashing library

use hmac::hmac;
use sha2::Sha256;

pub use crate::utils::{
    DEFAULT_SAFE_ITERATIONS, PasswordHashDecodeError, password_hash, password_hash_customized,
    password_verify,
};

mod utils;

/// PBKDF2-HMAC-SHA256 key derivation function
pub fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], iterations: u32, dklen: usize) -> Vec<u8> {
    fn f(password: &[u8], salt: &[u8], iterations: u32, block_index: u32) -> [u8; 32] {
        let mut u_input = Vec::with_capacity(salt.len() + 4);
        u_input.extend_from_slice(salt);
        u_input.extend_from_slice(&block_index.to_be_bytes());
        let mut u = hmac::<Sha256>(password, &u_input);
        let mut t = u;
        for _ in 1..iterations {
            u = hmac::<Sha256>(password, &u);
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
