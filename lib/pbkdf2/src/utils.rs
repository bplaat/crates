/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::pbkdf2_hmac_sha256;

/// Hash password using PBKDF2-HMAC-SHA256 returns string in PHC standard (https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md)
pub fn password_hash(password: &str) -> String {
    const ITERATIONS: u32 = 100_000;
    let mut salt = [0u8; 16];
    getrandom::getrandom(&mut salt).expect("Can't get random bytes");
    let hashed_password = pbkdf2_hmac_sha256(password.as_bytes(), &salt, ITERATIONS, 32);
    format!(
        "$pbkdf2-sha256$t={}${}${}",
        ITERATIONS,
        base64::encode(&salt, true),
        base64::encode(&hashed_password, true)
    )
}

/// Verify password using hash string in PHC standard
pub fn password_verify(password: &str, hash: &str) -> Result<bool, PasswordHashDecodeError> {
    let parts = hash.split('$').collect::<Vec<&str>>();
    let iterations = parts[2]
        .split('=')
        .nth(1)
        .ok_or(PasswordHashDecodeError)?
        .parse::<u32>()
        .map_err(|_| PasswordHashDecodeError)?;
    let salt = base64::decode(parts[3]).map_err(|_| PasswordHashDecodeError)?;
    let stored_hash = base64::decode(parts[4]).map_err(|_| PasswordHashDecodeError)?;
    let computed_hash = pbkdf2_hmac_sha256(password.as_bytes(), &salt, iterations, 32);
    Ok(stored_hash == computed_hash)
}

// MARK: PasswordHashDecodeError
/// Password hash decode error
#[derive(Debug)]
pub struct PasswordHashDecodeError;

impl Display for PasswordHashDecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Password hash decode error")
    }
}

impl Error for PasswordHashDecodeError {}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let hashed = password_hash("my_secure_password");
        assert!(password_verify("my_secure_password", &hashed).unwrap());
    }

    #[test]
    fn test_verify_incorrect_password() {
        let hashed = password_hash("my_secure_password");
        assert!(!password_verify("wrong_password", &hashed).unwrap());
    }

    #[test]
    fn test_hash_is_different_for_same_password() {
        let hashed1 = password_hash("my_secure_password");
        let hashed2 = password_hash("my_secure_password");
        assert_ne!(hashed1, hashed2);
    }

    #[test]
    fn test_verify_password_with_invalid_parts() {
        let invalid_hash = "$pbkdf2-sha256$t=100000$invalid*salt$inval&idhash";
        assert!(password_verify("password", invalid_hash).is_err());
    }
}
