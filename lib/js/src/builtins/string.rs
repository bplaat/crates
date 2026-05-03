/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::native;
use crate::value::{ArrayValue, ObjectValue, Value};

pub(crate) fn call_method(s: &str, method: &str, args: &[Value]) -> Option<Value> {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    match method {
        "charAt" => {
            let idx = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
            Some(
                chars
                    .get(idx)
                    .map(|c| Value::String(c.to_string()))
                    .unwrap_or(Value::String(String::new())),
            )
        }
        "charCodeAt" => {
            let idx = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
            Some(
                chars
                    .get(idx)
                    .map(|c| Value::Number(*c as u32 as f64))
                    .unwrap_or(Value::Number(f64::NAN)),
            )
        }
        "indexOf" => {
            let search = args.first().map(|v| v.to_string()).unwrap_or_default();
            let from = args.get(1).map(|v| v.to_number() as usize).unwrap_or(0);
            let haystack: String = chars[from.min(len)..].iter().collect();
            Some(
                haystack
                    .find(&search)
                    .map(|byte_pos| {
                        let char_pos = haystack[..byte_pos].chars().count();
                        Value::Number((from.min(len) + char_pos) as f64)
                    })
                    .unwrap_or(Value::Number(-1.0)),
            )
        }
        "lastIndexOf" => {
            let search = args.first().map(|v| v.to_string()).unwrap_or_default();
            let from_index = args
                .get(1)
                .map(|v| (v.to_number() as usize).min(len))
                .unwrap_or(len);
            let search_in: String = chars[..from_index.min(len)].iter().collect();
            Some(
                search_in
                    .rfind(&search)
                    .map(|byte_pos| {
                        let char_pos = search_in[..byte_pos].chars().count();
                        Value::Number(char_pos as f64)
                    })
                    .unwrap_or(Value::Number(-1.0)),
            )
        }
        "slice" => {
            let start = args
                .first()
                .map(|v| {
                    let n = v.to_number() as i64;
                    if n < 0 { (len as i64 + n).max(0) as usize } else { (n as usize).min(len) }
                })
                .unwrap_or(0);
            let end = args
                .get(1)
                .map(|v| {
                    let n = v.to_number() as i64;
                    if n < 0 { (len as i64 + n).max(0) as usize } else { (n as usize).min(len) }
                })
                .unwrap_or(len);
            let result: String =
                if start <= end { chars[start..end].iter().collect() } else { String::new() };
            Some(Value::String(result))
        }
        "substring" => {
            let a = args
                .first()
                .map(|v| v.to_number() as usize)
                .unwrap_or(0)
                .min(len);
            let b = args.get(1).map(|v| v.to_number() as usize).unwrap_or(len).min(len);
            let (start, end) = if a <= b { (a, b) } else { (b, a) };
            Some(Value::String(chars[start..end].iter().collect()))
        }
        "substr" => {
            let start = args
                .first()
                .map(|v| {
                    let n = v.to_number() as i64;
                    if n < 0 { (len as i64 + n).max(0) as usize } else { (n as usize).min(len) }
                })
                .unwrap_or(0);
            let length = args.get(1).map(|v| v.to_number() as usize).unwrap_or(len);
            let end = (start + length).min(len);
            Some(Value::String(chars[start..end].iter().collect()))
        }
        "toUpperCase" | "toLocaleUpperCase" => Some(Value::String(s.to_uppercase())),
        "toLowerCase" | "toLocaleLowerCase" => Some(Value::String(s.to_lowercase())),
        "trim" => Some(Value::String(s.trim().to_string())),
        "trimStart" | "trimLeft" => Some(Value::String(s.trim_start().to_string())),
        "trimEnd" | "trimRight" => Some(Value::String(s.trim_end().to_string())),
        "split" => {
            if args.is_empty() {
                return Some(Value::Array(ArrayValue {
                    elements: Rc::new(RefCell::new(vec![Value::String(s.to_string())])),
                }));
            }
            let sep = args[0].to_string();
            let limit = args.get(1).map(|v| v.to_number() as usize);
            let parts: Vec<Value> = if sep.is_empty() {
                chars.iter().map(|c| Value::String(c.to_string())).collect()
            } else {
                s.split(&sep as &str).map(|p| Value::String(p.to_string())).collect()
            };
            let parts =
                if let Some(lim) = limit { parts.into_iter().take(lim).collect() } else { parts };
            Some(Value::Array(ArrayValue {
                elements: Rc::new(RefCell::new(parts)),
            }))
        }
        "replace" => {
            let search = args.first().map(|v| v.to_string()).unwrap_or_default();
            let replacement = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            Some(Value::String(s.replacen(&search, &replacement, 1)))
        }
        "replaceAll" => {
            let search = args.first().map(|v| v.to_string()).unwrap_or_default();
            let replacement = args.get(1).map(|v| v.to_string()).unwrap_or_default();
            Some(Value::String(s.replace(&search, &replacement)))
        }
        "concat" => {
            let mut result = s.to_string();
            for arg in args {
                result.push_str(&arg.to_string());
            }
            Some(Value::String(result))
        }
        "includes" => {
            let search = args.first().map(|v| v.to_string()).unwrap_or_default();
            Some(Value::Boolean(s.contains(&search as &str)))
        }
        "startsWith" => {
            let search = args.first().map(|v| v.to_string()).unwrap_or_default();
            let pos = args.get(1).map(|v| v.to_number() as usize).unwrap_or(0);
            let slice: String = chars[pos.min(len)..].iter().collect();
            Some(Value::Boolean(slice.starts_with(&search as &str)))
        }
        "endsWith" => {
            let search = args.first().map(|v| v.to_string()).unwrap_or_default();
            let end_pos =
                args.get(1).map(|v| v.to_number() as usize).unwrap_or(len).min(len);
            let slice: String = chars[..end_pos].iter().collect();
            Some(Value::Boolean(slice.ends_with(&search as &str)))
        }
        "repeat" => {
            let count = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
            Some(Value::String(s.repeat(count)))
        }
        "padStart" => {
            let target = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
            let fill = args.get(1).map(|v| v.to_string()).unwrap_or_else(|| " ".to_string());
            if len >= target {
                return Some(Value::String(s.to_string()));
            }
            let needed = target - len;
            let pad: String = fill.chars().cycle().take(needed).collect();
            Some(Value::String(format!("{pad}{s}")))
        }
        "padEnd" => {
            let target = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
            let fill = args.get(1).map(|v| v.to_string()).unwrap_or_else(|| " ".to_string());
            if len >= target {
                return Some(Value::String(s.to_string()));
            }
            let needed = target - len;
            let pad: String = fill.chars().cycle().take(needed).collect();
            Some(Value::String(format!("{s}{pad}")))
        }
        "at" => {
            let idx = args.first().map(|v| v.to_number() as i64).unwrap_or(0);
            let actual = if idx < 0 { len as i64 + idx } else { idx } as usize;
            Some(chars.get(actual).map(|c| Value::String(c.to_string())).unwrap_or(Value::Undefined))
        }
        "toString" | "valueOf" => Some(Value::String(s.to_string())),
        _ => None,
    }
}

pub(super) fn register(env: &Rc<RefCell<IndexMap<String, Value>>>) {
    let mut map = IndexMap::new();
    map.insert(
        "fromCharCode".to_string(),
        native(|args| {
            let s: String = args
                .iter()
                .map(|v| char::from_u32(v.to_number() as u32).unwrap_or('\0'))
                .collect();
            Value::String(s)
        }),
    );
    map.insert(
        "fromCodePoint".to_string(),
        native(|args| {
            let s: String = args
                .iter()
                .filter_map(|v| char::from_u32(v.to_number() as u32))
                .collect();
            Value::String(s)
        }),
    );
    // __call__ makes String(x) coerce to string
    map.insert(
        "__call__".to_string(),
        native(|args| Value::String(args.first().map(|v| v.to_string()).unwrap_or_default())),
    );
    env.borrow_mut().insert(
        "String".to_string(),
        Value::Object(ObjectValue {
            properties: Rc::new(RefCell::new(map)),
        }),
    );

    // Boolean callable
    env.borrow_mut().insert(
        "Boolean".to_string(),
        native(|args| Value::Boolean(args.first().map(|v| v.is_truthy()).unwrap_or(false))),
    );
}
