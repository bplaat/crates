/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::BTreeMap;
use std::io;
use std::sync::{Arc, LazyLock, Mutex, mpsc};
use std::time::Instant;

use bwindow::EventLoopProxy;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use small_websocket::{Message, WebSocket};

use std::path::PathBuf;

use crate::config::{self, CONFIG};
use crate::dmx::{
    DMX_STATE, FIXTURE_OUTPUTS, FixtureOutput, FixtureProp, FixtureState, Mode, Tempo,
};
use crate::stage::{FixtureProfile, FixtureType, OpenStage, STAGE, Stage};
use crate::usb::ErrorCategory;

// MARK: UsbStatus
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "state", content = "category", rename_all = "camelCase")]
pub(crate) enum UsbStatus {
    Connected,
    Disconnected,
    Error(ErrorCategory),
}

pub(crate) static USB_STATUS: Mutex<UsbStatus> = Mutex::new(UsbStatus::Disconnected);

pub(crate) fn set_usb_status(status: UsbStatus) {
    let mut current = USB_STATUS.lock().expect("Failed to lock USB status");
    if *current == status {
        return;
    }
    *current = status;
    drop(current);
    broadcast(&IpcMessage::UsbStatusChanged { status });
}

// MARK: Broadcast
static BROADCAST_SENDER: LazyLock<Option<mpsc::Sender<String>>> = LazyLock::new(|| {
    let (sender, receiver) = mpsc::channel::<String>();
    std::thread::Builder::new()
        .name("ipc-broadcast".to_string())
        .spawn(move || {
            for message in receiver {
                send_to_connections(None, &message);
            }
        })
        .ok()
        .map(|_| sender)
});

/// Broadcast a message to all connections without blocking the caller on slow clients
pub(crate) fn broadcast(message: &IpcMessage) {
    if BROADCAST_SENDER
        .as_ref()
        .is_none_or(|sender| sender.send(message.to_json()).is_err())
    {
        warn!("IPC broadcast thread stopped");
    }
}

const MIN_BPM: f32 = 20.0;
const MAX_BPM: f32 = 300.0;

// MARK: Stage files
/// Switch to another stage file, remember it for the next launch and notify all connections
pub(crate) fn open_stage(path: PathBuf, stage: Stage) {
    DMX_STATE
        .lock()
        .expect("Failed to lock DMX state")
        .fixtures
        .clear();
    *STAGE.lock().expect("Failed to lock stage") = Some(OpenStage {
        path: path.clone(),
        stage: stage.clone(),
    });

    let mut config = CONFIG.lock().expect("Failed to lock config");
    let config = config.as_mut().expect("Config not loaded");
    config.last_stage = Some(path.clone());
    if let Err(error) = config.save() {
        warn!("Can't save config.json: {error}");
    }
    broadcast(&IpcMessage::StageOpened { path, stage });
}

// MARK: IpcMessage
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(crate) enum IpcMessage {
    // State
    Start,
    Stop,
    GetState,
    GetStateResponse {
        state: State,
    },
    GetStage,
    #[serde(skip_deserializing)]
    GetStageResponse {
        path: PathBuf,
        stage: Stage,
        #[serde(rename = "fixtureTypes")]
        fixture_types: Vec<&'static FixtureProfile>,
        #[serde(rename = "dmxLength")]
        dmx_length: usize,
    },
    SetStage {
        stage: Stage,
    },
    #[serde(skip_deserializing)]
    StageOpened {
        path: PathBuf,
        stage: Stage,
    },
    GetUsbStatus,
    GetUsbStatusResponse {
        status: UsbStatus,
    },
    UsbStatusChanged {
        status: UsbStatus,
    },
    SetFixtureProp {
        fixtures: Vec<u32>,
        prop: FixtureProp,
    },
    FixtureOutputs {
        outputs: BTreeMap<u32, FixtureOutput>,
    },
    SetMode {
        mode: Mode,
    },
    /// Tap tempo, also sets the downbeat to now
    SetBpm {
        bpm: f32,
    },
}

