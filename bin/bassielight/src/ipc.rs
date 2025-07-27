/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use small_websocket::{Message, WebSocket};
use tiny_webview::EventLoopProxy;

use crate::CONFIG;
use crate::config::{Config, DMX_SWITCHES_LENGTH};
use crate::dmx::{Color, DMX_STATE, Mode};

// MARK: IpcMessage
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct State {
    pub config: Config,
    pub color: Color,
    pub toggle_color: Color,
    pub toggle_speed: Option<u64>,
    pub strobe_speed: Option<u64>,
    pub mode: Mode,
    pub switches_toggle: [bool; DMX_SWITCHES_LENGTH],
    pub switches_press: [bool; DMX_SWITCHES_LENGTH],
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(crate) enum IpcMessage {
    GetState,
    GetStateResponse {
        state: State,
    },
    SetColor {
        color: Color,
    },
    SetToggleColor {
        #[serde(rename = "toggleColor")]
        toggle_color: Color,
    },
    SetToggleSpeed {
        #[serde(rename = "toggleSpeed")]
        toggle_speed: Option<u64>,
    },
    SetStrobeSpeed {
        #[serde(rename = "strobeSpeed")]
        strobe_speed: Option<u64>,
    },
    SetSwitchesToggle {
        #[serde(rename = "switchesToggle")]
        switches_toggle: [bool; 4],
    },
    SetSwitchesPress {
        #[serde(rename = "switchesPress")]
        switches_press: [bool; 4],
    },
    SetMode {
        mode: Mode,
    },
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
        println!("[RUST] Sending IPC message: {message}");
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
            println!("[RUST] Broadcasting IPC message: {message}");
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
    let mut dmx_state = DMX_STATE.lock().expect("Failed to lock DMX state");
    let message = serde_json::from_str(message).expect("Failed to parse IPC message");
    println!("[RUST] Received IPC message: {message:?}");
    match message {
        IpcMessage::GetState => {
            let state = State {
                config: CONFIG
                    .lock()
                    .expect("Failed to lock config")
                    .clone()
                    .expect("Config is not set"),
                color: dmx_state.color,
                toggle_color: dmx_state.toggle_color,
                toggle_speed: dmx_state.toggle_speed.map(|d| d.as_millis() as u64),
                strobe_speed: dmx_state.strobe_speed.map(|d| d.as_millis() as u64),
                mode: dmx_state.mode,
                switches_toggle: dmx_state.switches_toggle,
                switches_press: dmx_state.switches_press,
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
        IpcMessage::SetToggleColor { toggle_color } => {
            dmx_state.toggle_color = toggle_color;
            connection.broadcast(
                serde_json::to_string(&IpcMessage::SetToggleColor { toggle_color })
                    .expect("Failed to serialize IPC message"),
            );
        }
        IpcMessage::SetToggleSpeed { toggle_speed } => {
            dmx_state.toggle_speed = toggle_speed.map(Duration::from_millis);
            dmx_state.is_toggle_color = toggle_speed.is_some();
            connection.broadcast(
                serde_json::to_string(&IpcMessage::SetToggleSpeed { toggle_speed })
                    .expect("Failed to serialize IPC message"),
            );
        }
        IpcMessage::SetStrobeSpeed { strobe_speed } => {
            dmx_state.strobe_speed = strobe_speed.map(Duration::from_millis);
            dmx_state.is_strobe = strobe_speed.is_some();
            connection.broadcast(
                serde_json::to_string(&IpcMessage::SetStrobeSpeed { strobe_speed })
                    .expect("Failed to serialize IPC message"),
            );
        }
        IpcMessage::SetSwitchesToggle { switches_toggle } => {
            dmx_state.switches_toggle = switches_toggle;
            connection.broadcast(
                serde_json::to_string(&IpcMessage::SetSwitchesToggle { switches_toggle })
                    .expect("Failed to serialize IPC message"),
            );
        }
        IpcMessage::SetSwitchesPress { switches_press } => {
            dmx_state.switches_press = switches_press;
            connection.broadcast(
                serde_json::to_string(&IpcMessage::SetSwitchesPress { switches_press })
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
