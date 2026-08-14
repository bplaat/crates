/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple Cargo plugin that builds macOS application bundles.

use std::fs;
use std::process::{ExitCode, exit};

use crate::manifest::Manifest;

mod appex;
mod args;
mod bundle;
mod manifest;
mod support;

fn read_manifest(path: &str) -> Manifest {
    let manifest_path = format!("{path}/Cargo.toml");
    basic_toml::from_str(&fs::read_to_string(&manifest_path).unwrap_or_else(|error| {
        eprintln!("Can't read {manifest_path} file: {error}");
        exit(1);
    }))
    .unwrap_or_else(|error| {
        eprintln!("Can't parse {manifest_path} file: {error}");
        exit(1);
    })
}

fn main() -> ExitCode {
    if !cfg!(target_os = "macos") {
        eprintln!("cargo-bundle can only be run on macOS");
        return ExitCode::FAILURE;
    }

    let args = args::parse_args();
    if args.help {
        args::help();
        return ExitCode::SUCCESS;
    }
    if args.version {
        println!("cargo-bundle {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    let manifest = read_manifest(&args.path);
    let bundle_metadata = &manifest.package.metadata.bundle;
    println!(
        "Bundling {} v{} ({})",
        bundle_metadata.name, manifest.package.version, args.path
    );

    let target_dir = format!("target/bundle/{}", manifest.package.name);
    bundle::build(&args.path, &target_dir, &manifest);
    appex::build_all(&args.path, &target_dir, &manifest);
    bundle::sign(&args.path, &target_dir, bundle_metadata);

    if args.zip {
        bundle::create_zip(&target_dir, bundle_metadata);
    }
    if args.dmg {
        bundle::create_dmg(&target_dir, bundle_metadata);
    }
    ExitCode::SUCCESS
}
