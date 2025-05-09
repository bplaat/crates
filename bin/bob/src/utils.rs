/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{fs, io};

use crate::Project;

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
        format!("{} bytes", bytes)
    }
}

pub(crate) fn resolve_source_file_path(source_file: &str, project: &Project) -> String {
    source_file
        .replace("$source_dir", &format!("{}/src", project.manifest_dir))
        .replace(
            "$source_gen_dir",
            &format!(
                "{}/target/{}/src-gen",
                project.manifest_dir, project.profile
            ),
        )
}

pub(crate) fn create_file_and_parent_dirs(path: impl AsRef<Path>) -> io::Result<File> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    File::create(path)
}

pub(crate) fn write_file_when_different(path: &str, contents: &str) -> io::Result<()> {
    if let Ok(existing_contents) = fs::read_to_string(path) {
        if existing_contents == contents {
            return Ok(());
        }
    }
    let mut f = create_file_and_parent_dirs(path)?;
    f.write_all(contents.as_bytes())?;
    Ok(())
}

pub(crate) fn index_files(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    let entries = fs::read_dir(dir).expect("Can't read directory");
    for entry in entries {
        let entry = entry.expect("Can't read directory entry");
        let path = entry.path();
        if path.is_dir() {
            files.extend(index_files(&path.to_string_lossy()));
        } else {
            files.push(path.to_string_lossy().to_string());
        }
    }
    files
}
