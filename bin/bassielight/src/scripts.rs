/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::BTreeMap;
use std::io;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

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
    /// Last error of each script that stopped by an error
    pub errors: BTreeMap<String, String>,
    version: u64,
}

pub(crate) static SCRIPTS: Mutex<Scripts> = Mutex::new(Scripts {
    sources: BTreeMap::new(),
    errors: BTreeMap::new(),
    version: 0,
});

/// Read all scripts in a stage folder
pub(crate) fn read_scripts(folder: &Path) -> BTreeMap<String, String> {
    let mut sources = BTreeMap::new();
    for path in std::fs::read_dir(folder.join(SCRIPTS_DIR))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
    {
        if path.extension() == Some("lua".as_ref())
            && let (Some(name), Ok(source)) = (path.file_stem(), std::fs::read_to_string(&path))
        {
            sources.insert(name.to_string_lossy().into_owned(), source);
        }
    }
    sources
}

/// Use new script sources, notifying all connections when they changed
pub(crate) fn set_sources(sources: BTreeMap<String, String>) {
    let mut scripts = SCRIPTS.lock().expect("Failed to lock scripts");
    if scripts.sources == sources {
        return;
    }
    scripts.sources = sources.clone();
    scripts.version += 1;
    drop(scripts);
    ipc::broadcast(&IpcMessage::ScriptsChanged { scripts: sources });
}

/// Script names are file names, so only simple names are allowed
pub(crate) fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ' '))
}

pub(crate) fn save_script(folder: &Path, name: &str, source: &str) -> io::Result<()> {
    let scripts = folder.join(SCRIPTS_DIR);
    std::fs::create_dir_all(&scripts)?;
    std::fs::write(scripts.join(format!("{name}.lua")), source)?;
    set_sources(read_scripts(folder));
    Ok(())
}

pub(crate) fn delete_script(folder: &Path, name: &str) -> io::Result<()> {
    std::fs::remove_file(folder.join(SCRIPTS_DIR).join(format!("{name}.lua")))?;
    set_sources(read_scripts(folder));
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
const PROPS: [&str; 17] = [
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
    "flash_on",
    "flash_press",
    "flash_intensity",
    "flash_speed",
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
    id: u32,
    field: TweenField,
    from: TweenValue,
    to: TweenValue,
    start: f64,
    beats: f64,
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
    sources: BTreeMap<String, String>,
    version: u64,
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
            sources: BTreeMap::new(),
            version: 0,
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
                lua.app_data_mut::<Frame>()
                    .expect("Frame is set")
                    .commands
                    .extend(commands);
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

    /// Run the scripts that are due at this beat and apply what they did to the DMX state
    pub(crate) fn update(&mut self, beat: f64, tempo: Tempo, stage: &Stage) {
        let mut errors = Vec::new();
        let mut finished = Vec::new();

        // Restart running scripts when the sources changed, start and stop scripts like requested
        {
            let scripts = SCRIPTS.lock().expect("Failed to lock scripts");
            if scripts.version != self.version {
                self.version = scripts.version;
                self.sources = scripts.sources.clone();
                self.running.clear();
                self.tweens.clear();
            }
        }
        let (wanted, states) = {
            let state = DMX_STATE.lock().expect("Failed to lock DMX state");
            (state.running_scripts.clone(), state.fixtures.clone())
        };
        self.running.retain(|name, _| wanted.contains(name));
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
                    _ => {
                        errors.push((
                            name.clone(),
                            "Use wait() or sync() instead of coroutine.yield()".into(),
                        ));
                        break;
                    }
                };
            }
        }
        let commands = self
            .lua
            .remove_app_data::<Frame>()
            .map(|frame| frame.commands)
            .unwrap_or_default();

        // Apply the commands and tweens
        let mut broadcasts = Vec::new();
        let mut state = DMX_STATE.lock().expect("Failed to lock DMX state");
        for command in commands {
            match command {
                Command::Set { id, prop } => {
                    let field = TweenField::of(prop).map(|(field, _)| field);
                    self.tweens
                        .retain(|tween| !(tween.id == id && Some(tween.field) == field));
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

        // Scripts that ended or failed are no longer running
        let stopped = !errors.is_empty() || !finished.is_empty();
        for (name, _) in &errors {
            state.running_scripts.remove(name);
            self.running.remove(name);
        }
        for name in &finished {
            state.running_scripts.remove(name);
            self.running.remove(name);
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

    #[test]
    fn runs_scripts_on_the_beat() {
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
        set_sources(BTreeMap::from([
            (
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
            ),
        ]));
        DMX_STATE.lock().unwrap().running_scripts.insert("blink".to_string());

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
}
