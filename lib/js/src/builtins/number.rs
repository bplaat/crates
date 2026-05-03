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

pub(crate) fn call_method(n: f64, method: &str, args: &[Value]) -> Option<Result<Value, String>> {
    match method {
        "toFixed" => {
            let digits = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
            Some(Ok(Value::String(format!("{n:.digits$}"))))
        }
        "toPrecision" => {
            let prec = args.first().map(|v| v.to_number() as usize).unwrap_or(6);
            Some(Ok(Value::String(
                format!("{n:.prec$e}").chars().take(prec + 1).collect(),
            )))
        }
        "toExponential" => {
            let digits = args.first().map(|v| v.to_number() as usize).unwrap_or(6);
            let s = format!("{n:.digits$e}");
            if let Some(e_pos) = s.find('e') {
                let (mantissa, exp_part) = s.split_at(e_pos);
                let exp_num: i32 = exp_part[1..].parse().unwrap_or(0);
                if exp_num >= 0 {
                    Some(Ok(Value::String(format!("{mantissa}e+{exp_num}"))))
                } else {
                    Some(Ok(Value::String(format!("{mantissa}e{exp_num}"))))
                }
            } else {
                Some(Ok(Value::String(s)))
            }
        }
        "toString" => {
            let radix = args.first().map(|v| v.to_number() as u32).unwrap_or(10);
            if radix == 10 {
                Some(Ok(Value::String(crate::value::number_to_js_string(n))))
            } else {
                let n_int = n as i64;
                Some(Ok(Value::String(match radix {
                    2 => format!("{n_int:b}"),
                    8 => format!("{n_int:o}"),
                    16 => format!("{n_int:x}"),
                    _ => format!("{n_int}"),
                })))
            }
        }
        "valueOf" => Some(Ok(Value::Number(n))),
        "toLocaleString" => Some(Ok(Value::String(crate::value::number_to_js_string(n)))),
        _ => None,
    }
}

pub(super) fn register(env: &Rc<RefCell<IndexMap<String, Value>>>) {
    let mut map = IndexMap::new();
    map.insert("MAX_VALUE".to_string(), Value::Number(f64::MAX));
    map.insert("MIN_VALUE".to_string(), Value::Number(f64::MIN_POSITIVE));
    map.insert(
        "POSITIVE_INFINITY".to_string(),
        Value::Number(f64::INFINITY),
    );
    map.insert(
        "NEGATIVE_INFINITY".to_string(),
        Value::Number(f64::NEG_INFINITY),
    );
    map.insert("NaN".to_string(), Value::Number(f64::NAN));
    map.insert("EPSILON".to_string(), Value::Number(f64::EPSILON));
    map.insert(
        "MAX_SAFE_INTEGER".to_string(),
        Value::Number(9007199254740991.0),
    );
    map.insert(
        "MIN_SAFE_INTEGER".to_string(),
        Value::Number(-9007199254740991.0),
    );
    map.insert(
        "isNaN".to_string(),
        native(|a| Value::Boolean(matches!(a.first(), Some(Value::Number(n)) if n.is_nan()))),
    );
    map.insert(
        "isFinite".to_string(),
        native(|a| Value::Boolean(matches!(a.first(), Some(Value::Number(n)) if n.is_finite()))),
    );
    map.insert(
        "isInteger".to_string(),
        native(|a| {
            Value::Boolean(
                matches!(a.first(), Some(Value::Number(n)) if n.fract() == 0.0 && n.is_finite()),
            )
        }),
    );
    map.insert("isSafeInteger".to_string(), native(|a| {
        Value::Boolean(matches!(a.first(), Some(Value::Number(n)) if n.fract() == 0.0 && n.abs() <= 9007199254740991.0))
    }));
    map.insert(
        "parseInt".to_string(),
        native(|args| {
            let s = args.first().map(|v| v.to_string()).unwrap_or_default();
            let radix = args.get(1).map(|v| v.to_number() as u32).unwrap_or(10);
            let trimmed = s.trim();
            let valid: String = trimmed.chars().take_while(|c| c.is_digit(radix)).collect();
            if valid.is_empty() {
                Value::Number(f64::NAN)
            } else {
                Value::Number(i64::from_str_radix(&valid, radix).unwrap_or(0) as f64)
            }
        }),
    );
    map.insert(
        "parseFloat".to_string(),
        native(|args| Value::Number(args.first().map(|v| v.to_number()).unwrap_or(f64::NAN))),
    );
    // __call__ makes this object callable: Number(x) coerces x to number
    map.insert(
        "__call__".to_string(),
        native(|args| Value::Number(args.first().map(|v| v.to_number()).unwrap_or(0.0))),
    );
    env.borrow_mut().insert(
        "Number".to_string(),
        Value::Object(ObjectValue {
            properties: Rc::new(RefCell::new(map)),
        }),
    );
}
