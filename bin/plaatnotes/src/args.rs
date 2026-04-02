/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use clap::Parser;

#[derive(Parser)]
#[command(about = "PlaatNotes note-taking app", version)]
pub(crate) struct Args {
    #[arg(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[arg(long, help = "Email address", value_name = "EMAIL")]
    pub email: Option<String>,
}

#[derive(clap::Subcommand, PartialEq, Eq)]
pub(crate) enum Subcommand {
    #[command(name = "serve")]
    Serve,
    #[command(name = "serve-e2e")]
    ServeE2e,
    #[command(name = "import-google-keep")]
    ImportGoogleKeep { path: Option<String> },
    Help,
    Version,
}

pub(crate) fn subcommand_help() {
    println!(
        "Usage: plaatnotes [SUBCOMMAND]

Subcommands:
  serve                   Start the HTTP server (default)
  serve-e2e               Start the HTTP server with in-memory test database (for E2E testing)
  import-google-keep      Import notes from a Google Keep Takeout folder or zip
    --email <email>         Email of the user to import notes for
  help                    Print this help message
  version                 Print the version number"
    );
}
