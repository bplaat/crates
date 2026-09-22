/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

#[allow(unused_imports)]
use sha2::Digest as _;
use sha2::Sha256;

pub use crate::utils::{
    DEFAULT_SAFE_ITERATIONS, MAX_PASSWORD_HASH_ITERATIONS, PasswordHashDecodeError, password_hash,
    password_hash_customized, password_verify,
};

mod utils;

/// PBKDF2-HMAC-SHA256 key derivation function
pub fn pbkdf2_hmac_sha256(password: &[u8], salt: &[u8], iterations: u32, dklen: usize) -> Vec<u8> {
    assert!(
        iterations > 0,
        "PBKDF2 iterations must be greater than zero"
    );
    assert!(
        dklen.div_ceil(32) <= u32::MAX as usize,
        "PBKDF2 derived key is too large"
    );

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

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut key_block = [0; 64];
    if key.len() > key_block.len() {
        key_block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut inner_pad = key_block;
    let mut outer_pad = key_block;
    for byte in &mut inner_pad {
        *byte ^= 0x36;
    }
    for byte in &mut outer_pad {
        *byte ^= 0x5c;
    }

    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(message);
    let inner = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner);
    outer.finalize().into()
}
