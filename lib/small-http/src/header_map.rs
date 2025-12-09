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
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Insert header
    pub fn insert(&mut self, name: String, value: String) {
        self.0.push((name, value));
    }
}

impl<'a> IntoIterator for &'a HeaderMap {
    type Item = (&'a str, &'a str);
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, (String, String)>,
        fn(&(String, String)) -> (&str, &str),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().map(|(n, v)| (n.as_str(), v.as_str()))
    }
}
