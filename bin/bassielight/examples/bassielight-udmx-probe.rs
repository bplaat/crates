/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Hardware diagnostics using production uDMX code and all-zero channel values.

use std::error::Error;
use std::thread::sleep;
use std::time::{Duration, Instant};

use rusb::{Context, UsbContext};

#[allow(dead_code)]
#[path = "../src/usb.rs"]
mod usb;

fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "device".into());
    let length = parse_argument(2, 512)?;
    let iterations = parse_argument(3, 5)?;
    let delay_ms = parse_argument(4, 23)?;
    let start = parse_argument(5, 0)?;
    if length == 0 || start.checked_add(length).is_none_or(|end| end > 512) {
        return Err("uDMX channel ranges must fit within 512 channels".into());
    }
    let delay = Duration::from_millis(delay_ms as u64);
    let frame = vec![0; length];
    if mode == "connection" {
        if start != 0 {
            return Err("Connection diagnostics require a zero starting channel".into());
        }
        let mut connection = usb::UdmxConnection::new(Instant::now());
        if connection.poll(Instant::now()) != Some(usb::ConnectionEvent::Connected) {
            return Err("Cannot open uDMX 16c0:05dc".into());
        }
        for iteration in 1..=iterations {
            let started = Instant::now();
            let event = connection.send(started, &frame);
            println!("{iteration}: {event:?} in {:?}", started.elapsed());
            sleep(delay);
        }
        return Ok(());
    }
    if mode != "device" {
        return Err(
            "Use device for range transfers or connection for cached production output".into(),
        );
    }
    let context = Context::new()?;
    let device = context
        .devices()?
        .iter()
        .find(|device| {
            device.device_descriptor().is_ok_and(|descriptor| {
                descriptor.vendor_id() == 0x16c0 && descriptor.product_id() == 0x05dc
            })
        })
        .ok_or("uDMX 16c0:05dc is not connected")?;
    let handle = device.open()?;
    match handle.set_active_configuration(1) {
        Ok(()) | Err(rusb::Error::Busy) => {}
        Err(error) => return Err(error.into()),
    }
    handle.claim_interface(0)?;
    let mut times = Vec::new();
    let mut errors = 0;
    println!(
        "uDMX probe: request_type=0x40, start={start}, length={length}, iterations={iterations}, delay_ms={delay_ms}"
    );
    for iteration in 1..=iterations {
        let started = Instant::now();
        let result = usb::write_udmx_range(&handle, start as u16, &frame);
        let elapsed = started.elapsed();
        if result.is_err() {
            errors += 1;
        }
        times.push(elapsed);
        println!("{iteration}: {result:?} in {elapsed:?}");
        sleep(delay);
    }
    times.sort_unstable();
    if !times.is_empty() {
        println!(
            "Summary: {errors}/{iterations} errors, min={:?}, median={:?}, max={:?}",
            times[0],
            times[times.len() / 2],
            times[times.len() - 1]
        );
    }
    Ok(())
}

fn parse_argument(index: usize, default: usize) -> Result<usize, Box<dyn Error>> {
    std::env::args()
        .nth(index)
        .map_or(Ok(default), |value| Ok(value.parse()?))
}
