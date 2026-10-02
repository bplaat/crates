/*
 * Copyright (c) 2025 Leonard van der Plaat
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::warn;
use serde::{Deserialize, Deserializer, Serialize};

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
    #[serde(rename = "chauvet_intimidator_beam_140sr")]
    ChauvetIntimidatorBeam140SR,
    #[serde(rename = "chauvet_amhaze_stadium")]
    ChauvetAmhazeStadium,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum FixtureKind {
    Rgb,
    Switch,
    Strobe,
    MovingHead,
    Haze,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub(crate) enum Channel {
    Red,
    Green,
    Blue,
    Dimmer,
    /// Value used by the built-in music program
    Music(u8),
    Switch,
    /// Strobe flash or preset speed, slow to fast
    Speed,
    /// Built-in preset selection
    Preset,
    Pan,
    Tilt,
    /// Color wheel slot closest to the color
    ColorWheel,
    Gobo,
    /// Beam focus, big to small
    Focus,
    /// Built-in movement macro
    Movement,
    /// Pan, tilt and movement macro speed, fast to slow
    MovementSpeed,
    /// Closed for black, open otherwise
    Shutter,
    /// Turn on the discharge lamp through the fixture function channel
    LampOn,
    /// Haze output volume, low to high
    Haze,
    /// Fan speed of a hazer, slow to fast
    Fan,
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
    #[serde(rename = "movingHead", skip_serializing_if = "Option::is_none")]
    pub moving_head: Option<&'static MovingHead>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MovingHead {
    #[serde(skip)]
    pub color_wheel: &'static [WheelColor],
    pub gobos: &'static [Gobo],
    pub movements: &'static [Movement],
    /// Color wheel, gobo wheel and movement values for auto mode
    #[serde(skip)]
    pub auto_color: u8,
    #[serde(skip)]
    pub auto_gobo: u8,
    #[serde(skip)]
    pub auto_movement: u8,
}

#[derive(Debug)]
pub(crate) struct WheelColor {
    pub value: u8,
    /// Approximate color of the light through the filter
    pub color: Color,
}

#[derive(Debug, Serialize)]
pub(crate) struct Gobo {
    pub name: &'static str,
    /// Pattern the visualization draws
    pub shape: &'static str,
    #[serde(skip)]
    pub value: u8,
}

#[derive(Debug, Serialize)]
pub(crate) struct Movement {
    pub name: &'static str,
    #[serde(skip)]
    pub value: u8,
}

impl MovingHead {
    /// Color wheel slot that looks most like the color
    pub(crate) fn wheel_color(&self, color: Color) -> &WheelColor {
        let distance = |slot: &&WheelColor| {
            let [r, g, b] = [
                slot.color.r as i32 - color.r as i32,
                slot.color.g as i32 - color.g as i32,
                slot.color.b as i32 - color.b as i32,
            ];
            r * r + g * g + b * b
        };
        self.color_wheel
            .iter()
            .min_by_key(distance)
            .expect("Color wheel is empty")
    }
}

const fn wheel_color(value: u8, color: u32) -> WheelColor {
    WheelColor {
        value,
        color: Color::from_u32(color),
    }
}

const fn gobo(name: &'static str, shape: &'static str, value: u8) -> Gobo {
    Gobo { name, shape, value }
}

const fn movement(name: &'static str, value: u8) -> Movement {
    Movement { name, value }
}

const INTIMIDATOR_BEAM_140SR: MovingHead = MovingHead {
    color_wheel: &[
        wheel_color(1, 0xffffff),  // White
        wheel_color(5, 0xff0000),  // Red
        wheel_color(9, 0xffd000),  // Yellow
        wheel_color(13, 0x40c0ff), // Sky blue
        wheel_color(17, 0x00ff00), // Green
        wheel_color(21, 0xa0ff00), // Lime green
        wheel_color(25, 0xfff0d8), // Natural white
        wheel_color(29, 0xb080ff), // Lavender
        wheel_color(33, 0xffff60), // Canary yellow
        wheel_color(37, 0xff00ff), // Magenta
        wheel_color(41, 0x00ffff), // Cyan
        wheel_color(45, 0xffe5b0), // Warm white
        wheel_color(49, 0xc080ff), // Lilac
        wheel_color(53, 0xb0d8ff), // Pale blue
        wheel_color(59, 0x6000ff), // Ultraviolet
    ],
    // Shapes from the Beam 140SR gobo wheel pictures in the manual
    gobos: &[
        gobo("Dot", "dot", 5),
        gobo("Tiny ring", "ringSmall", 9),
        gobo("Small ring", "ringSmall", 13),
        gobo("Medium ring", "ringMedium", 17),
        gobo("Large ring", "ringLarge", 21),
        gobo("Shards", "shards", 25),
        gobo("Cross", "cross", 29),
        gobo("Dot ring", "dotRing", 33),
        gobo("Swirl", "vortex", 37),
        gobo("Flower", "flower", 41),
        gobo("Dot disc", "dotDisc", 45),
        gobo("Squares", "squares", 49),
        gobo("Wave", "wave", 53),
        gobo("Big dots", "dotScatter", 57),
        gobo("Dot cloud", "dotCloud", 61),
        gobo("Arrows", "arrows", 65),
        gobo("Dot line", "dotLine", 69),
    ],
    movements: &[
        movement("Macro 1", 15),
        movement("Macro 2", 31),
        movement("Macro 3", 47),
        movement("Macro 4", 63),
        movement("Macro 5", 79),
        movement("Macro 6", 95),
        movement("Macro 7", 111),
        movement("Macro 8", 127),
        movement("Sound macro 1", 143),
        movement("Sound macro 2", 159),
        movement("Sound macro 3", 175),
        movement("Sound macro 4", 191),
        movement("Sound macro 5", 207),
        movement("Sound macro 6", 223),
        movement("Sound macro 7", 239),
        movement("Sound macro 8", 251),
    ],
    // Rainbow color cycling, gobo cycling and sound active movement
    auto_color: 160,
    auto_gobo: 160,
    auto_movement: 143,
};

#[derive(Debug, Serialize)]
pub(crate) struct Presets {
    /// Channel layout while a preset runs
    pub channels: &'static [Channel],
    pub list: &'static [Preset],
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
};

impl FixtureType {
    pub(crate) const ALL: [FixtureType; 9] = [
        FixtureType::AmericanDJP56Led,
        FixtureType::AmericanDJMegaTripar,
        FixtureType::AyraCompar10,
        FixtureType::AyraCompar20,
        FixtureType::ShowtecMultidimMKII,
        FixtureType::ShowtecTitanStrobe,
        FixtureType::JbSystemsTubeled,
        FixtureType::ChauvetIntimidatorBeam140SR,
        FixtureType::ChauvetAmhazeStadium,
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
                moving_head: None,
            },
            FixtureType::AmericanDJMegaTripar => &FixtureProfile {
                r#type: FixtureType::AmericanDJMegaTripar,
                name: "American DJ Mega Tripar",
                kind: FixtureKind::Rgb,
                size: [225, 220],
                // In sound mode channel 5 controls microphone sensitivity, not strobing
                channels: &[Red, Green, Blue, Unused, Music(255), Music(240), Dimmer],
                presets: None,
                moving_head: None,
            },
            FixtureType::AyraCompar10 => &FixtureProfile {
                r#type: FixtureType::AyraCompar10,
                name: "Ayra Compar 10",
                kind: FixtureKind::Rgb,
                size: [170, 170],
                // 8-channel mode, with full microphone sensitivity in sound mode
                channels: &[Dimmer, Unused, Red, Green, Blue, Unused, Unused, Music(255)],
                presets: None,
                moving_head: None,
            },
            FixtureType::AyraCompar20 => &FixtureProfile {
                r#type: FixtureType::AyraCompar20,
                name: "Ayra Compar 20",
                kind: FixtureKind::Rgb,
                // Not in the specs, estimated
                size: [250, 250],
                channels: &[Dimmer, Unused, Red, Green, Blue, Music(255)],
                presets: None,
                moving_head: None,
            },
            FixtureType::ShowtecMultidimMKII => &FixtureProfile {
                r#type: FixtureType::ShowtecMultidimMKII,
                name: "SHOWTEC Multidim MKII",
                kind: FixtureKind::Switch,
                size: [212, 188],
                channels: &[Switch; DMX_SWITCHES_LENGTH],
                presets: None,
                moving_head: None,
            },
            FixtureType::ShowtecTitanStrobe => &FixtureProfile {
                r#type: FixtureType::ShowtecTitanStrobe,
                name: "SHOWTEC Titan Strobe",
                kind: FixtureKind::Strobe,
                size: [495, 265],
                channels: &[Speed, Dimmer],
                presets: None,
                moving_head: None,
            },
            FixtureType::JbSystemsTubeled => &FixtureProfile {
                r#type: FixtureType::JbSystemsTubeled,
                name: "JB Systems TUBELED",
                kind: FixtureKind::Rgb,
                // One 1m tube, the diameter is estimated
                size: [1000, 40],
                channels: &[Unused, Red, Green, Blue],
                presets: Some(&TUBELED_PRESETS),
                moving_head: None,
            },
            FixtureType::ChauvetIntimidatorBeam140SR => &FixtureProfile {
                r#type: FixtureType::ChauvetIntimidatorBeam140SR,
                name: "Chauvet Intimidator Beam 140SR",
                kind: FixtureKind::MovingHead,
                size: [322, 220],
                // 14 channel mode
                channels: &[
                    Pan,
                    Unused,
                    Tilt,
                    Unused,
                    MovementSpeed,
                    ColorWheel,
                    Gobo,
                    // Prism
                    Unused,
                    Focus,
                    Dimmer,
                    Shutter,
                    Unused,
                    LampOn,
                    Movement,
                ],
                presets: None,
                moving_head: Some(&INTIMIDATOR_BEAM_140SR),
            },
            FixtureType::ChauvetAmhazeStadium => &FixtureProfile {
                r#type: FixtureType::ChauvetAmhazeStadium,
                name: "Chauvet Amhaze Stadium",
                kind: FixtureKind::Haze,
                size: [275, 406],
                channels: &[Fan, Haze],
                presets: None,
                moving_head: None,
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

/// Rect in the room that selects or blacks out its fixtures and groups and toggles its scripts
/// when pressed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Button {
    pub id: u32,
    /// Shows the target names when empty
    pub label: String,
    /// Center position and size in centimeters
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub action: ButtonAction,
    /// Older stages have a single `target`
    #[serde(default, alias = "target", deserialize_with = "one_or_many")]
    pub targets: Vec<ButtonTarget>,
}

/// What pressing a button does with its fixtures and groups
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ButtonAction {
    #[default]
    Select,
    /// Toggle the blackout of the fixtures, they keep their other settings
    Blackout,
}

fn one_or_many<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<ButtonTarget>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(ButtonTarget),
        Many(Vec<ButtonTarget>),
    }
    Ok(match Option::<OneOrMany>::deserialize(deserializer)? {
        None => Vec::new(),
        Some(OneOrMany::One(target)) => vec![target],
        Some(OneOrMany::Many(targets)) => targets,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(crate) enum ButtonTarget {
    Fixture { id: u32 },
    Group { id: u32 },
    Script { name: String },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Stage {
    pub room: Room,
    pub fixtures: Vec<Fixture>,
    pub groups: Vec<Group>,
    pub buttons: Vec<Button>,
}

/// The opened stage folder and its unsaved edits
pub(crate) struct OpenStage {
    pub path: PathBuf,
    pub stage: Stage,
    /// Last read or written stage.json, to notice changes by others
    pub json: String,
    /// Canonical saved contents, used to compare edits independently of file formatting
    pub saved_json: String,
}

pub(crate) static STAGE: Mutex<Option<OpenStage>> = Mutex::new(None);

// MARK: Stage folder
/// A stage is a folder with this extension, holding the stage.json, the scripts folder and docs
/// for AI agents that edit it
pub(crate) const STAGE_EXTENSION: &str = "stage";
const STAGE_FILE: &str = "stage.json";
pub(crate) const SCRIPTS_DIR: &str = "scripts";
const AGENTS_DOCS: &str = include_str!("scripting.md");

impl OpenStage {
    pub(crate) fn open(folder: &Path, dmx_length: usize) -> Result<OpenStage, String> {
        let json = std::fs::read_to_string(folder.join(STAGE_FILE))
            .map_err(|error| format!("Can't read {STAGE_FILE}: {error}"))?;
        let stage = Stage::parse(&json, dmx_length)?;
        if let Err(error) = write_docs(folder) {
            warn!("Can't write stage docs: {error}");
        }
        Ok(OpenStage {
            path: folder.to_path_buf(),
            saved_json: stage.to_json(),
            stage,
            json,
        })
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.stage.to_json() != self.saved_json
    }

    pub(crate) fn save(&mut self) -> io::Result<()> {
        let json = self.stage.to_json();
        std::fs::write(self.path.join(STAGE_FILE), &json)?;
        self.saved_json = json.clone();
        self.json = json;
        Ok(())
    }
}

/// Open a picked path: a stage folder, the stage.json in one, or a single json stage file of
/// older versions which is copied into a new stage folder next to it
pub(crate) fn open_path(path: &Path, dmx_length: usize) -> Result<OpenStage, String> {
    let parent = path.parent().filter(|parent| {
        path.file_name() == Some(STAGE_FILE.as_ref())
            && parent.extension() == Some(STAGE_EXTENSION.as_ref())
    });
    let folder = if path.is_dir() {
        path.to_path_buf()
    } else if let Some(parent) = parent {
        parent.to_path_buf()
    } else {
        let json = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        let stage = Stage::parse(&json, dmx_length)?;
        // The stage.json of older versions becomes the default stage
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy())
            .filter(|stem| stem != "stage")
            .unwrap_or("Default".into());
        let folder = path.with_file_name(format!("{name}.{STAGE_EXTENSION}"));
        if !folder.exists() {
            create_folder(&folder, &stage, None).map_err(|error| error.to_string())?;
        }
        folder
    };
    OpenStage::open(&folder, dmx_length)
}

/// Add the stage extension to a picked path when it's missing
pub(crate) fn with_extension(path: PathBuf) -> PathBuf {
    if path.extension() == Some(STAGE_EXTENSION.as_ref()) {
        path
    } else {
        path.with_extension(STAGE_EXTENSION)
    }
}

/// Create a stage folder with the stage and the scripts of another stage folder
pub(crate) fn create_folder(
    folder: &Path,
    stage: &Stage,
    scripts_from: Option<&Path>,
) -> io::Result<()> {
    let scripts = folder.join(SCRIPTS_DIR);
    std::fs::create_dir_all(&scripts)?;
    std::fs::write(folder.join(STAGE_FILE), stage.to_json())?;
    if let Some(from) = scripts_from {
        crate::scripts::copy_scripts(&from.join(SCRIPTS_DIR), &scripts)?;
    }
    write_docs(folder)
}

/// Docs for AI agents like Claude Code and Codex, so they can edit the stage and its scripts
fn write_docs(folder: &Path) -> io::Result<()> {
    std::fs::write(
        folder.join("AGENTS.md"),
        format!("{AGENTS_DOCS}\n{}", fixture_docs()),
    )?;
    let claude = folder.join("CLAUDE.md");
    if !claude.exists() {
        std::fs::write(claude, "@AGENTS.md\n")?;
    }
    Ok(())
}

/// Reference of all supported fixture types
fn fixture_docs() -> String {
    let mut docs = String::from("## Fixture types\n");
    for profile in FixtureType::ALL.map(FixtureType::profile) {
        let kind = serde_json::to_value(profile.kind).expect("Failed to serialize kind");
        let r#type = serde_json::to_value(profile.r#type).expect("Failed to serialize type");
        docs.push_str(&format!(
            "\n### {}\n\n- `type`: `{}`\n- `kind`: `{}`\n- DMX channels: {}\n",
            profile.name,
            r#type.as_str().unwrap_or_default(),
            kind.as_str().unwrap_or_default(),
            profile.channels.len()
        ));
        let names = |names: Vec<&str>| names.join(", ");
        if let Some(presets) = profile.presets {
            docs.push_str(&format!(
                "- `preset`: {}\n",
                names(presets.list.iter().map(|preset| preset.name).collect())
            ));
        }
        if let Some(head) = profile.moving_head {
            docs.push_str(&format!(
                "- `gobo`: {}\n- `movement`: {}\n",
                names(head.gobos.iter().map(|gobo| gobo.name).collect()),
                names(
                    head.movements
                        .iter()
                        .map(|movement| movement.name)
                        .collect()
                )
            ));
        }
    }
    docs
}

impl Stage {
    pub(crate) fn parse(json: &str, dmx_length: usize) -> Result<Stage, String> {
        let mut stage: Stage =
            serde_json::from_str(json).map_err(|error| format!("Invalid stage file: {error}"))?;
        stage.assign_fixture_ids();
        stage.validate(dmx_length)?;
        Ok(stage)
    }

    pub(crate) fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Failed to serialize stage")
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
            r#"{"type":"ayra_compar_20","name":"Ayra Compar 20","kind":"rgb","size":[250,250],"channels":[{"type":"dimmer"},{"type":"unused"},{"type":"red"},{"type":"green"},{"type":"blue"},{"type":"music","value":255}]}"#
        );
        assert_eq!(FixtureType::ShowtecMultidimMKII.channel_count(), 4);
        assert_eq!(
            serde_json::to_string(&FixtureType::ChauvetIntimidatorBeam140SR).unwrap(),
            r#""chauvet_intimidator_beam_140sr""#
        );
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
            "buttons":[
                {"id":1,"label":"","x":100,"y":50,"width":120,"height":40,"target":{"type":"group","id":1}},
                {"id":2,"label":"","x":100,"y":50,"width":120,"height":40,"target":null},
                {"id":3,"label":"","x":100,"y":50,"width":120,"height":40,"action":"blackout",
                    "targets":[{"type":"group","id":1},{"type":"fixture","id":2}]}
            ]}"#,
        )
        .expect("Failed to parse stage");
        assert!(stage.groups[0].hide_outline);
        assert_eq!(stage.buttons[0].action, ButtonAction::Select);
        assert!(matches!(
            stage.buttons[0].targets[..],
            [ButtonTarget::Group { id: 1 }]
        ));
        assert!(stage.buttons[1].targets.is_empty());
        assert_eq!(stage.buttons[2].action, ButtonAction::Blackout);
        assert!(matches!(
            stage.buttons[2].targets[..],
            [
                ButtonTarget::Group { id: 1 },
                ButtonTarget::Fixture { id: 2 }
            ]
        ));
        let json = stage.to_json();
        assert!(json.contains(r#""targets""#) && !json.contains(r#""target""#));
    }

    #[test]
    fn picks_closest_color_wheel_slot() {
        let profile = FixtureType::ChauvetIntimidatorBeam140SR.profile();
        let head = profile.moving_head.expect("Should be a moving head");
        let slot = |color| head.wheel_color(Color::from_u32(color)).value;
        assert_eq!(slot(0xff0000), 5);
        assert_eq!(slot(0xffffff), 1);
        assert_eq!(slot(0x00ffff), 41);
        assert_eq!(FixtureType::ChauvetIntimidatorBeam140SR.channel_count(), 14);
        assert_eq!(head.gobos.len(), 17);
        assert_eq!(head.gobos[0].value, 5);
        assert_eq!(head.gobos[16].value, 69);
        assert_eq!(head.auto_gobo, 160);
        assert_eq!(head.auto_color, 160);
        assert_eq!(
            profile.channels,
            &[
                Channel::Pan,
                Channel::Unused,
                Channel::Tilt,
                Channel::Unused,
                Channel::MovementSpeed,
                Channel::ColorWheel,
                Channel::Gobo,
                Channel::Unused,
                Channel::Focus,
                Channel::Dimmer,
                Channel::Shutter,
                Channel::Unused,
                Channel::LampOn,
                Channel::Movement,
            ]
        );
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

    #[test]
    fn tracks_unsaved_changes_and_only_clears_after_successful_save() {
        let folder =
            std::env::temp_dir().join(format!("bassielight-dirty-test-{}", std::process::id()));
        std::fs::create_dir_all(&folder).expect("Failed to create test folder");
        let stage = Stage::default();
        // Formatting differences on disk must not mark a newly opened document dirty.
        let json = serde_json::to_string(&stage).expect("Failed to serialize stage");
        std::fs::write(folder.join(STAGE_FILE), &json).expect("Failed to write test stage");
        let mut open = OpenStage::open(&folder, 512).expect("Failed to open test stage");
        assert!(!open.is_dirty());
        open.stage.room.width += 100;
        assert!(open.is_dirty());
        assert_eq!(
            std::fs::read_to_string(folder.join(STAGE_FILE)).expect("Failed to read stage"),
            json
        );
        open.stage.room.width -= 100;
        assert!(!open.is_dirty());
        open.stage.room.width += 100;
        open.save().expect("Failed to save stage");
        assert!(!open.is_dirty());
        assert_eq!(
            std::fs::read_to_string(folder.join(STAGE_FILE)).expect("Failed to read stage"),
            open.stage.to_json()
        );
        let saved = open.json.clone();
        open.stage.room.height += 100;
        std::fs::remove_dir_all(&folder).expect("Failed to remove test folder");
        assert!(open.save().is_err());
        assert!(open.is_dirty());
        assert_eq!(open.json, saved);
    }
}
