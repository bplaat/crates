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
use crate::scripts;
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
/// Switch to another stage folder, remember it for the next launch and notify all connections
pub(crate) fn open_stage(open: OpenStage) {
    {
        let mut dmx_state = DMX_STATE.lock().expect("Failed to lock DMX state");
        dmx_state.fixtures.clear();
        dmx_state.running_scripts.clear();
    }
    let (path, stage) = (open.path.clone(), open.stage.clone());
    scripts::SCRIPTS
        .lock()
        .expect("Failed to lock scripts")
        .errors
        .clear();
    scripts::set_sources(scripts::read_scripts(&path));
    *STAGE.lock().expect("Failed to lock stage") = Some(open);

    let mut config = CONFIG.lock().expect("Failed to lock config");
    let config = config.as_mut().expect("Config not loaded");
    config.last_stage = Some(path.clone());
    if let Err(error) = config.save() {
        warn!("Can't save config.json: {error}");
    }
    broadcast(&IpcMessage::StageOpened { path, stage });
    scripts::broadcast_status();
}

// MARK: IpcMessage
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(crate) enum IpcMessage {
    // State
    Start {
        /// Performing on the stage page, external changes are picked up afterwards
        #[serde(default)]
        freeze: bool,
    },
    Stop,
    GetState,
    #[serde(skip_deserializing)]
    GetStateResponse {
        state: State,
    },
    GetStage,
    #[serde(skip_deserializing)]
    GetStageResponse {
        path: PathBuf,
        stage: Stage,
        dirty: bool,
        #[serde(rename = "fixtureTypes")]
        fixture_types: Vec<&'static FixtureProfile>,
        #[serde(rename = "dmxLength")]
        dmx_length: usize,
    },
    SaveStage,
    #[serde(skip_deserializing)]
    SaveStageResponse {
        error: Option<String>,
    },
    #[serde(skip_deserializing)]
    StageDirty {
        path: PathBuf,
        dirty: bool,
    },
    SetStage {
        stage: Stage,
    },
    #[serde(skip_deserializing)]
    StageOpened {
        path: PathBuf,
        stage: Stage,
    },
    // Scripts
    GetScripts,
    #[serde(skip_deserializing)]
    GetScriptsResponse {
        scripts: BTreeMap<String, String>,
        running: Vec<String>,
        errors: BTreeMap<String, String>,
    },
    SaveScript {
        name: String,
        source: String,
    },
    DeleteScript {
        name: String,
    },
    StartScript {
        name: String,
    },
    StopScript {
        name: String,
    },
    #[serde(skip_deserializing)]
    ScriptsChanged {
        scripts: BTreeMap<String, String>,
    },
    #[serde(skip_deserializing)]
    ScriptsRunning {
        running: Vec<String>,
        errors: BTreeMap<String, String>,
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
    #[serde(skip_deserializing)]
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
                | IpcMessage::StageDirty { .. }
                | IpcMessage::SaveStageResponse { .. }
                | IpcMessage::GetScriptsResponse { .. }
                | IpcMessage::ScriptsChanged { .. }
                | IpcMessage::ScriptsRunning { .. }
                | IpcMessage::GetUsbStatusResponse { .. }
                | IpcMessage::UsbStatusChanged { .. }
                | IpcMessage::FixtureOutputs { .. }
        )
    }
}

#[derive(Debug, Serialize)]
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

fn stage_folder() -> PathBuf {
    STAGE
        .lock()
        .expect("Failed to lock stage")
        .as_ref()
        .expect("Stage not loaded")
        .path
        .clone()
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
        IpcMessage::Start { freeze } => {
            dmx_state.is_running = true;
            dmx_state.freeze = freeze;
            connection.broadcast(&message);
        }
        IpcMessage::Stop => {
            dmx_state.is_running = false;
            dmx_state.freeze = false;
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
            let (path, stage, dirty) = {
                let open_stage = STAGE.lock().expect("Failed to lock stage");
                let open = open_stage.as_ref().expect("Stage not loaded");
                (open.path.clone(), open.stage.clone(), open.is_dirty())
            };
            let response = IpcMessage::GetStageResponse {
                path,
                dirty,
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
                let _ = connection.send(
                    IpcMessage::StageDirty {
                        path: open.path.clone(),
                        dirty: open.is_dirty(),
                    }
                    .to_json(),
                );
                return connection
                    .send(IpcMessage::SetStage { stage }.to_json())
                    .is_ok();
            }
            dmx_state
                .fixtures
                .retain(|id, _| stage.fixtures.iter().any(|f| f.id == *id));
            open.stage = stage.clone();
            connection.broadcast(&message);
            broadcast(&IpcMessage::StageDirty {
                path: open.path.clone(),
                dirty: open.is_dirty(),
            });
        }

        IpcMessage::SaveStage => {
            let mut open_stage = STAGE.lock().expect("Failed to lock stage");
            let open = open_stage.as_mut().expect("Stage not loaded");
            let error = if open.is_dirty() {
                open.save().err().map(|error| error.to_string())
            } else {
                None
            };
            broadcast(&IpcMessage::StageDirty {
                path: open.path.clone(),
                dirty: open.is_dirty(),
            });
            if connection
                .send(IpcMessage::SaveStageResponse { error }.to_json())
                .is_err()
            {
                return false;
            }
        }

        // Scripts
        IpcMessage::GetScripts => {
            let response = {
                let scripts = scripts::SCRIPTS.lock().expect("Failed to lock scripts");
                IpcMessage::GetScriptsResponse {
                    scripts: scripts.sources.clone(),
                    running: dmx_state.running_scripts.iter().cloned().collect(),
                    errors: scripts.errors.clone(),
                }
            };
            if connection.send(response.to_json()).is_err() {
                return false;
            }
        }
        IpcMessage::SaveScript {
            ref name,
            ref source,
        } => {
            if !scripts::is_valid_name(name) {
                warn!("Rejecting invalid script name: {name}");
                return true;
            }
            scripts::SCRIPTS
                .lock()
                .expect("Failed to lock scripts")
                .errors
                .remove(name);
            let folder = stage_folder();
            if let Err(error) = scripts::save_script(&folder, name, source) {
                warn!("Can't save script {name}: {error}");
            }
            broadcast(&scripts::status(&dmx_state));
        }
        IpcMessage::DeleteScript { ref name } => {
            if !scripts::is_valid_name(name) {
                warn!("Rejecting invalid script name: {name}");
                return true;
            }
            dmx_state.running_scripts.remove(name);
            if let Err(error) = scripts::delete_script(&stage_folder(), name) {
                warn!("Can't delete script {name}: {error}");
            }
            broadcast(&scripts::status(&dmx_state));
        }
        IpcMessage::StartScript { ref name } => {
            scripts::SCRIPTS
                .lock()
                .expect("Failed to lock scripts")
                .errors
                .remove(name);
            dmx_state.running_scripts.insert(name.clone());
            broadcast(&scripts::status(&dmx_state));
        }
        IpcMessage::StopScript { ref name } => {
            dmx_state.running_scripts.remove(name);
            broadcast(&scripts::status(&dmx_state));
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
        | IpcMessage::StageDirty { .. }
        | IpcMessage::SaveStageResponse { .. }
        | IpcMessage::GetScriptsResponse { .. }
        | IpcMessage::ScriptsChanged { .. }
        | IpcMessage::ScriptsRunning { .. }
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
