/*
 * Copyright (c) 2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::{Deserialize, DeserializeError, Deserializer};

struct JsonDeserializer {
    json: String,
    pos: usize,
}

impl JsonDeserializer {
    fn new(json: String) -> Self {
        JsonDeserializer { json, pos: 0 }
    }
}

impl Deserializer for JsonDeserializer {
    fn deserialize_bool(&mut self) -> Result<bool, DeserializeError> {
        if self.json[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(true)
        } else if self.json[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(false)
        } else {
            Err(DeserializeError)
        }
    }

    fn deserialize_i8(&mut self) -> Result<i8, DeserializeError> {
        let end = self.json[self.pos..]
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(self.json.len() - self.pos);
        let value = self.json[self.pos..self.pos + end]
            .parse()
            .map_err(|_| DeserializeError)?;
        self.pos += end;
        Ok(value)
    }

    fn deserialize_i16(&mut self) -> Result<i16, DeserializeError> {
        unimplemented!()
    }
    fn deserialize_i32(&mut self) -> Result<i32, DeserializeError> {
        unimplemented!()
    }
    fn deserialize_i64(&mut self) -> Result<i64, DeserializeError> {
        unimplemented!()
    }
    fn deserialize_u8(&mut self) -> Result<u8, DeserializeError> {
        unimplemented!()
    }
    fn deserialize_u16(&mut self) -> Result<u16, DeserializeError> {
        unimplemented!()
    }
    fn deserialize_u32(&mut self) -> Result<u32, DeserializeError> {
        unimplemented!()
    }
    fn deserialize_u64(&mut self) -> Result<u64, DeserializeError> {
        unimplemented!()
    }
    fn deserialize_f32(&mut self) -> Result<f32, DeserializeError> {
        unimplemented!()
    }
    fn deserialize_f64(&mut self) -> Result<f64, DeserializeError> {
        unimplemented!()
    }
    fn deserialize_str(&mut self) -> Result<&str, DeserializeError> {
        unimplemented!()
    }
    fn deserialize_bytes(&mut self) -> Result<&[u8], DeserializeError> {
        unimplemented!()
    }
    fn deserialize_none(&mut self) -> Result<(), DeserializeError> {
        unimplemented!()
    }

    fn deserialize_start_seq(&mut self) -> Result<usize, DeserializeError> {
        unimplemented!()
    }
    fn deserialize_end_seq(&mut self) -> Result<(), DeserializeError> {
        unimplemented!()
    }

    fn deserialize_start_struct(&mut self, _name: &str) -> Result<usize, DeserializeError> {
        unimplemented!()
    }
    fn deserialize_field(&mut self) -> Option<&str> {
        unimplemented!()
    }
    fn deserialize_end_struct(&mut self) -> Result<(), DeserializeError> {
        unimplemented!()
    }

    fn deserialize_skip(&mut self) -> Result<(), DeserializeError> {
        unimplemented!()
    }
}

fn from_string<T: Deserialize>(json: &str) -> Result<T, DeserializeError> {
    let mut deserializer = JsonDeserializer::new(json.to_string());
    T::deserialize(&mut deserializer)
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;
    use crate::de::DeserializeError;
    use crate::Deserialize;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Color {
        Red,
        Green,
        Blue,
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

    struct Person {
        name: String,
        age: u8,
        color: Color,
    }

    impl Deserialize for Person {
        fn deserialize(deserializer: &mut dyn Deserializer) -> Result<Self, DeserializeError> {
            let mut name = None;
            let mut age = None;
            let mut color = None;
            deserializer.deserialize_start_struct("Person")?;
            while let Some(field) = deserializer.deserialize_field() {
                match field {
                    "name" => name = Some(Deserialize::deserialize(deserializer)?),
                    "age" => age = Some(Deserialize::deserialize(deserializer)?),
                    "color" => color = Some(Deserialize::deserialize(deserializer)?),
                    _ => deserializer.deserialize_skip()?,
                }
            }
            deserializer.deserialize_end_struct()?;
            Ok(Person {
                name: name.ok_or(DeserializeError)?,
                age: age.ok_or(DeserializeError)?,
                color: color.ok_or(DeserializeError)?,
            })
        }
    }

    #[test]
    fn test_struct_deserialize() {
        let person = from_string::<Person>(r#"{"age":30,"color":"red","name":"Alice"}"#).unwrap();
        assert_eq!(person.name, "Alice");
        assert_eq!(person.age, 30);
    }

    #[test]
    fn test_vec_deserialize() {
        let vec = from_string::<Vec<u8>>("[1,2,3]").unwrap();
        assert_eq!(vec, vec![1, 2, 3]);

        let people = from_string::<Vec<Person>>(
            r#"[{"color":"red","name":"Alice","age":30},{"age":25,"name":"Bob","color":"green"}]"#,
        )
        .unwrap();
        assert_eq!(people.len(), 2);
        assert_eq!(people[0].name, "Alice");
        assert_eq!(people[0].age, 30);
        assert_eq!(people[0].color, Color::Red);
        assert_eq!(people[1].name, "Bob");
        assert_eq!(people[1].age, 25);
        assert_eq!(people[1].color, Color::Green);
    }
}
