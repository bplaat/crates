/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A simple INI file parser library

use std::collections::HashMap;

/// Config file (.ini file)
#[derive(Default, Clone)]
pub struct ConfigFile {
    groups: HashMap<String, Group>,
}

#[derive(Default, Clone)]
struct Group {
    properties: HashMap<String, String>,
}

impl ConfigFile {
    /// Create a new empty config file
    pub fn new() -> Self {
        ConfigFile::default()
    }

    /// Load config file from path
    pub fn load_from_path(path: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        Self::load_from_str(&std::fs::read_to_string(path)?)
    }

    /// Load config file from string
    pub fn load_from_str(s: &str) -> Result<Self, std::io::Error> {
        let mut config: ConfigFile = ConfigFile::new();
        let mut current_group = String::new();

        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }

            // Remove inline comments (after ';' or '#')
            let line = match line.find([';', '#']) {
                Some(idx) => &line[..idx],
                None => line,
            }
            .trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_group = line[1..line.len() - 1].trim().to_string();
                config.groups.entry(current_group.clone()).or_default();
            } else if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let mut value = value.trim().to_string();

                // Escape quotes
                if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                    value = value[1..value.len() - 1].to_string();
                }

                config
                    .groups
                    .entry(current_group.clone())
                    .or_default()
                    .properties
                    .insert(key, value);
            }
        }

        Ok(config)
    }

    /// Get all group names
    pub fn groups(&self) -> impl Iterator<Item = &str> {
        self.groups.keys().map(|s| s.as_str())
    }

    /// Get all keys in a group
    pub fn keys(&self, group: &str) -> Option<impl Iterator<Item = &str>> {
        self.groups
            .get(group)
            .map(|s| s.properties.keys().map(|k| k.as_str()))
    }

    /// Read a string value
    pub fn read_string(&self, group: &str, key: &str) -> Option<&str> {
        self.groups
            .get(group)
            .and_then(|s| s.properties.get(key).map(|s| s.as_str()))
    }

    /// Read a boolean value
    pub fn read_bool(&self, group: &str, key: &str) -> Option<bool> {
        self.read_string(group, key)
            .and_then(|v| match v.to_lowercase().as_str() {
                "true" | "1" => Some(true),
                "false" | "0" => Some(false),
                _ => None,
            })
    }

    /// Read an integer value
    pub fn read_i32(&self, group: &str, key: &str) -> Option<i32> {
        self.read_string(group, key).and_then(|v| v.parse().ok())
    }

    /// Read an integer value
    pub fn read_u32(&self, group: &str, key: &str) -> Option<u32> {
        self.read_string(group, key).and_then(|v| v.parse().ok())
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parsing() {
        let ini_str = r#"
            ; This is a comment
               # This is another comment

            [group1]
              key1 =  value1
            key2   =  "value2 with spaces"
            key3 = true          ; test
            key4   = 42

            [group2]
            keyA = "  valueA"
               keyB = false
            keyC = 100
        "#;

        let config = ConfigFile::load_from_str(ini_str).unwrap();

        let groups: Vec<_> = config.groups().collect();
        assert!(groups.contains(&"group1"));
        assert!(groups.contains(&"group2"));

        let group1_keys: Vec<_> = config.keys("group1").unwrap().collect();
        assert!(group1_keys.contains(&"key1"));
        assert!(group1_keys.contains(&"key2"));
        assert!(group1_keys.contains(&"key3"));
        assert!(group1_keys.contains(&"key4"));

        let group2_keys: Vec<_> = config.keys("group2").unwrap().collect();
        assert!(group2_keys.contains(&"keyA"));
        assert!(group2_keys.contains(&"keyB"));
        assert!(group2_keys.contains(&"keyC"));

        assert_eq!(config.read_string("group1", "key1"), Some("value1"));
        assert_eq!(
            config.read_string("group1", "key2"),
            Some("value2 with spaces")
        );
        assert_eq!(config.read_bool("group1", "key3"), Some(true));
        assert_eq!(config.read_i32("group1", "key4"), Some(42));

        assert_eq!(config.read_string("group2", "keyA"), Some("  valueA"));
        assert_eq!(config.read_bool("group2", "keyB"), Some(false));
        assert_eq!(config.read_u32("group2", "keyC"), Some(100));
    }
}
