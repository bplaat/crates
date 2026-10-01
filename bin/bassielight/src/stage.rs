/*
 * Copyright (c) 2025 Leonard van der Plaat
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::dmx::Color;

pub(crate) const DMX_SWITCHES_LENGTH: usize = 4;

// MARK: FixtureType
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum FixtureType {
    #[serde(rename = "american_dj_p56led")]
    AmericanDJP56Led,
    #[serde(rename = "american_dj_mega_tripar")]
    AmericanDJMegaTripar,
    #[serde(rename = "ayra_compar_10")]
    AyraCompar10,
    #[serde(rename = "ayra_compar_20")]
    AyraCompar20,
    #[serde(rename = "showtec_multidim_mkii")]
    ShowtecMultidimMKII,
    #[serde(rename = "showtec_titan_strobe")]
    ShowtecTitanStrobe,
    #[serde(rename = "jb_systems_tubeled")]
    JbSystemsTubeled,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum FixtureKind {
    Rgb,
    Switch,
    Strobe,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub(crate) enum Channel {
    Red,
    Green,
    Blue,
    Dimmer,
    /// Value that starts the built-in music program
    Music(u8),
    Switch,
    /// Strobe flash or preset speed, slow to fast
    Speed,
    /// Built-in preset selection
    Preset,
    Unused,
}

#[derive(Debug, Serialize)]
pub(crate) struct FixtureProfile {
    #[serde(rename = "type")]
    pub r#type: FixtureType,
    pub name: &'static str,
    pub kind: FixtureKind,
    /// Front size in millimeters, width and height
    pub size: [u32; 2],
    pub channels: &'static [Channel],
    /// Built-in presets the fixture can run instead of its channels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presets: Option<&'static Presets>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Presets {
    /// Channel layout while a preset runs
    pub channels: &'static [Channel],
    pub list: &'static [Preset],
    /// Preset that runs in auto mode
    #[serde(skip)]
    pub auto: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct Preset {
    pub name: &'static str,
    #[serde(skip)]
    pub value: u8,
    /// Color of static presets, shown in the visualization
    pub color: Option<Color>,
}

const fn preset(name: &'static str, value: u8) -> Preset {
    Preset {
        name,
        value,
        color: None,
    }
}

const fn static_preset(name: &'static str, value: u8, color: u32) -> Preset {
    Preset {
        name,
        value,
        color: Some(Color::from_u32(color)),
    }
}

/// The TUBELED controller selects a preset when channel 1 is not zero, otherwise channels 2 to 4
/// are the red, green and blue of all connected tubes
const TUBELED_PRESETS: Presets = Presets {
    channels: &[
        Channel::Preset,
        Channel::Speed,
        Channel::Unused,
        Channel::Unused,
    ],
    list: &[
        static_preset("Blackout", 1, 0x000000),
        static_preset("Static red", 6, 0xff0000),
        static_preset("Static green", 12, 0x00ff00),
        static_preset("Static yellow", 18, 0xffff00),
        static_preset("Static blue", 24, 0x0000ff),
        static_preset("Static purple", 30, 0xff00ff),
        static_preset("Static cyan", 36, 0x00ffff),
        static_preset("Static white", 42, 0xffffff),
        preset("Fast change", 48),
        preset("Slow flow 1", 54),
        preset("Fast flow 1", 60),
        preset("Fast flow 2", 66),
        preset("Black run", 72),
        preset("Roll chase", 78),
        preset("Roll color", 84),
        preset("Color 1/4", 90),
        preset("Color 1 1/4", 96),
        preset("Color 1 1/2", 102),
        preset("Color flash", 108),
        preset("Blue & white flow", 114),
        preset("Red & green flow", 120),
        preset("Green & blue flow", 126),
        preset("Red & blue flow", 132),
        preset("Red & green chase 1", 138),
        preset("Red & green chase 2", 144),
        preset("Red & blue chase 1", 150),
        preset("Red & blue chase 2", 156),
        preset("Red & white chase 1", 162),
        preset("Red & white chase 2", 168),
        preset("Blue & green chase 1", 174),
        preset("Blue & green chase 2", 180),
        preset("White & green chase 1", 186),
        preset("White & green chase 2", 192),
        preset("Rainbow chase 1", 198),
        preset("Rainbow chase 2", 204),
        preset("Rainbow chase 3", 210),
        preset("Rainbow chase 4", 216),
        preset("Rainbow chase 8", 222),
    ],
    // The controller has no sound mode, run a rainbow chase instead
    auto: 33,
};

impl FixtureType {
    pub(crate) const ALL: [FixtureType; 7] = [
        FixtureType::AmericanDJP56Led,
        FixtureType::AmericanDJMegaTripar,
        FixtureType::AyraCompar10,
        FixtureType::AyraCompar20,
        FixtureType::ShowtecMultidimMKII,
        FixtureType::ShowtecTitanStrobe,
        FixtureType::JbSystemsTubeled,
    ];

    pub(crate) const fn profile(self) -> &'static FixtureProfile {
        use Channel::*;
        match self {
            FixtureType::AmericanDJP56Led => &FixtureProfile {
                r#type: FixtureType::AmericanDJP56Led,
                name: "American DJ P56 LED",
                kind: FixtureKind::Rgb,
                size: [227, 227],
                channels: &[Red, Green, Blue, Unused, Unused, Music(224)],
                presets: None,
            },
            FixtureType::AmericanDJMegaTripar => &FixtureProfile {
                r#type: FixtureType::AmericanDJMegaTripar,
                name: "American DJ Mega Tripar",
                kind: FixtureKind::Rgb,
                size: [225, 220],
                channels: &[Red, Green, Blue, Unused, Unused, Music(240), Dimmer],
                presets: None,
            },
            FixtureType::AyraCompar10 => &FixtureProfile {
                r#type: FixtureType::AyraCompar10,
                name: "Ayra Compar 10",
                kind: FixtureKind::Rgb,
                size: [170, 170],
                channels: &[Dimmer, Unused, Red, Green, Blue, Unused, Unused, Music(221)],
                presets: None,
            },
            FixtureType::AyraCompar20 => &FixtureProfile {
                r#type: FixtureType::AyraCompar20,
                name: "Ayra Compar 20",
                kind: FixtureKind::Rgb,
                // Not in the specs, estimated
                size: [250, 250],
                channels: &[Dimmer, Unused, Red, Green, Blue, Music(221)],
                presets: None,
            },
            FixtureType::ShowtecMultidimMKII => &FixtureProfile {
                r#type: FixtureType::ShowtecMultidimMKII,
                name: "SHOWTEC Multidim MKII",
                kind: FixtureKind::Switch,
                size: [212, 188],
                channels: &[Switch; DMX_SWITCHES_LENGTH],
                presets: None,
            },
            FixtureType::ShowtecTitanStrobe => &FixtureProfile {
                r#type: FixtureType::ShowtecTitanStrobe,
                name: "SHOWTEC Titan Strobe",
                kind: FixtureKind::Strobe,
                size: [495, 265],
                channels: &[Speed, Dimmer],
                presets: None,
            },
            FixtureType::JbSystemsTubeled => &FixtureProfile {
                r#type: FixtureType::JbSystemsTubeled,
                name: "JB Systems TUBELED",
                kind: FixtureKind::Rgb,
                // One 1m tube, the diameter is estimated
                size: [1000, 40],
                channels: &[Unused, Red, Green, Blue],
                presets: Some(&TUBELED_PRESETS),
            },
        }
    }

    pub(crate) const fn channel_count(self) -> usize {
        self.profile().channels.len()
    }
}

// MARK: Stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Fixture {
    #[serde(default)]
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: FixtureType,
    pub addr: usize,
    /// Position in the room in centimeters
    #[serde(default)]
    pub x: u32,
    #[serde(default)]
    pub y: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switches: Option<Vec<String>>,
}

/// Rectangular room dimensions in centimeters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Room {
    pub width: u32,
    pub height: u32,
}

impl Default for Room {
    fn default() -> Self {
        Room {
            width: 800,
            height: 500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Group {
    pub id: u32,
    pub name: String,
    pub fixtures: Vec<u32>,
    /// Don't draw the bounding rect, the group is then selected with a button
    #[serde(default)]
    pub hide_outline: bool,
}

/// Rect in the room that selects a group or fixture when pressed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Button {
    pub id: u32,
    /// Shows the target name when empty
    pub label: String,
    /// Center position and size in centimeters
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub target: Option<ButtonTarget>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(crate) enum ButtonTarget {
    Fixture { id: u32 },
    Group { id: u32 },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Stage {
    pub room: Room,
    pub fixtures: Vec<Fixture>,
    pub groups: Vec<Group>,
    pub buttons: Vec<Button>,
}

/// The opened stage file, edits are saved to it right away
pub(crate) struct OpenStage {
    pub path: PathBuf,
    pub stage: Stage,
}

pub(crate) static STAGE: Mutex<Option<OpenStage>> = Mutex::new(None);

impl Stage {
    pub(crate) fn load(path: &Path, dmx_length: usize) -> Result<Stage, String> {
        let file = File::open(path).map_err(|error| error.to_string())?;
        let mut stage: Stage = serde_json::from_reader(io::BufReader::new(file))
            .map_err(|error| format!("Invalid stage file: {error}"))?;
        stage.assign_fixture_ids();
        stage.validate(dmx_length)?;
        Ok(stage)
    }

    pub(crate) fn save(&self, path: &Path) -> io::Result<()> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self).map_err(io::Error::other)
    }

    /// Give fixtures from stages without (unique) ids sequential ids
    fn assign_fixture_ids(&mut self) {
        let mut ids = HashSet::new();
        if self.fixtures.iter().all(|f| f.id != 0 && ids.insert(f.id)) {
            return;
        }
        for (index, fixture) in self.fixtures.iter_mut().enumerate() {
            fixture.id = index as u32 + 1;
        }
    }

    pub(crate) fn validate(&self, dmx_length: usize) -> Result<(), String> {
        let mut ids = HashSet::new();
        for fixture in &self.fixtures {
            if !ids.insert(fixture.id) {
                return Err(format!("Duplicate fixture id {}", fixture.id));
            }
            if fixture.addr == 0 || fixture.addr - 1 + fixture.r#type.channel_count() > dmx_length {
                return Err(format!("Fixture {} address is out of range", fixture.name));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(id: u32, addr: usize) -> Fixture {
        Fixture {
            id,
            name: format!("Fixture {id}"),
            r#type: FixtureType::AmericanDJP56Led,
            addr,
            x: 0,
            y: 0,
            switches: None,
        }
    }

    #[test]
    fn assigns_ids_to_legacy_fixtures() {
        let mut stage: Stage = serde_json::from_str(
            r#"{"fixtures":[{"name":"A","type":"ayra_compar_10","addr":1},{"name":"B","type":"ayra_compar_20","addr":9}]}"#,
        )
        .expect("Failed to parse legacy stage");
        stage.assign_fixture_ids();
        assert_eq!(stage.fixtures[0].id, 1);
        assert_eq!(stage.fixtures[1].id, 2);
        assert_eq!(stage.room.width, Room::default().width);
    }

    #[test]
    fn serializes_fixture_profiles() {
        let json = serde_json::to_string(FixtureType::AyraCompar20.profile())
            .expect("Failed to serialize fixture profile");
        assert_eq!(
            json,
            r#"{"type":"ayra_compar_20","name":"Ayra Compar 20","kind":"rgb","size":[250,250],"channels":[{"type":"dimmer"},{"type":"unused"},{"type":"red"},{"type":"green"},{"type":"blue"},{"type":"music","value":221}]}"#
        );
        assert_eq!(FixtureType::ShowtecMultidimMKII.channel_count(), 4);
    }

    #[test]
    fn serializes_tubeled_presets() {
        let json = serde_json::to_value(FixtureType::JbSystemsTubeled.profile())
            .expect("Failed to serialize fixture profile");
        let list = json["presets"]["list"]
            .as_array()
            .expect("TUBELED should have presets");
        assert_eq!(list.len(), 38);
        assert_eq!(
            list[1],
            serde_json::json!({ "name": "Static red", "color": 0xff0000 })
        );
        assert_eq!(list[37]["color"], serde_json::Value::Null);
        assert_eq!(json["presets"]["channels"][0]["type"], "preset");
        assert!(FixtureType::AyraCompar20.profile().presets.is_none());
    }

    #[test]
    fn parses_buttons_and_hidden_outlines() {
        let stage: Stage = serde_json::from_str(
            r#"{"groups":[{"id":1,"name":"Odd","fixtures":[1],"hide_outline":true}],
            "buttons":[{"id":1,"label":"","x":100,"y":50,"width":120,"height":40,"target":{"type":"group","id":1}}]}"#,
        )
        .expect("Failed to parse stage");
        assert!(stage.groups[0].hide_outline);
        assert!(matches!(
            stage.buttons[0].target,
            Some(ButtonTarget::Group { id: 1 })
        ));
    }

    #[test]
    fn validates_fixture_addresses_and_ids() {
        let mut stage = Stage {
            fixtures: vec![fixture(1, 1), fixture(2, 507)],
            ..Stage::default()
        };
        assert!(stage.validate(512).is_ok());
        stage.fixtures[1].addr = 508;
        assert!(stage.validate(512).is_err());
        stage.fixtures[1].addr = 0;
        assert!(stage.validate(512).is_err());
        stage.fixtures[1] = fixture(1, 7);
        assert!(stage.validate(512).is_err());
    }
}
