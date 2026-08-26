/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use glob::{MatchOptions, Pattern, glob_with};

use super::CleanupResult;
use crate::catalog::PathRule;

#[derive(Default)]
pub(super) struct ScanStats {
    pub(super) files: u64,
    pub(super) bytes: u64,
    errors: Vec<String>,
}

pub(super) fn scan_rules(rules: &[PathRule], cancelled: &AtomicBool, result: &mut CleanupResult) {
    for rule in rules {
        if cancelled.load(Ordering::Relaxed) {
            return;
        }
        match expand_rule(rule) {
            Ok(roots) => scan_roots(&roots, cancelled, result),
            Err(error) => result.errors.push(error),
        }
    }
}

pub(super) fn clean_rules(rules: &[PathRule], result: &mut CleanupResult) {
    for rule in rules {
        match expand_rule(rule) {
            Ok(roots) => clean_roots(&roots, result),
            Err(error) => result.errors.push(error),
        }
    }
}

fn expand_rule(rule: &PathRule) -> Result<Vec<PathBuf>, String> {
    let base = rule
        .root
        .resolve()
        .ok_or_else(|| format!("Environment root is unavailable for {}", rule.path))?;
    let escaped_base = Pattern::escape(&base.to_string_lossy());
    let pattern_text = format!(
        "{escaped_base}{}{path}",
        std::path::MAIN_SEPARATOR,
        path = rule.path
    );
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };
    let mut paths = Vec::new();
    let entries = glob_with(&pattern_text, options).map_err(|error| error.to_string())?;
    for entry in entries {
        match entry {
            Ok(path) if is_safe_resolved_path(&base, &path) => paths.push(path),
            Ok(path) => {
                return Err(format!(
                    "Skipped unsafe path outside or linked through {}: {}",
                    base.display(),
                    path.display()
                ));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

pub(super) fn scan_roots(roots: &[PathBuf], cancelled: &AtomicBool, result: &mut CleanupResult) {
    let mut seen = HashSet::new();
    for root in roots {
        if cancelled.load(Ordering::Relaxed) || !root.exists() || !seen.insert(root.clone()) {
            continue;
        }
        result.available = true;
        let mut stats = ScanStats::default();
        scan_path(root, cancelled, &mut stats);
        result.files += stats.files;
        result.bytes += stats.bytes;
        result.errors.extend(stats.errors);
    }
}

pub(super) fn scan_path(path: &Path, cancelled: &AtomicBool, stats: &mut ScanStats) {
    if cancelled.load(Ordering::Relaxed) {
        return;
    }
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => {
            stats.errors.push(format!("{}: {error}", path.display()));
            return;
        }
    };
    if is_reparse(&metadata) {
        stats
            .errors
            .push(format!("Skipped reparse point {}", path.display()));
        return;
    }
    if metadata.is_file() {
        stats.files += 1;
        stats.bytes += metadata.len();
        return;
    }
    if !metadata.is_dir() {
        return;
    }
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => {
            stats.errors.push(format!("{}: {error}", path.display()));
            return;
        }
    };
    for entry in entries {
        match entry {
            Ok(entry) => scan_path(&entry.path(), cancelled, stats),
            Err(error) => stats.errors.push(error.to_string()),
        }
    }
}

pub(super) fn clean_roots(roots: &[PathBuf], result: &mut CleanupResult) {
    let mut seen = HashSet::new();
    for root in roots {
        if !root.exists() || !seen.insert(root.clone()) {
            continue;
        }
        let metadata = match fs::symlink_metadata(root) {
            Ok(metadata) => metadata,
            Err(error) => {
                result.errors.push(format!("{}: {error}", root.display()));
                continue;
            }
        };
        if is_reparse(&metadata) {
            result.skipped += 1;
            result
                .errors
                .push(format!("Skipped reparse point {}", root.display()));
            continue;
        }
        if metadata.is_file() {
            delete_file(root, &metadata, result);
            continue;
        }
        let entries = match fs::read_dir(root) {
            Ok(entries) => entries,
            Err(error) => {
                result.errors.push(format!("{}: {error}", root.display()));
                continue;
            }
        };
        for entry in entries {
            match entry {
                Ok(entry) => delete_tree(&entry.path(), result),
                Err(error) => {
                    result.skipped += 1;
                    result.errors.push(error.to_string());
                }
            }
        }
    }
}

fn delete_tree(path: &Path, result: &mut CleanupResult) {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => {
            result.skipped += 1;
            result.errors.push(format!("{}: {error}", path.display()));
            return;
        }
    };
    if is_reparse(&metadata) {
        result.skipped += 1;
        result
            .errors
            .push(format!("Skipped reparse point {}", path.display()));
    } else if metadata.is_file() {
        delete_file(path, &metadata, result);
    } else if metadata.is_dir() {
        let mut before = ScanStats::default();
        scan_path(path, &AtomicBool::new(false), &mut before);

        // On Windows, std::fs::remove_dir_all opens children relative to their
        // parent handles with FILE_OPEN_REPARSE_POINT and OBJ_DONT_REPARSE. This
        // prevents a concurrent junction swap from redirecting elevated cleanup.
        match fs::remove_dir_all(path) {
            Ok(()) => {
                result.cleaned_files += before.files;
                result.cleaned_bytes += before.bytes;
            }
            Err(error) => {
                let mut remaining = ScanStats::default();
                scan_path(path, &AtomicBool::new(false), &mut remaining);
                result.cleaned_files += before.files.saturating_sub(remaining.files);
                result.cleaned_bytes += before.bytes.saturating_sub(remaining.bytes);
                result.skipped += remaining.files.max(1);
                result.errors.push(format!("{}: {error}", path.display()));
            }
        }
    }
}

fn delete_file(path: &Path, metadata: &fs::Metadata, result: &mut CleanupResult) {
    match fs::remove_file(path) {
        Ok(()) => {
            result.cleaned_files += 1;
            result.cleaned_bytes += metadata.len();
        }
        Err(error) => {
            result.skipped += 1;
            result.errors.push(format!("{}: {error}", path.display()));
        }
    }
}

pub(super) fn is_within(base: &Path, candidate: &Path) -> bool {
    candidate.starts_with(base)
}

fn is_safe_resolved_path(base: &Path, candidate: &Path) -> bool {
    if !is_within(base, candidate) {
        return false;
    }
    let (Ok(canonical_base), Ok(canonical_candidate)) =
        (fs::canonicalize(base), fs::canonicalize(candidate))
    else {
        return false;
    };
    if !is_within(&canonical_base, &canonical_candidate) {
        return false;
    }
    candidate
        .ancestors()
        .take_while(|path| is_within(base, path))
        .all(|path| fs::symlink_metadata(path).is_ok_and(|metadata| !is_reparse(&metadata)))
}

pub(super) fn is_reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}
