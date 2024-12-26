/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple persons REST API example

fn main() {
    openapi_generator::generate_schemas_build(
        "openapi.yaml",
        format!(
            "{}/persons_api.rs",
            std::env::var("OUT_DIR").expect("OUT_DIR not set")
        ),
    );
}
