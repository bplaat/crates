/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use serde::{Deserialize, Serialize};

use crate::dmx::Mode;

#[allow(clippy::enum_variant_names)]
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub(crate) enum IpcMessage {
    #[serde(rename = "setColor")]
    SetColor { color: u32 },
    #[serde(rename = "setToggleColor")]
    SetToggleColor { color: u32 },
    #[serde(rename = "setToggleSpeed")]
    SetToggleSpeed { speed: Option<u64> },
    #[serde(rename = "setStrobeSpeed")]
    SetStrobeSpeed { speed: Option<u64> },
    #[serde(rename = "setMode")]
    SetMode { mode: Mode },
}
