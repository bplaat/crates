/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use argparse::{Parser, Subcommand as SubcommandParser};
use directories::UserDirs;

#[derive(Clone, Copy, PartialEq, Eq, SubcommandParser)]
pub(crate) enum Subcommand {
    #[arg(alias = "d", help = "Download album or artist")]
    Download,
    #[arg(alias = "l", help = "List all albums of artist")]
    List,
    #[arg(alias = "h", help = "Print this help message")]
    Help,
    #[arg(alias = "v", help = "Print the version number")]
    Version,
}

#[derive(Parser)]
#[arg(name = "music-dl")]
pub(crate) struct Args {
    #[arg(subcommand)]
    pub subcommand: Subcommand,
    #[arg(positional, value = "query")]
    pub query: String,
    #[arg(
        short = 'o',
        long = "output",
        value = "dir",
        help = "Change output directory"
    )]
    pub output_dir: String,
    #[arg(short = 'i', long = "id", help = "Query is a Deezer ID")]
    pub is_id: bool,
    #[arg(short = 'a', long = "artist", help = "Query is an artist name")]
    pub is_artist: bool,
    #[arg(short = 's', long = "with-singles", help = "Include singles of artist")]
    pub with_singles: bool,
    #[arg(short = 'c', long = "with-cover", help = "Also download cover image")]
    pub with_cover: bool,
}

impl Default for Args {
    fn default() -> Self {
        let user_dirs = UserDirs::new().expect("Can't get user dirs");
        Self {
            subcommand: Subcommand::Help,
            query: String::new(),
            output_dir: user_dirs.audio_dir().display().to_string(),
            is_id: false,
            is_artist: false,
            with_singles: false,
            with_cover: false,
        }
    }
}
