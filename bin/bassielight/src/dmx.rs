/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::{BTreeMap, BTreeSet};
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
use crate::stage::{Channel, DMX_SWITCHES_LENGTH, Fixture, FixtureKind, STAGE};
use crate::{scripts, usb};

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
    /// Index of the moving head gobo, `None` for open
    pub gobo: Option<usize>,
    /// Focus, big to small
    pub focus: f32,
    /// Index of the running built-in movement macro, `None` to stand still
    pub movement: Option<usize>,
    pub movement_speed: f32,
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
        gobo: None,
        focus: 0.0,
        movement: None,
        movement_speed: 0.5,
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
            FixtureProp::Gobo(gobo) => self.gobo = gobo,
            FixtureProp::Focus(focus) => self.focus = focus,
            FixtureProp::Movement(movement) => self.movement = movement,
            FixtureProp::MovementSpeed(movement_speed) => self.movement_speed = movement_speed,
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
    Gobo(Option<usize>),
    Focus(f32),
    Movement(Option<usize>),
    MovementSpeed(f32),
}

// MARK: DmxState
#[derive(Clone)]
pub(crate) struct DmxState {
    pub is_running: bool,
    /// Performing on the stage page, changes to the stage folder are picked up afterwards
    pub freeze: bool,
    pub mode: Mode,
    pub tempo: Tempo,
    pub fixtures: BTreeMap<u32, FixtureState>,
    pub running_scripts: BTreeSet<String>,
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
    freeze: false,
    mode: Mode::Manual,
    tempo: Tempo {
        bpm: DEFAULT_BPM,
        downbeat: None,
    },
    fixtures: BTreeMap::new(),
    running_scripts: BTreeSet::new(),
});

