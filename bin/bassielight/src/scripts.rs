/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::{io, mem};

use log::info;
use mlua::thread::ThreadStatus;
use mlua::{HookTriggers, Lua, LuaOptions, MultiValue, StdLib, Table, Thread, Value, VmState};
use serde::Serialize;

use crate::dmx::{Color, DMX_STATE, DmxState, FixtureProp, FixtureState, Mode, Tempo};
use crate::ipc::{self, IpcMessage};
use crate::stage::{FixtureKind, FixtureProfile, SCRIPTS_DIR, Stage};

// MARK: Script files
/// Scripts of the opened stage folder, the version changes when any source changes
pub(crate) struct Scripts {
    pub sources: BTreeMap<String, String>,
    pub folders: BTreeSet<String>,
    /// Last error of each script that stopped by an error
    pub errors: BTreeMap<String, String>,
    version: u64,
}

pub(crate) static SCRIPTS: Mutex<Scripts> = Mutex::new(Scripts {
    sources: BTreeMap::new(),
    folders: BTreeSet::new(),
    errors: BTreeMap::new(),
    version: 0,
});

#[derive(Default)]
pub(crate) struct ScriptFiles {
    pub sources: BTreeMap<String, String>,
    pub folders: BTreeSet<String>,
}

impl From<BTreeMap<String, String>> for ScriptFiles {
    fn from(sources: BTreeMap<String, String>) -> Self {
        Self {
            sources,
            ..Self::default()
        }
    }
}

/// Read real folders recursively, keeping script IDs relative to scripts/ without .lua.
pub(crate) fn read_scripts(folder: &Path) -> ScriptFiles {
    fn read(directory: &Path, prefix: &str, files: &mut ScriptFiles) {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let filename = entry.file_name();
            let Some(filename) = filename.to_str() else {
                continue;
            };
            let name = if prefix.is_empty() {
                filename.to_string()
            } else {
                format!("{prefix}/{filename}")
            };
            if kind.is_dir() && is_valid_name(&name) {
                files.folders.insert(name.clone());
                read(&entry.path(), &name, files);
            } else if kind.is_file()
                && let Some(name) = name.strip_suffix(".lua")
                && is_valid_name(name)
                && let Ok(source) = std::fs::read_to_string(entry.path())
            {
                files.sources.insert(name.to_string(), source);
            }
        }
    }
    let mut files = ScriptFiles::default();
    read(&folder.join(SCRIPTS_DIR), "", &mut files);
    files
}

/// Notify on folder changes too; empty folders don't restart running scripts.
pub(crate) fn set_sources(files: impl Into<ScriptFiles>) {
    let files = files.into();
    let mut scripts = SCRIPTS.lock().expect("Failed to lock scripts");
    if scripts.sources == files.sources && scripts.folders == files.folders {
        return;
    }
    if scripts.sources != files.sources {
        scripts.sources = files.sources.clone();
        scripts.version += 1;
    }
    scripts.folders = files.folders.clone();
    drop(scripts);
    ipc::broadcast(&IpcMessage::ScriptsChanged {
        scripts: files.sources,
        folders: files.folders.into_iter().collect(),
    });
}

/// Simple relative paths, with no empty, absolute, traversal or platform-specific components.
pub(crate) fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && name.split('/').all(|part| {
            !part.is_empty()
                && part.len() <= 64
                && part.trim() == part
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ' '))
        })
}

fn script_directory(folder: &Path, name: &str, create: bool) -> io::Result<PathBuf> {
    if !name.is_empty() && !is_valid_name(name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid script folder",
        ));
    }
    let mut directory = folder.join(SCRIPTS_DIR);
    if create {
        std::fs::create_dir_all(&directory)?;
    }
    for part in std::iter::once("").chain(name.split('/').filter(|part| !part.is_empty())) {
        if !part.is_empty() {
            directory.push(part);
        }
        if create && !directory.exists() {
            std::fs::create_dir(&directory)?;
        }
        if !std::fs::symlink_metadata(&directory)?.file_type().is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Script folders must be real directories",
            ));
        }
    }
    Ok(directory)
}

fn script_path(folder: &Path, name: &str, create: bool) -> io::Result<PathBuf> {
    if !is_valid_name(name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid script path",
        ));
    }
    let (parent, file) = name.rsplit_once('/').unwrap_or(("", name));
    let path = script_directory(folder, parent, create)?.join(format!("{file}.lua"));
    if std::fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Scripts must be real files",
        ));
    }
    Ok(path)
}

pub(crate) fn create_script_folder(folder: &Path, name: &str) -> io::Result<()> {
    if !is_valid_name(name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid script folder",
        ));
    }
    script_directory(folder, name, true)?;
    set_sources(read_scripts(folder));
    Ok(())
}

pub(crate) fn save_script(folder: &Path, name: &str, source: &str) -> io::Result<()> {
    std::fs::write(script_path(folder, name, true)?, source)?;
    set_sources(read_scripts(folder));
    Ok(())
}

pub(crate) fn delete_script(folder: &Path, name: &str) -> io::Result<()> {
    std::fs::remove_file(script_path(folder, name, false)?)?;
    set_sources(read_scripts(folder));
    Ok(())
}

