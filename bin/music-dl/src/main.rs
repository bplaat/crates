/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]

use crate::args::parse_args;

mod args;
mod cli;
mod downloader;
mod services;
mod structs;
mod utils;

fn main() {
    let args = parse_args();
    cli::run(&args);
}
