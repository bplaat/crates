/*
 * Copyright (c) 2025 Leonard van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};

// Constants
pub(crate) const DMX_LENGTH: usize = 512;
pub(crate) const DMX_FPS: u64 = 44;
pub(crate) const DMX_SWITCHES_LENGTH: usize = 4;

/// Types of DMX fixtures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum FixtureType {
    #[serde(rename = "p56led")]
    P56Led,
    #[serde(rename = "american_dj_mega_tripar")]
    AmericanDJMegaTripar,
    #[serde(rename = "ayra_compar_10")]
    AyraCompar10,
    #[serde(rename = "multidim_mkii")]
    MultiDimMKII,
}

/// DMX fixture configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Fixture {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: FixtureType,
    pub addr: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switches: Option<Vec<String>>,
}

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    pub fixtures: Vec<Fixture>,
    pub dmx_length: usize,
    pub dmx_fps: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            fixtures: Vec::new(),
            dmx_length: DMX_LENGTH,
            dmx_fps: DMX_FPS,
        }
    }
}

/// Loads the configuration from `path`. If the file does not exist, creates it with default values.
pub(crate) fn load_config(path: impl AsRef<Path>) -> Config {
    if let Ok(file) = File::open(path.as_ref()) {
        serde_json::from_reader(file).expect("Can't read and/or parse config.json")
    } else {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).expect("Can't create directories");
        }

        let default_conf = Config::default();
        let mut file = File::create(path).expect("Can't open config.json");
        serde_json::to_writer_pretty(&mut file, &default_conf).expect("Can't write config.json");
        default_conf
    }
}
