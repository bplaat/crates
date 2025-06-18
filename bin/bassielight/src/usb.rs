/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use rusb::{Context, Device, UsbContext};

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
