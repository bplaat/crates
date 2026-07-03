/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use argparse::{Parser, Subcommand as SubcommandParser};

#[derive(PartialEq, Eq, SubcommandParser)]
pub(crate) enum Subcommand {
    #[arg(default, help = "Start the HTTP server")]
    Serve,
    #[arg(
        name = "serve-e2e",
        help = "Start the HTTP server with in-memory test database"
    )]
    ServeE2e,
    #[arg(
        name = "import-google-keep",
        help = "Import notes from a Google Keep Takeout folder or zip"
    )]
    ImportGoogleKeep,
    #[arg(help = "Print this help message")]
    Help,
    #[arg(help = "Print the version number")]
    Version,
}

#[derive(Parser)]
#[arg(name = "plaatnotes")]
pub(crate) struct Args {
    #[arg(subcommand)]
    pub subcommand: Subcommand,
    #[arg(positional, command = "import-google-keep", value = "path")]
    pub path: Option<String>,
    #[arg(
        long = "email",
        value = "email",
        help = "Email of the user to import notes for"
    )]
    pub email: Option<String>,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            subcommand: Subcommand::Serve,
            path: None,
            email: None,
        }
    }
}
