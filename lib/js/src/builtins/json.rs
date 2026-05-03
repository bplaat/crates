/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::{is_internal_key, make_obj, native};
use crate::value::{ArrayValue, ObjectValue, Value};

pub(super) fn register(env: &Rc<RefCell<IndexMap<String, Value>>>) {
    env.borrow_mut().insert(
        "JSON".to_string(),
        make_obj(&[
            (
                "parse",
                native(|args| {
                    let s = args.first().map(|v| v.to_string()).unwrap_or_default();
                    match serde_json::from_str::<serde_json::Value>(&s) {
                        Ok(v) => json_to_value(v),
                        Err(_) => Value::Undefined,
                    }
                }),
            ),
            (
                "stringify",
                native(|args| {
                    let val = args.first().cloned().unwrap_or(Value::Undefined);
                    let space = args.get(2).map(|v| v.to_number() as usize).unwrap_or(0);
                    let json_val = value_to_json(&val);
                    let result = if space > 0 {
                        serde_json::to_string_pretty(&json_val).unwrap_or_default()
                    } else {
                        serde_json::to_string(&json_val).unwrap_or_default()
                    };
                    Value::String(result)
                }),
            ),
        ]),
    );
}

pub(crate) fn json_to_value(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or(f64::NAN)),
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            let elements = arr.into_iter().map(json_to_value).collect();
            Value::Array(ArrayValue {
                elements: Rc::new(RefCell::new(elements)),
            })
        }
        serde_json::Value::Object(obj) => {
            let mut map = IndexMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_value(v));
            }
            Value::Object(ObjectValue {
                properties: Rc::new(RefCell::new(map)),
            })
        }
    }
}

pub(crate) fn value_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Undefined | Value::Function(..) | Value::NativeFunction(_) => serde_json::Value::Null,
        Value::Null => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Number(n) => {
            if n.is_nan() || n.is_infinite() {
                serde_json::Value::Null
            } else if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                serde_json::Value::Number(serde_json::Number::from(*n as i64))
            } else {
                serde_json::Value::Number(
                    serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0)),
                )
            }
        }
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Array(arr) => {
            serde_json::Value::Array(arr.borrow().iter().map(value_to_json).collect())
        }
        Value::Object(obj) => {
            let mut map = serde_json::Map::new();
            for (k, v) in obj.borrow().iter() {
                if !is_internal_key(k)
                    && !matches!(
                        v,
                        Value::Function(..) | Value::NativeFunction(_) | Value::Undefined
                    )
                {
                    map.insert(k.clone(), value_to_json(v));
                }
            }
            serde_json::Value::Object(map)
        }
    }
}
