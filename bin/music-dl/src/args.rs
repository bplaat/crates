/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use clap::Parser;

#[derive(Clone, Copy, PartialEq, Eq, clap::Subcommand)]
pub(crate) enum Subcommand {
    #[command(alias = "d")]
    Download,
    #[command(alias = "l")]
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

#[derive(Parser)]
#[command(about = "Download music from Deezer/YouTube")]
struct RawArgs {
    #[arg(subcommand)]
    subcommand: Option<Subcommand>,

    /// Search query or ID (positional)
    query: Option<String>,

    #[arg(short = 'o', long, help = "Output directory", value_name = "DIR")]
    output_dir: Option<String>,

    #[arg(short = 'i', long = "id", help = "Treat query as an ID")]
    is_id: bool,

    #[arg(short = 'a', long = "artist", help = "Treat query as an artist name")]
    is_artist: bool,

    #[arg(short = 's', long = "with-singles", help = "Include singles")]
    with_singles: bool,

    #[arg(short = 'c', long = "with-cover", help = "Download cover art")]
    with_cover: bool,
}

pub(crate) fn parse_args() -> Args {
    use directories::UserDirs;
    let raw = RawArgs::parse();
    Args {
        subcommand: raw.subcommand.unwrap_or(Subcommand::Help),
        query: raw.query.unwrap_or_default(),
        output_dir: raw.output_dir.unwrap_or_else(|| {
            UserDirs::new()
                .expect("Can't get user dirs")
                .audio_dir()
                .display()
                .to_string()
        }),
        is_id: raw.is_id,
        is_artist: raw.is_artist,
        with_singles: raw.with_singles,
        with_cover: raw.with_cover,
    }
}
