/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(feature = "tls")]
use native_tls::TlsStream;

use crate::header_map::HeaderMap;
use crate::request::{FetchError, Request};
use crate::response::Response;
use crate::KEEP_ALIVE_TIMEOUT;

// MARK: MaybeHttpsStream
pub(crate) enum MaybeHttpsStream {
    Plain(TcpStream),
    #[cfg(feature = "tls")]
    Tls(TlsStream<TcpStream>),
}

impl MaybeHttpsStream {
    pub(crate) fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        match self {
            Self::Plain(s) => s.set_read_timeout(dur),
            #[cfg(feature = "tls")]
            Self::Tls(s) => s.get_ref().set_read_timeout(dur),
        }
    }
}

impl Read for MaybeHttpsStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Plain(s) => s.read(buf),
            #[cfg(feature = "tls")]
            Self::Tls(s) => s.read(buf),
        }
    }
}

impl Write for MaybeHttpsStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Plain(s) => s.write(buf),
            #[cfg(feature = "tls")]
            Self::Tls(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Plain(s) => s.flush(),
            #[cfg(feature = "tls")]
            Self::Tls(s) => s.flush(),
        }
    }
}

// MARK: HTTP Client
/// HTTP client
#[derive(Default, Clone)]
pub struct Client {
    connection_pool: Arc<Mutex<ConnectionPool>>,
    headers: HeaderMap,
}

impl Client {
    /// Create a new HTTP client
    pub fn new() -> Self {
        Self::default()
    }

    /// Set header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Fetch a request
    pub fn fetch(&mut self, mut request: Request) -> Result<Response, FetchError> {
        // Add client headers to request
        for (name, value) in &self.headers {
            request = request.header(name, value);
        }

        // Build connection key and address
        let host = request.url.host().ok_or(FetchError)?.to_string();
        let is_https = request.url.scheme() == "https";
        let port = request
            .url
            .port()
            .unwrap_or(if is_https { 443 } else { 80 });
        let conn_key = format!("{}://{}:{}", request.url.scheme(), host, port);
        let tcp_addr = format!("{host}:{port}");

        // Get or create connection
        let mut stream = self
            .connection_pool
            .lock()
            .expect("Can't lock connection pool")
            .take_connection(&conn_key, &tcp_addr, is_https, &host)
            .ok_or(FetchError)?;
        stream
            .set_read_timeout(Some(KEEP_ALIVE_TIMEOUT))
            .map_err(|_| FetchError)?;

        // Send request and read response
        request.write_to_stream(&mut stream, true);
        let res = Response::read_from_stream(&mut stream).map_err(|_| FetchError)?;

        // Return connection
        self.connection_pool
            .lock()
            .expect("Can't lock connection pool")
            .return_connection(&conn_key, stream);
        Ok(res)
    }
}

// MARK: ConnectionPool
#[derive(Default)]
struct ConnectionPool {
    connections: HashMap<String, Vec<MaybeHttpsStream>>,
}

impl ConnectionPool {
    fn take_connection(
        &mut self,
        key: &str,
        tcp_addr: &str,
        is_https: bool,
        host: &str,
    ) -> Option<MaybeHttpsStream> {
        if !self.connections.contains_key(key) {
            self.connections.insert(key.to_string(), Vec::new());
        }

        if let Some(connections) = self.connections.get_mut(key) {
            if let Some(conn) = connections.pop() {
                return Some(conn);
            }

            let tcp = TcpStream::connect(tcp_addr).ok()?;

            #[cfg(feature = "tls")]
            if is_https {
                use native_tls::TlsConnector;
                let connector = TlsConnector::new().ok()?;
                let tls = connector.connect(host, tcp).ok()?;
                return Some(MaybeHttpsStream::Tls(tls));
            }
            // Suppress unused warnings when tls feature is disabled
            let _ = is_https;
            let _ = host;

            return Some(MaybeHttpsStream::Plain(tcp));
        }

        None
    }

    fn return_connection(&mut self, key: &str, conn: MaybeHttpsStream) {
        if let Some(connections) = self.connections.get_mut(key) {
            connections.push(conn);
        }
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::io::{Read, Write};
    use std::net::{Ipv4Addr, TcpListener};
    use std::thread;

    use super::*;

    #[test]
    fn test_client_multiple_requests() {
        // Start test server
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let server_addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            loop {
                let mut buf = [0; 512];
                _ = stream.read(&mut buf);
                stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: closed\r\n\r\ntest",
                    )
                    .unwrap();
            }
        });

        // Create client and fetch multiple requests
        let mut client = Client::new();
        for _ in 0..10 {
            client
                .fetch(Request::get(format!("http://{server_addr}/")))
                .unwrap();
        }
    }
}
