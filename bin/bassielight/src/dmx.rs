/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::BTreeMap;
use std::env;
use std::fmt::{self, Display};
use std::io::{self, IsTerminal};
use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, Instant};

use log::{info, trace, warn};
use serde::{Deserialize, Serialize};

use crate::config::{CONFIG, DMX_LENGTH};
use crate::ipc::{self, IpcMessage, UsbStatus};
use crate::stage::{Channel, DMX_SWITCHES_LENGTH, FixtureKind, STAGE};
use crate::usb;

// MARK: Color
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub(crate) const BLACK: Color = Color { r: 0, g: 0, b: 0 };

    pub(crate) const fn from_u32(value: u32) -> Color {
        Color {
            r: ((value >> 16) & 0xFF) as u8,
            g: ((value >> 8) & 0xFF) as u8,
            b: (value & 0xFF) as u8,
        }
    }

    pub(crate) fn apply_intensity(self, intensity: f32) -> Color {
        Color {
            r: (self.r as f32 * intensity) as u8,
            g: (self.g as f32 * intensity) as u8,
            b: (self.b as f32 * intensity) as u8,
        }
    }

    pub(crate) fn tween(self, other: Color, t: f32) -> Color {
        Color {
            r: (self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8,
            g: (self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8,
            b: (self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value: u32 = ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32);
        serializer.serialize_u32(value)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Color::from_u32(u32::deserialize(deserializer)?))
    }
}

// MARK: Mode
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Mode {
    Black,
    Manual,
    Auto,
}

// MARK: ToggleTween
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ToggleTween {
    Direct,
    Linear,
    Ease,
}

// MARK: FixtureState
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FixtureState {
    pub color: Color,
    pub toggle_color: Color,
    pub intensity: f32,
    pub toggle_tween: ToggleTween,
    /// Beats between toggle color changes
    pub toggle_speed: Option<f32>,
    /// Beats per strobe flash
    pub strobe_speed: Option<f32>,
    pub switches_toggle: [bool; DMX_SWITCHES_LENGTH],
    pub switches_press: [bool; DMX_SWITCHES_LENGTH],
    pub flash_on: bool,
    pub flash_press: bool,
    pub flash_intensity: f32,
    pub flash_speed: f32,
    /// Index of the running built-in preset, `None` for the own color
    pub preset: Option<usize>,
    pub preset_speed: f32,
}

impl FixtureState {
    pub(crate) const DEFAULT: FixtureState = FixtureState {
        color: Color::BLACK,
        toggle_color: Color::BLACK,
        intensity: 1.0,
        toggle_tween: ToggleTween::Direct,
        toggle_speed: None,
        strobe_speed: None,
        switches_toggle: [false; DMX_SWITCHES_LENGTH],
        switches_press: [false; DMX_SWITCHES_LENGTH],
        flash_on: false,
        flash_press: false,
        flash_intensity: 1.0,
        flash_speed: 0.5,
        preset: None,
        preset_speed: 0.5,
    };

    pub(crate) const fn apply(&mut self, prop: FixtureProp) {
        match prop {
            FixtureProp::Color(color) => self.color = color,
            FixtureProp::ToggleColor(toggle_color) => self.toggle_color = toggle_color,
            FixtureProp::Intensity(intensity) => self.intensity = intensity,
            FixtureProp::ToggleTween(toggle_tween) => self.toggle_tween = toggle_tween,
            FixtureProp::ToggleSpeed(toggle_speed) => self.toggle_speed = toggle_speed,
            FixtureProp::StrobeSpeed(strobe_speed) => self.strobe_speed = strobe_speed,
            FixtureProp::SwitchToggle { index, on } => {
                if index < DMX_SWITCHES_LENGTH {
                    self.switches_toggle[index] = on;
                }
            }
            FixtureProp::SwitchPress { index, on } => {
                if index < DMX_SWITCHES_LENGTH {
                    self.switches_press[index] = on;
                }
            }
            FixtureProp::FlashOn(flash_on) => self.flash_on = flash_on,
            FixtureProp::FlashPress(flash_press) => self.flash_press = flash_press,
            FixtureProp::FlashIntensity(flash_intensity) => self.flash_intensity = flash_intensity,
            FixtureProp::FlashSpeed(flash_speed) => self.flash_speed = flash_speed,
            FixtureProp::Preset(preset) => self.preset = preset,
            FixtureProp::PresetSpeed(preset_speed) => self.preset_speed = preset_speed,
        }
    }
}

// MARK: FixtureProp
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum FixtureProp {
    Color(Color),
    ToggleColor(Color),
    Intensity(f32),
    ToggleTween(ToggleTween),
    ToggleSpeed(Option<f32>),
    StrobeSpeed(Option<f32>),
    SwitchToggle { index: usize, on: bool },
    SwitchPress { index: usize, on: bool },
    FlashOn(bool),
    FlashPress(bool),
    FlashIntensity(f32),
    FlashSpeed(f32),
    Preset(Option<usize>),
    PresetSpeed(f32),
}

// MARK: DmxState
#[derive(Clone)]
pub(crate) struct DmxState {
    pub is_running: bool,
    pub mode: Mode,
    pub tempo: Tempo,
    pub fixtures: BTreeMap<u32, FixtureState>,
}

impl DmxState {
    pub(crate) fn fixture(&self, id: u32) -> FixtureState {
        self.fixtures
            .get(&id)
            .copied()
            .unwrap_or(FixtureState::DEFAULT)
    }
}

pub(crate) static DMX_STATE: Mutex<DmxState> = Mutex::new(DmxState {
    is_running: false,
    mode: Mode::Manual,
    tempo: Tempo {
        bpm: DEFAULT_BPM,
        downbeat: None,
    },
    fixtures: BTreeMap::new(),
});

// MARK: FixtureOutput
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum FixtureOutput {
    /// Output color, `None` when the fixture runs its music program
    Rgb(Option<Color>),
    Switch(Vec<bool>),
    Strobe {
        intensity: f32,
        speed: f32,
    },
}

/// Last output of each fixture
pub(crate) static FIXTURE_OUTPUTS: Mutex<BTreeMap<u32, FixtureOutput>> =
    Mutex::new(BTreeMap::new());

// MARK: Tempo
pub(crate) const DEFAULT_BPM: f32 = 120.0;

/// Shared beat clock, all toggles and strobes derive their phase from it so fixtures stay in
/// sync with each other and with the music
#[derive(Debug, Clone, Copy)]
pub(crate) struct Tempo {
    pub bpm: f32,
    /// Moment of the last tap tempo, `None` until the first tap
    pub downbeat: Option<Instant>,
}

impl Tempo {
    /// Beats elapsed since the downbeat
    fn beats(&self, elapsed: Duration) -> f64 {
        elapsed.as_secs_f64() * self.bpm as f64 / 60.0
    }

    fn seconds(&self, beats: f32) -> f64 {
        beats as f64 * 60.0 / self.bpm as f64
    }
}

/// Position of a DMX frame on the beat clock
struct Clock {
    tempo: Tempo,
    beat: f64,
    frame: u64,
    fps: f64,
}

impl Clock {
    fn color(&self, state: &FixtureState) -> Color {
        // One flash per strobe period, rounded to whole frames so every flash is equally long
        if let Some(beats) = state.strobe_speed {
            let period = (self.tempo.seconds(beats) * self.fps).round().max(2.0) as u64;
            if self.frame % period >= period / 2 {
                return Color::BLACK;
            }
        }

        let Some(beats) = state.toggle_speed else {
            return state.color;
        };
        let cycle = self.beat / beats as f64;
        let (from, to) = if cycle as u64 % 2 == 1 {
            (state.color, state.toggle_color)
        } else {
            (state.toggle_color, state.color)
        };
        let t = cycle.fract() as f32;
        let t = match state.toggle_tween {
            ToggleTween::Direct => 1.0,
            ToggleTween::Linear => t,
            ToggleTween::Ease => t * t * (3.0 - 2.0 * t),
        };
        from.tween(to, t)
    }
}

// MARK: DMX Thread
pub(crate) fn dmx_thread() {
    let (dmx_length, dmx_fps) = {
        let config = CONFIG.lock().expect("Failed to lock config");
        let config = config.as_ref().expect("Config not loaded");
        (config.dmx_length, config.dmx_fps)
    };
    let frame_interval = Duration::from_secs_f64(1.0 / dmx_fps.max(1) as f64);
    let mut connection = usb::UdmxConnection::new(Instant::now());
    let clock_start = Instant::now();
    let mut next_frame = Instant::now();
    let mut dmx = Vec::new();
    let use_colors = io::stdout().is_terminal()
        && env::var_os("NO_COLOR").is_none()
        && env::var_os("CI").is_none();

    loop {
        log_connection_event(connection.poll(Instant::now()));
        let dmx_state = DMX_STATE.lock().expect("Failed to lock DMX state").clone();
        if !dmx_state.is_running {
            // FIXME: Create async framework don't do micro sleeps
            sleep(Duration::from_millis(100));
            next_frame = Instant::now();
            continue;
        }

        // Frames are timed by fixed deadlines so send jitter doesn't shift the beat clock
        let frame_time = next_frame;
        let tempo = dmx_state.tempo;
        let elapsed = frame_time.saturating_duration_since(tempo.downbeat.unwrap_or(clock_start));
        let clock = Clock {
            tempo,
            beat: tempo.beats(elapsed),
            frame: (elapsed.as_secs_f64() * dmx_fps as f64) as u64,
            fps: dmx_fps as f64,
        };
        let fixtures = STAGE
            .lock()
            .expect("Failed to lock stage")
            .as_ref()
            .expect("Stage not loaded")
            .stage
            .fixtures
            .clone();

        // Only send channels up to the last one used by a fixture to keep transfers short
        let send_length = fixtures
            .iter()
            .map(|f| f.addr - 1 + f.r#type.channel_count())
            .max()
            .unwrap_or(dmx_length)
            .min(dmx_length)
            .min(DMX_LENGTH);

        // Update DMX data
        dmx.clear();
        dmx.resize(dmx_length, 0);
        let mut outputs = BTreeMap::new();
        for fixture in &fixtures {
            let profile = fixture.r#type.profile();
            let channels = &mut dmx[fixture.addr - 1..][..profile.channels.len()];
            match profile.kind {
                FixtureKind::Switch => {
                    let state = dmx_state.fixture(fixture.id);
                    let mut switches = Vec::new();
                    for (value, channel) in channels.iter_mut().zip(profile.channels) {
                        if *channel == Channel::Switch {
                            let index = switches.len();
                            let is_on = dmx_state.mode == Mode::Manual
                                && (state.switches_toggle.get(index) == Some(&true)
                                    || state.switches_press.get(index) == Some(&true));
                            *value = if is_on { 255 } else { 0 };
                            switches.push(is_on);
                        }
                    }
                    outputs.insert(fixture.id, FixtureOutput::Switch(switches));
                }
                FixtureKind::Strobe => {
                    let state = dmx_state.fixture(fixture.id);
                    // A strobe has no music program, so it keeps flashing in auto mode
                    let is_on = state.flash_on || state.flash_press;
                    let (intensity, speed) = if dmx_state.mode == Mode::Black || !is_on {
                        (0.0, 0.0)
                    } else {
                        (state.flash_intensity, state.flash_speed)
                    };
                    for (value, channel) in channels.iter_mut().zip(profile.channels) {
                        *value = match channel {
                            Channel::Dimmer => (intensity * 255.0) as u8,
                            Channel::Speed => (speed * 255.0) as u8,
                            _ => 0,
                        };
                    }
                    outputs.insert(fixture.id, FixtureOutput::Strobe { intensity, speed });
                }
                FixtureKind::Rgb => {
                    let state = dmx_state.fixture(fixture.id);

                    // A running built-in preset replaces the color channels, auto mode always runs one
                    let preset = profile.presets.and_then(|presets| {
                        let index = match dmx_state.mode {
                            Mode::Black => None,
                            Mode::Manual => state.preset,
                            Mode::Auto => Some(presets.auto),
                        }?;
                        Some((presets.channels, presets.list.get(index)?))
                    });
                    if let Some((preset_channels, preset)) = preset {
                        for (value, channel) in channels.iter_mut().zip(preset_channels) {
                            *value = match channel {
                                Channel::Preset => preset.value,
                                Channel::Speed => (state.preset_speed * 255.0) as u8,
                                _ => 0,
                            };
                        }
                        outputs.insert(fixture.id, FixtureOutput::Rgb(preset.color));
                        continue;
                    }

                    let color = clock.color(&state);
                    let color_with_intensity = color.apply_intensity(state.intensity);

                    // Fixtures without a dimmer channel get the intensity applied to their color
                    let rgb = if profile.channels.contains(&Channel::Dimmer) {
                        color
                    } else {
                        color_with_intensity
                    };
                    for (value, channel) in channels.iter_mut().zip(profile.channels) {
                        *value = match (*channel, dmx_state.mode) {
                            (Channel::Red, Mode::Manual) => rgb.r,
                            (Channel::Green, Mode::Manual) => rgb.g,
                            (Channel::Blue, Mode::Manual) => rgb.b,
                            (Channel::Dimmer, Mode::Manual) => (state.intensity * 255.0) as u8,
                            (Channel::Music(music), Mode::Auto) => music,
                            _ => 0,
                        };
                    }
                    outputs.insert(
                        fixture.id,
                        FixtureOutput::Rgb(match dmx_state.mode {
                            Mode::Black => Some(Color::BLACK),
                            Mode::Manual => Some(color_with_intensity),
                            Mode::Auto => None,
                        }),
                    );
                }
            }
        }

        if let Some(FixtureOutput::Rgb(Some(color))) = outputs.values().next() {
            if use_colors {
                trace!(
                    "Color: \x1b[38;2;{};{};{}m{color}\x1b[0m",
                    color.r, color.g, color.b
                );
            } else {
                trace!("Color: {color}");
            }
        }
        let mut fixture_outputs = FIXTURE_OUTPUTS
            .lock()
            .expect("Failed to lock fixture outputs");
        if *fixture_outputs != outputs {
            ipc::broadcast(&IpcMessage::FixtureOutputs {
                outputs: outputs.clone(),
            });
            *fixture_outputs = outputs;
        }
        drop(fixture_outputs);

        log_connection_event(connection.send(Instant::now(), &dmx[..send_length]));
        next_frame += frame_interval;
        let now = Instant::now();
        if next_frame < now {
            // Fell behind, continue from now instead of sending a burst of frames
            next_frame = now;
        }
        sleep(next_frame - now);
    }
}

fn log_connection_event(event: Option<usb::ConnectionEvent>) {
    match event {
        Some(usb::ConnectionEvent::Connected) => {
            info!("uDMX device connected");
            ipc::set_usb_status(UsbStatus::Connected);
        }
        Some(usb::ConnectionEvent::Recovered) => {
            info!("uDMX communication recovered");
            ipc::set_usb_status(UsbStatus::Connected);
        }
        Some(usb::ConnectionEvent::Disconnected(category)) => {
            warn!("uDMX device disconnected: {category}");
            ipc::set_usb_status(status_for_error(category));
        }
        Some(usb::ConnectionEvent::Error(category)) => {
            warn!("uDMX error: {category}");
            ipc::set_usb_status(status_for_error(category));
        }
        None => {}
    }
}

const fn status_for_error(category: usb::ErrorCategory) -> UsbStatus {
    if matches!(category, usb::ErrorCategory::NoDevice) {
        UsbStatus::Disconnected
    } else {
        UsbStatus::Error(category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Color = Color { r: 255, g: 0, b: 0 };
    const BLUE: Color = Color { r: 0, g: 0, b: 255 };

    fn clock(beat: f64, frame: u64) -> Clock {
        Clock {
            tempo: Tempo {
                bpm: 120.0,
                downbeat: None,
            },
            beat,
            frame,
            fps: 44.0,
        }
    }

    #[test]
    fn toggles_colors_on_the_beat() {
        let state = FixtureState {
            color: RED,
            toggle_color: BLUE,
            toggle_speed: Some(1.0),
            ..FixtureState::DEFAULT
        };
        assert_eq!(clock(0.5, 0).color(&state), RED);
        assert_eq!(clock(1.5, 0).color(&state), BLUE);
        assert_eq!(clock(2.5, 0).color(&state), RED);
        assert_eq!(
            clock(0.5, 0).color(&FixtureState {
                toggle_speed: None,
                ..state
            }),
            RED
        );
    }

    #[test]
    fn strobes_with_whole_frame_flashes() {
        // 1/4 beat at 120 BPM is 125ms, which rounds to 6 frames: 3 on and 3 off
        let state = FixtureState {
            color: RED,
            strobe_speed: Some(0.25),
            ..FixtureState::DEFAULT
        };
        let flashes: Vec<bool> = (0..12)
            .map(|frame| clock(0.0, frame).color(&state) == RED)
            .collect();
        assert_eq!(
            flashes,
            [
                true, true, true, false, false, false, true, true, true, false, false, false
            ]
        );
    }

    #[test]
    fn maps_connection_failures_to_gui_status() {
        assert_eq!(
            status_for_error(usb::ErrorCategory::NoDevice),
            UsbStatus::Disconnected
        );
        assert_eq!(
            status_for_error(usb::ErrorCategory::Access),
            UsbStatus::Error(usb::ErrorCategory::Access)
        );
    }
}
