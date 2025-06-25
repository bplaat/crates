/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple and small websocket library for the [small-http](lib/small-http) library

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use sha1::{Digest, Sha1};
use small_http::{Request, Response, Status};

/// WebSocket message
#[derive(Debug, Clone)]
pub enum Message {
    /// Text message
    Text(String),
    /// Binary message
    Binary(Vec<u8>),
    /// Ping message
    Ping(Vec<u8>),
    /// Pong message
    Pong(Vec<u8>),
    /// Close message
    Close(Option<u16>, Option<String>),
}

/// WebSocket connection
#[derive(Clone)]
pub struct WebSocket {
    stream: Arc<Mutex<TcpStream>>,
}

impl PartialEq for WebSocket {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.stream, &other.stream)
    }
}
impl Eq for WebSocket {}

impl WebSocket {
    fn new(stream: TcpStream) -> Self {
        WebSocket {
            stream: Arc::new(Mutex::new(stream)),
        }
    }

    /// Connect to a WebSocket server
    #[cfg(feature = "client")]
    pub fn connect(url: impl AsRef<str>) -> Result<Self, ConnectError> {
        let parsed_url = url::Url::parse(url.as_ref()).map_err(|_| ConnectError)?;
        let mut stream = TcpStream::connect(format!(
            "{}:{}",
            parsed_url.host().expect("URL should have a host"),
            parsed_url.port().unwrap_or(80)
        ))
        .map_err(|_| ConnectError)?;

        let mut random_key = [0u8; 16];
        getrandom::fill(&mut random_key).expect("Can't generate random key");
        let random_key = BASE64_STANDARD.encode(random_key);
        let req = Request::get(url.as_ref())
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", &random_key);
        req.write_to_stream(&mut stream, false);

        let res = Response::read_from_stream(&mut stream).map_err(|_| ConnectError)?;
        if res.status != Status::SwitchingProtocols {
            return Err(ConnectError);
        }
        let websocket_accept = res
            .headers
            .get("Sec-WebSocket-Accept")
            .ok_or(ConnectError)?;
        let mut sha1 = Sha1::new();
        sha1.update(random_key.as_bytes());
        sha1.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
        let expected_accept = BASE64_STANDARD.encode(sha1.finalize());
        if *websocket_accept != expected_accept {
            eprintln!(
                "WebSocket connection failed: expected Sec-WebSocket-Accept header to be '{}', got '{}'",
                expected_accept, websocket_accept
            );
            return Err(ConnectError);
        }

        Ok(WebSocket::new(stream))
    }

