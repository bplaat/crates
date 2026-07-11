/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::path::{Component, Path};

pub(crate) fn password_hash(password: &str) -> String {
    let mut salt = [0u8; 16];
    getrandom::fill(&mut salt).expect("Can't get random bytes");
    pbkdf2::password_hash_customized(password, &salt, crate::consts::PBKDF2_ITERATIONS)
}

pub(crate) fn validate_project_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 63 {
        return false;
    }
    let bytes = name.as_bytes();
    (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && (bytes[name.len() - 1].is_ascii_lowercase() || bytes[name.len() - 1].is_ascii_digit())
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}

pub(crate) fn normalize_base_dir(base_dir: &str) -> Option<String> {
    let raw = base_dir.trim();
    if raw.is_empty() {
        return Some(String::new());
    }
    if Path::new(raw).is_absolute() || raw.starts_with('/') {
        return None;
    }
    let trimmed = raw.trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = Vec::new();
    for component in Path::new(trimmed).components() {
        match component {
            Component::Normal(part) => {
                let part = part.to_str()?;
                if part.is_empty() || part.contains('\\') || part.chars().any(|c| c.is_control()) {
                    return None;
                }
                parts.push(part);
            }
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return None;
            }
        }
    }

    Some(parts.join("/"))
}
