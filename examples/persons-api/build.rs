/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple persons REST API example

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("Should be some");
    openapi_generator::generate_schemas_build(
        "openapi.yaml",
        format!("{out_dir}/persons_api.rs"),
        openapi_generator::Generator::Rust,
    );
}
