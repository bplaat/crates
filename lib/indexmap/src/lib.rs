/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A order preserving map implementation.

use std::collections::HashMap;
use std::hash::Hash;

// MARK: IndexMap
/// A order preserving map implementation
pub struct IndexMap<K: Eq + Hash + Clone, V> {
    map: HashMap<K, V>,
    order: Vec<K>,
}

impl<K: Eq + Hash + Clone, V> Default for IndexMap<K, V> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            order: Vec::new(),
        }
    }
}

impl<K: Eq + Hash + Clone, V> IndexMap<K, V> {
    /// Create a new IndexMap
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a key value pair
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if !self.map.contains_key(&key) {
            self.order.push(key.clone());
        }
        self.map.insert(key, value)
    }

    /// Get a value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    /// Get a mutable reference to a value by key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    /// Remove a value by key
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(value) = self.map.remove(key) {
            self.order.retain(|k| k != key);
            Some(value)
        } else {
            None
        }
    }

    /// Check if the map contains a key
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Get the number of elements in the map
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clear the map
    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }

    /// Get an iterator over the key value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.order
            .iter()
            .filter_map(move |k| self.map.get(k).map(|v| (k, v)))
    }

    /// Get an iterator over the keys
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.order.iter()
    }

    /// Get an iterator over the values
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.order.iter().filter_map(move |k| self.map.get(k))
    }
}

#[cfg(feature = "berde")]
impl<K: Eq + Hash + Clone + ToString, V: berde::Serialize> berde::Serialize for IndexMap<K, V> {
    fn serialize(&self, serializer: &mut dyn berde::Serializer) {
        serializer.serialize_start_struct("IndexMap", self.len());
        for (key, value) in self.iter() {
            serializer.serialize_field(&key.to_string());
            value.serialize(serializer);
        }
        serializer.serialize_end_struct();
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut map = IndexMap::new();
        assert_eq!(map.insert("key1", 10), None);
        assert_eq!(map.get(&"key1"), Some(&10));
        assert_eq!(map.insert("key1", 20), Some(10));
        assert_eq!(map.get(&"key1"), Some(&20));
    }

    #[test]
    fn test_remove() {
        let mut map = IndexMap::new();
        map.insert("key1", 10);
        map.insert("key2", 20);
        assert_eq!(map.remove(&"key1"), Some(10));
        assert_eq!(map.get(&"key1"), None);
        assert_eq!(map.get(&"key2"), Some(&20));
    }

    #[test]
    fn test_order_preservation() {
        let mut map = IndexMap::new();
        map.insert("key1", 10);
        map.insert("key2", 20);
        map.insert("key3", 30);
        let keys: Vec<_> = map.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, vec!["key1", "key2", "key3"]);
    }

    #[test]
    fn test_iter() {
        let mut map = IndexMap::new();
        map.insert("key1", 10);
        map.insert("key2", 20);
        let items: Vec<_> = map.iter().collect();
        assert_eq!(items, vec![(&"key1", &10), (&"key2", &20)]);
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut map = IndexMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        map.insert("key1", 10);
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut map = IndexMap::new();
        map.insert("key1", 10);
        map.insert("key2", 20);
        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.get(&"key1"), None);
        assert_eq!(map.get(&"key2"), None);
    }

    #[test]
    fn test_keys_and_values() {
        let mut map = IndexMap::new();
        map.insert("key1", 10);
        map.insert("key2", 20);
        let keys: Vec<_> = map.keys().collect();
        assert_eq!(keys, vec![&"key1", &"key2"]);
        let values: Vec<_> = map.values().collect();
        assert_eq!(values, vec![&10, &20]);
    }

    #[cfg(feature = "berde")]
    #[test]
    fn test_serialize() {
        let mut map = IndexMap::new();
        map.insert("key1", 10);
        map.insert("key2", 20);
        let serialized = berde::ser::json::to_string(&map);
        assert_eq!(serialized, r#"{"key1":10,"key2":20}"#);
    }
}