    /// Get the underlying TCP stream peer address
    pub fn peer_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.stream.lock().expect("Can't get lock").peer_addr()
    }

    /// Receive WebSocket message
    pub fn recv(&mut self) -> std::io::Result<Message> {
        let mut stream = self.stream.lock().expect("Can't get lock");
        let mut buf = [0; 1024];
        match stream.read(&mut buf) {
            Ok(0) => Ok(Message::Close(None, Some("Connection closed".to_string()))),
            Ok(_) => Self::parse_message(&buf).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid WebSocket frame")
            }),
            Err(e) => Err(e),
        }
    }

    /// Receive WebSocket message without blocking
    pub fn recv_non_blocking(&mut self) -> std::io::Result<Option<Message>> {
        let mut stream = self.stream.lock().expect("Can't get lock");
        stream.set_nonblocking(true)?;
        let mut buf = [0; 1024];
        match stream.read(&mut buf) {
            Ok(0) => Ok(Some(Message::Close(
                None,
                Some("Connection closed".to_string()),
            ))),
            Ok(_) => Self::parse_message(&buf).map(Some).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid WebSocket frame")
            }),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn parse_message(buf: &[u8]) -> Option<Message> {
        // Parse WebSocket frame
        let opcode = buf[0] & 0x0F;
        let masked = (buf[1] & 0x80) != 0;
        let payload_len = (buf[1] & 0x7F) as usize;

        // Handle payload length
        let (payload_offset, payload_len) = match payload_len {
            126 => {
                let len = u16::from_be_bytes([buf[2], buf[3]]) as usize;
                (4, len)
            }
            127 => {
                let len = u64::from_be_bytes([
                    buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9],
                ]) as usize;
                (10, len)
            }
            len => (2, len),
        };

        // Get masking key if present
        let (mask_offset, mask) = if masked {
            (
                payload_offset + 4,
                [
                    buf[payload_offset],
                    buf[payload_offset + 1],
                    buf[payload_offset + 2],
                    buf[payload_offset + 3],
                ],
            )
        } else {
            (payload_offset, [0; 4])
        };

        // Unmask and collect payload
        let mut payload = Vec::with_capacity(payload_len);
        for i in 0..payload_len {
            let byte = buf[mask_offset + i];
            payload.push(if masked { byte ^ mask[i % 4] } else { byte });
        }

        // Return appropriate message type
        match opcode {
            0x1 => Some(Message::Text(
                String::from_utf8_lossy(&payload).into_owned(),
            )),
            0x2 => Some(Message::Binary(payload)),
            0x8 => {
                let code = if payload.len() >= 2 {
                    Some(u16::from_be_bytes([payload[0], payload[1]]))
                } else {
                    None
                };
                let reason = if payload.len() > 2 {
                    Some(String::from_utf8_lossy(&payload[2..]).into_owned())
                } else {
                    None
                };
                Some(Message::Close(code, reason))
            }
            0x9 => Some(Message::Ping(payload)),
            0xA => Some(Message::Pong(payload)),
            _ => None,
        }
    }

    /// Write a WebSocket message
    pub fn send(&mut self, message: Message) -> std::io::Result<()> {
        let mut frame = Vec::new();
        match message {
            Message::Text(text) => {
                frame.push(0x81); // Text frame
                let payload = text.into_bytes();
                self.write_frame(&mut frame, &payload)?;
            }
            Message::Binary(data) => {
                frame.push(0x82); // Binary frame
                self.write_frame(&mut frame, &data)?;
            }
            Message::Ping(data) => {
                frame.push(0x89); // Ping frame
                self.write_frame(&mut frame, &data)?;
            }
            Message::Pong(data) => {
                frame.push(0x8A); // Pong frame
                self.write_frame(&mut frame, &data)?;
            }
            Message::Close(code, reason) => {
                frame.push(0x88); // Close frame
                let mut payload = Vec::new();
                if let Some(c) = code {
                    payload.extend_from_slice(&c.to_be_bytes());
                }
                if let Some(r) = reason {
                    payload.extend_from_slice(r.as_bytes());
                }
                self.write_frame(&mut frame, &payload)?;
            }
        }
        self.stream
            .lock()
            .expect("Can't get lock")
            .write_all(&frame)
    }

    fn write_frame(&self, frame: &mut Vec<u8>, payload: &[u8]) -> std::io::Result<()> {
        let payload_len = payload.len();
        if payload_len <= 125 {
            frame.push(payload_len as u8);
        } else if payload_len <= 65535 {
            frame.push(126);
            frame.extend_from_slice(&(payload_len as u16).to_be_bytes());
        } else {
            frame.push(127);
            frame.extend_from_slice(&(payload_len as u64).to_be_bytes());
        }
        frame.extend_from_slice(payload);
        Ok(())
    }
}

/// ConnectError
#[derive(Debug)]
pub struct ConnectError;

impl Display for ConnectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Connect error")
    }
}

impl Error for ConnectError {}

/// Upgrade HTTP request to WebSocket connection
pub fn upgrade(request: &Request, handler: impl FnOnce(WebSocket) + Send + 'static) -> Response {
    let mut res = Response::with_status(Status::SwitchingProtocols)
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade");
    if let Some(key) = request.headers.get("Sec-WebSocket-Key") {
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
        res = res.header(
            "Sec-WebSocket-Accept",
            BASE64_STANDARD.encode(hasher.finalize()),
        );
    }
    res = res.takeover(|stream| handler(WebSocket::new(stream)));
    res
}

// MARK: Tests
#[cfg(test)]
mod test {
    use std::net::{Ipv4Addr, TcpListener};

    use super::*;

    #[test]
    fn test_websocket_server_client() {
        // Create WebSocket server
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            small_http::serve(listener, |req| {
                upgrade(req, |mut ws| {
                    loop {
                        if let Message::Text(text) = ws.recv().expect("Failed to receive message") {
                            ws.send(Message::Text(text)).unwrap();
                        }
                    }
                })
            });
        });

        // Connect WebSocket client
        let mut ws = WebSocket::connect(format!("ws://{}:{}/", addr.ip(), addr.port())).unwrap();
        ws.send(Message::Text("Hello".to_string())).unwrap();
        if let Message::Text(text) = ws.recv().unwrap() {
            assert_eq!(text, "Hello")
        }
    }
}
