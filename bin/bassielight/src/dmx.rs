/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::thread::sleep;
use std::time::{Duration, SystemTime};

use rusb::{Context, Device};

use crate::config::{Config, FixtureType};

pub(crate) static mut x_color: u32 = 0;
pub(crate) static mut x_toggle_color: u32 = 0;
pub(crate) static mut x_toggle_speed: Option<Duration> = None;
pub(crate) static mut x_is_toggle_color: bool = false;
pub(crate) static mut x_strobe_speed: Option<Duration> = None;
pub(crate) static mut x_is_strobe: bool = false;

/// Starts the DMX output thread for the given device using the given configuration.
pub(crate) fn dmx_thread(device: Device<Context>, config: Config) {
    let handle = device.open().expect("Can't open uDMX device");

    let mut dmx = vec![0u8; config.dmx_length];
    let mut toggle_color_time = SystemTime::now();
    let mut strobe_time = SystemTime::now();

    loop {
        dmx.fill(0);

        if let Some(toggle_speed) = unsafe { x_toggle_speed } {
            if SystemTime::now()
                .duration_since(toggle_color_time)
                .expect("Time went backwards")
                > toggle_speed
            {
                unsafe { x_is_toggle_color = !x_is_toggle_color };
                toggle_color_time = SystemTime::now();
            }
        }

        if let Some(strobe_speed) = unsafe { x_strobe_speed } {
            if SystemTime::now()
                .duration_since(strobe_time)
                .expect("Time went backwards")
                > strobe_speed
            {
                unsafe { x_is_strobe = !x_is_strobe };
                strobe_time = SystemTime::now();
            }
        }

        for fixture in &config.fixtures {
            if fixture.r#type == FixtureType::P56Led {
                let base_addr = fixture.addr - 1;
                let color = unsafe {
                    if x_is_strobe {
                        0x000000
                    } else if x_is_toggle_color {
                        x_toggle_color
                    } else {
                        x_color
                    }
                };
                dmx[base_addr] = (color >> 16) as u8;
                dmx[base_addr + 1] = (color >> 8) as u8;
                dmx[base_addr + 2] = color as u8;
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
