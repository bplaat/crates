/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

pub(crate) fn password_hash(password: &str) -> String {
    let mut salt = [0u8; 16];
    getrandom::fill(&mut salt).expect("Can't get random bytes");
    pbkdf2::password_hash_customized(password, &salt, crate::consts::PBKDF2_ITERATIONS)
}
