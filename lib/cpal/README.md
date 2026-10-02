# cpal

A minimal replacement for the [`cpal`](https://crates.io/crates/cpal) crate that provides the
audio input streams used in this workspace.

## Supported API

- Get the default host and its default input device
- Read the default input config, this is always mono `f32` at the device sample rate
- Build an input stream with `f32` samples for any channel count and sample rate, the platform
  converts the audio of the device
- Play, pause and drop a stream

Output streams, device enumeration, device names, other sample formats, and callback timestamps are
not implemented. The `timeout` of `build_input_stream` is ignored.

## Platforms

| Platform        | Backend                 | Additional requirements                                                                  |
| --------------- | ----------------------- | ---------------------------------------------------------------------------------------- |
| Windows         | waveIn with wave mapper | None                                                                                     |
| macOS           | AudioQueue              | Bundled apps need `NSMicrophoneUsageDescription` and a sandbox `audio-input` entitlement |
| Linux           | ALSA `default` device   | The system libasound2 library                                                            |
| Other platforms | None                    | There are no devices                                                                     |

On macOS the error callback is never called, the queue records silence when microphone access is
denied.

### Linux

Install the ALSA runtime library, development headers are not required. Ubuntu / Debian:

```sh
sudo apt install libasound2
```

## Example

```rs
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn print_levels() -> Result<(), cpal::Error> {
    let device = cpal::default_host()
        .default_input_device()
        .ok_or(cpal::ErrorKind::DeviceNotAvailable)?;
    let config = device.default_input_config()?;
    let stream = device.build_input_stream(
        config.config(),
        |data: &[f32], _: &cpal::InputCallbackInfo| {
            let peak = data.iter().fold(0.0f32, |peak, sample| peak.max(sample.abs()));
            println!("{peak:.3}");
        },
        |err| eprintln!("{err}"),
        None,
    )?;
    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(5));
    Ok(())
}
```

## License

Copyright © 2026 [Bastiaan van der Plaat](https://github.com/bplaat)

Licensed under the [MIT](../../LICENSE) license.
