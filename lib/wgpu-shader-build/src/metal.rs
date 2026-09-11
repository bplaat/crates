/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use crate::Result;
use crate::output::write_if_changed;

pub(crate) fn compile(source: &Path, air: &Path, library: &Path) -> Result<()> {
    let temporary_air = TemporaryFile::new(air);
    let temporary_library = TemporaryFile::new(library);
    let mut compiler = Command::new("xcrun");
    compiler.args(["-sdk", "macosx", "metal", "-std=macos-metal2.1"]);
    if let Some(target) = env::var_os("MACOSX_DEPLOYMENT_TARGET") {
        let mut argument = OsString::from("-mmacosx-version-min=");
        argument.push(target);
        compiler.arg(argument);
    }
    compiler
        .arg("-c")
        .arg(source)
        .arg("-o")
        .arg(&temporary_air.0);
    run(&mut compiler, "Metal AIR compilation")?;
    run(
        Command::new("xcrun")
            .args(["-sdk", "macosx", "metallib"])
            .arg(&temporary_air.0)
            .arg("-o")
            .arg(&temporary_library.0),
        "Metal library linking",
    )?;
    let air_bytes = fs::read(&temporary_air.0)
        .map_err(|e| format!("{}: read compiled AIR: {e}", temporary_air.0.display()))?;
    let library_bytes = fs::read(&temporary_library.0).map_err(|e| {
        format!(
            "{}: read compiled Metal library: {e}",
            temporary_library.0.display()
        )
    })?;
    write_if_changed(air, &air_bytes)?;
    write_if_changed(library, &library_bytes)
}

fn run(command: &mut Command, operation: &str) -> Result<()> {
    let output = command.output().map_err(|e| {
        format!(
            "{operation}: failed to start {:?}: {e}",
            command.get_program()
        )
    })?;
    if output.status.success() {
        return Ok(());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let diagnostics = [stdout.trim(), stderr.trim()]
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    Err(format!("{operation} failed:\n{diagnostics}").into())
}

struct TemporaryFile(PathBuf);

impl TemporaryFile {
    fn new(path: &Path) -> Self {
        let mut temporary = path.as_os_str().to_owned();
        temporary.push(format!(".{}.tmp", std::process::id()));
        Self(temporary.into())
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
