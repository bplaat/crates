/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use small_websocket::{Message, WebSocket};
use tiny_webview::Webview;

use crate::dmx::{DMX_STATE, Mode};

// MARK: IpcMessage
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct State {
    pub color: u32,
    pub toggle_color: u32,
    pub toggle_speed: Option<u64>,
    pub strobe_speed: Option<u64>,
    pub mode: Mode,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(crate) enum IpcMessage {
    GetState,
    GetStateResponse { state: State },
    SetColor { color: u32 },
    SetToggleColor { color: u32 },
    SetToggleSpeed { speed: Option<u64> },
    SetStrobeSpeed { speed: Option<u64> },
    SetMode { mode: Mode },
}

// MARK: IpcConnection
pub(crate) static IPC_CONNECTIONS: Mutex<Vec<IpcConnection>> = Mutex::new(Vec::new());

pub(crate) enum IpcConnection {
    WebviewIpc(Arc<Mutex<Webview>>),
    WebSocket(WebSocket),
}

impl PartialEq for IpcConnection {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::WebviewIpc(a), Self::WebviewIpc(b)) => Arc::ptr_eq(a, b),
            (Self::WebSocket(a), Self::WebSocket(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for IpcConnection {}

impl IpcConnection {
    pub(crate) fn send(&mut self, message: String) {
        println!("[RUST] Sending IPC message: {}", message);
        match self {
            Self::WebviewIpc(webview) => webview
                .lock()
                .expect("Can't lock webview")
                .send_ipc_message(message),

            Self::WebSocket(ws) => ws
                .send(Message::Text(message))
                .expect("Failed to send IPC message"),
        }
    }

    pub(crate) fn broadcast(&mut self, message: String) {
        println!("[RUST] Broadcasting IPC message: {}", message);
        let mut connections = IPC_CONNECTIONS
            .lock()
            .expect("Failed to lock IPC connections");
        for connection in connections.iter_mut() {
            if connection != self {
                connection.send(message.clone());
            }
        }
    }
}

// MARK: IPC Message Handler
pub(crate) fn ipc_message_handler(mut connection: IpcConnection, message: &str) {
    let mut dmx_state = DMX_STATE.lock().expect("Failed to lock DMX state");
    let message = serde_json::from_str(message).expect("Failed to parse IPC message");
    println!("[RUST] Received IPC message: {:?}", message);
    match message {
        IpcMessage::GetState => {
            let state = State {
                color: dmx_state.color,
                toggle_color: dmx_state.toggle_color,
                toggle_speed: dmx_state.toggle_speed.map(|d| d.as_millis() as u64),
                strobe_speed: dmx_state.strobe_speed.map(|d| d.as_millis() as u64),
                mode: dmx_state.mode,
            };
            connection.send(
                serde_json::to_string(&IpcMessage::GetStateResponse { state })
                    .expect("Failed to serialize IPC response"),
            );
        }

        IpcMessage::SetColor { color } => {
            dmx_state.color = color;
            connection.broadcast(
                serde_json::to_string(&IpcMessage::SetColor { color })
                    .expect("Failed to serialize IPC message"),
            );
        }
        IpcMessage::SetToggleColor { color } => {
            dmx_state.toggle_color = color;
            connection.broadcast(
                serde_json::to_string(&IpcMessage::SetToggleColor { color })
                    .expect("Failed to serialize IPC message"),
            );
        }
        IpcMessage::SetToggleSpeed { speed } => {
            dmx_state.toggle_speed = speed.map(Duration::from_millis);
            dmx_state.is_toggle_color = speed.is_some();
            connection.broadcast(
                serde_json::to_string(&IpcMessage::SetToggleSpeed { speed })
                    .expect("Failed to serialize IPC message"),
            );
        }
        IpcMessage::SetStrobeSpeed { speed } => {
            dmx_state.strobe_speed = speed.map(Duration::from_millis);
            dmx_state.is_strobe = speed.is_some();
            connection.broadcast(
                serde_json::to_string(&IpcMessage::SetStrobeSpeed { speed })
                    .expect("Failed to serialize IPC message"),
            );
        }
        IpcMessage::SetMode { mode } => {
            dmx_state.mode = mode;
            connection.broadcast(
                serde_json::to_string(&IpcMessage::SetMode { mode })
                    .expect("Failed to serialize IPC message"),
            );
        }

        _ => {}
    }
}
