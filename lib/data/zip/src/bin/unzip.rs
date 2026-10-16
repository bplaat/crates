/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! List or extract a ZIP archive.

use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::path::{Component, Path};
use std::{env, io};

use zip::ZipArchive;

fn ensure_directory(root: &Path, relative: &Path) -> Result<(), String> {
    let mut path = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(format!("unsafe archive path: {}", relative.display()));
        };
        path.push(name);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Ok(_) => return Err(format!("{} is not a directory", path.display())),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir(&path).map_err(|error| format!("{}: {error}", path.display()))?;
            }
            Err(error) => return Err(format!("{}: {error}", path.display())),
        }
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let (list, archive_path, destination) = match args.as_slice() {
        [flag, archive] if flag == OsStr::new("-l") => (true, archive, None),
        [archive] => (false, archive, None),
        [archive, flag, directory] if flag == OsStr::new("-d") => {
            (false, archive, Some(Path::new(directory)))
        }
        _ => return Err("usage: unzip [-l] archive [-d directory]".into()),
    };
    let archive_file = File::open(archive_path)
        .map_err(|error| format!("{}: {error}", Path::new(archive_path).display()))?;
    let mut archive = ZipArchive::new(archive_file).map_err(|error| error.to_string())?;
    let root = if list {
        None
    } else {
        let directory = destination.unwrap_or(Path::new("."));
        fs::create_dir_all(directory)
            .map_err(|error| format!("{}: {error}", directory.display()))?;
        Some(fs::canonicalize(directory).map_err(|error| error.to_string())?)
    };
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let name = entry.name().to_owned();
        let relative = Path::new(&name);
        if relative.components().next().is_none()
            || relative
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(format!("unsafe archive path: {name}"));
        }
        if let Some(root) = &root {
            if name.ends_with('/') {
                ensure_directory(root, relative)?;
            } else {
                if let Some(parent) = relative.parent() {
                    ensure_directory(root, parent)?;
                }
                let path = root.join(relative);
                let mut output = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                    .map_err(|error| format!("{}: {error}", path.display()))?;
                if let Err(error) = io::copy(&mut entry, &mut output) {
                    drop(output);
                    let _ = fs::remove_file(&path);
                    return Err(format!("{}: {error}", path.display()));
                }
            }
        }
        println!("{name}");
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("unzip: {error}");
        std::process::exit(1);
    }
}
