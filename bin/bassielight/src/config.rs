/*
 * Copyright (c) 2025 Leonard van der Plaat
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use log::warn;
use serde::{Deserialize, Serialize};

use crate::stage::{self, OpenStage, STAGE_EXTENSION, Stage};

// Constants
pub(crate) const DMX_LENGTH: usize = 512;
pub(crate) const DMX_FPS: u64 = 44;
const DEFAULT_STAGE: &str = "Default";

/// App settings, the stage itself lives in a separate stage file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct Config {
    pub dmx_length: usize,
    pub dmx_fps: u64,
    pub last_stage: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            dmx_length: DMX_LENGTH,
            dmx_fps: DMX_FPS,
            last_stage: None,
        }
    }
}

pub(crate) static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

impl Config {
    /// App data directory with the settings and the default stage file
    pub(crate) fn dir() -> PathBuf {
        let project_dirs =
            directories::ProjectDirs::from("nl", "bplaat", "BassieLight").expect("Can't get dirs");
        let config_dir = project_dirs.config_dir();
        std::fs::create_dir_all(&config_dir).expect("Can't create directories");
        config_dir
    }

    pub(crate) fn load() -> Config {
        let config: Config = match File::open(Config::dir().join("config.json")) {
            Ok(file) => serde_json::from_reader(io::BufReader::new(file))
                .expect("Can't read and/or parse config.json"),
            Err(_) => Config::default(),
        };
        if config.dmx_length == 0 || config.dmx_length > DMX_LENGTH {
            panic!("Invalid config.json: DMX length must be between 1 and {DMX_LENGTH}");
        }
        if !(1..=DMX_FPS).contains(&config.dmx_fps) {
            panic!("Invalid config.json: DMX FPS must be between 1 and {DMX_FPS}");
        }
        config
    }

    pub(crate) fn save(&self) -> io::Result<()> {
        let file = File::create(Config::dir().join("config.json"))?;
        serde_json::to_writer_pretty(file, self).map_err(io::Error::other)
    }

    /// Open the last stage folder, falling back to the default stage folder next to the settings
    pub(crate) fn load_stage(&self) -> OpenStage {
        if let Some(path) = &self.last_stage {
            match stage::open_path(path, self.dmx_length) {
                Ok(open) => return open,
                Err(error) => warn!("Can't open last stage {}: {error}", path.display()),
            }
        }

        let folder = Config::dir().join(format!("{DEFAULT_STAGE}.{STAGE_EXTENSION}"));
        if !folder.exists() {
            stage::create_folder(&folder, &Stage::default(), None)
                .expect("Can't create the default stage");
        }
        OpenStage::open(&folder, self.dmx_length)
            .unwrap_or_else(|error| panic!("Can't open {}: {error}", folder.display()))
    }
}

pub(crate) fn dmx_length() -> usize {
    CONFIG
        .lock()
        .expect("Failed to lock config")
        .as_ref()
        .expect("Config not loaded")
        .dmx_length
}
