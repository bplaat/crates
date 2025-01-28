/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::net::TcpStream;

use crate::request::Request;
use crate::response::Response;

// MARK: Fetch
/// Fetch request with http client
pub fn fetch(req: Request) -> Result<Response, FetchError> {
    let authority = req.url.authority.as_ref().expect("Invalid request url");
    let mut stream = TcpStream::connect(format!(
        "{}:{}",
        authority.host,
        authority.port.unwrap_or(80)
    ))
    .map_err(|_| FetchError)?;
    req.write_to_stream(&mut stream);
    Response::read_from_stream(&mut stream).map_err(|_| FetchError)
}

// MARK: FetchError
#[derive(Debug)]
pub struct FetchError;

impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Fetch error")
    }
}

impl Error for FetchError {}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::io::Write;
    use std::net::{Ipv4Addr, TcpListener};
    use std::thread;

    use super::*;
    use crate::enums::Status;

    #[test]
    fn test_fetch_http1_0() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let server_addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .write_all(b"HTTP/1.0 200 OK\r\nContent-Length: 4\r\n\r\ntest")
                .unwrap();
        });

        let res = fetch(Request::with_url(format!("http://{}/", server_addr))).unwrap();
        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.body, "test".as_bytes());
    }

    #[test]
    fn test_fetch_http1_1() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let server_addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: closed\r\n\r\ntest",
                )
                .unwrap();
        });

        let res = fetch(Request::with_url(format!("http://{}/", server_addr))).unwrap();
        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.body, "test".as_bytes());
    }
}
