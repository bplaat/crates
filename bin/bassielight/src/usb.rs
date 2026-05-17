/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use log::error;
use rusb::{Context, Device, DeviceHandle, UsbContext};

/// Finds the first uDMX device matching vendor and product id.
pub(crate) fn find_udmx_device() -> Option<Device<Context>> {
    let context = Context::new().ok()?;
    for device in context.devices().ok()?.iter() {
        let device_desc = device.device_descriptor().ok()?;
        if device_desc.vendor_id() == 0x16c0 && device_desc.product_id() == 0x05dc {
            return Some(device);
        }
    }
    None
}

/// Opens the first uDMX device and returns a handle ready for transfers.
pub(crate) fn open_udmx_handle() -> Option<DeviceHandle<Context>> {
    let device = find_udmx_device()?;
    let handle = match device.open() {
        Ok(h) => h,
        Err(err) => {
            error!("Failed to open uDMX device: {err}");
            return None;
        }
    };
    if let Err(err) = handle.set_active_configuration(1) {
        // BUSY is acceptable - device may already be configured
        if err != rusb::Error::Busy {
            error!("Failed to set uDMX configuration: {err}");
            return None;
        }
    }
    if let Err(err) = handle.claim_interface(0) {
        error!("Failed to claim uDMX interface: {err}");
        return None;
    }
    Some(handle)
}
