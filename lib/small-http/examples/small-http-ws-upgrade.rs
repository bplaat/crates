/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple small-http server websocket upgrade example

use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener};

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use sha1::{Digest, Sha1};
use small_http::{Request, Response, Status};

fn handler(req: &Request) -> Response {
    let path = req.url.path();
    println!("{} {}", req.method, path);

    if path == "/ws" {
        // Send WebSocket upgrade response
        let mut res = Response::with_status(Status::SwitchingProtocols)
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade");
        if let Some(key) = req.headers.get("Sec-WebSocket-Key") {
            let mut hasher = Sha1::new();
            hasher.update(key.as_bytes());
            hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
            res = res.header(
                "Sec-WebSocket-Accept",
                BASE64_STANDARD.encode(hasher.finalize()),
            );
        }
        res = res.takeover(|mut stream| {
            println!(
                "Client connected: {}",
                stream.peer_addr().expect("Can't get client addr")
            );
            loop {
                let mut buf = [0; 1024];
                let n = stream.read(&mut buf).expect("Failed to read from stream");
                if n == 0 {
                    break;
                }

                // Parse incoming WebSocket frame
                let fin = (buf[0] & 0x80) != 0;
                let opcode = buf[0] & 0x0F;
                let masked = (buf[1] & 0x80) != 0;
                let payload_len = (buf[1] & 0x7F) as usize;
                println!(
                    "Recv frame: fin={}, opcode={}, masked={}, len={}",
                    fin, opcode, masked, payload_len
                );

                // Handle text frame
                if opcode == 0x1 {
                    // Read text frame
                    let mut payload = Vec::with_capacity(payload_len);
                    if masked {
                        let mask_key = &buf[2..6];
                        for i in 0..payload_len {
                            payload.push(buf[6 + i] ^ mask_key[i % 4]);
                        }
                    } else {
                        payload.extend_from_slice(&buf[2..2 + payload_len]);
                    }
                    let text = String::from_utf8_lossy(&payload);
                    println!("Recv frame text: {}", text);

                    // Echo back the text frame
                    let response_frame = [
                        0x81,              // FIN + Text frame
                        payload_len as u8, // Payload length
                    ];
                    stream
                        .write_all(&response_frame)
                        .expect("Failed to write to stream");
                    stream.write_all(&payload).expect("Failed to write payload");
                }
            }
            println!("Client disconnected");
        });
        return res;
    }

    Response::with_status(Status::NotFound)
        .header("Content-Type", "text/html")
        .body("<h1>404 Not Found</h1>")
}

fn main() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 8080))
        .unwrap_or_else(|_| panic!("Can't bind to port"));
    small_http::serve(listener, handler);
}
