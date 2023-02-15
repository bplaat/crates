use std::{
    collections::HashMap,
    iter::Peekable
};

#[allow(dead_code)]
#[derive(Debug)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn new_array() -> JsonValue {
        JsonValue::Array(Vec::new())
    }

    pub fn new_object() -> JsonValue {
        JsonValue::Object(HashMap::new())
    }

    pub fn parse(json: &str) -> Option<JsonValue> {
        let mut chars = json.chars().peekable();
        return JsonValue::parse_item(&mut chars);
    }

    fn parse_item<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> Option<JsonValue> {
        JsonValue::parse_skip_whitespace(chars);
        if let Some(char) = chars.peek() {
            if *char == 'n' {
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                return Some(JsonValue::Null);
            }
            if *char == 't' {
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                return Some(JsonValue::Bool(true));
            }
            if *char == 'f' {
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                chars.next();
                return Some(JsonValue::Bool(false));
            }
            if char.is_digit(10) || *char == '-' {
                let mut number = String::new();
                let mut is_float = false;
                while let Some(char) = chars.peek() {
                    if *char == '.' {
                        is_float = true;
                    }
                    if char.is_digit(10) || *char == '.' {
                        number.push(chars.next().unwrap());
                        continue;
                    }
                    break;
                }
                if is_float {
                    return Some(JsonValue::Float(number.parse().unwrap()));
                } else {
                    return Some(JsonValue::Int(number.parse().unwrap()));
                }
            }
            if *char == '"' {
                chars.next(); // "
                let mut string = String::new();
                while let Some(char) = chars.peek() {
                    if *char != '"' {
                        string.push(chars.next().unwrap());
                        continue;
                    }
                    break;
                }
                chars.next(); // "
                return Some(JsonValue::String(string));
            }
            if *char == '[' {
                chars.next(); // [
                let mut array = JsonValue::new_array();
                loop {
                    if let Some(item) = JsonValue::parse_item(chars) {
                        array.push(item);
                    }
                    JsonValue::parse_skip_whitespace(chars);
                    if let Some(char) = chars.peek() {
                        if *char == ',' {
                            chars.next(); // ,
                            continue;
                        }
                        if *char == ']' {
                            break;
                        }
                        panic!("Unexpected character: {}", char);
                    }
                }
                chars.next(); // ]
                return Some(array);
            }
            if *char == '{' {
                chars.next(); // {
                let mut object = JsonValue::new_object();
                loop {
                    JsonValue::parse_skip_whitespace(chars);
                    chars.next(); // "
                    let mut key = String::new();
                    while let Some(char) = chars.peek() {
                        if *char != '"' {
                            key.push(chars.next().unwrap());
                            continue;
                        }
                        break;
                    }
                    chars.next(); // "
                    JsonValue::parse_skip_whitespace(chars);
                    chars.next(); // :
                    if let Some(value) = JsonValue::parse_item(chars) {
                        object.insert(key.as_str(), value);
                    }
                    JsonValue::parse_skip_whitespace(chars);
                    if let Some(char) = chars.peek() {
                        if *char == ',' {
                            chars.next(); // ,
                            continue;
                        }
                        if *char == '}' {
                            break;
                        }
                        panic!("Unexpected character: {}", char);
                    }
                }
                chars.next(); // }
                return Some(object);
            }
        }
        None
    }

    fn parse_skip_whitespace<I: Iterator<Item = char>>(chars: &mut Peekable<I>) {
        while let Some(char) = chars.peek() {
            if *char == ' ' || *char == '\t' || *char == '\r' || *char == '\n' {
                chars.next();
                continue;
            }
            return;
        }
    }

    pub fn push(&mut self, value: JsonValue) {
        match self {
            JsonValue::Array(array) => array.push(value),
            _ => panic!("JsonValue not an array"),
        }
    }

    pub fn insert(&mut self, key: &str, value: JsonValue) {
        match self {
            JsonValue::Object(object) => {
                object.insert(key.to_string(), value);
            }
            _ => panic!("JsonValue not an object"),
        }
    }
}

impl ToString for JsonValue {
    fn to_string(&self) -> String {
        match self {
            JsonValue::Null => String::from("null"),
            JsonValue::Bool(bool) => String::from(if *bool { "true" } else {"false"}),
            JsonValue::Int(int) => int.to_string(),
            JsonValue::Float(float) => float.to_string(),
            JsonValue::String(string) => {
                let mut sb = String::from('"');
                sb.push_str(string);
                sb.push('"');
                sb
            }
            JsonValue::Array(array) => {
                let mut sb = String::from('[');
                for (index, item) in array.iter().enumerate() {
                    sb.push_str(item.to_string().as_str());
                    if index != array.len() - 1 {
                        sb.push(',');
                    }
                }
                sb.push(']');
                sb
            }
            JsonValue::Object(object) => {
                let mut sb = String::from('{');
                for (index, key) in object.keys().into_iter().enumerate() {
                    sb.push('"');
                    sb.push_str(key.as_str());
                    sb.push_str("\":");
                    sb.push_str(object[key].to_string().as_str());
                    if index != object.len() - 1 {
                        sb.push(',');
                    }
                }
                sb.push('}');
                sb
            }
        }
    }
}

pub trait ToJson {
    fn to_json(&self) -> JsonValue;
}

impl<T> ToJson for Vec<T>
where
    T: ToJson,
{
    fn to_json(&self) -> JsonValue {
        let mut array_json = JsonValue::new_array();
        for item in self {
            array_json.push(item.to_json());
        }
        array_json
    }
}
