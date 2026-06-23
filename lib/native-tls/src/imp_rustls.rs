/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! rustls TLS implementation for the `vendored` feature.
//!
//! Uses rustls with the ring backend and webpki-roots for embedded CA certificates.
//! Fully self-contained: no system OpenSSL or C compilation required.

use std::io::{self, Read, Write};
use std::sync::Arc;

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore};

use crate::{Error, HandshakeError};

// MARK: TlsConnector
/// A TLS connector using rustls with embedded CA roots
pub struct TlsConnector {
    config: Arc<ClientConfig>,
}

impl TlsConnector {
    /// Create a new TLS connector
    pub fn new() -> Result<Self, Error> {
        let mut roots = RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        Ok(Self {
            config: Arc::new(config),
        })
    }

    /// Perform a TLS handshake over the given stream
    pub fn connect<S: Read + Write>(
        &self,
        domain: &str,
        stream: S,
    ) -> Result<TlsStream<S>, HandshakeError<S>> {
        self.connect_inner(domain, stream)
            .map_err(HandshakeError::new)
    }

    fn connect_inner<S: Read + Write>(
        &self,
        domain: &str,
        mut stream: S,
    ) -> Result<TlsStream<S>, Error> {
        let server_name = ServerName::try_from(domain.to_string())
            .map_err(|e| Error(format!("Invalid domain name: {e}")))?;
        let mut conn = ClientConnection::new(self.config.clone(), server_name)
            .map_err(|e| Error(e.to_string()))?;
        while conn.is_handshaking() {
            conn.complete_io(&mut stream)
                .map_err(|e| Error(e.to_string()))?;
        }
        Ok(TlsStream { conn, stream })
    }
}

impl Default for TlsConnector {
    fn default() -> Self {
        Self::new().expect("TlsConnector::new() failed")
    }
}

// MARK: TlsStream
/// A TLS stream backed by rustls
pub struct TlsStream<S> {
    conn: ClientConnection,
    stream: S,
}

impl<S> TlsStream<S> {
    /// Returns a reference to the underlying stream
    pub fn get_ref(&self) -> &S {
        &self.stream
    }

    /// Returns a mutable reference to the underlying stream
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.stream
    }
}

impl<S: Read + Write> Read for TlsStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        rustls::Stream::new(&mut self.conn, &mut self.stream).read(buf)
    }
}

impl<S: Read + Write> Write for TlsStream<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        rustls::Stream::new(&mut self.conn, &mut self.stream).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        rustls::Stream::new(&mut self.conn, &mut self.stream).flush()
    }
}
