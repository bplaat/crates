/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#[cfg(feature = "multi-threaded")]
use std::io::BufRead;
use std::io::{BufReader, Write};
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
        let stream = stream.expect("Failed to accept connection");
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("Can't set read timeout");

        // Read incoming request
        let mut reader = BufReader::new(stream);
        let client_addr = reader
            .get_ref()
            .peer_addr()
            .expect("Can't get tcp stream client addr");
        match Request::read_from_reader(&mut reader, client_addr) {
            Ok(request) => {
                // Handle request and write response
                let mut response = handler(&request);
                response.write_to_stream(reader.get_mut(), &request, false);

                // If the response has a takeover function, start thread and move tcp stream
                if let Some(takeover) = response.takeover.take() {
                    std::thread::spawn(move || takeover(reader));
                }
            }
            Err(err) => {
                // Invalid request received
                _ = write!(reader.get_mut(), "HTTP/1.0 400 Bad Request\r\n\r\n");
                cfg_select! {
                    feature = "log" => log::error!("Invalid http request: {err:?}"),
                    _ => eprintln!("[small-http] Error invalid http request: {err:?}"),
                }
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
    // FIXME: The current thread pool doesn't spawn extra threads so http server could be overwhelmed with long running requests.
    let num_threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let pool = threadpool::ThreadPool::new(num_threads * 8);

    // Listen for incoming tcp clients
    for stream in listener.incoming() {
        let stream = stream.expect("Failed to accept connection");
        stream
            .set_read_timeout(Some(crate::KEEP_ALIVE_TIMEOUT))
            .expect("Can't set read timeout");

        let handler = handler.clone();
        pool.execute(move || {
            let mut reader = BufReader::new(stream);
            loop {
                // Wait for data to be available
                match reader.fill_buf() {
                    Ok([]) => {
                        return;
                    }
                    Ok(_) => {} // Data available continue
                    Err(err) => {
                        if err.kind() != std::io::ErrorKind::WouldBlock
                            && err.kind() != std::io::ErrorKind::TimedOut
                        {
                            cfg_select! {
                                feature = "log" => log::error!("Peeking tcp stream: {err:?}"),
                                _ => eprintln!("[small-http] Error peeking tcp stream: {err:?}"),
                            }
                        }
                        return;
                    }
                }

                // Read incoming request
                let client_addr = reader
                    .get_ref()
                    .peer_addr()
                    .expect("Can't get tcp stream client addr");
                match Request::read_from_reader(&mut reader, client_addr) {
                    Ok(request) => {
                        // Handle request and write response
                        let mut response = handler(&request);
                        response.write_to_stream(reader.get_mut(), &request, true);

                        // If the response has a takeover function, start thread and move tcp stream
                        if let Some(takeover) = response.takeover.take() {
                            std::thread::spawn(move || takeover(reader));
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
                        _ = write!(reader.get_mut(), "HTTP/1.0 400 Bad Request\r\n\r\n");
                        cfg_select! {
                            feature = "log" => log::error!("Invalid http request: {err:?}"),
                            _ => eprintln!("[small-http] Error invalid http request: {err:?}"),
                        }
                        return;
                    }
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
    use std::sync::mpsc;
    use std::thread;

    use super::*;
    use crate::enums::Status;
    use crate::request::Request;

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
    fn test_takeover_preserves_buffered_bytes() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("Failed to bind address");
        let addr = listener.local_addr().unwrap();
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            serve_single_threaded(listener, move |_req| {
                let sender = sender.clone();
                Response::with_status(Status::SwitchingProtocols).takeover(move |mut reader| {
                    let mut protocol_data = [0; 5];
                    reader.read_exact(&mut protocol_data).unwrap();
                    sender.send(protocol_data).unwrap();
                })
            });
        });

        let mut stream = TcpStream::connect(addr).expect("Failed to connect to server");
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\nhello")
            .expect("Failed to write to stream");

        assert_eq!(
            receiver.recv_timeout(Duration::from_secs(1)).unwrap(),
            *b"hello"
        );
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
    #[cfg(feature = "multi-threaded")]
    fn test_serve_pipelined_requests() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("Failed to bind address");
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            serve(listener, |request| {
                Response::with_status(Status::Ok).body(request.url.path())
            });
        });

        let mut stream = TcpStream::connect(addr).expect("Failed to connect to server");
        stream
            .write_all(
                b"GET /first HTTP/1.1\r\nHost: localhost\r\n\r\nGET /second HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
            )
            .expect("Failed to write pipelined requests");
        let mut reader = BufReader::new(stream);

        let first = Response::read_from_buffered_stream(&mut reader).unwrap();
        assert_eq!(first.body, b"/first");
        let second = Response::read_from_buffered_stream(&mut reader).unwrap();
        assert_eq!(second.body, b"/second");
    }

    #[test]
    #[cfg(feature = "cgi")]
    #[allow(unsafe_code)]
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

    #[test]
    fn test_various_methods() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            serve_single_threaded(listener, |req| {
                Response::with_status(Status::Ok).header("X-Method", req.method.to_string())
            });
        });

        for (req, expected_method) in [
            (Request::get(format!("http://{addr}/")), "GET"),
            (Request::post(format!("http://{addr}/")), "POST"),
            (Request::put(format!("http://{addr}/")), "PUT"),
            (Request::delete(format!("http://{addr}/")), "DELETE"),
            (Request::patch(format!("http://{addr}/")), "PATCH"),
        ] {
            let res = req.fetch().unwrap();
            assert_eq!(res.headers.get("X-Method").unwrap(), expected_method);
        }
    }

    #[test]
    fn test_various_status_codes() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            serve_single_threaded(listener, |req| match req.url.path() {
                "/created" => Response::with_status(Status::Created),
                "/no-content" => Response::with_status(Status::NoContent),
                "/bad-request" => Response::with_status(Status::BadRequest),
                "/not-found" => Response::with_status(Status::NotFound),
                "/internal-server-error" => Response::with_status(Status::InternalServerError),
                _ => Response::with_status(Status::Ok),
            });
        });

        for (path, expected_status) in [
            ("/", Status::Ok),
            ("/created", Status::Created),
            ("/no-content", Status::NoContent),
            ("/bad-request", Status::BadRequest),
            ("/not-found", Status::NotFound),
            ("/internal-server-error", Status::InternalServerError),
        ] {
            let res = Request::get(format!("http://{addr}{path}"))
                .fetch()
                .unwrap();
            assert_eq!(res.status, expected_status);
        }
    }
}