// MARK: FixtureOutput
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum FixtureOutput {
    /// Output color, `None` when the fixture runs its music program
    Rgb(Option<Color>),
    /// Output color and shape of the gobo in the beam
    MovingHead {
        color: Option<Color>,
        gobo: Option<&'static str>,
    },
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
#[derive(Debug, Clone, Copy, PartialEq)]
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

        self.toggle_color(state)
    }

    fn toggle_color(&self, state: &FixtureState) -> Color {
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

const SHUTTER_OPEN: u8 = 255;
const LAMP_ON: u8 = 90;

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
    let mut engine = scripts::Engine::new().expect("Failed to start the script engine");
    let use_colors = io::stdout().is_terminal()
        && env::var_os("NO_COLOR").is_none()
        && env::var_os("CI").is_none();

    loop {
        log_connection_event(connection.poll(Instant::now()));
        let tempo = {
            let dmx_state = DMX_STATE.lock().expect("Failed to lock DMX state");
            dmx_state.is_running.then_some(dmx_state.tempo)
        };
        let Some(tempo) = tempo else {
            // FIXME: Create async framework don't do micro sleeps
            sleep(Duration::from_millis(100));
            next_frame = Instant::now();
            continue;
        };

        // Frames are timed by fixed deadlines so send jitter doesn't shift the beat clock
        let frame_time = next_frame;
        let elapsed = frame_time.saturating_duration_since(tempo.downbeat.unwrap_or(clock_start));
        let clock = Clock {
            tempo,
            beat: tempo.beats(elapsed),
            frame: (elapsed.as_secs_f64() * dmx_fps as f64) as u64,
            fps: dmx_fps as f64,
        };
        let stage = STAGE
            .lock()
            .expect("Failed to lock stage")
            .as_ref()
            .expect("Stage not loaded")
            .stage
            .clone();
        let fixtures = &stage.fixtures;

        // Scripts change the fixture states before they are sent
        engine.update(clock.beat, tempo, &stage);
        let dmx_state = DMX_STATE.lock().expect("Failed to lock DMX state").clone();

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
        let outputs = render_fixtures(fixtures, &dmx_state, &clock, &mut dmx);

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

/// Render a frame without hardware or IPC so mode transitions can be checked independently.
fn render_fixtures(
    fixtures: &[Fixture],
    dmx_state: &DmxState,
    clock: &Clock,
    dmx: &mut [u8],
) -> BTreeMap<u32, FixtureOutput> {
    let mut outputs = BTreeMap::new();
    for fixture in fixtures {
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
            FixtureKind::Rgb | FixtureKind::MovingHead => {
                let state = dmx_state.fixture(fixture.id);

                // Built-in presets replace the color channels only in manual mode
                let preset = profile.presets.and_then(|presets| {
                    let index = match dmx_state.mode {
                        Mode::Manual => state.preset,
                        Mode::Black | Mode::Auto => None,
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

                let mut color = clock.color(&state);
                let is_open = color != Color::BLACK;

                // Moving heads use the closest color wheel slot and close the shutter for black,
                // the wheel keeps the current toggle color while the strobe closes the shutter
                let head = profile.moving_head;
                let wheel = head.map(|head| {
                    let color = clock.toggle_color(&state);
                    head.wheel_color(if color == Color::BLACK {
                        state.color
                    } else {
                        color
                    })
                });
                if let Some(wheel) = wheel
                    && is_open
                {
                    color = wheel.color;
                }
                let gobo = head.and_then(|head| head.gobos.get(state.gobo?));
                let movement = head
                    .and_then(|head| head.movements.get(state.movement?))
                    .map_or(0, |movement| movement.value);
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
                        (Channel::ColorWheel, Mode::Manual) => wheel.map_or(0, |wheel| wheel.value),
                        (Channel::Gobo, Mode::Manual) => gobo.map_or(0, |gobo| gobo.value),
                        (Channel::Focus, _) => (state.focus * 255.0) as u8,
                        (Channel::Movement, Mode::Manual) => movement,
                        // The fixture moves fast to slow, the slider slow to fast
                        (Channel::MovementSpeed, _) => ((1.0 - state.movement_speed) * 255.0) as u8,
                        (Channel::Shutter, Mode::Manual) if is_open => SHUTTER_OPEN,
                        (Channel::LampOn, _) => LAMP_ON,
                        // Moving heads cycle colors and gobos and move to the sound
                        (Channel::ColorWheel, Mode::Auto) => head.map_or(0, |head| head.auto_color),
                        (Channel::Gobo, Mode::Auto) => head.map_or(0, |head| head.auto_gobo),
                        (Channel::Movement, Mode::Auto) => {
                            head.map_or(0, |head| head.auto_movement)
                        }
                        (Channel::Shutter, Mode::Auto) => SHUTTER_OPEN,
                        (Channel::Dimmer, Mode::Auto) => 255,
                        // Without a movement macro moving heads point to the center
                        (Channel::Pan | Channel::Tilt, _) => 128,
                        _ => 0,
                    };
                }
                let output_color = match dmx_state.mode {
                    Mode::Black => Some(Color::BLACK),
                    Mode::Manual => Some(color_with_intensity),
                    Mode::Auto
                        if head.is_some()
                            || profile
                                .channels
                                .iter()
                                .any(|channel| matches!(channel, Channel::Music(_))) =>
                    {
                        None
                    }
                    Mode::Auto => Some(Color::BLACK),
                };
                outputs.insert(
                    fixture.id,
                    if head.is_some() {
                        FixtureOutput::MovingHead {
                            color: output_color,
                            gobo: match dmx_state.mode {
                                Mode::Manual => gobo.map(|gobo| gobo.shape),
                                _ => None,
                            },
                        }
                    } else {
                        FixtureOutput::Rgb(output_color)
                    },
                );
            }
        }
    }
    outputs
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
    use crate::stage::FixtureType;

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

    fn render_fixture(
        fixture_type: FixtureType,
        mode: Mode,
        fixture_state: FixtureState,
        clock: Clock,
    ) -> (Vec<u8>, FixtureOutput) {
        let fixture = Fixture {
            id: 1,
            name: String::new(),
            r#type: fixture_type,
            addr: 3,
            x: 0,
            y: 0,
            switches: None,
        };
        let state = DmxState {
            is_running: true,
            freeze: false,
            mode,
            tempo: clock.tempo,
            fixtures: BTreeMap::from([(1, fixture_state)]),
            running_scripts: BTreeSet::new(),
        };
        let count = fixture_type.channel_count();
        let mut channels = vec![42; count + 4];
        let outputs = render_fixtures(&[fixture], &state, &clock, &mut channels);
        assert_eq!(&channels[..2], &[42; 2]);
        assert_eq!(&channels[count + 2..], &[42; 2]);
        (channels[2..count + 2].to_vec(), outputs[&1].clone())
    }

    #[test]
    fn rgb_profiles_render_colors_dimming_toggles_and_strobes() {
        use FixtureType::*;
        // RGB indices and master dimmer indices from each fixture's DMX chart.
        for (fixture_type, rgb, dimmer) in [
            (AmericanDJP56Led, [0, 1, 2], None),
            (AmericanDJMegaTripar, [0, 1, 2], Some(6)),
            (AyraCompar10, [2, 3, 4], Some(0)),
            (AyraCompar20, [2, 3, 4], Some(0)),
            (JbSystemsTubeled, [1, 2, 3], None),
        ] {
            for color in [
                0x000000, 0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0xff00ff, 0x00ffff, 0xffffff,
                0x123456,
            ] {
                let color = Color::from_u32(color);
                for intensity in [0.0, 0.5, 1.0] {
                    let state = FixtureState {
                        color,
                        intensity,
                        ..FixtureState::DEFAULT
                    };
                    let (channels, output) =
                        render_fixture(fixture_type, Mode::Manual, state, clock(0.0, 0));
                    let mut expected = vec![0; fixture_type.channel_count()];
                    let rendered = color.apply_intensity(intensity);
                    let channel_color = if dimmer.is_some() { color } else { rendered };
                    for (index, value) in
                        rgb.into_iter()
                            .zip([channel_color.r, channel_color.g, channel_color.b])
                    {
                        expected[index] = value;
                    }
                    if let Some(index) = dimmer {
                        expected[index] = (intensity * 255.0) as u8;
                    }
                    assert_eq!(channels, expected, "{fixture_type:?}, {color}, {intensity}");
                    assert_eq!(output, FixtureOutput::Rgb(Some(rendered)));
                }
            }
            let state = FixtureState {
                color: RED,
                toggle_color: BLUE,
                toggle_speed: Some(1.0),
                strobe_speed: Some(0.25),
                ..FixtureState::DEFAULT
            };
            for (beat, frame, color) in [(0.5, 0, RED), (1.5, 0, BLUE), (1.5, 3, Color::BLACK)] {
                let (channels, output) =
                    render_fixture(fixture_type, Mode::Manual, state, clock(beat, frame));
                assert_eq!(
                    rgb.map(|index| channels[index]),
                    [color.r, color.g, color.b]
                );
                assert_eq!(output, FixtureOutput::Rgb(Some(color)));
            }
        }
    }

    #[test]
    fn rgb_auto_and_blackout_match_fixture_dmx_charts() {
        use FixtureType::*;
        for (fixture_type, expected) in [
            (AmericanDJP56Led, vec![0, 0, 0, 0, 0, 224]),
            (AmericanDJMegaTripar, vec![0, 0, 0, 0, 255, 240, 255]),
            (AyraCompar10, vec![255, 0, 0, 0, 0, 0, 0, 255]),
            (AyraCompar20, vec![255, 0, 0, 0, 0, 255]),
            (JbSystemsTubeled, vec![0; 4]),
        ] {
            let state = FixtureState {
                color: RED,
                intensity: 0.0,
                preset: Some(33),
                ..FixtureState::DEFAULT
            };
            let (channels, _) = render_fixture(fixture_type, Mode::Auto, state, clock(0.0, 0));
            assert_eq!(channels, expected, "{fixture_type:?}");
            let (channels, output) =
                render_fixture(fixture_type, Mode::Black, state, clock(0.0, 0));
            assert_eq!(
                channels,
                vec![0; fixture_type.channel_count()],
                "{fixture_type:?}"
            );
            assert_eq!(output, FixtureOutput::Rgb(Some(Color::BLACK)));
        }
    }

    #[test]
    fn moving_head_holds_toggle_color_during_strobe_blackout() {
        let state = FixtureState {
            color: RED,
            toggle_color: Color::from_u32(0x00ff00),
            toggle_speed: Some(1.0),
            strobe_speed: Some(0.25),
            intensity: 0.5,
            ..FixtureState::DEFAULT
        };
        let (on, _) = render_fixture(
            FixtureType::ChauvetIntimidatorBeam140SR,
            Mode::Manual,
            state,
            clock(1.5, 0),
        );
        let (off, output) = render_fixture(
            FixtureType::ChauvetIntimidatorBeam140SR,
            Mode::Manual,
            state,
            clock(1.5, 3),
        );
        assert_eq!(on, [128, 0, 128, 0, 127, 17, 0, 0, 0, 127, 255, 0, 90, 0]);
        assert_eq!(off[5], on[5]);
        assert_eq!(off[10], 0);
        assert_eq!(
            output,
            FixtureOutput::MovingHead {
                color: Some(Color::BLACK),
                gobo: None
            }
        );
        let (auto, _) = render_fixture(
            FixtureType::ChauvetIntimidatorBeam140SR,
            Mode::Auto,
            state,
            clock(1.5, 3),
        );
        assert_eq!(
            auto,
            [128, 0, 128, 0, 127, 160, 160, 0, 0, 255, 255, 0, 90, 143]
        );
        let (black, _) = render_fixture(
            FixtureType::ChauvetIntimidatorBeam140SR,
            Mode::Black,
            state,
            clock(1.5, 0),
        );
        assert_eq!(black[9], 0);
        assert_eq!(black[10], 0);
    }

    #[test]
    fn auto_mode_lights_music_fixtures_and_blacks_out_tubes() {
        let clock = clock(0.0, 0);
        let fixtures = [FixtureType::AyraCompar10, FixtureType::JbSystemsTubeled]
            .into_iter()
            .enumerate()
            .map(|(index, r#type)| Fixture {
                id: index as u32 + 1,
                name: String::new(),
                r#type,
                addr: index * 8 + 1,
                x: 0,
                y: 0,
                switches: None,
            })
            .collect::<Vec<_>>();
        let mut state = DmxState {
            is_running: true,
            freeze: false,
            mode: Mode::Auto,
            tempo: clock.tempo,
            fixtures: BTreeMap::from([
                (
                    1,
                    FixtureState {
                        intensity: 0.0,
                        ..FixtureState::DEFAULT
                    },
                ),
                (
                    2,
                    FixtureState {
                        color: RED,
                        preset: Some(33),
                        ..FixtureState::DEFAULT
                    },
                ),
            ]),
            running_scripts: BTreeSet::new(),
        };
        let mut channels = [0; 12];
        let outputs = render_fixtures(&fixtures, &state, &clock, &mut channels);
        assert_eq!(&channels[..8], &[255, 0, 0, 0, 0, 0, 0, 255]);
        assert_eq!(&channels[8..], &[0; 4]);
        assert_eq!(outputs[&1], FixtureOutput::Rgb(None));
        assert_eq!(outputs[&2], FixtureOutput::Rgb(Some(Color::BLACK)));

        // The selected tube chase is preserved for returning to manual mode.
        state.mode = Mode::Manual;
        render_fixtures(&fixtures, &state, &clock, &mut channels);
        assert_eq!(&channels[8..], &[198, 127, 0, 0]);

        // Direct RGB must clear the program selector and replace the speed channel with red.
        state
            .fixtures
            .get_mut(&2)
            .expect("Tube state missing")
            .preset = None;
        render_fixtures(&fixtures, &state, &clock, &mut channels);
        assert_eq!(&channels[8..], &[0, 255, 0, 0]);
        state
            .fixtures
            .get_mut(&2)
            .expect("Tube state missing")
            .preset = Some(33);
        render_fixtures(&fixtures, &state, &clock, &mut channels);
        assert_eq!(&channels[8..], &[198, 127, 0, 0]);

        // Returning from a running chase to auto must clear every tube channel.
        state.mode = Mode::Auto;
        render_fixtures(&fixtures, &state, &clock, &mut channels);
        assert_eq!(&channels[8..], &[0; 4]);

        state.mode = Mode::Black;
        render_fixtures(&fixtures, &state, &clock, &mut channels);
        assert_eq!(channels, [0; 12]);
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
