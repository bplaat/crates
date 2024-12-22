/*
 * Copyright (c) 2024 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple persons REST API example

fn main() {
    openapi_generator::generate_schemas("openapi.yaml");
}
