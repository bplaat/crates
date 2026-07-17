/*
 * Copyright (c) 2024-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::process::{capture, npm_program, run};
use crate::utils::collect_source_files;

mod build;
mod checks;
mod metadata;
mod process;
mod utils;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Os {
    Linux,
    Macos,
    Windows,
}

impl Os {
    fn detect() -> Result<Self> {
        match env::consts::OS {
            "linux" => Ok(Self::Linux),
            "macos" => Ok(Self::Macos),
            "windows" => Ok(Self::Windows),
            other => bail!("unsupported OS: {other}"),
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Macos => "macos",
            Self::Windows => "windows",
        }
    }
}

struct Xtask {
    root: PathBuf,
    os: Os,
}

impl Xtask {
    fn new() -> Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .context("xtask must live at <workspace>/xtask")?
            .to_owned();
        env::set_current_dir(&root)
            .with_context(|| format!("failed to enter {}", root.display()))?;
        Ok(Self {
            root,
            os: Os::detect()?,
        })
    }

    fn run(&self, task: &str) -> Result<()> {
        match task {
            "build-pages" => self.build_pages(),
            "build-bundle" => self.build_bundle(),
            "clean" => self.clean(),
            "check" => self.check(),
            "check-shared" => self.check_shared(),
            "check-rust" => self.check_rust(),
            "check-e2e" => self.check_e2e(),
            "coverage" => self.coverage(),
            "install" => self.install(),
            "help" | "--help" | "-h" => {
                Self::print_help();
                Ok(())
            }
            other => {
                Self::print_help();
                bail!("unknown task: {other}")
            }
        }
    }

    fn print_help() {
        println!("Repository tasks");
        println!();
        println!("Usage: cargo xtask <task>");
        println!();
        println!("Tasks:");
        println!("  check          Run every check");
        println!("  check-shared   Run platform-independent checks");
        println!("  check-rust     Format, lint, and test Rust code");
        println!("  check-e2e      Run PlaatNotes browser tests");
        println!("  coverage       Generate Rust coverage");
        println!("  build-pages    Build GitHub Pages artifacts");
        println!("  build-bundle   Build native application bundles");
        println!("  clean          Remove generated files");
        println!("  install        Install command-line and GUI applications");
    }

    fn ensure_npm_deps(&self) -> Result<()> {
        if !self.root.join("node_modules").is_dir() {
            run(Command::new(npm_program(self.os)).arg("ci"))?;
        }
        Ok(())
    }

    fn cargo_metadata(&self) -> Result<Value> {
        let output = capture(Command::new("cargo").args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
        ]))?;
        serde_json::from_slice(&output.stdout).context("failed to parse cargo metadata")
    }

    fn source_files(&self, extensions: &[&str]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        collect_source_files(&self.root, &mut files)?;
        if !extensions.is_empty() {
            files.retain(|file| {
                file.extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extensions.contains(&extension))
            });
        }
        files.sort();
        Ok(files)
    }
}

fn main() -> Result<()> {
    let task = env::args().nth(1).unwrap_or_else(|| "check".to_owned());
    Xtask::new()?.run(&task)
}

// MARK: Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_name_matches_variant() {
        assert_eq!(Os::Linux.name(), "linux");
        assert_eq!(Os::Macos.name(), "macos");
        assert_eq!(Os::Windows.name(), "windows");
    }

    #[test]
    fn os_detect_matches_current_platform() -> Result<()> {
        let expected = match env::consts::OS {
            "linux" => Os::Linux,
            "macos" => Os::Macos,
            "windows" => Os::Windows,
            _ => return Ok(()),
        };
        assert!(Os::detect()? == expected);
        Ok(())
    }
}
