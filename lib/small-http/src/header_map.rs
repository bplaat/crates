/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

/// HeaderMap
#[derive(Default, Clone)]
pub struct HeaderMap(Vec<(String, String)>);

impl HeaderMap {
    /// Create new HeaderMap
    pub fn new() -> Self {
        Self::default()
    }

    /// Get header value
    pub fn get(&self, name: &str) -> Option<&String> {
        self.0.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    /// Insert header
    pub fn insert(&mut self, name: String, value: String) {
        self.0.push((name, value));
    }

    /// Iterate over headers
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.0.iter().map(|(n, v)| (n, v))
    }
}
