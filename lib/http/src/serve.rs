/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::io::{self, Write};
use std::net::TcpListener;
use std::time::Duration;

use threadpool::ThreadPool;

use crate::request::Request;
use crate::response::Response;
use crate::version::Version;

const WORKER_THREADS: usize = 512;
pub(crate) const KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(5);

/// Start HTTP server
pub fn serve<F>(listener: TcpListener, handler: F)
where
    F: Fn(&Request) -> Response + Clone + Send + 'static,
{
    let pool = ThreadPool::new(WORKER_THREADS);
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
            match Request::read_from_stream(&mut stream) {
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
