/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Encode or decode base64 data from a file or standard input.

use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::path::Path;
use std::{env, fs};

use base64::Engine;
use base64::prelude::BASE64_STANDARD;

fn run() -> Result<(), String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let (decode, path) = match args.as_slice() {
        [] => (false, None),
        [flag] if flag == OsStr::new("-d") => (true, None),
        [path] => (false, Some(path)),
        [flag, path] if flag == OsStr::new("-d") => (true, Some(path)),
        _ => return Err("usage: base64 [-d] [file]".into()),
    };
    let mut input = Vec::new();
    match path {
        Some(path) if path != OsStr::new("-") => {
            input = fs::read(path)
                .map_err(|error| format!("{}: {error}", Path::new(path).display()))?;
        }
        _ => {
            io::stdin()
                .read_to_end(&mut input)
                .map_err(|error| error.to_string())?;
        }
    }
    let output = if decode {
        input.retain(|byte| !byte.is_ascii_whitespace());
        BASE64_STANDARD
            .decode(input)
            .map_err(|error| error.to_string())?
    } else {
        let encoded = BASE64_STANDARD.encode(input);
        let mut wrapped = Vec::with_capacity(encoded.len() + encoded.len().div_ceil(76));
        for line in encoded.as_bytes().chunks(76) {
            wrapped.extend_from_slice(line);
            wrapped.push(b'\n');
        }
        wrapped
    };
    io::stdout()
        .write_all(&output)
        .map_err(|error| error.to_string())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("base64: {error}");
        std::process::exit(1);
    }
}
