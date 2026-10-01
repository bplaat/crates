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

- Design your room, fixtures and groups in the built-in editor
- Select fixtures or groups on a live visualization of the room and control their lights
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
- [SHOWTEC Titan Strobe](https://www.manualslib.com/manual/1569275/Showtec-Titan-Strobe.html)
- [JB Systems TUBELED Controller](https://www.manualslib.com/manual/1158327/Jb-Systems-Tubeled.html)
    - Controls its connected array of [JB Systems TUBELED](https://www.manualslib.com/manual/1165842/Jb-Systems-Tubeled.html) tubes as one fixture
    - Own colors or one of the 38 built-in presets
- [JB Systems TUBELED Controller](https://www.manualslib.com/manual/1158327/Jb-Systems-Tubeled.html)
    - Connected to [JB Systems TUBELED](https://www.manualslib.com/manual/1165842/Jb-Systems-Tubeled.html)

## Installation

Build the latest release from source and run it. BassieLight reconnects to the
first matching uDMX automatically when it is plugged in.

### Windows

BassieLight uses the WinUSB driver. Install it with [Zadig](https://zadig.akeo.ie/):

1. Connect uDMX and open **Options > List All Devices**.
2. Select the device and check that its USB ID is `16C0:05DC`.
3. Open **Device > Load Preset Device** and load
   [`meta/windows/bassielight-udmx-zadig.cfg`](meta/windows/bassielight-udmx-zadig.cfg).
4. Check that the target driver is **WinUSB** (presets cannot select it), then choose
   **Install Driver** or **Replace Driver** and reconnect uDMX.

To remove stale Zadig driver packages for uDMX, unplug it and run
[`remove-zadig-udmx.ps1`](meta/windows/remove-zadig-udmx.ps1) in an elevated PowerShell.
It lists matching packages; run it again with `-Apply` to delete them, then reinstall WinUSB.

### Linux

Install the supplied udev rule and reconnect uDMX:

```sh
sudo cp meta/linux/60-bassielight-udmx.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

The rule grants the logged-in user access, so BassieLight does not need root.

## License

Copyright © 2023-2026 [Bastiaan van der Plaat](https://bplaat.nl/)

Licensed under the [MIT](../../LICENSE) license.
