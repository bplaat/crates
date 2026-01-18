# BassieLight

<div>

<img align="left" src="docs/images/icon.svg" width="96" height="96" />

<br/>

<p>
    A simple DMX512 lights controller GUI compatible with the <a href="https://www.anyma.ch/research/udmx/">uDMX</a> and various fixtures
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

### Installation

On Windows install the WinUSB driver with [Zadig](https://zadig.akeo.ie/) for the uDMX device.

Build the latest release from source and run it.

## License

Copyright &copy; 2023-2025 [Bastiaan van der Plaat](https://bplaat.nl/)

Licensed under the [MIT](../../LICENSE) license.
