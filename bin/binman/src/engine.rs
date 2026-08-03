/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use glob::{MatchOptions, glob_with};
use serde::Serialize;

use crate::catalog::{CleanupDefinition, CleanupKind, PathRoot, PathRule};

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CleanupResult {
    pub(crate) id: String,
    pub(crate) available: bool,
    pub(crate) files: u64,
    pub(crate) bytes: u64,
    pub(crate) unknown_size: bool,
    pub(crate) cleaned_files: u64,
    pub(crate) cleaned_bytes: u64,
    pub(crate) skipped: u64,
    pub(crate) errors: Vec<String>,
}

#[derive(Default)]
struct ScanStats {
    files: u64,
    bytes: u64,
    errors: Vec<String>,
}

pub(crate) fn scan(
    cleanup: &CleanupDefinition,
    cancelled: &AtomicBool,
    is_elevated: bool,
) -> CleanupResult {
    let mut result = CleanupResult {
        id: cleanup.id.clone(),
        ..CleanupResult::default()
    };

    match &cleanup.kind {
        CleanupKind::Paths { rules } => scan_rules(rules, cancelled, &mut result),
        CleanupKind::RecycleBin => {
            if let (Some(root), Some(sid)) = (PathRoot::SystemDrive.resolve(), current_user_sid()) {
                scan_roots(
                    &[root.join("$Recycle.Bin").join(sid)],
                    cancelled,
                    &mut result,
                );
            }
        }
        CleanupKind::WindowsUpdate => {
            if let Some(root) = PathRoot::Windows.resolve() {
                scan_roots(
                    &[root.join("SoftwareDistribution").join("Download")],
                    cancelled,
                    &mut result,
                );
            }
        }
        CleanupKind::DeliveryOptimization => {
            if let Some(root) = PathRoot::Windows.resolve() {
                scan_roots(
                    &[root.join("ServiceProfiles/NetworkService/AppData/Local/Microsoft/Windows/DeliveryOptimization/Cache")],
                    cancelled,
                    &mut result,
                );
            }
        }
        CleanupKind::DismComponentStore => scan_dism(&mut result),
        CleanupKind::DockerBuildCache => scan_docker_build_cache(&mut result),
        CleanupKind::UvCache if !is_elevated => scan_uv_cache(cancelled, &mut result),
        CleanupKind::UvCache => {}
    }
    result
}