/// Copy the complete script tree when a stage is duplicated, including empty folders.
pub(crate) fn copy_scripts(source: &Path, destination: &Path) -> io::Result<()> {
    std::fs::create_dir_all(destination)?;
    if destination
        .canonicalize()?
        .starts_with(source.canonicalize()?)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot copy scripts into their own tree",
        ));
    }
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if kind.is_dir() {
            copy_scripts(&entry.path(), &target)?;
        } else if kind.is_file() {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Which scripts run and which stopped with an error
pub(crate) fn status(dmx_state: &DmxState) -> IpcMessage {
    IpcMessage::ScriptsRunning {
        running: dmx_state.running_scripts.iter().cloned().collect(),
        errors: SCRIPTS
            .lock()
            .expect("Failed to lock scripts")
            .errors
            .clone(),
    }
}

/// Notify all connections which scripts run and which stopped with an error
pub(crate) fn broadcast_status() {
    let message = status(&DMX_STATE.lock().expect("Failed to lock DMX state"));
    ipc::broadcast(&message);
}

// MARK: Props
const COLORS: [(&str, u32); 11] = [
    ("black", 0x000000),
    ("white", 0xffffff),
    ("red", 0xff0000),
    ("green", 0x00ff00),
    ("blue", 0x0000ff),
    ("yellow", 0xffff00),
    ("magenta", 0xff00ff),
    ("cyan", 0x00ffff),
    ("orange", 0xff8000),
    ("purple", 0x8000ff),
    ("pink", 0xff40a0),
];

/// Props that are ignored for fixtures that don't have them
const PROPS: [&str; 24] = [
    "color",
    "toggle_color",
    "intensity",
    "toggle_tween",
    "toggle_speed",
    "strobe_speed",
    "preset",
    "preset_speed",
    "gobo",
    "focus",
    "movement",
    "movement_speed",
    "switches",
    "switch_on",
    "switch_all_press",
    "flash_on",
    "flash_press",
    "flash_intensity",
    "flash_speed",
    "haze_on",
    "haze_press",
    "haze_volume",
    "fan_speed",
    "blackout",
];

fn error(message: impl Into<String>) -> mlua::Error {
    mlua::Error::runtime(message.into())
}

fn color(value: &Value) -> mlua::Result<Color> {
    match value {
        Value::Integer(color) => Ok(Color::from_u32(*color as u32)),
        Value::Number(color) => Ok(Color::from_u32(*color as u32)),
        Value::String(name) => {
            let name = name.to_str()?;
            let color = match name.strip_prefix('#') {
                Some(hex) => u32::from_str_radix(hex, 16).ok(),
                None => COLORS
                    .iter()
                    .find(|(color, _)| color.eq_ignore_ascii_case(&name))
                    .map(|(_, color)| *color),
            };
            color
                .map(Color::from_u32)
                .ok_or_else(|| error(format!("Unknown color {name}")))
        }
        _ => Err(error("A color should be a name, #rrggbb string or number")),
    }
}

fn number(value: &Value) -> mlua::Result<f32> {
    match value {
        Value::Integer(number) => Ok(*number as f32),
        Value::Number(number) => Ok(*number as f32),
        _ => Err(error("Expected a number")),
    }
}

fn fraction(value: &Value) -> mlua::Result<f32> {
    Ok(number(value)?.clamp(0.0, 1.0))
}

fn boolean(value: &Value) -> mlua::Result<bool> {
    value
        .as_boolean()
        .ok_or_else(|| error("Expected a boolean"))
}

/// Beats, or `false` for off
fn speed(value: &Value) -> mlua::Result<Option<f32>> {
    match value {
        Value::Boolean(false) | Value::Nil => Ok(None),
        _ => number(value)
            .ok()
            .filter(|beats| *beats > 0.0)
            .map(Some)
            .ok_or_else(|| error("A speed should be beats or false")),
    }
}

/// Item by name or number from 1, or `false` for none
fn index<'a>(
    value: &Value,
    names: impl ExactSizeIterator<Item = &'a str>,
) -> mlua::Result<Option<usize>> {
    let count = names.len();
    match value {
        Value::Boolean(false) | Value::Nil => Ok(None),
        Value::Integer(number) if (1..=count as i64).contains(number) => {
            Ok(Some(*number as usize - 1))
        }
        Value::String(name) => {
            let name = name.to_str()?;
            names
                .enumerate()
                .find(|(_, item)| item.eq_ignore_ascii_case(&name))
                .map(|(index, _)| Some(index))
                .ok_or_else(|| error(format!("Unknown name {name}")))
        }
        _ => Err(error(format!(
            "Expected a name, a number from 1 to {count} or false"
        ))),
    }
}

fn from_name<T: serde::de::DeserializeOwned>(value: &Value) -> mlua::Result<T> {
    let name = value
        .as_string()
        .ok_or_else(|| error("Expected a name"))?
        .to_str()?
        .to_string();
    serde_json::from_value(serde_json::Value::String(name.clone()))
        .map_err(|_| error(format!("Unknown name {name}")))
}

/// Props of a script props table for a fixture, props it doesn't have are left out
fn parse_prop(
    profile: &FixtureProfile,
    key: &str,
    value: &Value,
) -> mlua::Result<Vec<FixtureProp>> {
    use FixtureProp as P;
    let colored = matches!(profile.kind, FixtureKind::Rgb | FixtureKind::MovingHead);
    let strobe = profile.kind == FixtureKind::Strobe;
    let haze = profile.kind == FixtureKind::Haze;
    let presets = profile.presets;
    let head = profile.moving_head;
    let prop = match key {
        "color" if colored => P::Color(color(value)?),
        "toggle_color" if colored => P::ToggleColor(color(value)?),
        "intensity" if colored => P::Intensity(fraction(value)?),
        "toggle_tween" if colored => P::ToggleTween(from_name(value)?),
        "toggle_speed" if colored => P::ToggleSpeed(speed(value)?),
        "strobe_speed" if colored => P::StrobeSpeed(speed(value)?),
        "preset" => match presets {
            Some(presets) => {
                P::Preset(index(value, presets.list.iter().map(|preset| preset.name))?)
            }
            None => return Ok(Vec::new()),
        },
        "preset_speed" if presets.is_some() => P::PresetSpeed(fraction(value)?),
        "gobo" | "movement" => {
            let Some(head) = head else {
                return Ok(Vec::new());
            };
            match key {
                "gobo" => P::Gobo(index(value, head.gobos.iter().map(|gobo| gobo.name))?),
                _ => P::Movement(index(
                    value,
                    head.movements.iter().map(|movement| movement.name),
                )?),
            }
        }
        "movement_speed" if head.is_some() => P::MovementSpeed(fraction(value)?),
        "focus" if head.is_some() => P::Focus(fraction(value)?),
        "switch_on" if profile.kind == FixtureKind::Switch => P::SwitchOn(boolean(value)?),
        "switch_all_press" if profile.kind == FixtureKind::Switch => {
            P::SwitchAllPress(boolean(value)?)
        }
        "switches" if profile.kind == FixtureKind::Switch => {
            let switches = value
                .as_table()
                .ok_or_else(|| error("Expected a list of booleans"))?;
            return switches
                .sequence_values::<bool>()
                .enumerate()
                .map(|(index, on)| Ok(P::SwitchToggle { index, on: on? }))
                .collect();
        }
        "flash_on" if strobe => P::FlashOn(boolean(value)?),
        "flash_press" if strobe => P::FlashPress(boolean(value)?),
        "flash_intensity" if strobe => P::FlashIntensity(fraction(value)?),
        "flash_speed" if strobe => P::FlashSpeed(fraction(value)?),
        "haze_on" if haze => P::HazeOn(boolean(value)?),
        "haze_press" if haze => P::HazePress(boolean(value)?),
        "haze_volume" if haze => P::HazeVolume(fraction(value)?),
        "fan_speed" if haze => P::FanSpeed(fraction(value)?),
        "blackout" => P::Blackout(boolean(value)?),
        _ if PROPS.contains(&key) => return Ok(Vec::new()),
        _ => return Err(error(format!("Unknown prop {key}"))),
    };
    Ok(vec![prop])
}

