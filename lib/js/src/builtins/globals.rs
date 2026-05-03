/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::native;
use crate::value::{ObjectValue, Value};

pub(super) fn register(env: &Rc<RefCell<IndexMap<String, Value>>>) {
    // Global constants
    env.borrow_mut().insert(
        "globalThis".to_string(),
        Value::Object(ObjectValue {
            properties: env.clone(),
        }),
    );
    env.borrow_mut()
        .insert("Infinity".to_string(), Value::Number(f64::INFINITY));
    env.borrow_mut()
        .insert("NaN".to_string(), Value::Number(f64::NAN));
    env.borrow_mut()
        .insert("undefined".to_string(), Value::Undefined);

    // Type testing
    env.borrow_mut().insert(
        "isNaN".to_string(),
        native(|args| Value::Boolean(args.first().is_none_or(|v| v.to_number().is_nan()))),
    );
    env.borrow_mut().insert(
        "isFinite".to_string(),
        native(|args| Value::Boolean(args.first().is_some_and(|v| v.to_number().is_finite()))),
    );

    // Parsing
    env.borrow_mut().insert(
        "parseInt".to_string(),
        native(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return Value::Number(f64::NAN);
            }
            let radix = args.get(1).map(|v| v.to_number() as u32).unwrap_or(0);
            let (src, radix) = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                (&trimmed[2..], 16u32)
            } else if trimmed.starts_with("0b") || trimmed.starts_with("0B") {
                (&trimmed[2..], 2u32)
            } else if trimmed.starts_with("0o") || trimmed.starts_with("0O") {
                (&trimmed[2..], 8u32)
            } else {
                (trimmed, if radix == 0 { 10 } else { radix })
            };
            let (neg, digits) = if let Some(stripped) = src.strip_prefix('-') {
                (true, stripped)
            } else if let Some(stripped) = src.strip_prefix('+') {
                (false, stripped)
            } else {
                (false, src)
            };
            let valid: String = digits.chars().take_while(|c| c.is_digit(radix)).collect();
            if valid.is_empty() {
                return Value::Number(f64::NAN);
            }
            let n = i64::from_str_radix(&valid, radix).unwrap_or(0);
            Value::Number(if neg { -(n as f64) } else { n as f64 })
        }),
    );
    env.borrow_mut().insert(
        "parseFloat".to_string(),
        native(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            Value::Number(s.trim().parse::<f64>().unwrap_or(f64::NAN))
        }),
    );

    // URI encoding
    env.borrow_mut().insert(
        "encodeURIComponent".to_string(),
        native(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let encoded: String = s
                .bytes()
                .flat_map(|b| {
                    if b.is_ascii_alphanumeric()
                        || matches!(
                            b,
                            b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')'
                        )
                    {
                        vec![b as char]
                    } else {
                        format!("%{b:02X}").chars().collect::<Vec<_>>()
                    }
                })
                .collect();
            Value::String(encoded)
        }),
    );
    env.borrow_mut().insert(
        "decodeURIComponent".to_string(),
        native(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let mut result = String::new();
            let mut chars = s.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '%' {
                    let h1 = chars.next().unwrap_or('0');
                    let h2 = chars.next().unwrap_or('0');
                    if let Ok(byte) = u8::from_str_radix(&format!("{h1}{h2}"), 16) {
                        result.push(byte as char);
                    } else {
                        result.push('%');
                        result.push(h1);
                        result.push(h2);
                    }
                } else {
                    result.push(c);
                }
            }
            Value::String(result)
        }),
    );
    env.borrow_mut().insert(
        "encodeURI".to_string(),
        native(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let encoded: String = s
                .bytes()
                .flat_map(|b| {
                    if b.is_ascii_alphanumeric()
                        || matches!(
                            b,
                            b'-' | b'_'
                                | b'.'
                                | b'!'
                                | b'~'
                                | b'*'
                                | b'\''
                                | b'('
                                | b')'
                                | b';'
                                | b','
                                | b'/'
                                | b'?'
                                | b':'
                                | b'@'
                                | b'&'
                                | b'='
                                | b'+'
                                | b'$'
                                | b'#'
                        )
                    {
                        vec![b as char]
                    } else {
                        format!("%{b:02X}").chars().collect::<Vec<_>>()
                    }
                })
                .collect();
            Value::String(encoded)
        }),
    );
    env.borrow_mut().insert(
        "decodeURI".to_string(),
        native(|args| {
            // Simplified: pass-through for already-decoded strings
            Value::String(args.first().map(|v| v.to_string()).unwrap_or_default())
        }),
    );
}
