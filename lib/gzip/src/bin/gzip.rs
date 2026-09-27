/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Compress or decompress one file from the command line.

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

fn run() -> Result<(), String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let (decode, source) = match args.as_slice() {
        [path] => (false, Path::new(path)),
        [flag, path] if flag == OsStr::new("-d") => (true, Path::new(path)),
        _ => return Err("usage: gzip [-d] path".into()),
    };
    let destination = if decode {
        if source.extension() != Some(OsStr::new("gz")) {
            return Err("decompression path must end in .gz".into());
        }
        let mut path = source.to_path_buf();
        path.set_extension("");
        path
    } else {
        let mut path = OsString::from(source.as_os_str());
        path.push(".gz");
        PathBuf::from(path)
    };
    let input = fs::read(source).map_err(|error| format!("{}: {error}", source.display()))?;
    let output = if decode {
        gzip::decompress(&input).ok_or("invalid gzip data")?
    } else {
        gzip::compress(&input)
    };
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)
        .map_err(|error| format!("{}: {error}", destination.display()))?;
    if let Err(error) = file.write_all(&output).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(&destination);
        return Err(format!("{}: {error}", destination.display()));
    }
    drop(file);
    fs::remove_file(source).map_err(|error| format!("{}: {error}", source.display()))?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("gzip: {error}");
        std::process::exit(1);
    }
}
