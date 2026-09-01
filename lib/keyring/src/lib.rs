/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

use std::fmt::{self, Display, Formatter};

cfg_select! {
    target_os = "macos" => {
        mod macos;
        use macos as imp;
    }
    windows => {
        mod windows;
        use windows as imp;
    }
    any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ) => {
        mod libsecret;
        use libsecret as imp;
    }
    _ => {
        compile_error!("Unsupported platform");
    }
}

/// An error returned by the system credential store.
#[derive(Debug)]
pub enum Error {
    /// The requested credential does not exist.
    NoEntry,
    /// A service or account contains an embedded NUL byte.
    InvalidInput,
    /// The platform credential store failed.
    Platform(String),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoEntry => formatter.write_str("credential not found"),
            Self::InvalidInput => {
                formatter.write_str("service and account must not contain NUL bytes")
            }
            Self::Platform(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for Error {}

/// A result returned by the system credential store.
pub type Result<T> = std::result::Result<T, Error>;

/// A password identified by a service and account name.
#[derive(Debug, Clone)]
pub struct Entry {
    service: String,
    account: String,
}

impl Entry {
    /// Creates an entry in the platform credential store.
    pub fn new(service: &str, account: &str) -> Result<Self> {
        if service.contains('\0') || account.contains('\0') {
            return Err(Error::InvalidInput);
        }
        Ok(Self {
            service: service.to_string(),
            account: account.to_string(),
        })
    }

    /// Stores or replaces this entry's password.
    pub fn set_password(&self, password: &str) -> Result<()> {
        imp::set_password(&self.service, &self.account, password)
    }

    /// Loads this entry's password.
    pub fn get_password(&self) -> Result<String> {
        imp::get_password(&self.service, &self.account)
    }

    /// Deletes this entry's password.
    pub fn delete_credential(&self) -> Result<()> {
        imp::delete_credential(&self.service, &self.account)
    }
}

#[cfg(test)]
mod tests {
    use zeroize::Zeroizing;

    use super::*;

    #[test]
    fn rejects_embedded_nul() {
        assert!(matches!(
            Entry::new("service\0suffix", "account"),
            Err(Error::InvalidInput)
        ));
        assert!(matches!(
            Entry::new("service", "account\0suffix"),
            Err(Error::InvalidInput)
        ));
    }

    #[test]
    #[ignore = "uses the user's native credential store"]
    fn native_store_round_trip() {
        let account = format!("test-{}", std::process::id());
        let entry = Entry::new("nl.bplaat.keyring.tests", &account).expect("entry creation failed");
        entry.set_password("test password").expect("store failed");
        let password = Zeroizing::new(entry.get_password().expect("load failed"));
        entry.set_password("").expect("store empty failed");
        let empty_password = Zeroizing::new(entry.get_password().expect("load empty failed"));
        entry.delete_credential().expect("delete failed");
        assert_eq!(password.as_str(), "test password");
        assert!(empty_password.is_empty());
    }
}
