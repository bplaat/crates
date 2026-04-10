/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple and small websocket library for the [small-http](lib/small-http) library

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use sha1::Sha1;
use small_http::{Request, Response, Status};

// Maximum allowed WebSocket frame payload in bytes (64 KiB)
const MAX_FRAME_PAYLOAD: usize = 64 * 1024;

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
        // Need at least 2 bytes for the frame header
        if buf.len() < 2 {
            return None;
        }

        // Parse WebSocket frame
        let opcode = buf[0] & 0x0F;
        let masked = (buf[1] & 0x80) != 0;
        let payload_len = (buf[1] & 0x7F) as usize;

        // Handle payload length
        let (payload_offset, payload_len) = match payload_len {
            126 => {
                if buf.len() < 4 {
                    return None;
                }
                let len = u16::from_be_bytes([buf[2], buf[3]]) as usize;
                (4, len)
            }
            127 => {
                if buf.len() < 10 {
                    return None;
                }
                let len = u64::from_be_bytes([
                    buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9],
                ]) as usize;
                (10, len)
            }
            len => (2, len),
        };

        // Reject frames larger than the maximum allowed payload
        if payload_len > MAX_FRAME_PAYLOAD {
            return None;
        }

        // Get masking key if present
        let (mask_offset, mask) = if masked {
            if buf.len() < payload_offset + 4 {
                return None;
            }
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

        // Bounds check before slicing the buffer
        if mask_offset + payload_len > buf.len() {
            return None;
        }

        // Unmask and collect payload
        let mut payload = buf[mask_offset..mask_offset + payload_len].to_vec();
        if masked {
            for (byte, &key) in payload.iter_mut().zip(mask.iter().cycle()) {
                *byte ^= key;
            }
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

/// Upgrade HTTP request to WebSocket connection.
/// Returns a 400 Bad Request response if the request does not conform to RFC 6455.
pub fn upgrade(request: &Request, handler: impl FnOnce(WebSocket) + Send + 'static) -> Response {
    // Validate required WebSocket upgrade headers (RFC 6455 Section 4.2.1)
    let upgrade_ok = request
        .headers
        .get("Upgrade")
        .is_some_and(|v| v.eq_ignore_ascii_case("websocket"));
    let connection_ok = request
        .headers
        .get("Connection")
        .is_some_and(|v| v.to_ascii_lowercase().contains("upgrade"));
    let version_ok = request
        .headers
        .get("Sec-WebSocket-Version")
        .is_some_and(|v| v == "13");
    let key = request.headers.get("Sec-WebSocket-Key");

    if !upgrade_ok || !connection_ok || !version_ok || key.is_none() {
        return Response::with_status(Status::BadRequest).body("400 Bad Request");
    }

    let mut res = Response::with_status(Status::SwitchingProtocols)
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade");
    let mut hasher = Sha1::new();
    hasher.update(key.expect("checked above").as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    res = res.header(
        "Sec-WebSocket-Accept",
        BASE64_STANDARD.encode(hasher.finalize()),
    );
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

    // Build a minimal unmasked WebSocket frame: FIN + opcode, then length, then payload
    fn make_frame(opcode: u8, payload: &[u8]) -> Vec<u8> {
        let mut frame = vec![0x80 | opcode];
        let len = payload.len();
        if len <= 125 {
            frame.push(len as u8);
        } else {
            frame.push(126);
            frame.extend_from_slice(&(len as u16).to_be_bytes());
        }
        frame.extend_from_slice(payload);
        frame
    }

    #[test]
    fn test_parse_text_frame() {
        let frame = make_frame(0x1, b"Hello");
        let msg = WebSocket::parse_message(&frame).unwrap();
        assert!(matches!(msg, Message::Text(t) if t == "Hello"));
    }

    #[test]
    fn test_parse_binary_frame() {
        let frame = make_frame(0x2, &[0xDE, 0xAD, 0xBE, 0xEF]);
        let msg = WebSocket::parse_message(&frame).unwrap();
        assert!(matches!(msg, Message::Binary(b) if b == [0xDE, 0xAD, 0xBE, 0xEF]));
    }

    #[test]
    fn test_parse_ping_pong_frames() {
        let ping = make_frame(0x9, b"ping-data");
        assert!(
            matches!(WebSocket::parse_message(&ping).unwrap(), Message::Ping(b) if b == b"ping-data")
        );

        let pong = make_frame(0xA, b"pong-data");
        assert!(
            matches!(WebSocket::parse_message(&pong).unwrap(), Message::Pong(b) if b == b"pong-data")
        );
    }

    #[test]
    fn test_parse_close_frame_with_code_and_reason() {
        // Close frame: 2-byte code (1000 = 0x03E8) + reason
        let mut payload = vec![0x03u8, 0xE8]; // 1000
        payload.extend_from_slice(b"bye");
        let frame = make_frame(0x8, &payload);
        match WebSocket::parse_message(&frame).unwrap() {
            Message::Close(code, reason) => {
                assert_eq!(code, Some(1000));
                assert_eq!(reason.as_deref(), Some("bye"));
            }
            _ => panic!("expected Close"),
        }
    }

    #[test]
    fn test_parse_close_frame_no_payload() {
        let frame = make_frame(0x8, &[]);
        match WebSocket::parse_message(&frame).unwrap() {
            Message::Close(code, reason) => {
                assert_eq!(code, None);
                assert_eq!(reason, None);
            }
            _ => panic!("expected Close"),
        }
    }

    #[test]
    fn test_parse_masked_frame() {
        // Client-masked text frame with key [0x37, 0xFA, 0x21, 0x3D] and payload "Hello"
        let mask = [0x37u8, 0xFA, 0x21, 0x3D];
        let plain = b"Hello";
        let masked: Vec<u8> = plain
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ mask[i % 4])
            .collect();

        let mut frame = vec![0x81u8, 0x80 | 5u8]; // FIN+text, MASKED+5
        frame.extend_from_slice(&mask);
        frame.extend_from_slice(&masked);

        let msg = WebSocket::parse_message(&frame).unwrap();
        assert!(matches!(msg, Message::Text(t) if t == "Hello"));
    }

    #[test]
    fn test_parse_medium_length_frame() {
        // 200-byte binary payload uses 2-byte extended length (126 marker)
        let payload = vec![0xABu8; 200];
        let frame = make_frame(0x2, &payload);
        assert_eq!(frame[1], 126); // extended length marker
        assert_eq!(u16::from_be_bytes([frame[2], frame[3]]), 200);
        let msg = WebSocket::parse_message(&frame).unwrap();
        assert!(matches!(msg, Message::Binary(b) if b.len() == 200));
    }

    #[test]
    fn test_parse_unknown_opcode_returns_none() {
        let frame = make_frame(0x3, b"data"); // 0x3 is reserved/unknown
        assert!(WebSocket::parse_message(&frame).is_none());
    }

    #[test]
    fn test_parse_truncated_frame_returns_none() {
        // Frame header says 10 bytes but buffer has only 3
        let frame = vec![0x82u8, 10u8, 0x01]; // binary, 10 bytes, only 1 provided
        assert!(WebSocket::parse_message(&frame).is_none());
    }

    #[test]
    fn test_upgrade_accept_key() {
        // RFC 6455 Section 1.3 example: known input/output pair
        let req = Request::new()
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==");
        let res = upgrade(&req, |_ws| {});
        assert_eq!(res.status, Status::SwitchingProtocols);
        assert_eq!(
            res.headers.get("Sec-WebSocket-Accept").unwrap(),
            "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
        );
    }

    #[test]
    fn test_upgrade_missing_headers_returns_bad_request() {
        let req = Request::new(); // no WebSocket headers
        let res = upgrade(&req, |_ws| {});
        assert_eq!(res.status, Status::BadRequest);
    }
}
