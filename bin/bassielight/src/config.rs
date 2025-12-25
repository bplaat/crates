/*
 * Copyright (c) 2025 Leonard van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs::File;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// Constants
pub(crate) const DMX_LENGTH: usize = 512;
pub(crate) const DMX_FPS: u64 = 44;
pub(crate) const DMX_SWITCHES_LENGTH: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum FixtureType {
    #[serde(rename = "american_dj_p56led")]
    AmericanDJP56Led,
    #[serde(rename = "american_dj_mega_tripar")]
    AmericanDJMegaTripar,
    #[serde(rename = "ayra_compar_10")]
    AyraCompar10,
    #[serde(rename = "showtec_multidim_mkii")]
    ShowtecMultidimMKII,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Fixture {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: FixtureType,
    pub addr: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switches: Option<Vec<String>>,
}

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

impl Config {
    fn default_path() -> PathBuf {
        if !cfg!(debug_assertions) {
            let project_dirs = directories::ProjectDirs::from("nl", "bplaat", "BassieLight")
                .expect("Can't get dirs");
            let config_dir = project_dirs.config_dir();
            std::fs::create_dir_all(&config_dir).expect("Can't create directories");
            config_dir.join("config.json")
        } else {
            PathBuf::from("config.json")
        }
    }

    pub(crate) fn load() -> Config {
        let path = Config::default_path();
        if let Ok(file) = File::open(&path) {
            serde_json::from_reader(file).expect("Can't read and/or parse config.json")
        } else {
            let default_conf = Config::default();
            let mut file = File::create(&path).expect("Can't open config.json");
            serde_json::to_writer_pretty(&mut file, &default_conf)
                .expect("Can't write config.json");
            default_conf
        }
    }
}
