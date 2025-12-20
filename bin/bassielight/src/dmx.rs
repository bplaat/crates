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

use crate::config::{Config, DMX_SWITCHES_LENGTH, FixtureType};

// MARK: Color
#[derive(Debug, Copy, Clone)]
pub(crate) struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub(crate) const BLACK: Color = Color { r: 0, g: 0, b: 0 };
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value: u32 = ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32);
        serializer.serialize_u32(value)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        Ok(Color {
            r: ((value >> 16) & 0xFF) as u8,
            g: ((value >> 8) & 0xFF) as u8,
            b: (value & 0xFF) as u8,
        })
    }
}

// MARK: Mode
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Mode {
    Black,
    Manual,
    Auto,
}

// MARK: DmxState
#[derive(Clone)]
pub(crate) struct DmxState {
    pub is_running: bool,
    pub mode: Mode,
    pub color: Color,
    pub toggle_color: Color,
    pub intensity: u8,
    pub toggle_speed: Option<Duration>,
    pub is_toggle_color: bool,
    pub strobe_speed: Option<Duration>,
    pub is_strobe: bool,
    pub switches_toggle: [bool; DMX_SWITCHES_LENGTH],
    pub switches_press: [bool; DMX_SWITCHES_LENGTH],
}

pub(crate) static DMX_STATE: Mutex<DmxState> = Mutex::new(DmxState {
    is_running: false,
    mode: Mode::Manual,
    color: Color::BLACK,
    toggle_color: Color::BLACK,
    intensity: 0xff,
    toggle_speed: None,
    is_toggle_color: false,
    strobe_speed: None,
    is_strobe: false,
    switches_toggle: [false; DMX_SWITCHES_LENGTH],
    switches_press: [false; DMX_SWITCHES_LENGTH],
});

// MARK: DMX Thread
/// Starts the DMX output thread for the given device using the given configuration.
pub(crate) fn dmx_thread(device: Device<Context>, config: Config) {
    let handle = device.open().expect("Can't open uDMX device");

    let mut dmx = vec![0u8; config.dmx_length];
    let mut toggle_color_time = SystemTime::now();
    let mut strobe_time = SystemTime::now();

    loop {
        let mut dmx_state = DMX_STATE.lock().expect("Failed to lock DMX state").clone();
        if !dmx_state.is_running {
            // FIXME: Create async framework don't do micro sleeps
            sleep(Duration::from_millis(100));
            continue;
        }

        // Update timers
        if let Some(toggle_speed) = dmx_state.toggle_speed
            && SystemTime::now()
                .duration_since(toggle_color_time)
                .expect("Time went backwards")
                > toggle_speed
        {
            dmx_state.is_toggle_color = !dmx_state.is_toggle_color;
            toggle_color_time = SystemTime::now();
        }
        if let Some(strobe_speed) = dmx_state.strobe_speed
            && SystemTime::now()
                .duration_since(strobe_time)
                .expect("Time went backwards")
                > strobe_speed
        {
            dmx_state.is_strobe = !dmx_state.is_strobe;
            strobe_time = SystemTime::now();
        }

        // Update DMX data
        dmx.fill(0);
        for fixture in &config.fixtures {
            match fixture.r#type {
                FixtureType::AmericanDJP56Led
                | FixtureType::AmericanDJMegaTripar
                | FixtureType::AyraCompar10 => {
                    let base_addr = fixture.addr - 1;
                    let color = if dmx_state.is_strobe {
                        Color::BLACK
                    } else if dmx_state.is_toggle_color {
                        dmx_state.toggle_color
                    } else {
                        dmx_state.color
                    };

                    // American DJ P56 LED
                    if fixture.r#type == FixtureType::AmericanDJP56Led {
                        if dmx_state.mode == Mode::Manual {
                            dmx[base_addr] =
                                (color.r as u32 * dmx_state.intensity as u32 / 255) as u8;
                            dmx[base_addr + 1] =
                                (color.g as u32 * dmx_state.intensity as u32 / 255) as u8;
                            dmx[base_addr + 2] =
                                (color.b as u32 * dmx_state.intensity as u32 / 255) as u8;
                        }
                        if dmx_state.mode == Mode::Auto {
                            dmx[base_addr + 5] = 224;
                        }
                    }

                    // American DJ Mega Tripar
                    if fixture.r#type == FixtureType::AmericanDJMegaTripar {
                        if dmx_state.mode == Mode::Manual {
                            dmx[base_addr] = color.r;
                            dmx[base_addr + 1] = color.g;
                            dmx[base_addr + 2] = color.b;
                            dmx[base_addr + 6] = dmx_state.intensity;
                        }
                        if dmx_state.mode == Mode::Auto {
                            dmx[base_addr + 5] = 240;
                        }
                    }

                    // Ayra Compar 10
                    if fixture.r#type == FixtureType::AyraCompar10 {
                        if dmx_state.mode == Mode::Manual {
                            dmx[base_addr] = dmx_state.intensity;
                            dmx[base_addr + 2] = color.r;
                            dmx[base_addr + 3] = color.g;
                            dmx[base_addr + 4] = color.b;
                        }
                        if dmx_state.mode == Mode::Auto {
                            dmx[base_addr + 7] = 221;
                        }
                    }
                }

                FixtureType::ShowtecMultidimMKII => {
                    let base_addr = fixture.addr - 1;
                    if dmx_state.mode == Mode::Manual {
                        for i in 0..DMX_SWITCHES_LENGTH {
                            if dmx_state.switches_toggle[i] || dmx_state.switches_press[i] {
                                dmx[base_addr + i] = 255;
                            } else {
                                dmx[base_addr + i] = 0;
                            }
                        }
                    } else {
                        for i in 0..DMX_SWITCHES_LENGTH {
                            dmx[base_addr + i] = 0;
                        }
                    }
                }
            }
        }

        // Send DMX data
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
