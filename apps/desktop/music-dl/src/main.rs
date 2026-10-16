/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![doc = include_str!("../README.md")]
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use crate::args::parse_args;

mod args;
mod cli;
mod downloader;
mod gui;
mod services;
mod structs;
mod utils;

fn main() {
    if std::env::args().len() == 1 {
        gui::run();
    } else {
        let args = parse_args();
        cli::run(&args);
    }
}
