/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! An user agent parser library

use std::{env, fs};

use serde::{Deserialize, Serialize};

// MARK: Rules
#[derive(Deserialize, Serialize)]
struct Rules {
    user_agent_parsers: Vec<Rule>,
    os_parsers: Vec<Rule>,
}

#[derive(Deserialize, Serialize)]
struct Rule {
    regex: String,
    family_replacement: Option<String>,
    v1_replacement: Option<String>,
    v2_replacement: Option<String>,
    v3_replacement: Option<String>,
    os_replacement: Option<String>,
    os_v1_replacement: Option<String>,
    os_v2_replacement: Option<String>,
    os_v3_replacement: Option<String>,
    os_v4_replacement: Option<String>,
}

// MARK: Main
fn main() {
    let rules = serde_yaml::from_str::<Rules>(include_str!("regexes.yaml"))
        .expect("Can't parse regexes.yaml");
    fs::write(
        format!(
            "{}/rules.bin",
            env::var("OUT_DIR").expect("OUT_DIR not set")
        ),
        postcard::to_allocvec(&rules).expect("Can't serialize rules"),
    )
    .expect("Can't write to rules.bin");
}
