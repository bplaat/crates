/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple HTTPS GET example using native-tls directly

use std::io::{Read, Write};
use std::net::TcpStream;

use native_tls::TlsConnector;

fn main() {
    let stream = TcpStream::connect("example.com:443").expect("Can't connect");
    let connector = TlsConnector::new().expect("Can't create TLS connector");
    let mut tls = connector
        .connect("example.com", stream)
        .expect("TLS handshake failed");

    tls.write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n")
        .expect("Can't write request");

    let mut response = String::new();
    tls.read_to_string(&mut response)
        .expect("Can't read response");
    println!("{response}");
}
