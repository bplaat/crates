/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use crate::ser::{Serialize, Serializer};

fn percent_encoding(string: &str) -> String {
    string
        .replace("%", "%25")
        .replace(" ", "%20")
        .replace("!", "%21")
        .replace("\"", "%22")
        .replace("#", "%23")
        .replace("$", "%24")
        .replace("&", "%26")
        .replace("'", "%27")
        .replace("(", "%28")
        .replace(")", "%29")
        .replace("*", "%2A")
        .replace("+", "%2B")
        .replace(",", "%2C")
        .replace("/", "%2F")
        .replace(":", "%3A")
        .replace(";", "%3B")
        .replace("=", "%3D")
        .replace("?", "%3F")
        .replace("@", "%40")
        .replace("[", "%5B")
        .replace("]", "%5D")
}

/// URL encoded serializer
#[derive(Default)]
struct UrlEncodedSerializer {
    output: String,
}

impl UrlEncodedSerializer {
    /// Create a new URL encoded serializer
    fn new() -> Self {
        Self::default()
    }

    /// Get the output
    fn output(self) -> String {
        self.output
    }

    fn append_ampersand(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with('=') {
            self.output.push('&');
        }
    }
}

impl Serializer for UrlEncodedSerializer {
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
        self.output.push_str(&percent_encoding(&value.to_string()));
    }

    fn serialize_str(&mut self, value: &str) {
        self.output.push_str(&percent_encoding(value));
    }

    fn serialize_bytes(&mut self, _value: &[u8]) {
        unimplemented!();
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
        unimplemented!();
    }

    fn serialize_end_seq(&mut self) {
        unimplemented!();
    }

    fn serialize_element(&mut self, _value: &dyn Serialize) {
        unimplemented!();
    }

    // Struct
    fn serialize_start_struct(&mut self, _name: &str, _len: usize) {}

    fn serialize_end_struct(&mut self) {}

    fn serialize_field(&mut self, name: &str, value: &dyn Serialize) {
        self.append_ampersand();
        self.output.push_str(name);
        self.output.push('=');
        value.serialize(self);
    }
}

/// Convert a value to a URL encoded string
pub fn to_string<T: Serialize>(value: &T) -> String {
    let mut serializer = UrlEncodedSerializer::new();
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

    #[derive(crate::Serialize)]
    struct Person {
        name: String,
        age: u8,
        color: Color,
    }

    #[test]
    fn test_struct_serialize() {
        let person = Person {
            name: "Bastiaan van der Plaat".to_string(),
            age: 22,
            color: Color::Red,
        };
        assert_eq!(
            to_string(&person),
            "name=Bastiaan%20van%20der%20Plaat&age=22&color=red"
        );
    }
}
