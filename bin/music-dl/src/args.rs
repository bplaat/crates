/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::env;
use std::process::exit;

use crate::utils::user_music_dir;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Subcommand {
    Download,
    List,
    Help,
    Version,
}

pub(crate) struct Args {
    pub subcommand: Subcommand,
    pub query: String,
    pub output_dir: String,
    pub is_id: bool,
    pub is_artist: bool,
    pub with_singles: bool,
    pub with_cover: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            subcommand: Subcommand::Help,
            query: String::new(),
            output_dir: user_music_dir(),
            is_id: false,
            is_artist: false,
            with_singles: false,
            with_cover: false,
        }
    }
}

pub(crate) fn parse_args() -> Args {
    let mut args = Args::default();
    let mut args_iter = env::args().skip(1);
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "d" | "download" => args.subcommand = Subcommand::Download,
            "l" | "list" => args.subcommand = Subcommand::List,
            "h" | "help" => args.subcommand = Subcommand::Help,
            "v" | "version" | "-v" | "--version" => args.subcommand = Subcommand::Version,
            "-o" | "--output" => args.output_dir = args_iter.next().expect("Invalid argument"),
            "-i" | "--id" => args.is_id = true,
            "-a" | "--artist" => args.is_artist = true,
            "-s" | "--with-singles" => args.with_singles = true,
            "-c" | "--with-cover" => args.with_cover = true,
            _ => {
                if args.query.is_empty() {
                    args.query = arg;
                } else {
                    eprintln!("Unknown argument: {}", arg);
                    exit(1);
                }
            }
        }
    }
    args
}
