/*
 * Copyright (c) 2025 Leonard van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs::File;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Types of DMX fixtures.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FixtureType {
    P56Led,
}

/// DMX fixture configuration.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Fixture {
    /// Unique name identifier for the fixture.
    pub name: String,
    /// Fixture type (e.g., "p56led").
    #[serde(rename = "type")]
    pub r#type: FixtureType,
    /// DMX channel address (1-indexed) for the fixture's first channel.
    pub addr: usize,
}

/// Application configuration.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    /// List of fixtures.
    pub fixtures: Vec<Fixture>,
    /// DMX buffer length.
    pub dmx_length: usize,
    /// DMX frames per second.
    pub dmx_fps: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            fixtures: vec![],
            dmx_length: 512,
            dmx_fps: 44,
        }
    }
}

/// Loads the configuration from `path`. If the file does not exist, creates it with default values.
pub(crate) fn load_config(path: impl AsRef<Path>) -> Result<Config> {
    if !path.as_ref().exists() {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let default_conf = Config::default();
        let mut file = File::create(path)?;
        serde_json::to_writer_pretty(&mut file, &default_conf)?;
        return Ok(default_conf);
    }

    let file = File::open(path)?;
    let config = serde_json::from_reader(file)?;
    Ok(config)
}
