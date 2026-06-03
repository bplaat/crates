/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;

#[cfg(target_os = "macos")]
mod imp_macos;
#[cfg(target_os = "macos")]
use imp_macos as imp;

#[cfg(target_os = "windows")]
mod imp_windows;
#[cfg(target_os = "windows")]
use imp_windows as imp;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod imp_openssl;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use imp_openssl as imp;

// MARK: Error
/// TLS error
#[derive(Debug)]
pub struct Error(String);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

// MARK: HandshakeError
/// Error that can occur during the TLS handshake
#[derive(Debug)]
pub struct HandshakeError<S> {
    error: Error,
    _stream: PhantomData<S>,
}

impl<S> HandshakeError<S> {
    pub(crate) fn new(error: Error) -> Self {
        Self {
            error,
            _stream: PhantomData,
        }
    }

    /// Returns the underlying TLS error
    pub fn error(&self) -> &Error {
        &self.error
    }
}

impl<S> Display for HandshakeError<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl<S: fmt::Debug> std::error::Error for HandshakeError<S> {}

// MARK: TlsConnector
/// A TLS connector for creating TLS connections
pub use imp::TlsConnector;
// MARK: TlsStream
/// A TLS stream wrapping an underlying I/O stream
pub use imp::TlsStream;

// MARK: Tests
#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    use super::*;

    // Helper: open a TLS connection and issue a simple HTTP/1.0 GET request.
    // HTTP/1.0 causes the server to close the connection after the response,
    // so read_to_string terminates cleanly.
    fn https_get(host: &str, path: &str) -> String {
        let tcp = TcpStream::connect(format!("{host}:443")).expect("TCP connect failed");
        let connector = TlsConnector::new().expect("TlsConnector::new failed");
        let mut tls = connector.connect(host, tcp).expect("TLS handshake failed");
        write!(tls, "GET {path} HTTP/1.0\r\nHost: {host}\r\n\r\n").expect("write failed");
        let mut response = String::new();
        tls.read_to_string(&mut response).unwrap_or_default();
        response
    }

    #[test]
    fn test_handshake_succeeds() {
        let tcp = TcpStream::connect("example.com:443").expect("TCP connect failed");
        let connector = TlsConnector::new().expect("TlsConnector::new failed");
        // If this does not panic, the handshake succeeded
        connector
            .connect("example.com", tcp)
            .expect("TLS handshake failed");
    }

    #[test]
    fn test_https_get_returns_http_response() {
        let response = https_get("example.com", "/");
        assert!(
            response.starts_with("HTTP/"),
            "Expected HTTP response, got: {response}"
        );
    }

    #[test]
    fn test_https_get_response_contains_html() {
        let response = https_get("example.com", "/");
        let lower = response.to_lowercase();
        assert!(
            lower.contains("<html"),
            "Expected HTML in body, got: {response}"
        );
    }

    #[test]
    fn test_multiple_sequential_connections() {
        for i in 0..3 {
            let response = https_get("example.com", "/");
            assert!(
                response.starts_with("HTTP/"),
                "Connection {i}: expected HTTP response"
            );
        }
    }

    #[test]
    fn test_sni_hostname_validated() {
        // wrong.host.badssl.com serves a cert for *.badssl.com, not for
        // wrong.host.badssl.com (second-level wildcard doesn't match).
        // The TLS connector must reject this.
        let tcp = match TcpStream::connect("wrong.host.badssl.com:443") {
            Ok(s) => s,
            Err(_) => return, // skip if no network
        };
        let connector = TlsConnector::new().expect("TlsConnector::new failed");
        let result = connector.connect("wrong.host.badssl.com", tcp);
        assert!(
            result.is_err(),
            "Expected hostname validation failure for wrong.host.badssl.com"
        );
    }

    #[test]
    fn test_expired_cert_rejected() {
        let tcp = match TcpStream::connect("expired.badssl.com:443") {
            Ok(s) => s,
            Err(_) => return, // skip if no network
        };
        let connector = TlsConnector::new().expect("TlsConnector::new failed");
        let result = connector.connect("expired.badssl.com", tcp);
        assert!(
            result.is_err(),
            "Expected expired certificate to be rejected"
        );
    }

    #[test]
    fn test_self_signed_cert_rejected() {
        let tcp = match TcpStream::connect("self-signed.badssl.com:443") {
            Ok(s) => s,
            Err(_) => return, // skip if no network
        };
        let connector = TlsConnector::new().expect("TlsConnector::new failed");
        let result = connector.connect("self-signed.badssl.com", tcp);
        assert!(
            result.is_err(),
            "Expected self-signed certificate to be rejected"
        );
    }

    #[test]
    fn test_different_hosts() {
        // Verify connections to two different hosts both work
        let r1 = https_get("example.com", "/");
        let r2 = https_get("www.google.com", "/");
        assert!(r1.starts_with("HTTP/"), "example.com failed");
        assert!(r2.starts_with("HTTP/"), "google.com failed");
    }
}
