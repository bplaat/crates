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

use crate::stage::{OpenStage, Stage};

// Constants
pub(crate) const DMX_LENGTH: usize = 512;
pub(crate) const DMX_FPS: u64 = 44;
const STAGE_FILE_NAME: &str = "stage.json";

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
        config
    }

    pub(crate) fn save(&self) -> io::Result<()> {
        let file = File::create(Config::dir().join("config.json"))?;
        serde_json::to_writer_pretty(file, self).map_err(io::Error::other)
    }

    /// Open the last stage file, falling back to a fresh stage file in the config directory
    pub(crate) fn load_stage(&self) -> OpenStage {
        if let Some(path) = &self.last_stage {
            match Stage::load(path, self.dmx_length) {
                Ok(stage) => {
                    return OpenStage {
                        path: path.clone(),
                        stage,
                    };
                }
                Err(error) => warn!("Can't open last stage {}: {error}", path.display()),
            }
        }

        let path = Config::dir().join(STAGE_FILE_NAME);
        let stage = if path.exists() {
            Stage::load(&path, self.dmx_length)
                .unwrap_or_else(|error| panic!("Can't open {}: {error}", path.display()))
        } else {
            // Move the stage out of the config.json of older versions, otherwise start fresh
            let stage: Stage = File::open(Config::dir().join("config.json"))
                .ok()
                .and_then(|file| serde_json::from_reader(io::BufReader::new(file)).ok())
                .unwrap_or_default();
            stage.save(&path).expect("Can't write stage.json");
            stage
        };
        OpenStage { path, stage }
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