pub(crate) fn clean(cleanup: &CleanupDefinition, is_elevated: bool) -> CleanupResult {
    let mut result = CleanupResult {
        id: cleanup.id.clone(),
        available: true,
        ..CleanupResult::default()
    };

    let running: Vec<_> = cleanup
        .process_names
        .iter()
        .filter(|name| process_is_running(name))
        .collect();
    if !running.is_empty() {
        result.skipped = 1;
        result.errors.push(format!(
            "Close {} before cleaning this cache",
            running
                .iter()
                .map(|name| name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
        return result;
    }

    match &cleanup.kind {
        CleanupKind::Paths { rules } => clean_rules(rules, &mut result),
        CleanupKind::RecycleBin => {
            if let (Some(root), Some(sid)) = (PathRoot::SystemDrive.resolve(), current_user_sid()) {
                clean_roots(&[root.join("$Recycle.Bin").join(sid)], &mut result);
            }
        }
        CleanupKind::WindowsUpdate => clean_windows_update(&mut result),
        CleanupKind::DeliveryOptimization => clean_delivery_optimization(&mut result),
        CleanupKind::DismComponentStore => run_dism_cleanup(&mut result),
        CleanupKind::DockerBuildCache => clean_docker_build_cache(&mut result),
        CleanupKind::UvCache if !is_elevated => clean_uv_cache(&mut result),
        CleanupKind::UvCache => result
            .errors
            .push("uv cleanup is disabled while Binman is elevated".to_string()),
    }
    result
}

fn scan_rules(rules: &[PathRule], cancelled: &AtomicBool, result: &mut CleanupResult) {
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

fn clean_rules(rules: &[PathRule], result: &mut CleanupResult) {
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
    let pattern = base.join(&rule.path);
    let pattern_text = pattern.to_string_lossy();
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };
    let mut paths = Vec::new();
    let entries = glob_with(&pattern_text, options).map_err(|error| error.to_string())?;
    for entry in entries {
        match entry {
            Ok(path) if is_within(&base, &path) => paths.push(path),
            Ok(_) => return Err(format!("Resolved path escaped {}", base.display())),
            Err(error) => return Err(error.to_string()),
        }
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn scan_roots(roots: &[PathBuf], cancelled: &AtomicBool, result: &mut CleanupResult) {
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

fn scan_path(path: &Path, cancelled: &AtomicBool, stats: &mut ScanStats) {
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

fn clean_roots(roots: &[PathBuf], result: &mut CleanupResult) {
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

fn clean_windows_update(result: &mut CleanupResult) {
    let Some(windows) = PathRoot::Windows.resolve() else {
        result
            .errors
            .push("Windows directory is unavailable".to_string());
        return;
    };
    let target = windows.join("SoftwareDistribution").join("Download");
    let mut stopped = Vec::new();
    let mut ready = true;
    for name in ["bits", "wuauserv"] {
        match service_state(name) {
            Some(1) => {}
            Some(4) => {
                if service_control("stop", name) && wait_for_service_state(name, 1) {
                    stopped.push(name);
                } else {
                    result
                        .errors
                        .push(format!("Could not stop the {name} service"));
                    ready = false;
                    break;
                }
            }
            Some(_) => {
                result.errors.push(format!("The {name} service is busy"));
                ready = false;
                break;
            }
            None => {
                result
                    .errors
                    .push(format!("Could not query the {name} service"));
                ready = false;
                break;
            }
        }
    }
    if ready {
        clean_roots(&[target], result);
    }
    for name in stopped {
        if !service_control("start", name) || !wait_for_service_state(name, 4) {
            result
                .errors
                .push(format!("Could not restart the {name} service"));
        }
    }
}

pub(crate) fn system_executable(relative_path: &str) -> Option<PathBuf> {
    if Path::new(relative_path).is_absolute()
        || Path::new(relative_path)
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }
    let path = PathRoot::Windows
        .resolve()?
        .join("System32")
        .join(relative_path);
    trusted_executable(path)
}

fn docker_executable() -> Option<PathBuf> {
    trusted_executable(
        PathRoot::ProgramFiles
            .resolve()?
            .join("Docker/Docker/resources/bin/docker.exe"),
    )
}

fn trusted_executable(path: PathBuf) -> Option<PathBuf> {
    let metadata = fs::symlink_metadata(&path).ok()?;
    (metadata.is_file() && !is_reparse(&metadata)).then_some(path)
}

fn process_is_running(name: &str) -> bool {
    let Some(executable) = system_executable("tasklist.exe") else {
        return false;
    };
    Command::new(executable)
        .args(["/FI", &format!("IMAGENAME eq {name}"), "/FO", "CSV", "/NH"])
        .output()
        .is_ok_and(|output| {
            let text = String::from_utf8_lossy(&output.stdout);
            text.lines().any(|line| {
                line.trim_start_matches('\u{feff}')
                    .trim_start_matches('"')
                    .split('"')
                    .next()
                    .is_some_and(|image| image.eq_ignore_ascii_case(name))
            })
        })
}

fn current_user_sid() -> Option<String> {
    let output = Command::new(system_executable("whoami.exe")?)
        .args(["/user", "/fo", "csv", "/nh"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.split('"')
        .find(|field| field.starts_with("S-1-"))
        .map(str::to_string)
}

pub(crate) fn disk_free_space() -> Option<u64> {
    let output = Command::new(system_executable("WindowsPowerShell/v1.0/powershell.exe")?)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "[Console]::Write((Get-PSDrive -Name $env:SystemDrive.TrimEnd(':')).Free)",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

fn service_state(name: &str) -> Option<u32> {
    let output = Command::new(system_executable("sc.exe")?)
        .args(["query", name])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_service_state(&String::from_utf8_lossy(&output.stdout))
}

fn parse_service_state(output: &str) -> Option<u32> {
    output.lines().find_map(|line| {
        let value = line
            .split_once(':')?
            .1
            .split_whitespace()
            .next()?
            .parse()
            .ok()?;
        (1..=7).contains(&value).then_some(value)
    })
}

fn service_control(action: &str, name: &str) -> bool {
    let Some(executable) = system_executable("sc.exe") else {
        return false;
    };
    Command::new(executable)
        .args([action, name])
        .status()
        .is_ok_and(|status| status.success())
}

fn wait_for_service_state(name: &str, expected: u32) -> bool {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if service_state(name) == Some(expected) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(200));
    }
}

fn clean_delivery_optimization(result: &mut CleanupResult) {
    let Some(executable) = system_executable("WindowsPowerShell/v1.0/powershell.exe") else {
        result
            .errors
            .push("Windows PowerShell is unavailable".to_string());
        return;
    };
    let status = Command::new(executable)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Delete-DeliveryOptimizationCache -Force",
        ])
        .status();
    if !status.is_ok_and(|status| status.success()) {
        result
            .errors
            .push("Delivery Optimization cleanup failed".to_string());
    }
}

fn scan_dism(result: &mut CleanupResult) {
    result.available = true;
    result.unknown_size = true;
    let Some(executable) = system_executable("dism.exe") else {
        result.errors.push("DISM is unavailable".to_string());
        return;
    };
    match Command::new(executable)
        .args(["/Online", "/Cleanup-Image", "/AnalyzeComponentStore"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            result.files = parse_reclaimable_packages(&text).unwrap_or_default();
        }
        Ok(output) => result
            .errors
            .push(format!("DISM exited with {}", output.status)),
        Err(error) => result.errors.push(format!("Could not run DISM: {error}")),
    }
}

fn run_dism_cleanup(result: &mut CleanupResult) {
    result.unknown_size = true;
    let Some(executable) = system_executable("dism.exe") else {
        result.errors.push("DISM is unavailable".to_string());
        return;
    };
    match Command::new(executable)
        .args(["/Online", "/Cleanup-Image", "/StartComponentCleanup"])
        .status()
    {
        Ok(status) if status.success() => {}
        Ok(status) => result.errors.push(format!("DISM exited with {status}")),
        Err(error) => result.errors.push(format!("Could not run DISM: {error}")),
    }
}

fn scan_docker_build_cache(result: &mut CleanupResult) {
    let Some(executable) = docker_executable() else {
        return;
    };
    match Command::new(executable).args(["builder", "du"]).output() {
        Ok(output) if output.status.success() => {
            result.available = true;
            result.unknown_size = true;
        }
        Ok(_) | Err(_) => {}
    }
}

fn clean_docker_build_cache(result: &mut CleanupResult) {
    result.unknown_size = true;
    let Some(executable) = docker_executable() else {
        result.errors.push("Docker is unavailable".to_string());
        return;
    };
    match Command::new(executable)
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

fn uv_cache_dir() -> Option<PathBuf> {
    let output = Command::new("uv.exe")
        .args(["cache", "dir"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!path.is_empty()).then(|| PathBuf::from(path))
}

fn scan_uv_cache(cancelled: &AtomicBool, result: &mut CleanupResult) {
    if let Some(path) = uv_cache_dir() {
        scan_roots(&[path], cancelled, result);
    }
}

fn clean_uv_cache(result: &mut CleanupResult) {
    let before = uv_cache_dir().map(|path| {
        let mut stats = ScanStats::default();
        scan_path(&path, &AtomicBool::new(false), &mut stats);
        stats
    });
    match Command::new("uv.exe").args(["cache", "clean"]).output() {
        Ok(output) if output.status.success() => {
            if let Some(before) = before {
                let mut remaining = ScanStats::default();
                if let Some(path) = uv_cache_dir() {
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

fn parse_reclaimed_size(output: &str) -> Option<u64> {
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

fn parse_reclaimable_packages(output: &str) -> Option<u64> {
    output.lines().find_map(|line| {
        line.strip_prefix("Number of Reclaimable Packages :")
            .and_then(|value| value.trim().parse().ok())
    })
}

fn is_within(base: &Path, candidate: &Path) -> bool {
    candidate.starts_with(base)
}

#[cfg(windows)]
fn is_reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_dism_package_count() {
        let output = "Number of Reclaimable Packages : 10\r\n";
        assert_eq!(parse_reclaimable_packages(output), Some(10));
    }

    #[test]
    fn parses_external_reclaimed_sizes() {
        assert_eq!(
            parse_reclaimed_size("Total reclaimed space: 1.25GB\n"),
            Some(1_250_000_000)
        );
        assert_eq!(parse_reclaimed_size("Total: 42.5MB\n"), Some(42_500_000));
        assert_eq!(parse_reclaimed_size("No size here"), None);
    }

    #[test]
    fn parses_service_state_after_service_type() {
        let output =
            "TYPE : 20 WIN32_SHARE_PROCESS\r\nSTATE : 4 RUNNING\r\nWIN32_EXIT_CODE : 0\r\n";
        assert_eq!(parse_service_state(output), Some(4));
    }

    #[test]
    fn trusted_system_tool_paths_reject_parent_components() {
        assert!(system_executable("../evil.exe").is_none());
    }

    #[test]
    fn containment_rejects_siblings() {
        assert!(is_within(
            Path::new("C:\\Root"),
            Path::new("C:\\Root\\Cache")
        ));
        assert!(!is_within(Path::new("C:\\Root"), Path::new("C:\\Other")));
    }

    #[test]
    fn scanning_is_read_only_and_cleaning_keeps_the_root() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("binman-test-{unique}"));
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("fixture directory should be created");
        fs::write(nested.join("cache.bin"), b"cache").expect("fixture file should be created");

        let mut scan_result = CleanupResult::default();
        scan_roots(
            std::slice::from_ref(&root),
            &AtomicBool::new(false),
            &mut scan_result,
        );
        assert_eq!(scan_result.files, 1);
        assert_eq!(scan_result.bytes, 5);
        assert!(nested.join("cache.bin").exists());

        let mut clean_result = CleanupResult::default();
        clean_roots(std::slice::from_ref(&root), &mut clean_result);
        assert!(root.exists());
        assert!(
            fs::read_dir(&root)
                .expect("fixture root should remain")
                .next()
                .is_none()
        );
        assert_eq!(clean_result.cleaned_files, 1);
        fs::remove_dir(&root).expect("empty fixture root should be removable");
    }
}
