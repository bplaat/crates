/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::str::FromStr;
use std::time::Duration;

// MARK: LogEntry
pub(crate) struct LogEntry {
    pub path: String,
    pub mtime: Duration,
    pub hash: Option<Vec<u8>>,
}

impl FromStr for LogEntry {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(' ').collect();
        if parts.len() != 3 && parts.len() != 4 {
            return Err("Invalid log entry format".to_string());
        }

        let path = parts[0].to_string();

        let mtime_secs = parts[1]
            .parse::<u64>()
            .map_err(|_| "Invalid modified time".to_string())?;
        let mtime_nanos = parts[2]
            .parse::<u32>()
            .map_err(|_| "Invalid modified time".to_string())?;

        let hash = if parts.len() == 4 {
            let hash_str = parts[3];
            let mut hash = Vec::with_capacity(hash_str.len() / 2);
            for i in (0..hash_str.len()).step_by(2) {
                hash.push(
                    u8::from_str_radix(&hash_str[i..i + 2], 16)
                        .map_err(|_| "Invalid hash format".to_string())?,
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
pub(crate) struct Log {
    file: File,
    entries: Vec<LogEntry>,
}

impl Log {
    pub(crate) fn new(path: &str) -> Self {
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)
            .unwrap_or_else(|_| panic!("Can't open file: {path}"));

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .unwrap_or_else(|_| panic!("Can't read file: {path}"));

        let entries = match contents
            .lines()
            .map(|line| line.parse::<LogEntry>())
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(entries) => entries,
            Err(_) => {
                // Truncate the file if corrupt and return empty log
                file.set_len(0)
                    .unwrap_or_else(|_| panic!("Can't truncate file: {path}"));
                Vec::new()
            }
        };
        Log { file, entries }
    }

    pub(crate) fn get(&self, path: &str) -> Option<&LogEntry> {
        self.entries.iter().rev().find(|entry| entry.path == path)
    }

    pub(crate) fn add(&mut self, entry: LogEntry) {
        writeln!(self.file, "{entry}").unwrap_or_else(|_| panic!("Can't write to file"));
        self.entries.push(entry);
    }
}
