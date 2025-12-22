/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::io::Write;
use std::net::TcpListener;
use std::time::Duration;

use crate::request::Request;
use crate::response::Response;

/// Start HTTP server single threaded
pub fn serve_single_threaded(
    listener: TcpListener,
    handler: impl Fn(&Request) -> Response + 'static,
) {
    // Listen for incoming tcp clients
    for stream in listener.incoming() {
        let mut stream = stream.expect("Failed to accept connection");
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("Can't set read timeout");

        // Read incoming request
        let client_addr = stream
            .peer_addr()
            .expect("Can't get tcp stream client addr");
        match Request::read_from_stream(&mut stream, client_addr) {
            Ok(request) => {
                // Handle request and write response
                let mut response = handler(&request);
                response.write_to_stream(
                    &mut stream,
                    &request,
                    request.headers.get("Connection").is_some(),
                );

                // If the response has a takeover function, start thread and move tcp stream
                if let Some(takeover) = response.takeover.take() {
                    std::thread::spawn(move || takeover(stream));
                }
            }
            Err(err) => {
                // Invalid request received
                _ = write!(stream, "HTTP/1.0 400 Bad Request\r\n\r\n");
                println!("Error: Invalid http request: {err:?}");
            }
        }
    }
}

/// Start HTTP server
#[cfg(feature = "multi-threaded")]
pub fn serve(
    listener: TcpListener,
    handler: impl Fn(&Request) -> Response + Clone + Send + 'static,
) {
    // Create thread pool with workers
    let num_threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let pool = threadpool::ThreadPool::new(num_threads * 64);

    // Listen for incoming tcp clients
    for stream in listener.incoming() {
        let mut stream = stream.expect("Failed to accept connection");
        stream
            .set_read_timeout(Some(crate::KEEP_ALIVE_TIMEOUT))
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
                Err(err) => {
                    if err.kind() != std::io::ErrorKind::WouldBlock
                        && err.kind() != std::io::ErrorKind::TimedOut
                    {
                        println!("Error: {err:?}");
                    }
                    return;
                }
            }

            // Read incoming request
            let client_addr = stream
                .peer_addr()
                .expect("Can't get tcp stream client addr");
            match Request::read_from_stream(&mut stream, client_addr) {
                Ok(request) => {
                    // Handle request and write response
                    let mut response = handler(&request);
                    response.write_to_stream(&mut stream, &request, true);

                    // If the response has a takeover function, start thread and move tcp stream
                    if let Some(takeover) = response.takeover.take() {
                        std::thread::spawn(move || takeover(stream));
                        return;
                    }

                    // Close connection if HTTP/1.0 or Connection: close
                    if request.version == crate::enums::Version::Http1_0
                        || request.headers.get("Connection") == Some("close")
                    {
                        return;
                    }
                }
                Err(err) => {
                    // Invalid request received
                    _ = write!(stream, "HTTP/1.0 400 Bad Request\r\n\r\n");
                    println!("Error: Invalid http request: {err:?}");
                    return;
                }
            }
        });
    }
}

/// Serve CGI requests
#[cfg(feature = "cgi")]
pub fn serve_cgi(handler: impl Fn(&Request) -> Response) {
    let request = match Request::from_cgi_env() {
        Ok(req) => req,
        Err(_) => {
            println!("HTTP/1.0 400 Bad Request\r\n\r\n");
            _ = std::io::stdout().lock().flush();
            return;
        }
    };
    let response = handler(&request);
    let mut stdout = std::io::stdout().lock();
    response.write_to_cgi_stdout(&mut stdout);
    _ = stdout.flush();
}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::io::Read;
    use std::net::{Ipv4Addr, TcpStream};
    use std::thread;

    use super::*;
    use crate::enums::Status;

    #[test]
    fn test_serve_single_threaded() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("Failed to bind address");
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            serve_single_threaded(listener, |_req| Response::with_status(Status::Ok));
        });

        let mut stream = TcpStream::connect(addr).expect("Failed to connect to server");
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .expect("Failed to write to stream");

        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .expect("Failed to read from stream");
        assert!(response.starts_with(b"HTTP/1.1 200 OK"));
    }

    #[test]
    #[cfg(feature = "multi-threaded")]
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

    #[test]
    #[cfg(feature = "cgi")]
    fn test_parse_cgi_get() {
        use std::env;
        env::set_var("GATEWAY_INTERFACE", "CGI/1.1");
        env::set_var("REQUEST_METHOD", "GET");
        env::set_var("SERVER_PROTOCOL", "HTTP/1.1");
        env::set_var("PATH_INFO", "/test.txt");
        env::set_var("QUERY_STRING", "x=1&y=2");
        serve_cgi(|req| {
            assert_eq!(req.method.to_string(), "GET");
            assert_eq!(req.url.path(), "/test.txt");
            assert_eq!(req.url.query(), Some("x=1&y=2"));
            Response::with_status(Status::Ok)
        });
    }
}
