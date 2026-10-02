/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::thread::sleep;
use std::time::Duration;

use log::{info, warn};

use crate::config;
use crate::dmx::DMX_STATE;
use crate::ipc::{self, IpcMessage};
use crate::scripts;
use crate::stage::{STAGE, Stage};

const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Reload the stage and scripts when others like AI agents change the stage folder. While
/// performing on the stage page the setup is frozen, changes are picked up afterwards.
pub(crate) fn watch_thread() {
    // Last stage.json that couldn't be used, to warn about it only once
    let mut rejected = String::new();
    loop {
        sleep(POLL_INTERVAL);
        let is_frozen = {
            let dmx_state = DMX_STATE.lock().expect("Failed to lock DMX state");
            dmx_state.is_running && dmx_state.freeze
        };
        if is_frozen {
            continue;
        }

        let Some(folder) = STAGE
            .lock()
            .expect("Failed to lock stage")
            .as_ref()
            .map(|open| open.path.clone())
        else {
            continue;
        };
        if let Ok(json) = std::fs::read_to_string(folder.join("stage.json")) {
            let mut open_stage = STAGE.lock().expect("Failed to lock stage");
            if let Some(open) = open_stage.as_mut()
                && open.path == folder
                && !open.is_dirty()
                && json != open.json
                && json != rejected
            {
                match Stage::parse(&json, config::dmx_length()) {
                    Ok(stage) => {
                        info!("Reloading changed stage.json");
                        open.saved_json = stage.to_json();
                        open.stage = stage.clone();
                        open.json = json;
                        drop(open_stage);
                        ipc::broadcast(&IpcMessage::SetStage { stage });
                    }
                    Err(error) => {
                        warn!("Ignoring changed stage.json: {error}");
                        rejected = json;
                    }
                }
            }
        }
        scripts::set_sources(scripts::read_scripts(&folder));
    }
}
