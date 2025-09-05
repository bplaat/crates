/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::ser::{Serialize, Serializer};

// MARK: JSON Serializer
/// JSON serializer
#[derive(Default)]
struct JsonSerializer {
    output: String,
}

impl JsonSerializer {
    /// Create a new JSON serializer
    fn new() -> Self {
        Self::default()
    }

    /// Get the output
    fn output(self) -> String {
        self.output
    }
}

impl Serializer for JsonSerializer {
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
        self.output.push(value);
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

    // Option
    fn serialize_some(&mut self, value: &dyn Serialize) {
        value.serialize(self);
    }
    fn serialize_none(&mut self) {
        self.output.push_str("null");
    }

    // Seq
    fn serialize_start_seq(&mut self, _len: usize) {
        self.output.push('[');
    }

    fn serialize_end_seq(&mut self) {
        self.output.push(']');
    }

    fn serialize_element(&mut self, value: &dyn Serialize) {
        if !self.output.ends_with('[') {
            self.output.push(',');
        }
        value.serialize(self);
    }

    // Struct
    fn serialize_start_struct(&mut self, _name: &str, _len: usize) {
        self.output.push('{');
    }

    fn serialize_end_struct(&mut self) {
        self.output.push('}');
    }

    fn serialize_field(&mut self, name: &str, value: &dyn Serialize) {
        if !self.output.ends_with('{') {
            self.output.push(',');
        }
        self.output.push('"');
        self.output.push_str(name);
        self.output.push_str("\":");
        value.serialize(self);
    }
}

/// Convert a value to a JSON string
pub fn to_string<T: Serialize>(value: &T) -> String {
    let mut serializer = JsonSerializer::new();
    value.serialize(&mut serializer);
    serializer.output()
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::de::{Deserialize, DeserializeError, Deserializer};

    enum Color {
        // #[berde(rename = "red")]
        Red,
        // #[berde(rename = "green")]
        Green,
        // #[berde(rename = "blue")]
        Blue,
    }

    impl Serialize for Color {
        fn serialize(&self, serializer: &mut dyn Serializer) {
            match self {
                Color::Red => "red",
                Color::Green => "green",
                Color::Blue => "blue",
            }
            .serialize(serializer);
        }
    }

    impl Deserialize for Color {
        fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
            match deserializer.deserialize_str()? {
                "red" => Ok(Color::Red),
                "green" => Ok(Color::Green),
                "blue" => Ok(Color::Blue),
                _ => Err(DeserializeError),
            }
        }
    }

    #[derive(crate::Serialize, crate::Deserialize)]
    struct Person {
        name: String,
        age: u8,
        color: Color,
    }

    #[test]
    fn test_struct_serialize() {
        let person = Person {
            name: "Alice".to_string(),
            age: 30,
            color: Color::Red,
        };
        assert_eq!(
            to_string(&person),
            r#"{"name":"Alice","age":30,"color":"red"}"#
        );
    }

    #[test]
    fn test_vec_serialize() {
        let persons = vec![
            Person {
                name: "Alice".to_string(),
                age: 30,
                color: Color::Blue,
            },
            Person {
                name: "Bob".to_string(),
                age: 25,
                color: Color::Green,
            },
        ];
        assert_eq!(
            to_string(&persons),
            r#"[{"name":"Alice","age":30,"color":"blue"},{"name":"Bob","age":25,"color":"green"}]"#
        );
    }
}