impl IpcMessage {
    pub(crate) fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize IPC message")
    }

    const fn is_response(&self) -> bool {
        matches!(
            self,
            IpcMessage::GetStateResponse { .. }
                | IpcMessage::GetStageResponse { .. }
                | IpcMessage::StageOpened { .. }
                | IpcMessage::GetUsbStatusResponse { .. }
                | IpcMessage::UsbStatusChanged { .. }
                | IpcMessage::FixtureOutputs { .. }
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct State {
    pub mode: Mode,
    pub bpm: f32,
    pub fixtures: BTreeMap<u32, FixtureState>,
    pub fixture_outputs: BTreeMap<u32, FixtureOutput>,
}

// MARK: IpcConnection
pub(crate) static IPC_CONNECTIONS: Mutex<Vec<IpcConnection>> = Mutex::new(Vec::new());

pub(crate) enum IpcConnection {
    WebviewIpc(Arc<EventLoopProxy<crate::AppEvent>>),
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
    pub(crate) fn send(&mut self, message: String) -> io::Result<()> {
        match self {
            Self::WebviewIpc(event_loop_proxy) => {
                let _ = event_loop_proxy.send_user_event(crate::AppEvent::UserEvent(message));
                Ok(())
            }
            Self::WebSocket(ws) => ws.send(Message::Text(message)),
        }
    }

    /// Send a message to all other connections
    pub(crate) fn broadcast(&mut self, message: &IpcMessage) {
        send_to_connections(Some(self), &message.to_json());
    }
}

fn send_to_connections(sender: Option<&IpcConnection>, message: &str) {
    IPC_CONNECTIONS
        .lock()
        .expect("Failed to lock IPC connections")
        .retain_mut(|connection| {
            if sender.is_some_and(|sender| connection == sender) {
                return true;
            }
            if let Err(error) = connection.send(message.to_string()) {
                warn!("Removing failed IPC connection: {error}");
                false
            } else {
                true
            }
        });
}

// MARK: IPC Message Handler
pub(crate) fn ipc_message_handler(mut connection: IpcConnection, message: &str) -> bool {
    let message = match parse_client_message(message) {
        Ok(message) => message,
        Err(error) => {
            warn!("Rejecting invalid IPC message: {error}");
            return false;
        }
    };
    let mut dmx_state = DMX_STATE.lock().expect("Failed to lock DMX state");
    debug!("Received IPC message: {message:?}");
    match message {
        // State
        IpcMessage::Start => {
            dmx_state.is_running = true;
            connection.broadcast(&message);
        }
        IpcMessage::Stop => {
            dmx_state.is_running = false;
            connection.broadcast(&message);
        }
        IpcMessage::GetState => {
            let open_stage = STAGE.lock().expect("Failed to lock stage");
            let state = State {
                mode: dmx_state.mode,
                bpm: dmx_state.tempo.bpm,
                fixtures: open_stage
                    .iter()
                    .flat_map(|open| &open.stage.fixtures)
                    .map(|f| (f.id, dmx_state.fixture(f.id)))
                    .collect(),
                fixture_outputs: FIXTURE_OUTPUTS
                    .lock()
                    .expect("Failed to lock fixture outputs")
                    .clone(),
            };
            if connection
                .send(IpcMessage::GetStateResponse { state }.to_json())
                .is_err()
            {
                return false;
            }
        }
        IpcMessage::GetStage => {
            let (path, stage) = {
                let open_stage = STAGE.lock().expect("Failed to lock stage");
                let open = open_stage.as_ref().expect("Stage not loaded");
                (open.path.clone(), open.stage.clone())
            };
            let response = IpcMessage::GetStageResponse {
                path,
                stage,
                fixture_types: FixtureType::ALL.map(FixtureType::profile).to_vec(),
                dmx_length: config::dmx_length(),
            };
            if connection.send(response.to_json()).is_err() {
                return false;
            }
        }
        IpcMessage::SetStage { ref stage } => {
            let mut open_stage = STAGE.lock().expect("Failed to lock stage");
            let open = open_stage.as_mut().expect("Stage not loaded");
            if let Err(error) = stage.validate(config::dmx_length()) {
                // Resync the sender with the current stage
                warn!("Rejecting invalid stage: {error}");
                let stage = open.stage.clone();
                return connection
                    .send(IpcMessage::SetStage { stage }.to_json())
                    .is_ok();
            }
            if let Err(error) = stage.save(&open.path) {
                warn!("Can't save {}: {error}", open.path.display());
            }
            dmx_state
                .fixtures
                .retain(|id, _| stage.fixtures.iter().any(|f| f.id == *id));
            open.stage = stage.clone();
            connection.broadcast(&message);
        }
        IpcMessage::GetUsbStatus => {
            let status = *USB_STATUS.lock().expect("Failed to lock USB status");
            if connection
                .send(IpcMessage::GetUsbStatusResponse { status }.to_json())
                .is_err()
            {
                return false;
            }
        }

        IpcMessage::SetFixtureProp { ref fixtures, prop } => {
            for id in fixtures {
                dmx_state
                    .fixtures
                    .entry(*id)
                    .or_insert(FixtureState::DEFAULT)
                    .apply(prop);
            }
            connection.broadcast(&message);
        }
        IpcMessage::SetMode { mode } => {
            dmx_state.mode = mode;
            connection.broadcast(&message);
        }
        IpcMessage::SetBpm { bpm } => {
            dmx_state.tempo = Tempo {
                bpm: bpm.clamp(MIN_BPM, MAX_BPM),
                downbeat: Some(Instant::now()),
            };
            connection.broadcast(&message);
        }

        IpcMessage::GetStateResponse { .. }
        | IpcMessage::GetStageResponse { .. }
        | IpcMessage::StageOpened { .. }
        | IpcMessage::GetUsbStatusResponse { .. }
        | IpcMessage::UsbStatusChanged { .. }
        | IpcMessage::FixtureOutputs { .. } => unreachable!(),
    }
    true
}

fn parse_client_message(message: &str) -> Result<IpcMessage, &'static str> {
    let message: IpcMessage =
        serde_json::from_str(message).map_err(|_| "invalid JSON or message shape")?;
    if message.is_response() {
        Err("response-only message received from client")
    } else {
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_usb_status_messages() {
        let disconnected = serde_json::to_string(&IpcMessage::UsbStatusChanged {
            status: UsbStatus::Disconnected,
        })
        .expect("Failed to serialize disconnected status");
        assert_eq!(
            disconnected,
            r#"{"type":"usbStatusChanged","status":{"state":"disconnected"}}"#
        );

        let error = serde_json::to_string(&IpcMessage::UsbStatusChanged {
            status: UsbStatus::Error(ErrorCategory::Access),
        })
        .expect("Failed to serialize error status");
        assert_eq!(
            error,
            r#"{"type":"usbStatusChanged","status":{"state":"error","category":"access"}}"#
        );
    }

    #[test]
    fn rejects_malformed_and_response_only_client_messages() {
        assert!(parse_client_message("not JSON").is_err());
        assert!(
            parse_client_message(
                r#"{"type":"getUsbStatusResponse","status":{"state":"connected"}}"#
            )
            .is_err()
        );
        assert!(parse_client_message(r#"{"type":"getUsbStatus"}"#).is_ok());
    }

    #[test]
    fn parses_fixture_prop_messages() {
        let message = parse_client_message(
            r#"{"type":"setFixtureProp","fixtures":[1,2],"prop":{"toggleSpeed":null}}"#,
        )
        .expect("Failed to parse fixture prop message");
        assert!(matches!(
            message,
            IpcMessage::SetFixtureProp {
                ref fixtures,
                prop: FixtureProp::ToggleSpeed(None),
            } if fixtures == &[1, 2]
        ));
        assert!(matches!(
            parse_client_message(
                r#"{"type":"setFixtureProp","fixtures":[3],"prop":{"switchToggle":{"index":2,"on":true}}}"#,
            ),
            Ok(IpcMessage::SetFixtureProp {
                prop: FixtureProp::SwitchToggle { index: 2, on: true },
                ..
            })
        ));
        assert!(
            parse_client_message(r#"{"type":"fixtureOutputs","outputs":{"1":{"rgb":255}}}"#)
                .is_err()
        );
    }
}