fn parse_props(profile: &FixtureProfile, props: &Table) -> mlua::Result<Vec<FixtureProp>> {
    let mut result = Vec::new();
    for pair in props.pairs::<String, Value>() {
        let (key, value) = pair?;
        result.extend(parse_prop(profile, &key, &value)?);
    }
    Ok(result)
}

/// Prop of a fixture state for scripts, numbered items count from 1
fn get_prop(lua: &Lua, state: &FixtureState, key: &str) -> mlua::Result<Value> {
    let json = serde_json::to_value(state).expect("Failed to serialize fixture state");
    let camel_key = key
        .split('_')
        .enumerate()
        .fold(String::new(), |mut camel, (index, part)| {
            if index == 0 {
                camel.push_str(part);
            } else {
                let mut chars = part.chars();
                camel.extend(chars.next().map(|c| c.to_ascii_uppercase()));
                camel.extend(chars);
            }
            camel
        });
    let value = match key {
        "switches" => &json["switchesToggle"],
        _ if PROPS.contains(&key) => &json[camel_key.as_str()],
        _ => return Err(error(format!("Unknown prop {key}"))),
    };
    Ok(match value {
        serde_json::Value::Null => Value::Boolean(false),
        serde_json::Value::Bool(value) => Value::Boolean(*value),
        serde_json::Value::Number(number) if matches!(key, "preset" | "gobo" | "movement") => {
            Value::Integer(number.as_i64().unwrap_or_default() + 1)
        }
        serde_json::Value::Number(number) => match number.as_i64() {
            Some(integer) => Value::Integer(integer),
            None => Value::Number(number.as_f64().unwrap_or_default()),
        },
        serde_json::Value::String(string) => Value::String(lua.create_string(string)?),
        serde_json::Value::Array(items) => Value::Table(
            lua.create_sequence_from(items.iter().map(|item| item.as_bool().unwrap_or_default()))?,
        ),
        serde_json::Value::Object(_) => Value::Nil,
    })
}

// MARK: Tweens
#[derive(Debug, Clone, Copy, PartialEq)]
enum TweenField {
    Color,
    ToggleColor,
    Intensity,
    FlashIntensity,
    FlashSpeed,
    HazeVolume,
    FanSpeed,
    PresetSpeed,
    MovementSpeed,
    Focus,
}

#[derive(Debug, Clone, Copy)]
enum TweenValue {
    Color(Color),
    Number(f32),
}

impl TweenField {
    const fn of(prop: FixtureProp) -> Option<(TweenField, TweenValue)> {
        use TweenValue::{Color, Number};
        Some(match prop {
            FixtureProp::Color(color) => (TweenField::Color, Color(color)),
            FixtureProp::ToggleColor(color) => (TweenField::ToggleColor, Color(color)),
            FixtureProp::Intensity(value) => (TweenField::Intensity, Number(value)),
            FixtureProp::FlashIntensity(value) => (TweenField::FlashIntensity, Number(value)),
            FixtureProp::FlashSpeed(value) => (TweenField::FlashSpeed, Number(value)),
            FixtureProp::HazeVolume(value) => (TweenField::HazeVolume, Number(value)),
            FixtureProp::FanSpeed(value) => (TweenField::FanSpeed, Number(value)),
            FixtureProp::PresetSpeed(value) => (TweenField::PresetSpeed, Number(value)),
            FixtureProp::MovementSpeed(value) => (TweenField::MovementSpeed, Number(value)),
            FixtureProp::Focus(value) => (TweenField::Focus, Number(value)),
            _ => return None,
        })
    }

    const fn get(self, state: &FixtureState) -> TweenValue {
        match self {
            TweenField::Color => TweenValue::Color(state.color),
            TweenField::ToggleColor => TweenValue::Color(state.toggle_color),
            TweenField::Intensity => TweenValue::Number(state.intensity),
            TweenField::FlashIntensity => TweenValue::Number(state.flash_intensity),
            TweenField::FlashSpeed => TweenValue::Number(state.flash_speed),
            TweenField::HazeVolume => TweenValue::Number(state.haze_volume),
            TweenField::FanSpeed => TweenValue::Number(state.fan_speed),
            TweenField::PresetSpeed => TweenValue::Number(state.preset_speed),
            TweenField::MovementSpeed => TweenValue::Number(state.movement_speed),
            TweenField::Focus => TweenValue::Number(state.focus),
        }
    }

    fn prop(self, value: TweenValue) -> FixtureProp {
        match (self, value) {
            (TweenField::Color, TweenValue::Color(color)) => FixtureProp::Color(color),
            (TweenField::ToggleColor, TweenValue::Color(color)) => FixtureProp::ToggleColor(color),
            (TweenField::Intensity, TweenValue::Number(value)) => FixtureProp::Intensity(value),
            (TweenField::FlashIntensity, TweenValue::Number(value)) => {
                FixtureProp::FlashIntensity(value)
            }
            (TweenField::FlashSpeed, TweenValue::Number(value)) => FixtureProp::FlashSpeed(value),
            (TweenField::HazeVolume, TweenValue::Number(value)) => FixtureProp::HazeVolume(value),
            (TweenField::FanSpeed, TweenValue::Number(value)) => FixtureProp::FanSpeed(value),
            (TweenField::PresetSpeed, TweenValue::Number(value)) => FixtureProp::PresetSpeed(value),
            (TweenField::MovementSpeed, TweenValue::Number(value)) => {
                FixtureProp::MovementSpeed(value)
            }
            (TweenField::Focus, TweenValue::Number(value)) => FixtureProp::Focus(value),
            _ => unreachable!("Tween value doesn't match its field"),
        }
    }
}

