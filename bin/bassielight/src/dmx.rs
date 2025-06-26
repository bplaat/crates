/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use rusb::{Context, Device};
use serde::{Deserialize, Serialize};

use crate::config::{Config, FixtureType};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Mode {
    Black,
    Manual,
    Auto,
}

pub(crate) struct DmxState {
    pub mode: Mode,
    pub color: u32,
    pub toggle_color: u32,
    pub toggle_speed: Option<Duration>,
    pub is_toggle_color: bool,
    pub strobe_speed: Option<Duration>,
    pub is_strobe: bool,
}

pub(crate) static DMX_STATE: Mutex<DmxState> = Mutex::new(DmxState {
    mode: Mode::Black,
    color: 0x000000,
    toggle_color: 0x000000,
    toggle_speed: None,
    is_toggle_color: false,
    strobe_speed: None,
    is_strobe: false,
});

/// Starts the DMX output thread for the given device using the given configuration.
pub(crate) fn dmx_thread(device: Device<Context>, config: Config) {
    let handle = device.open().expect("Can't open uDMX device");

    let mut dmx = vec![0u8; config.dmx_length];
    let mut toggle_color_time = SystemTime::now();
    let mut strobe_time = SystemTime::now();

    loop {
        {
            let mut dmx_state = DMX_STATE.lock().expect("Failed to lock DMX state");
            dmx.fill(0);

            if let Some(toggle_speed) = dmx_state.toggle_speed {
                if SystemTime::now()
                    .duration_since(toggle_color_time)
                    .expect("Time went backwards")
                    > toggle_speed
                {
                    dmx_state.is_toggle_color = !dmx_state.is_toggle_color;
                    toggle_color_time = SystemTime::now();
                }
            }

            if let Some(strobe_speed) = dmx_state.strobe_speed {
                if SystemTime::now()
                    .duration_since(strobe_time)
                    .expect("Time went backwards")
                    > strobe_speed
                {
                    dmx_state.is_strobe = !dmx_state.is_strobe;
                    strobe_time = SystemTime::now();
                }
            }

            for fixture in &config.fixtures {
                if fixture.r#type == FixtureType::P56Led {
                    let base_addr = fixture.addr - 1;
                    let color = if dmx_state.is_strobe {
                        0x000000
                    } else if dmx_state.is_toggle_color {
                        dmx_state.toggle_color
                    } else {
                        dmx_state.color
                    };

                    if dmx_state.mode == Mode::Manual {
                        dmx[base_addr] = (color >> 16) as u8;
                        dmx[base_addr + 1] = (color >> 8) as u8;
                        dmx[base_addr + 2] = color as u8;
                    } else if dmx_state.mode == Mode::Black {
                        dmx[base_addr] = 0;
                        dmx[base_addr + 1] = 0;
                        dmx[base_addr + 2] = 0;
                    } else if dmx_state.mode == Mode::Auto {
                        dmx[base_addr + 5] = 225;
                    }
                }
            }
        }

        let _ = handle.write_control(
            rusb::request_type(
                rusb::Direction::Out,
                rusb::RequestType::Vendor,
                rusb::Recipient::Device,
            ),
            0x02,
            config.dmx_length as u16,
            0,
            &dmx,
            Duration::from_millis(0),
        );

        sleep(Duration::from_millis(1000 / config.dmx_fps));
    }
}
