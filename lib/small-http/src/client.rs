/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::header_map::HeaderMap;
use crate::request::{FetchError, Request};
use crate::response::Response;
use crate::KEEP_ALIVE_TIMEOUT;

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
        for (name, value) in self.headers.iter() {
            request = request.header(name, value);
        }

        // Get or create connection
        let addr = format!(
            "{}:{}",
            request.url.host().expect("No host in URL"),
            request.url.port().unwrap_or(80)
        );
        let mut stream = self
            .connection_pool
            .lock()
            .expect("Can't lock connection pool")
            .take_connection(&addr)
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
            .return_connection(&addr, stream);
        Ok(res)
    }
}

// MARK: ConnectionPool
#[derive(Default)]
struct ConnectionPool {
    connections: HashMap<String, Vec<TcpStream>>,
}

impl ConnectionPool {
    fn take_connection(&mut self, addr: &str) -> Option<TcpStream> {
        // Insert addr into connection pool if it doesn't exist
        if !self.connections.contains_key(addr) {
            self.connections.insert(addr.to_string(), Vec::new());
        }

        // Check if we have a connections for the addr
        if let Some(connections) = self.connections.get_mut(addr) {
            // Check if we have a connection available
            if let Some(conn) = connections.pop() {
                return Some(conn);
            }

            // Open connection and return it
            if let Ok(conn) = TcpStream::connect(addr) {
                return Some(conn);
            }
        }

        // No connection available
        None
    }

    fn return_connection(&mut self, addr: &str, conn: TcpStream) {
        // Insert connection back into pool
        if let Some(connections) = self.connections.get_mut(addr) {
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
