/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::io::{self, Write};
use std::net::TcpListener;
use std::thread;

use threadpool::ThreadPool;

use crate::enums::Version;
use crate::request::Request;
use crate::response::Response;
use crate::KEEP_ALIVE_TIMEOUT;

const WORK_THREAD_PER_CORE: usize = 64;

/// Start HTTP server
pub fn serve<F>(listener: TcpListener, handler: F)
where
    F: Fn(&Request) -> Response + Clone + Send + 'static,
{
    // Create thread pool with workers
    let num_cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let pool = ThreadPool::new(num_cores * WORK_THREAD_PER_CORE);

    // Listen for incoming tcp clients
    for mut stream in listener.incoming().flatten() {
        stream
            .set_read_timeout(Some(KEEP_ALIVE_TIMEOUT))
            .expect("Can't set read timeout");

        let handler = handler.clone();
        pool.execute(move || loop {
            // Wait for data to be available
            let mut buffer = [0; 1];
            match stream.peek(&mut buffer) {
                Ok(0) => {
                    return;
                }
                Ok(_) => {} // Data available continue
                Err(e) => {
                    if e.kind() != io::ErrorKind::WouldBlock && e.kind() != io::ErrorKind::TimedOut
                    {
                        println!("Error: {:?}", e);
                    }
                    return;
                }
            }

            // Read incoming request
            let client_addr = stream
                .peer_addr()
                .expect("Can't get tcp stream client addr");
            match Request::read_from_stream(&mut stream, client_addr) {
                Ok(req) => {
                    // Handle request
                    handler(&req).write_to_stream(&mut stream, &req);

                    // Close connection if HTTP/1.0 or Connection: close
                    if req.version == Version::Http1_0
                        || req.headers.get("Connection").map(|v| v.as_str()) == Some("close")
                    {
                        return;
                    }
                }
                Err(err) => {
                    // Invalid request received
                    _ = write!(stream, "HTTP/1.0 400 Bad Request\r\n\r\n");
                    println!("Error: Invalid http request: {:?}", err);
                    return;
                }
            }
        });
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::net::{Ipv4Addr, TcpStream};
    use std::thread;

    use io::Read;

    use super::*;
    use crate::enums::Status;

    #[test]
    fn test_serve() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("Failed to bind address");
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            serve(listener, |_req| Response::with_status(Status::Ok));
        });

        for _ in 0..10 {
            let mut stream = TcpStream::connect(addr).expect("Failed to connect to server");
            stream
                .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .expect("Failed to write to stream");

            let mut response = Vec::new();
            stream
                .read_to_end(&mut response)
                .expect("Failed to read from stream");
            assert!(response.starts_with(b"HTTP/1.1 200 OK"));
        }
    }
}
