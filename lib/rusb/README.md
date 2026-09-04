# rusb

A minimal replacement for the [`rusb`](https://crates.io/crates/rusb) crate that provides the
synchronous USB operations used in this workspace.

## Supported API

- Create a USB context and enumerate devices
- Read device vendor and product identifiers
- Open a device
- Select an active configuration
- Claim an interface
- Send synchronous host-to-device control transfers with a timeout
- Build USB request-type bytes with `request_type`

Bulk, interrupt, isochronous, and device-to-host transfers are not implemented. The crate also does
not provide hotplug callbacks, asynchronous operations, string descriptors, device reset, or kernel
driver management.

## Platforms

| Platform                                               | Backend             | Additional requirements                                                              |
| ------------------------------------------------------ | ------------------- | ------------------------------------------------------------------------------------ |
| Windows                                                | WinUSB              | The device must use the Microsoft WinUSB driver and register a device interface GUID |
| macOS 10.15 and newer                                  | IOKit and IOUSBHost | None                                                                                 |
| Linux, BSD, and other supported non-Apple Unix systems | libusb-1.0          | The system libusb-1.0 library and permission to access the USB device                |

The Windows backend discovers interface GUIDs from each device's registry properties. It does not
contain device-specific identifiers or a hard-coded interface GUID.

### Linux

Install the runtime library. The backend uses direct FFI declarations and links the versioned
libusb runtime, so development headers are not required.

Ubuntu / Debian:

```sh
sudo apt install libusb-1.0-0
```

Fedora:

```sh
sudo dnf install libusb1
```

USB device permissions must normally be configured with an appropriate udev rule; running the
application as root is not recommended.

BSD and other Unix systems need their system libusb-1.0 package or base library and suitable USB
device-node permissions.

## Example

```rs
use rusb::{Context, UsbContext};

fn print_usb_devices() -> Result<(), rusb::Error> {
    let context = Context::new()?;
    for device in context.devices()?.iter() {
        let descriptor = device.device_descriptor()?;
        println!(
            "{:04x}:{:04x}",
            descriptor.vendor_id(),
            descriptor.product_id()
        );
    }
    Ok(())
}
```

Native contexts and device references remain alive for as long as their devices and handles need
them. Native resources and claimed interfaces are released automatically when their Rust owners are
dropped. Platform FFI and unsafe code remain private to the backend modules.

## License

Copyright © 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
