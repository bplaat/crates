/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::Path;
use std::process::{Command, Stdio, exit};
use std::{fs, io};

pub(crate) fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.2} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.2} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} bytes")
    }
}

pub(crate) fn write_file_when_different(path: &str, contents: &str) -> io::Result<()> {
    if let Ok(existing_contents) = fs::read_to_string(path)
        && existing_contents == contents
    {
        return Ok(());
    }

    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

pub(crate) fn index_files(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    let entries = fs::read_dir(dir).unwrap_or_else(|_| {
        eprintln!("Can't read directory: {dir}");
        exit(1);
    });
    for entry in entries {
        let entry = entry.unwrap_or_else(|_| {
            eprintln!("Can't read directory: {dir}");
            exit(1);
        });
        let path = entry.path();
        if path.file_name().is_some_and(|name| name == ".DS_Store") {
            continue;
        }
        if path.is_dir() {
            files.extend(index_files(&path.to_string_lossy()));
        } else {
            files.push(path.to_string_lossy().to_string());
        }
    }
    files
}

pub(crate) fn spawn_service(program: &str, args: &[&str]) -> io::Result<()> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null()) // Detach stdin
        .stdout(Stdio::null()) // Detach stdout
        .stderr(Stdio::null()); // Detach stderr

    #[cfg(unix)]
    {
        // On Unix, use `setsid` to create a new session, detaching from the parent terminal
        use std::os::unix::process::CommandExt;
        command.process_group(0); // Set process group ID to 0 to start a new session
    }

    #[cfg(windows)]
    {
        // On Windows, use creation flags to detach the process
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const DETACHED_PROCESS: u32 = 0x00000008;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS);
    }

    // Spawn the process and forget it (do not wait for it)
    command.spawn()?;
    Ok(())
}
