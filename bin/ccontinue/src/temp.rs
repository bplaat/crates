/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Manages temporary file creation with a consistent naming scheme.
/// All temp files are placed in temp_dir/ccontinue for easy cleanup.
pub(crate) struct TempFileManager {
    base_dir: PathBuf,
}

impl TempFileManager {
    /// Create a new TempFileManager with temp_dir/ccontinue as base.
    pub(crate) fn new() -> Self {
        let mut base = std::env::temp_dir();
        base.push("ccontinue");
        std::fs::create_dir_all(&base).unwrap_or_else(|e| {
            eprintln!("[ERROR] Can't create temp dir: {e}");
            std::process::exit(1);
        });
        TempFileManager { base_dir: base }
    }

    /// Get the base temp directory path (temp_dir/ccontinue).
    pub(crate) fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// Create a temporary file path with the given extension.
    /// Returns a path like temp_dir/ccontinue/ccc_<pid>_<counter>.<ext>
    pub(crate) fn temp_file(&self, ext: &str) -> String {
        let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!("ccc_{}_{n}{ext}", std::process::id());
        let mut path = self.base_dir.clone();
        path.push(name);
        path.to_str()
            .expect("temp file path is valid UTF-8")
            .to_owned()
    }
}

impl Default for TempFileManager {
    fn default() -> Self {
        Self::new()
    }
}
