/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::fmt::{self, Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::str::FromStr;

// MARK: LogEntry
pub(crate) struct LogEntry {
    pub input: String,
    pub modified_time: u64,
    pub hash: Option<Vec<u8>>,
}

impl FromStr for LogEntry {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(' ').collect();
        if parts.len() != 2 && parts.len() != 3 {
            return Err("Invalid log entry format".to_string());
        }

        let input = parts[0].to_string();

        let modified_time = parts[1]
            .parse::<u64>()
            .map_err(|_| "Invalid modified time".to_string())?;

        let hash = if parts.len() == 3 {
            let hash_str = parts[2];
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
            input,
            modified_time,
            hash,
        })
    }
}

impl Display for LogEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.input, self.modified_time)?;
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
        let entries = contents
            .lines()
            .map(|line| {
                line.parse()
                    .unwrap_or_else(|_| panic!("Corrupt log file: {path}"))
            })
            .collect::<Vec<LogEntry>>();
        Log { file, entries }
    }

    pub(crate) fn get(&self, input: &str) -> Option<&LogEntry> {
        self.entries.iter().rev().find(|entry| entry.input == input)
    }

    pub(crate) fn add(&mut self, entry: LogEntry) {
        writeln!(self.file, "{entry}").unwrap_or_else(|_| panic!("Can't write to file"));
        self.entries.push(entry);
    }
}
