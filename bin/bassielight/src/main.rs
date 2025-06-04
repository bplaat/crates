/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![allow(non_upper_case_globals)]

use std::thread::sleep;
use std::time::{Duration, SystemTime};

use rusb::{Context, Device, UsbContext};
use serde::{Deserialize, Serialize};
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

const DMX_LENGTH: usize = 512;
const DMX_FPS: u64 = 44;

#[allow(clippy::enum_variant_names)]
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
enum IpcMessage {
    #[serde(rename = "setColor")]
    SetColor { color: u32 },
    #[serde(rename = "setToggleColor")]
    SetToggleColor { color: u32 },
    #[serde(rename = "setToggleSpeed")]
    SetToggleSpeed { speed: Option<u64> },
    #[serde(rename = "setStrobeSpeed")]
    SetStrobeSpeed { speed: Option<u64> },
}

// FIXME: Global state
static mut x_color: u32 = 0;

static mut x_toggle_color: u32 = 0;
static mut x_toggle_speed: Option<Duration> = None;
static mut x_is_toggle_color: bool = false;

static mut x_strobe_speed: Option<Duration> = None;
static mut x_is_strobe: bool = false;

fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("BassieLight")
        .with_inner_size(tao::dpi::LogicalSize::new(1024.0, 768.0))
        .build(&event_loop)
        .expect("Failed to create window");

    let _webview = WebViewBuilder::new()
        .with_html(include_str!("../app.html"))
        .with_ipc_handler(|req| {
            match serde_json::from_str(req.body()).expect("Can't parse message") {
                IpcMessage::SetColor { color } => unsafe {
                    x_color = color;
                },
                IpcMessage::SetToggleColor { color } => unsafe {
                    x_toggle_color = color;
                },
                IpcMessage::SetToggleSpeed { speed } => unsafe {
                    x_toggle_speed = speed.map(Duration::from_millis);
                    x_is_toggle_color = speed.is_some();
                },
                IpcMessage::SetStrobeSpeed { speed } => unsafe {
                    x_strobe_speed = speed.map(Duration::from_millis);
                    x_is_strobe = speed.is_some();
                },
            }
        })
        .build(&window)
        .expect("Failed to create webview");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => {
                std::thread::spawn(move || {
                    let device = find_udmx_device().expect("Can't find uDMX device");
                    dmx_thread(device);
                });
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}

fn find_udmx_device() -> Option<Device<Context>> {
    let context = Context::new().ok()?;
    for device in context.devices().ok()?.iter() {
        let device_desc = device.device_descriptor().ok()?;
        if device_desc.vendor_id() == 0x16c0 && device_desc.product_id() == 0x05dc {
            return Some(device);
        }
    }
    None
}

fn dmx_thread(device: Device<Context>) {
    let handle = device.open().expect("Can't open uDMX device");

    let mut dmx = [0u8; DMX_LENGTH];
    let mut toggle_color_time = SystemTime::now();
    let mut strobe_time = SystemTime::now();

    loop {
        // Tick
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

        let led_addrs = [0, 6, 12, 18, 24, 30];
        for addr in led_addrs {
            let color = unsafe {
                if x_is_strobe {
                    0x000000
                } else if x_is_toggle_color {
                    x_toggle_color
                } else {
                    x_color
                }
            };
            dmx[addr] = (color >> 16) as u8;
            dmx[addr + 1] = (color >> 8) as u8;
            dmx[addr + 2] = color as u8;
        }

        // Send
        _ = handle.write_control(
            rusb::request_type(
                rusb::Direction::Out,
                rusb::RequestType::Vendor,
                rusb::Recipient::Device,
            ),
            0x02,
            DMX_LENGTH as u16,
            0,
            &dmx,
            Duration::from_millis(0),
        );

        // Sleep
        sleep(Duration::from_millis(1000 / DMX_FPS));
    }
}
