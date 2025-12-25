/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple persons REST API example

use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Should be some"));
    openapi_generator::generate_schemas_build(
        "openapi.yaml",
        out_dir.join("persons_api.rs"),
        openapi_generator::Generator::Rust,
    );
}