impl TweenValue {
    fn lerp(self, to: TweenValue, t: f32) -> TweenValue {
        match (self, to) {
            (TweenValue::Color(from), TweenValue::Color(to)) => {
                TweenValue::Color(from.tween(to, t))
            }
            (TweenValue::Number(from), TweenValue::Number(to)) => {
                TweenValue::Number(from + (to - from) * t)
            }
            _ => to,
        }
    }
}

struct Tween {
    script: String,
    id: u32,
    field: TweenField,
    from: TweenValue,
    to: TweenValue,
    start: f64,
    beats: f64,
}

// MARK: Changes
/// Prop of a fixture, every switch is a prop of its own
type PropKey = (u32, mem::Discriminant<FixtureProp>, usize);

const fn prop_key(id: u32, prop: &FixtureProp) -> PropKey {
    let index = match prop {
        FixtureProp::SwitchToggle { index, .. } | FixtureProp::SwitchPress { index, .. } => *index,
        _ => 0,
    };
    (id, mem::discriminant(prop), index)
}

struct Change {
    /// Order of the first change, the script that changed a prop first found its original value
    order: u64,
    original: FixtureProp,
    /// Last value the script set
    written: FixtureProp,
}

/// What each running script changed, so stopping a script can put its fixtures back
#[derive(Default)]
struct Changes {
    scripts: BTreeMap<String, HashMap<PropKey, Change>>,
    next_order: u64,
}

impl Changes {
    fn record(&mut self, script: &str, id: u32, prop: FixtureProp, before: &FixtureState) {
        let order = &mut self.next_order;
        self.scripts
            .entry(script.to_string())
            .or_default()
            .entry(prop_key(id, &prop))
            .or_insert_with(|| {
                *order += 1;
                Change {
                    order: *order,
                    original: before.current(prop),
                    written: prop,
                }
            })
            .written = prop;
    }

    /// Forget what a script changed, returns the original values to put back when it is restored
    fn release(
        &mut self,
        script: &str,
        restore: bool,
        states: &BTreeMap<u32, FixtureState>,
    ) -> Vec<(u32, FixtureProp)> {
        let Some(changes) = self.scripts.remove(script) else {
            return Vec::new();
        };
        if !restore {
            return Vec::new();
        }
        let mut restores = Vec::new();
        for (key, change) in changes {
            // Props other running scripts changed stay theirs, a later one inherits the original
            if let Some(other) = self
                .scripts
                .values_mut()
                .filter_map(|changes| changes.get_mut(&key))
                .min_by_key(|other| other.order)
            {
                if other.order > change.order {
                    other.order = change.order;
                    other.original = change.original;
                }
                continue;
            }
            // Props the user changed since are kept
            let (id, ..) = key;
            if states
                .get(&id)
                .is_some_and(|state| state.current(change.written) == change.written)
            {
                restores.push((id, change.original));
            }
        }
        restores
    }
}

// MARK: Engine
/// Longest a script may run without waiting
const MAX_RUN_TIME: Duration = Duration::from_millis(50);
/// Scripts further behind skip ahead, when the stage was paused
const MAX_LAG_BEATS: f64 = 4.0;
const MAX_RESUMES_PER_FRAME: usize = 64;
const PRELUDE: &str = include_str!("prelude.lua");

enum Command {
    Set {
        id: u32,
        prop: FixtureProp,
    },
    Tween {
        id: u32,
        field: TweenField,
        to: TweenValue,
        start: f64,
        beats: f64,
    },
    Mode(Mode),
}

/// What scripts see and do while they run in a frame
struct Frame {
    stage: Stage,
    states: BTreeMap<u32, FixtureState>,
    /// Position of the running script in beats
    beat: f64,
    bpm: f32,
    deadline: Instant,
    commands: Vec<Command>,
}

struct Running {
    thread: Thread,
    /// Beat at which the script continues
    cursor: f64,
}

/// Runs the started scripts in the DMX thread, every frame scripts that are done waiting continue
pub(crate) struct Engine {
    lua: Lua,
    running: BTreeMap<String, Running>,
    tweens: Vec<Tween>,
    changes: Changes,
    sources: BTreeMap<String, String>,
    version: u64,
    stage_generation: u64,
    tempo: Option<Tempo>,
    last_beat: f64,
}

fn frame(lua: &Lua) -> mlua::Result<mlua::AppDataRef<'_, Frame>> {
    lua.app_data_ref::<Frame>()
        .ok_or_else(|| error("Scripts only run on the stage"))
}

fn serde_name(value: impl Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default()
}

impl Engine {
    pub(crate) fn new() -> mlua::Result<Engine> {
        // Scripts have no access to files or the system
        let lua = Lua::new_with(
            StdLib::STRING | StdLib::TABLE | StdLib::MATH | StdLib::UTF8 | StdLib::COROUTINE,
            LuaOptions::default(),
        )?;
        lua.set_global_hook(
            HookTriggers::new().every_nth_instruction(10_000),
            |lua, _| match lua.app_data_ref::<Frame>() {
                Some(frame) if Instant::now() > frame.deadline => {
                    Err(error("Script ran too long without wait() or sync()"))
                }
                _ => Ok(VmState::Continue),
            },
        )?;
        lua.load(PRELUDE)
            .set_name("@prelude.lua")
            .call::<()>(Engine::primitives(&lua)?)?;
        Ok(Engine {
            lua,
            running: BTreeMap::new(),
            tweens: Vec::new(),
            changes: Changes::default(),
            sources: BTreeMap::new(),
            version: 0,
            stage_generation: 0,
            tempo: None,
            last_beat: 0.0,
        })
    }

