# BassieLight

<div>

<img align="left" src="docs/images/icon.svg" width="96" height="96" />

<br/>

<p>
    A simple DMX512 lights controller GUI compatible with the <a href="https://www.anyma.ch/research/udmx/">uDMX</a> and various fixtures.
</p>

<br/>

</div>

## Features

- Create a setup with fixtures with a simple `config.json` file
- Control different Lights with the GUI
- Control setup with a remote device through the web interface

## Compatibility

- [uDMX USB DMX512 dongle](https://www.anyma.ch/research/udmx/)
- [American DJ P56P LED](https://www.manualslib.com/manual/530185/American-Dj-P56p-Led.html)
- [American DJ Mega Tripar](https://www.manualslib.com/manual/530164/American-Dj-Mega-Tripar-Profile.html)
    - 7 channel mode
- [Ayra Compar 10](https://www.manualslib.com/manual/1061771/Ayra-Compar-10.html)
    - 8 channel mode
- [Ayra Compar 20](https://www.manualslib.com/manual/1033103/Ayra-Compar-20.html)
    - 6 channel mode
- [SHOWTEC Multidim MKII](https://www.manualslib.com/manual/2115423/Showtec-Multidim-Mkii.html)

## Installation

Build the latest release from source and run it. BassieLight reconnects to the
first matching uDMX automatically when it is plugged in.

### Windows

BassieLight uses Microsoft's WinUSB driver. Download Zadig only from the
[official Zadig site](https://zadig.akeo.ie/), connect uDMX, and then:

1. Open **Options > List All Devices**.
2. Select the uDMX device and verify that the displayed USB ID is exactly
   `16C0:05DC`. Do not continue if the device name or ID differs.
3. Open **Device > Load Preset Device** and load
   [`meta/windows/bassielight-udmx-zadig.cfg`](meta/windows/bassielight-udmx-zadig.cfg).
4. Verify again that the target driver shown on the right is **WinUSB**, not
   libusbK or libusb-win32.
5. Choose **Install Driver** or **Replace Driver**, reconnect uDMX, and verify
   that BassieLight reports it as connected.

The preset matches only `USB\VID_16C0&PID_05DC` and registers interface GUID
`{0DD9BE09-BBEA-44A0-AB59-2F098406949C}`. Zadig device presets cannot select a
driver, so always confirm that the target driver is WinUSB before installing.

If stale Zadig packages prevent a clean installation, use the supplied
[`remove-zadig-udmx.ps1`](meta/windows/remove-zadig-udmx.ps1):

1. Unplug uDMX.
2. Start an elevated PowerShell, run the script without `-Apply`, and inspect
   every package it found.
3. Run it again with `-Apply` to delete only the confirmed packages.
4. Reboot if Windows still retains a stale device node.
5. Reconnect uDMX and reinstall WinUSB with the supplied preset.

The script only considers `oem*.inf` packages containing the exact uDMX
hardware ID, a Zadig/libwdi-style provider, and a WinUSB, libusbK, or
libusb-win32 service. It never targets Microsoft's inbox `winusb.inf`,
`winusb.sys`, or packages for other USB IDs.

### Linux

Install the supplied udev rule and reconnect uDMX:

```sh
sudo cp meta/linux/60-bassielight-udmx.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Run BassieLight as your normal desktop user, never as root. The rule grants
the active logged-in user access through `TAG+="uaccess"` while retaining mode
`0660`.

## License

Copyright © 2023-2026 [Bastiaan van der Plaat](https://bplaat.nl/)

Licensed under the [MIT](../../LICENSE) license.
