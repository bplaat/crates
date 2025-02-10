use std::collections::HashMap;

use crate::ser::{Serialize, Serializer};

/// TOML serializer
#[derive(Default)]
struct TomlSerializer {
    output: String,
    nested_maps: HashMap<String, String>,
    current_map: Option<String>,
}

impl TomlSerializer {
    /// Create a new TOML serializer
    fn new() -> Self {
        Self::default()
    }

    /// Get the output
    fn output(mut self) -> String {
        for (key, value) in self.nested_maps {
            self.output.push_str(&format!("\n[{}]\n{}", key, value));
        }
        self.output
    }
}

impl Serializer for TomlSerializer {
    // Primitives
    fn serialize_bool(&mut self, value: bool) {
        self.output.push_str(if value { "true" } else { "false" });
    }

    fn serialize_i8(&mut self, value: i8) {
        self.output.push_str(&value.to_string());
    }

    fn serialize_i16(&mut self, value: i16) {
        self.output.push_str(&value.to_string());
    }

    fn serialize_i32(&mut self, value: i32) {
        self.output.push_str(&value.to_string());
    }

    fn serialize_i64(&mut self, value: i64) {
        self.output.push_str(&value.to_string());
    }

    fn serialize_u8(&mut self, value: u8) {
        self.output.push_str(&value.to_string());
    }

    fn serialize_u16(&mut self, value: u16) {
        self.output.push_str(&value.to_string());
    }

    fn serialize_u32(&mut self, value: u32) {
        self.output.push_str(&value.to_string());
    }

    fn serialize_u64(&mut self, value: u64) {
        self.output.push_str(&value.to_string());
    }

    fn serialize_f32(&mut self, value: f32) {
        self.output.push_str(&value.to_string());
    }

    fn serialize_f64(&mut self, value: f64) {
        self.output.push_str(&value.to_string());
    }

    fn serialize_char(&mut self, value: char) {
        self.output.push('"');
        self.output.push(value);
        self.output.push('"');
    }

    fn serialize_str(&mut self, value: &str) {
        self.output.push('"');
        self.output.push_str(value);
        self.output.push('"');
    }

    fn serialize_bytes(&mut self, value: &[u8]) {
        self.output.push('[');
        for (i, byte) in value.iter().enumerate() {
            if i > 0 {
                self.output.push(',');
            }
            self.output.push_str(&byte.to_string());
        }
        self.output.push(']');
    }

    fn serialize_none(&mut self) {
        self.output.push_str("null");
    }

    // Sequence
    fn serialize_start_seq(&mut self, _len: usize) {
        self.output.push('[');
    }

    fn serialize_end_seq(&mut self) {
        self.output.push(']');
    }

    fn serialize_start_element(&mut self) {
        if !self.output.ends_with('[') {
            self.output.push(',');
        }
    }

    fn serialize_end_element(&mut self) {}

    // Map
    fn serialize_start_map(&mut self, name: &str, _len: usize) {
        self.current_map = Some(name.to_string());
    }

    fn serialize_end_map(&mut self) {
        self.current_map = None;
    }

    fn serialize_start_field(&mut self, name: &str) {
        if let Some(ref map_name) = self.current_map {
            let entry = self.nested_maps.entry(map_name.clone()).or_default();
            if !entry.is_empty() {
                entry.push('\n');
            }
            entry.push_str(name);
            entry.push_str(" = ");
        } else {
            if !self.output.is_empty() {
                self.output.push('\n');
            }
            self.output.push_str(name);
            self.output.push_str(" = ");
        }
    }

    fn serialize_end_field(&mut self) {}
}

/// Convert a value to a TOML string
pub fn to_string<T: Serialize>(value: &T) -> String {
    let mut serializer = TomlSerializer::new();
    value.serialize(&mut serializer);
    serializer.output()
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    enum Color {
        Red,
    }

    impl Serialize for Color {
        fn serialize(&self, serializer: &mut dyn Serializer) {
            match self {
                Color::Red => "red",
            }
            .serialize(serializer);
        }
    }

    struct Name {
        first: String,
        last: String,
    }

    impl Serialize for Name {
        fn serialize(&self, serializer: &mut dyn Serializer) {
            serializer.serialize_start_map("Name", 2);
            serializer.serialize_start_field("first");
            self.first.serialize(serializer);
            serializer.serialize_end_field();
            serializer.serialize_start_field("last");
            self.last.serialize(serializer);
            serializer.serialize_end_field();
            serializer.serialize_end_map();
        }
    }

    struct Person {
        name: Name,
        age: u8,
        color: Color,
    }
    impl Serialize for Person {
        fn serialize(&self, serializer: &mut dyn Serializer) {
            serializer.serialize_start_map("Person", 3);
            serializer.serialize_start_field("name");
            self.name.serialize(serializer);
            serializer.serialize_end_field();
            serializer.serialize_start_field("age");
            self.age.serialize(serializer);
            serializer.serialize_end_field();
            serializer.serialize_start_field("color");
            self.color.serialize(serializer);
            serializer.serialize_end_field();
            serializer.serialize_end_map();
        }
    }

    #[test]
    fn test_struct_serialize() {
        let person = Person {
            name: Name {
                first: "Alice".to_string(),
                last: "Smith".to_string(),
            },
            age: 30,
            color: Color::Red,
        };
        assert_eq!(
            to_string(&person),
            "age = 30\ncolor = \"red\"\n[name]\nfirst = \"Alice\"\nlast = \"Smith\"\n"
        );
    }
}
