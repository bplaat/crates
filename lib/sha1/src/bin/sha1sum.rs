/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Print SHA-1 checksums for files or standard input.

use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use sha1::Sha1;

fn hash(mut reader: impl Read) -> io::Result<[u8; 20]> {
    let mut hasher = Sha1::new();
    let mut buffer = [0; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finalize())
}

fn run() -> Result<(), String> {
    let mut paths = env::args_os().skip(1).collect::<Vec<_>>();
    if paths.is_empty() {
        paths.push("-".into());
    }
    for path in paths {
        let digest = if path == OsStr::new("-") {
            hash(io::stdin())
        } else {
            hash(
                File::open(&path)
                    .map_err(|error| format!("{}: {error}", Path::new(&path).display()))?,
            )
        }
        .map_err(|error| format!("{}: {error}", Path::new(&path).display()))?;
        for byte in digest {
            print!("{byte:02x}");
        }
        println!("  {}", path.to_string_lossy());
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("sha1sum: {error}");
        std::process::exit(1);
    }
}
