/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use super::filesystem::{ScanStats, scan_path, scan_roots};
use super::system::docker_executable;
use super::{CleanupResult, cancellable_output, command};

pub(super) fn scan_docker_build_cache(cancelled: &AtomicBool, result: &mut CleanupResult) {
    let Some(executable) = docker_executable() else {
        return;
    };
    let mut command = command(executable);
    command.args(["builder", "du"]);
    match cancellable_output(&mut command, cancelled) {
        Ok(Some(output)) if output.status.success() => {
            result.available = true;
            result.unknown_size = true;
        }
        Ok(_) | Err(_) => {}
    }
}

pub(super) fn clean_docker_build_cache(result: &mut CleanupResult) {
    result.unknown_size = true;
    let Some(executable) = docker_executable() else {
        result.errors.push("Docker is unavailable".to_string());
        return;
    };
    match command(executable)
        .args(["builder", "prune", "--force"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            result.cleaned_bytes = parse_reclaimed_size(&text).unwrap_or_default();
        }
        Ok(output) => result
            .errors
            .push(format!("Docker exited with {}", output.status)),
        Err(error) => result.errors.push(format!("Could not run Docker: {error}")),
    }
}

pub(super) fn uv_cache_dir(cancelled: Option<&AtomicBool>) -> Option<PathBuf> {
    let mut command = command("uv.exe");
    command.args(["cache", "dir"]);
    let output = match cancelled {
        Some(cancelled) => cancellable_output(&mut command, cancelled).ok()??,
        None => command.output().ok()?,
    };
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!path.is_empty()).then(|| PathBuf::from(path))
}

pub(super) fn scan_uv_cache(cancelled: &AtomicBool, result: &mut CleanupResult) {
    if let Some(path) = uv_cache_dir(Some(cancelled)) {
        scan_roots(&[path], cancelled, result);
    }
}

pub(super) fn clean_uv_cache(result: &mut CleanupResult) {
    let before = uv_cache_dir(None).map(|path| {
        let mut stats = ScanStats::default();
        scan_path(&path, &AtomicBool::new(false), &mut stats);
        stats
    });
    match command("uv.exe").args(["cache", "clean"]).output() {
        Ok(output) if output.status.success() => {
            if let Some(before) = before {
                let mut remaining = ScanStats::default();
                if let Some(path) = uv_cache_dir(None) {
                    scan_path(&path, &AtomicBool::new(false), &mut remaining);
                }
                result.cleaned_files = before.files.saturating_sub(remaining.files);
                result.cleaned_bytes = before.bytes.saturating_sub(remaining.bytes);
            }
        }
        Ok(output) => result
            .errors
            .push(format!("uv exited with {}", output.status)),
        Err(error) => result.errors.push(format!("Could not run uv: {error}")),
    }
}

pub(super) fn parse_reclaimed_size(output: &str) -> Option<u64> {
    let value = output
        .lines()
        .rev()
        .find_map(|line| line.split_once(':').map(|(_, value)| value.trim()))?;
    let unit_start = value.find(|character: char| character.is_ascii_alphabetic())?;
    let number: f64 = value[..unit_start].trim().parse().ok()?;
    let multiplier = match value[unit_start..].trim().to_ascii_lowercase().as_str() {
        "b" => 1.0,
        "kb" => 1_000.0,
        "mb" => 1_000_000.0,
        "gb" => 1_000_000_000.0,
        "tb" => 1_000_000_000_000.0,
        _ => return None,
    };
    Some((number * multiplier) as u64)
}
