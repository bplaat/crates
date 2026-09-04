/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Sends safe, all-zero DMX frames to an attached uDMX for hardware diagnostics.

use std::error::Error;
use std::thread::sleep;
use std::time::{Duration, Instant};

use rusb::{Context, DeviceHandle, Direction, Recipient, RequestType, UsbContext};

const UDMX_VENDOR_ID: u16 = 0x16c0;
const UDMX_PRODUCT_ID: u16 = 0x05dc;
const UDMX_CONFIGURATION: u8 = 1;
const UDMX_INTERFACE: u8 = 0;
const UDMX_SET_CHANNEL_RANGE: u8 = 0x02;
const TRANSFER_TIMEOUT: Duration = Duration::from_millis(500);
const TRANSFER_ATTEMPTS: usize = 3;

fn main() -> Result<(), Box<dyn Error>> {
    let recipient = match std::env::args().nth(1).as_deref() {
        None | Some("device") => Recipient::Device,
        Some("interface") => Recipient::Interface,
        Some(value) => return Err(format!("unknown recipient: {value}").into()),
    };
    let length = parse_argument(2, 512)?;
    let iterations = parse_argument(3, 5)?;
    let delay_ms = parse_argument(4, 40)?;
    if length == 0 || length > 512 {
        return Err("uDMX frames contain between 1 and 512 channels".into());
    }

    let context = Context::new()?;
    let device = context
        .devices()?
        .iter()
        .find(|device| {
            device.device_descriptor().is_ok_and(|descriptor| {
                descriptor.vendor_id() == UDMX_VENDOR_ID
                    && descriptor.product_id() == UDMX_PRODUCT_ID
            })
        })
        .ok_or("uDMX 16c0:05dc is not connected")?;
    let handle = device.open()?;
    handle.set_active_configuration(UDMX_CONFIGURATION)?;
    handle.claim_interface(UDMX_INTERFACE)?;

    let request_type = rusb::request_type(Direction::Out, RequestType::Vendor, recipient);
    let frame = vec![0; length];
    println!(
        "uDMX probe: request_type=0x{request_type:02x}, length={length}, iterations={iterations}, delay_ms={delay_ms}"
    );
    for iteration in 1..=iterations {
        let started = Instant::now();
        let (result, attempts) = send_frame(&handle, request_type, &frame);
        println!(
            "{iteration}: {result:?} after {attempts} attempt(s) in {:?}",
            started.elapsed()
        );
        sleep(Duration::from_millis(delay_ms as u64));
    }
    Ok(())
}

fn send_frame(
    handle: &DeviceHandle<Context>,
    request_type: u8,
    frame: &[u8],
) -> (Result<usize, rusb::Error>, usize) {
    for attempt in 1..=TRANSFER_ATTEMPTS {
        let result = handle.write_control(
            request_type,
            UDMX_SET_CHANNEL_RANGE,
            frame.len() as u16,
            0,
            frame,
            TRANSFER_TIMEOUT,
        );
        match result {
            Ok(transferred) if transferred == frame.len() => return (result, attempt),
            Err(
                rusb::Error::Io
                | rusb::Error::Timeout
                | rusb::Error::Pipe
                | rusb::Error::Interrupted
                | rusb::Error::Other,
            ) if attempt < TRANSFER_ATTEMPTS => {}
            _ => return (result, attempt),
        }
    }
    unreachable!()
}

fn parse_argument(index: usize, default: usize) -> Result<usize, Box<dyn Error>> {
    std::env::args()
        .nth(index)
        .map_or(Ok(default), |value| Ok(value.parse()?))
}
