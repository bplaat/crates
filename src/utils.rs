// MARK: Utils
use std::sync::LazyLock;

use regex::Regex;

use crate::types::Argument;

static RE_ARGUMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([_A-Za-z][_A-Za-z0-9 ]*[\**|\s+])\s*([_A-Za-z][_A-Za-z0-9]*)").unwrap()
});

pub fn to_snake_case(camel_case: &str) -> String {
    let mut s = String::new();
    for ch in camel_case.chars() {
        if ch.is_uppercase() {
            s.push('_');
            s.push(ch.to_lowercase().next().unwrap());
        } else {
            s.push(ch);
        }
    }
    if s.starts_with('_') {
        s[1..].to_owned()
    } else {
        s
    }
}

pub fn find_matching_close(text: &str, start: usize) -> usize {
    let bytes = text.as_bytes();
    let open_char = bytes[start];
    let close_char = if open_char == b'{' { b'}' } else { b')' };
    let mut depth = 0usize;
    let mut pos = start;
    while pos < bytes.len() {
        if bytes[pos] == open_char {
            depth += 1;
        } else if bytes[pos] == close_char {
            depth -= 1;
            if depth == 0 {
                return pos;
            }
        }
        pos += 1;
    }
    pos
}

pub fn parse_arguments(arguments_str: &str) -> Vec<Argument> {
    let mut arguments = Vec::new();
    if !arguments_str.trim().is_empty() {
        for argument_str in arguments_str.split(',') {
            if let Some(caps) = RE_ARGUMENT.captures(argument_str) {
                arguments.push(Argument {
                    name: caps[2].trim().to_owned(),
                    type_: caps[1].to_owned(),
                });
            }
        }
    }
    arguments
}