    /// Functions the prelude builds the script API on
    fn primitives(lua: &Lua) -> mlua::Result<Table> {
        let primitives = lua.create_table()?;
        primitives.set(
            "fixtures",
            lua.create_function(|lua, ()| {
                let frame = frame(lua)?;
                lua.create_sequence_from(
                    frame
                        .stage
                        .fixtures
                        .iter()
                        .map(|fixture| {
                            let table = lua.create_table()?;
                            table.set("id", fixture.id)?;
                            table.set("name", fixture.name.as_str())?;
                            table.set("type", serde_name(fixture.r#type))?;
                            table.set("kind", serde_name(fixture.r#type.profile().kind))?;
                            table.set("x", fixture.x)?;
                            table.set("y", fixture.y)?;
                            Ok::<_, mlua::Error>(table)
                        })
                        .collect::<mlua::Result<Vec<_>>>()?,
                )
            })?,
        )?;
        primitives.set(
            "group",
            lua.create_function(|lua, name: String| {
                let frame = frame(lua)?;
                Ok(frame
                    .stage
                    .groups
                    .iter()
                    .find(|group| group.name == name)
                    .map(|group| group.fixtures.clone()))
            })?,
        )?;
        primitives.set(
            "set",
            lua.create_function(|lua, (ids, props): (Vec<u32>, Table)| {
                let commands = {
                    let frame = frame(lua)?;
                    let mut commands = Vec::new();
                    for fixture in frame.stage.fixtures.iter().filter(|f| ids.contains(&f.id)) {
                        for prop in parse_props(fixture.r#type.profile(), &props)? {
                            commands.push(Command::Set {
                                id: fixture.id,
                                prop,
                            });
                        }
                    }
                    commands
                };
                // Later gets in the same frame see the new values
                let mut frame = lua.app_data_mut::<Frame>().expect("Frame is set");
                for command in &commands {
                    if let Command::Set { id, prop } = command {
                        frame
                            .states
                            .entry(*id)
                            .or_insert(FixtureState::DEFAULT)
                            .apply(*prop);
                    }
                }
                frame.commands.extend(commands);
                Ok(())
            })?,
        )?;
        primitives.set(
            "tween",
            lua.create_function(|lua, (ids, props, beats): (Vec<u32>, Table, f64)| {
                let commands = {
                    let frame = frame(lua)?;
                    let mut commands = Vec::new();
                    for fixture in frame.stage.fixtures.iter().filter(|f| ids.contains(&f.id)) {
                        for prop in parse_props(fixture.r#type.profile(), &props)? {
                            let (field, to) = TweenField::of(prop).ok_or_else(|| {
                                error("Only colors and props from 0 to 1 can be tweened")
                            })?;
                            commands.push(Command::Tween {
                                id: fixture.id,
                                field,
                                to,
                                start: frame.beat,
                                beats,
                            });
                        }
                    }
                    commands
                };
                lua.app_data_mut::<Frame>()
                    .expect("Frame is set")
                    .commands
                    .extend(commands);
                Ok(())
            })?,
        )?;
        primitives.set(
            "get",
            lua.create_function(|lua, (id, prop): (u32, String)| {
                let frame = frame(lua)?;
                let state = frame
                    .states
                    .get(&id)
                    .copied()
                    .unwrap_or(FixtureState::DEFAULT);
                get_prop(lua, &state, &prop)
            })?,
        )?;
        primitives.set("beat", lua.create_function(|lua, ()| Ok(frame(lua)?.beat))?)?;
        primitives.set("bpm", lua.create_function(|lua, ()| Ok(frame(lua)?.bpm))?)?;
        primitives.set(
            "mode",
            lua.create_function(|lua, name: Value| {
                let mode = from_name(&name)?;
                lua.app_data_mut::<Frame>()
                    .ok_or_else(|| error("Scripts only run on the stage"))?
                    .commands
                    .push(Command::Mode(mode));
                Ok(())
            })?,
        )?;
        let colors = lua.create_table()?;
        for (name, color) in COLORS {
            colors.set(name, color)?;
        }
        primitives.set("colors", colors)?;
        Ok(primitives)
    }

    /// Load a script in its own environment, it starts on the next beat
    fn start(&self, name: &str, beat: f64) -> mlua::Result<Running> {
        let source = self
            .sources
            .get(name)
            .ok_or_else(|| error(format!("There is no script {name}")))?;
        let env = self.lua.create_table()?;
        let meta = self.lua.create_table()?;
        meta.set("__index", self.lua.globals())?;
        env.set_metatable(Some(meta))?;
        let script = name.to_string();
        env.set(
            "print",
            self.lua.create_function(move |_, values: MultiValue| {
                let message = values
                    .iter()
                    .map(|value| value.to_string().unwrap_or_else(|_| format!("{value:?}")))
                    .collect::<Vec<_>>()
                    .join(" ");
                info!("[{script}] {message}");
                Ok(())
            })?,
        )?;
        let function = self
            .lua
            .load(source)
            .set_name(format!("@{name}.lua"))
            .set_environment(env)
            .into_function()?;
        Ok(Running {
            thread: self.lua.create_thread(function)?,
            cursor: beat.ceil(),
        })
    }

    /// Stop a running script, cancel its tweens and put back what it changed when restoring
    fn stop(
        &mut self,
        name: &str,
        restore: bool,
        state: &mut DmxState,
        broadcasts: &mut Vec<IpcMessage>,
    ) {
        self.running.remove(name);
        if restore {
            // Clients only get the end of a tween, so they get the value a cancelled tween reached
            self.tweens.retain(|tween| {
                if tween.script != name {
                    return true;
                }
                broadcasts.push(IpcMessage::SetFixtureProp {
                    fixtures: vec![tween.id],
                    prop: tween.field.prop(tween.field.get(&state.fixture(tween.id))),
                });
                false
            });
        }
        for (id, prop) in self.changes.release(name, restore, &state.fixtures) {
            state
                .fixtures
                .entry(id)
                .or_insert(FixtureState::DEFAULT)
                .apply(prop);
            broadcasts.push(IpcMessage::SetFixtureProp {
                fixtures: vec![id],
                prop,
            });
        }
    }

    /// Run the scripts that are due at this beat and apply what they did to the DMX state
    pub(crate) fn update(&mut self, beat: f64, tempo: Tempo, stage: &Stage) {
        let mut errors = Vec::new();
        let mut finished = Vec::new();
        let mut broadcasts = Vec::new();

        // Restart running scripts when the sources changed, start and stop scripts like requested
        let reload = {
            let scripts = SCRIPTS.lock().expect("Failed to lock scripts");
            let reload = scripts.version != self.version;
            if reload {
                self.version = scripts.version;
                self.sources = scripts.sources.clone();
            }
            reload
        };
        let (wanted, states) = {
            let mut state = DMX_STATE.lock().expect("Failed to lock DMX state");
            if state.stage_generation != self.stage_generation {
                // The fixtures of another stage have nothing to put back
                self.stage_generation = state.stage_generation;
                self.running.clear();
                self.tweens.clear();
                self.changes = Changes::default();
            }
            let stopped = self
                .running
                .keys()
                .filter(|name| reload || !state.running_scripts.contains(*name))
                .cloned()
                .collect::<Vec<_>>();
            for name in stopped {
                self.stop(&name, true, &mut state, &mut broadcasts);
            }
            (state.running_scripts.clone(), state.fixtures.clone())
        };
        for name in &wanted {
            if !self.running.contains_key(name) {
                match self.start(name, beat) {
                    Ok(running) => {
                        self.running.insert(name.clone(), running);
                    }
                    Err(error) => errors.push((name.clone(), error.to_string())),
                }
            }
        }

        // Move all script times along when the tempo changes, so tap tempo doesn't skip or stall them
        if self.tempo.is_some_and(|previous| previous != tempo) {
            let shift = beat - self.last_beat;
            self.running
                .values_mut()
                .for_each(|running| running.cursor += shift);
            self.tweens
                .iter_mut()
                .for_each(|tween| tween.start += shift);
        }
        self.tempo = Some(tempo);
        self.last_beat = beat;

        // Continue the scripts that are done waiting
        self.lua.set_app_data(Frame {
            stage: stage.clone(),
            states,
            beat,
            bpm: tempo.bpm,
            deadline: Instant::now(),
            commands: Vec::new(),
        });
        let mut commands = Vec::new();
        for (name, running) in &mut self.running {
            if beat - running.cursor > MAX_LAG_BEATS {
                running.cursor = beat;
            }
            let mut resumes = 0;
            while running.cursor <= beat {
                resumes += 1;
                if resumes > MAX_RESUMES_PER_FRAME {
                    errors.push((
                        name.clone(),
                        "Script waits too short, use wait(beats) with beats above 0".into(),
                    ));
                    break;
                }
                {
                    let mut frame = self.lua.app_data_mut::<Frame>().expect("Frame is set");
                    frame.beat = running.cursor;
                    frame.deadline = Instant::now() + MAX_RUN_TIME;
                }
                let values = match running.thread.resume::<MultiValue>(()) {
                    Ok(values) => values,
                    Err(error) => {
                        errors.push((name.clone(), error.to_string()));
                        break;
                    }
                };
                if running.thread.status() != ThreadStatus::Resumable {
                    finished.push(name.clone());
                    break;
                }
                // The prelude yields how long to wait
                let kind = values
                    .front()
                    .and_then(Value::as_string)
                    .and_then(|kind| kind.to_str().ok().map(|kind| kind.to_string()));
                // Lua passes whole beats as integers
                let beats = values
                    .get(1)
                    .and_then(|beats| number(beats).ok())
                    .map_or(1.0, f64::from)
                    .max(0.0);
                running.cursor = match kind.as_deref() {
                    Some("wait") => running.cursor + beats,
                    // Next multiple of the beats after the current position
                    Some("sync") if beats > 0.0 => {
                        ((running.cursor / beats + 1e-9).floor() + 1.0) * beats
                    }
                    Some("sync") => {
                        errors.push((name.clone(), "Use sync(beats) with beats above 0".into()));
                        break;
                    }
                    _ => {
                        errors.push((
                            name.clone(),
                            "Use wait() or sync() instead of coroutine.yield()".into(),
                        ));
                        break;
                    }
                };
            }
            let pending = mem::take(
                &mut self
                    .lua
                    .app_data_mut::<Frame>()
                    .expect("Frame is set")
                    .commands,
            );
            commands.extend(pending.into_iter().map(|command| (name.clone(), command)));
        }
        self.lua.remove_app_data::<Frame>();

        // Apply the commands and tweens
        let mut state = DMX_STATE.lock().expect("Failed to lock DMX state");
        for (script, command) in commands {
            match command {
                Command::Set { id, prop } => {
                    let field = TweenField::of(prop).map(|(field, _)| field);
                    self.tweens
                        .retain(|tween| !(tween.id == id && Some(tween.field) == field));
                    self.changes.record(&script, id, prop, &state.fixture(id));
                    state
                        .fixtures
                        .entry(id)
                        .or_insert(FixtureState::DEFAULT)
                        .apply(prop);
                    broadcasts.push(IpcMessage::SetFixtureProp {
                        fixtures: vec![id],
                        prop,
                    });
                }
                Command::Tween {
                    id,
                    field,
                    to,
                    start,
                    beats,
                } => {
                    self.tweens
                        .retain(|tween| !(tween.id == id && tween.field == field));
                    let from = field.get(&state.fixture(id));
                    self.tweens.push(Tween {
                        script,
                        id,
                        field,
                        from,
                        to,
                        start,
                        beats,
                    });
                }
                Command::Mode(mode) => {
                    state.mode = mode;
                    broadcasts.push(IpcMessage::SetMode { mode });
                }
            }
        }
        self.tweens.retain(|tween| {
            let t = if tween.beats > 0.0 {
                ((beat - tween.start) / tween.beats).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let prop = tween.field.prop(tween.from.lerp(tween.to, t as f32));
            // Tweens of scripts that ended run on, their values are kept
            if self.running.contains_key(&tween.script) {
                self.changes
                    .record(&tween.script, tween.id, prop, &state.fixture(tween.id));
            }
            state
                .fixtures
                .entry(tween.id)
                .or_insert(FixtureState::DEFAULT)
                .apply(prop);
            if t >= 1.0 {
                broadcasts.push(IpcMessage::SetFixtureProp {
                    fixtures: vec![tween.id],
                    prop,
                });
            }
            t < 1.0
        });

        // Failed scripts put back what they changed, scripts that ended keep it
        let stopped = !errors.is_empty() || !finished.is_empty();
        for (name, _) in &errors {
            state.running_scripts.remove(name);
            self.stop(name, true, &mut state, &mut broadcasts);
        }
        for name in &finished {
            state.running_scripts.remove(name);
            self.stop(name, false, &mut state, &mut broadcasts);
        }
        drop(state);
        if !errors.is_empty() {
            let mut scripts = SCRIPTS.lock().expect("Failed to lock scripts");
            for (name, message) in errors {
                log::warn!("Script {name} stopped: {message}");
                scripts.errors.insert(name, message);
            }
        }
        for message in &broadcasts {
            ipc::broadcast(message);
        }
        if stopped {
            broadcast_status();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stage::{Fixture, FixtureType};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    struct TestFolder(PathBuf);

    impl TestFolder {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "bassielight-scripts-{}-{nonce}",
                std::process::id()
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestFolder {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn nested_scripts_and_empty_folders_are_read_copied_and_deleted() {
        let _guard = TEST_LOCK.lock().unwrap();
        let folder = TestFolder::new();
        save_script(&folder.0, "Effects/Pars/Pulse", "wait(1)").unwrap();
        save_script(&folder.0, "Shows/Pulse", "wait(2)").unwrap();
        let version = SCRIPTS.lock().unwrap().version;
        create_script_folder(&folder.0, "Effects/Empty").unwrap();
        assert_eq!(SCRIPTS.lock().unwrap().version, version);
        let files = read_scripts(&folder.0);
        assert_eq!(
            files.sources,
            BTreeMap::from([
                ("Effects/Pars/Pulse".into(), "wait(1)".into()),
                ("Shows/Pulse".into(), "wait(2)".into()),
            ])
        );
        assert_eq!(
            files.folders,
            ["Effects", "Effects/Empty", "Effects/Pars", "Shows"]
                .map(String::from)
                .into()
        );
        let copy = folder.0.join("Copy.stage");
        crate::stage::create_folder(&copy, &Stage::default(), Some(&folder.0)).unwrap();
        let copied = read_scripts(&copy);
        assert_eq!(copied.sources, files.sources);
        assert_eq!(copied.folders, files.folders);
        delete_script(&folder.0, "Effects/Pars/Pulse").unwrap();
        let files = read_scripts(&folder.0);
        assert_eq!(files.sources.len(), 1);
        assert!(files.sources.contains_key("Shows/Pulse"));
        assert!(files.folders.contains("Effects/Pars"));
        assert!(copy_scripts(&folder.0.join(SCRIPTS_DIR), &folder.0.join("scripts/Copy")).is_err());
    }

    #[test]
    fn script_paths_reject_traversal_and_symlinks() {
        let _guard = TEST_LOCK.lock().unwrap();
        let folder = TestFolder::new();
        for path in [
            "",
            "../escape",
            "/absolute",
            "Effects//Bad",
            "Effects/",
            "Effects/../Bad",
            "Effects\\Bad",
            "C:/Bad",
            " Bad",
        ] {
            assert!(!is_valid_name(path), "{path}");
            assert!(save_script(&folder.0, path, "wait(1)").is_err());
            assert!(create_script_folder(&folder.0, path).is_err());
            assert!(delete_script(&folder.0, path).is_err());
        }
        #[cfg(unix)]
        {
            let outside = folder.0.join("Outside");
            std::fs::create_dir(&outside).unwrap();
            std::fs::create_dir(folder.0.join(SCRIPTS_DIR)).unwrap();
            std::fs::write(outside.join("Original.lua"), "original").unwrap();
            std::os::unix::fs::symlink(&outside, folder.0.join("scripts/Link")).unwrap();
            std::os::unix::fs::symlink(
                outside.join("Original.lua"),
                folder.0.join("scripts/Linked.lua"),
            )
            .unwrap();
            assert!(save_script(&folder.0, "Link/Attack", "bad").is_err());
            assert!(save_script(&folder.0, "Linked", "bad").is_err());
            assert!(delete_script(&folder.0, "Linked").is_err());
            assert!(read_scripts(&folder.0).sources.is_empty());
            assert!(read_scripts(&folder.0).folders.is_empty());
            assert_eq!(
                std::fs::read_to_string(outside.join("Original.lua")).unwrap(),
                "original"
            );
            assert!(!outside.join("Attack.lua").exists());
        }
    }

    #[test]
    fn runs_scripts_on_the_beat() {
        let _guard = TEST_LOCK.lock().unwrap();
        let stage = Stage {
            fixtures: vec![Fixture {
                id: 1,
                name: "Par".to_string(),
                r#type: FixtureType::AmericanDJP56Led,
                addr: 1,
                x: 0,
                y: 0,
                switches: None,
            }],
            ..Stage::default()
        };
        let tempo = Tempo {
            bpm: 120.0,
            downbeat: None,
        };
        let color = || DMX_STATE.lock().unwrap().fixture(1).color;
        let intensity = || DMX_STATE.lock().unwrap().fixture(1).intensity;

        let mut engine = Engine::new().expect("Failed to start engine");
        set_sources(BTreeMap::from([(
            "blink".to_string(),
            r##"
                local par = fixture("Par")
                par:set({ color = "red", intensity = 1 })
                wait(1)
                par:set({ color = "#0000ff" })
                par:tween({ intensity = 0 }, 2)
                wait(4)
                par:set({ glow = true })
                "##
            .to_string(),
        )]));
        DMX_STATE
            .lock()
            .unwrap()
            .running_scripts
            .insert("blink".to_string());

        engine.update(0.0, tempo, &stage);
        assert_eq!(color(), Color::from_u32(0xff0000));
        engine.update(0.5, tempo, &stage);
        assert_eq!(color(), Color::from_u32(0xff0000));
        engine.update(1.0, tempo, &stage);
        assert_eq!(color(), Color::from_u32(0x0000ff));
        engine.update(2.0, tempo, &stage);
        assert!((intensity() - 0.5).abs() < 0.01);
        engine.update(3.0, tempo, &stage);
        assert_eq!(intensity(), 0.0);
        engine.update(4.5, tempo, &stage);
        assert!(DMX_STATE.lock().unwrap().running_scripts.contains("blink"));

        // The unknown prop stops the script with an error
        engine.update(5.0, tempo, &stage);
        assert!(!DMX_STATE.lock().unwrap().running_scripts.contains("blink"));
        let errors = SCRIPTS.lock().unwrap().errors.clone();
        assert!(errors["blink"].contains("Unknown prop glow"), "{errors:?}");
    }

    #[test]
    fn stopping_a_layer_cancels_only_its_tweens_and_restores_its_fixtures() {
        let _guard = TEST_LOCK.lock().unwrap();
        let stage = Stage {
            fixtures: (1..=2)
                .map(|id| Fixture {
                    id,
                    name: format!("Par {id}"),
                    r#type: FixtureType::AmericanDJP56Led,
                    addr: id as usize * 6,
                    x: 0,
                    y: 0,
                    switches: None,
                })
                .collect(),
            ..Stage::default()
        };
        let tempo = Tempo {
            bpm: 120.0,
            downbeat: None,
        };
        set_sources(BTreeMap::from([
            ("Effects/Pars/a".into(), "fixture(1):set({ intensity = 0, color = 'red' }); fixture(1):tween({ intensity = 1 }, 4); wait(8)".into()),
            ("Shows/b".into(), "fixture(2):set({ intensity = 0, color = 'blue' }); fixture(2):tween({ intensity = 1 }, 4); wait(8)".into()),
        ]));
        {
            let mut state = DMX_STATE.lock().unwrap();
            state.fixtures.clear();
            state.running_scripts = ["Effects/Pars/a".into(), "Shows/b".into()].into();
        }
        let mut engine = Engine::new().unwrap();
        engine.update(0.0, tempo, &stage);
        engine.update(1.0, tempo, &stage);
        {
            let mut state = DMX_STATE.lock().unwrap();
            assert_eq!(state.fixture(1).intensity, 0.25);
            assert_eq!(state.fixture(2).intensity, 0.25);
            state.running_scripts.remove("Effects/Pars/a");
        }
        engine.update(2.0, tempo, &stage);
        {
            let mut state = DMX_STATE.lock().unwrap();
            assert_eq!(state.fixture(1).intensity, 1.0);
            assert_eq!(state.fixture(1).color, Color::BLACK);
            assert_eq!(state.fixture(2).intensity, 0.5);
            state.fixtures.get_mut(&1).unwrap().intensity = 0.8;
        }
        engine.update(3.0, tempo, &stage);
        let state = DMX_STATE.lock().unwrap();
        assert_eq!(state.fixture(1).intensity, 0.8);
        assert_eq!(state.fixture(2).intensity, 0.75);
    }

    #[test]
    fn stopping_scripts_restores_what_they_changed() {
        let _guard = TEST_LOCK.lock().unwrap();
        let stage = Stage {
            fixtures: vec![Fixture {
                id: 1,
                name: "Par".to_string(),
                r#type: FixtureType::AmericanDJP56Led,
                addr: 1,
                x: 0,
                y: 0,
                switches: None,
            }],
            ..Stage::default()
        };
        let tempo = Tempo {
            bpm: 120.0,
            downbeat: None,
        };
        set_sources(BTreeMap::from([
            ("a".into(), "fixture(1):set({ color = 'red' }); wait(100)".into()),
            (
                "b".into(),
                "fixture(1):set({ color = 'blue', intensity = 0.5 }); wait(100)".into(),
            ),
            ("scene".into(), "fixture(1):set({ toggle_speed = 1 })".into()),
            (
                "get".into(),
                "fixture(1):set({ color = 'white' }); assert(fixture(1):get('color') == 0xffffff); wait(100)".into(),
            ),
        ]));
        let set_running = |names: &[&str]| {
            DMX_STATE.lock().unwrap().running_scripts =
                names.iter().map(|name| name.to_string()).collect();
        };
        let state = || DMX_STATE.lock().unwrap().fixture(1);
        {
            let mut state = DMX_STATE.lock().unwrap();
            state.fixtures.clear();
            state
                .fixtures
                .entry(1)
                .or_insert(FixtureState::DEFAULT)
                .apply(FixtureProp::Color(Color::from_u32(0x00ff00)));
        }
        let mut engine = Engine::new().unwrap();

        // A later layer inherits the original of the earlier one it overrides
        set_running(&["a"]);
        engine.update(0.0, tempo, &stage);
        assert_eq!(state().color, Color::from_u32(0xff0000));
        set_running(&["a", "b"]);
        engine.update(1.0, tempo, &stage);
        assert_eq!(state().color, Color::from_u32(0x0000ff));
        set_running(&["b"]);
        engine.update(2.0, tempo, &stage);
        assert_eq!(state().color, Color::from_u32(0x0000ff));
        set_running(&[]);
        engine.update(3.0, tempo, &stage);
        assert_eq!(state().color, Color::from_u32(0x00ff00));
        assert_eq!(state().intensity, 1.0);

        // Values the user changed while the script ran are kept
        set_running(&["a"]);
        engine.update(4.0, tempo, &stage);
        DMX_STATE
            .lock()
            .unwrap()
            .fixtures
            .get_mut(&1)
            .unwrap()
            .apply(FixtureProp::Color(Color::from_u32(0xffff00)));
        set_running(&[]);
        engine.update(5.0, tempo, &stage);
        assert_eq!(state().color, Color::from_u32(0xffff00));

        // A script that ends keeps its values
        set_running(&["scene"]);
        engine.update(6.0, tempo, &stage);
        assert!(DMX_STATE.lock().unwrap().running_scripts.is_empty());
        assert_eq!(state().toggle_speed, Some(1.0));

        // Gets see the sets of the same frame
        set_running(&["get"]);
        engine.update(7.0, tempo, &stage);
        assert!(DMX_STATE.lock().unwrap().running_scripts.contains("get"));
        assert!(!SCRIPTS.lock().unwrap().errors.contains_key("get"));
    }
}
