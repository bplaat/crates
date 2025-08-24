/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fs;

pub(crate) fn copy_dir(src: &str, dst: &str) {
    for entry in fs::read_dir(src).expect("Failed to read resources directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        let dest_path = format!("{}/{}", dst, entry.file_name().to_string_lossy());
        if path.is_dir() {
            fs::create_dir_all(&dest_path).expect("Can't create resource subdirectory");
            copy_dir(&path.to_string_lossy(), &dest_path);
        } else {
            fs::copy(&path, &dest_path).expect("Failed to copy resource file");
        }
    }
}
