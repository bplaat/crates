/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use clap::Parser;

#[derive(Parser)]
#[command(about = "Bundle a Rust project into a macOS .app", version)]
pub(crate) struct Args {
    #[arg(short = 'p', long, help = "Build crate in <DIR>", value_name = "DIR", default_value = ".")]
    pub path: String,
}
