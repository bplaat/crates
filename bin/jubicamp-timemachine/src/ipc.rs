/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use small_websocket::{Message, WebSocket};
use tiny_webview::EventLoopProxy;

// MARK: IpcMessage
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(crate) enum IpcMessage {
    ShaderRandom,
    ShaderSetTimeout { timeout: u64 },
}

// MARK: IpcConnection
pub(crate) static IPC_CONNECTIONS: Mutex<Vec<IpcConnection>> = Mutex::new(Vec::new());

pub(crate) enum IpcConnection {
    WebviewIpc(Arc<EventLoopProxy>),
    WebSocket(WebSocket),
}

impl PartialEq for IpcConnection {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::WebviewIpc(_), Self::WebviewIpc(_)) => true,
            (Self::WebSocket(ws1), Self::WebSocket(ws2)) => ws1 == ws2,
            _ => false,
        }
    }
}
impl Eq for IpcConnection {}

impl IpcConnection {
    pub(crate) fn send(&mut self, message: String) {
        println!("[RUST] Sending IPC message: {}", message);
        match self {
            Self::WebviewIpc(event_loop_proxy) => event_loop_proxy.send_user_event(message),
            Self::WebSocket(ws) => ws
                .send(Message::Text(message))
                .expect("Failed to send IPC message"),
        }
    }

    pub(crate) fn broadcast(&mut self, message: String) {
        let mut connections = IPC_CONNECTIONS
            .lock()
            .expect("Failed to lock IPC connections");
        if connections.len() > 1 {
            println!("[RUST] Broadcasting IPC message: {}", message);
            for connection in connections.iter_mut() {
                if connection != self {
                    connection.send(message.clone());
                }
            }
        }
    }
}

// MARK: IPC Message Handler
pub(crate) fn ipc_message_handler(mut connection: IpcConnection, message: &str) {
    let message: IpcMessage = serde_json::from_str(message).expect("Failed to parse IPC message");
    println!("[RUST] Received IPC message: {:?}", message);
    connection.broadcast(serde_json::to_string(&message).expect("Failed to serialize IPC message"));
}
