/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::ser::{Serialize, Serializer};

/// YAML serializer
struct YamlSerializer {
    output: String,
    indent: i32,
    skip_indent: bool,
}

impl Default for YamlSerializer {
    fn default() -> Self {
        Self {
            output: String::new(),
            indent: -2,
            skip_indent: false,
        }
    }
}

impl YamlSerializer {
    /// Create a new YAML serializer
    fn new() -> Self {
        Self::default()
    }

    /// Get the output
    fn output(self) -> String {
        self.output
    }

    fn append_indent(&mut self) {
        if self.skip_indent {
            self.skip_indent = false;
            return;
        }
        for _ in 0..self.indent {
            self.output.push(' ');
        }
    }
}

impl Serializer for YamlSerializer {
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
        self.output.push_str(value);
    }

    fn serialize_bytes(&mut self, value: &[u8]) {
        self.output.push_str(&format!("{:?}", value));
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
        self.indent += 2;
    }

    fn serialize_end_seq(&mut self) {
        self.indent -= 2;
    }

    fn serialize_element(&mut self, value: &dyn Serialize) {
        self.append_indent();
        self.output.push_str("- ");
        self.skip_indent = true;
        value.serialize(self);
        if !self.output.ends_with("\n") {
            self.output.push('\n');
        }
    }

    // Struct
    fn serialize_start_struct(&mut self, _name: &str, _len: usize) {
        if self.output.ends_with(": ") {
            self.output.pop();
            self.output.push('\n');
        }
        self.indent += 2;
    }

    fn serialize_end_struct(&mut self) {
        self.indent -= 2;
    }

    fn serialize_field(&mut self, name: &str, value: &dyn Serialize) {
        self.append_indent();
        self.output.push_str(name);
        self.output.push_str(": ");
        value.serialize(self);
        if !self.output.ends_with("\n") {
            self.output.push('\n');
        }
    }
}

/// Convert a value to a YAML string
pub fn to_string<T: Serialize>(value: &T) -> String {
    let mut serializer = YamlSerializer::new();
    value.serialize(&mut serializer);
    serializer.output()
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    enum Color {
        Red,
        Green,
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

    #[derive(berde_derive::Serialize)]
    struct Name {
        first: String,
        last: String,
    }

    #[derive(berde_derive::Serialize)]
    struct Person {
        name: Name,
        age: u8,
        color: Color,
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
            "name:\n  first: Alice\n  last: Smith\nage: 30\ncolor: red\n"
        );
    }

    #[test]
    fn test_vec_serialize() {
        let persons = vec![
            Person {
                name: Name {
                    first: "Alice".to_string(),
                    last: "Smith".to_string(),
                },
                age: 30,
                color: Color::Blue,
            },
            Person {
                name: Name {
                    first: "Bob".to_string(),
                    last: "Johnson".to_string(),
                },
                age: 25,
                color: Color::Green,
            },
        ];
        assert_eq!(
            to_string(&persons),
            "- name:\n    first: Alice\n    last: Smith\n  age: 30\n  color: blue\n- name:\n    first: Bob\n    last: Johnson\n  age: 25\n  color: green\n"
        );
    }
}
