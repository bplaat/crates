/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::slice::{Iter, IterMut};

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

    /// Get number of headers
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get iterator
    pub fn iter(&self) -> Iter<'_, (String, String)> {
        self.0.iter()
    }
}

impl IntoIterator for HeaderMap {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<(String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a HeaderMap {
    type Item = &'a (String, String);
    type IntoIter = Iter<'a, (String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut HeaderMap {
    type Item = &'a mut (String, String);
    type IntoIter = IterMut<'a, (String, String)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
