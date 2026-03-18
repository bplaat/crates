/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::str::FromStr;
use std::time::Duration;

// MARK: LogEntry
/// A single entry in the build log recording a file's last-known state.
pub struct LogEntry {
    /// Path of the file (input or output).
    pub path: String,
    /// Last-seen modification time as seconds + subsecond nanos since UNIX epoch.
    pub mtime: Duration,
    /// SHA-1 hash of the file contents, or `None` for directories / empty files.
    pub hash: Option<Vec<u8>>,
}

impl FromStr for LogEntry {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(' ').collect();
        if parts.len() != 3 && parts.len() != 4 {
            return Err("invalid log entry format".to_string());
        }

        let path = parts[0].to_string();
        let mtime_secs = parts[1]
            .parse::<u64>()
            .map_err(|_| "invalid mtime secs".to_string())?;
        let mtime_nanos = parts[2]
            .parse::<u32>()
            .map_err(|_| "invalid mtime nanos".to_string())?;

        let hash = if parts.len() == 4 {
            let hash_str = parts[3];
            let mut hash = Vec::with_capacity(hash_str.len() / 2);
            for i in (0..hash_str.len()).step_by(2) {
                hash.push(
                    u8::from_str_radix(&hash_str[i..i + 2], 16)
                        .map_err(|_| "invalid hash format".to_string())?,
                );
            }
            Some(hash)
        } else {
            None
        };

        Ok(LogEntry {
            path,
            mtime: Duration::new(mtime_secs, mtime_nanos),
            hash,
        })
    }
}

impl Display for LogEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.path,
            self.mtime.as_secs(),
            self.mtime.subsec_nanos()
        )?;
        if let Some(hash) = &self.hash {
            write!(f, " ")?;
            for byte in hash {
                write!(f, "{byte:02x}")?;
            }
        }
        Ok(())
    }
}

// MARK: Log
/// Incremental build log that persists file state between runs.
///
/// On open the log is deduplicated (last entry per path wins) and rewritten to
/// keep the file size bounded -- mirroring the compaction Ninja performs on its
/// `.ninja_log`.
pub struct Log {
    file: File,
    entries: HashMap<String, LogEntry>,
}

impl Log {
    /// Open (or create) the log at `path`, deduplicating and compacting it.
    pub fn new(path: &str) -> Self {
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(path)
            .unwrap_or_else(|_| panic!("can't open log file: {path}"));

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .unwrap_or_else(|_| panic!("can't read log file: {path}"));

        // Parse all entries; on corrupt data truncate and start fresh.
        let raw: Option<Vec<LogEntry>> = contents
            .lines()
            .map(|line| line.parse::<LogEntry>().ok())
            .collect();

        let entries: HashMap<String, LogEntry> = match raw {
            Some(list) => {
                // Keep only the last entry per path (latest wins).
                let mut map: HashMap<String, LogEntry> = HashMap::new();
                for entry in list {
                    map.insert(entry.path.clone(), entry);
                }
                map
            }
            None => HashMap::new(),
        };

        // Rewrite the file with the compacted entries so it never grows without bound.
        file.set_len(0)
            .unwrap_or_else(|_| panic!("can't truncate log file: {path}"));
        file.seek(SeekFrom::Start(0))
            .unwrap_or_else(|_| panic!("can't seek log file: {path}"));
        for entry in entries.values() {
            writeln!(file, "{entry}").unwrap_or_else(|_| panic!("can't write log file: {path}"));
        }
        file.flush()
            .unwrap_or_else(|_| panic!("can't flush log file: {path}"));

        Log { file, entries }
    }

    /// Look up the last-recorded state for `path`.
    pub fn get(&self, path: &str) -> Option<&LogEntry> {
        self.entries.get(path)
    }

    /// Record new state for `path` (appended to the on-disk log and updated in memory).
    pub fn add(&mut self, entry: LogEntry) {
        writeln!(self.file, "{entry}").unwrap_or_else(|_| panic!("can't write to log file"));
        self.entries.insert(entry.path.clone(), entry);
    }
}
