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
        "Object".to_string(),
        make_obj(&[
            (
                "keys",
                native(|a| match a.first() {
                    Some(Value::Object(obj)) => {
                        let keys: Vec<Value> = obj
                            .borrow()
                            .keys()
                            .filter(|k| !is_internal_key(k))
                            .map(|k| Value::String(k.clone()))
                            .collect();
                        Value::Array(ArrayValue {
                            elements: Rc::new(RefCell::new(keys)),
                        })
                    }
                    _ => Value::Array(ArrayValue {
                        elements: Rc::new(RefCell::new(vec![])),
                    }),
                }),
            ),
            (
                "values",
                native(|a| match a.first() {
                    Some(Value::Object(obj)) => {
                        let vals: Vec<Value> = obj
                            .borrow()
                            .iter()
                            .filter(|(k, _)| !is_internal_key(k))
                            .map(|(_, v)| v.clone())
                            .collect();
                        Value::Array(ArrayValue {
                            elements: Rc::new(RefCell::new(vals)),
                        })
                    }
                    _ => Value::Array(ArrayValue {
                        elements: Rc::new(RefCell::new(vec![])),
                    }),
                }),
            ),
            (
                "entries",
                native(|a| match a.first() {
                    Some(Value::Object(obj)) => {
                        let entries: Vec<Value> = obj
                            .borrow()
                            .iter()
                            .filter(|(k, _)| !is_internal_key(k))
                            .map(|(k, v)| {
                                Value::Array(ArrayValue {
                                    elements: Rc::new(RefCell::new(vec![
                                        Value::String(k.clone()),
                                        v.clone(),
                                    ])),
                                })
                            })
                            .collect();
                        Value::Array(ArrayValue {
                            elements: Rc::new(RefCell::new(entries)),
                        })
                    }
                    _ => Value::Array(ArrayValue {
                        elements: Rc::new(RefCell::new(vec![])),
                    }),
                }),
            ),
            (
                "assign",
                native(|args| {
                    if let Some(Value::Object(target)) = args.first() {
                        for source in args.iter().skip(1) {
                            if let Value::Object(src) = source {
                                for (k, v) in src.borrow().iter() {
                                    if !is_internal_key(k) {
                                        target.borrow_mut().insert(k.clone(), v.clone());
                                    }
                                }
                            }
                        }
                        Value::Object(target.clone())
                    } else {
                        Value::Undefined
                    }
                }),
            ),
            (
                "create",
                native(|args| {
                    if let Some(Value::Object(proto)) = args.first() {
                        let map = proto.borrow().clone();
                        Value::Object(ObjectValue {
                            properties: Rc::new(RefCell::new(map)),
                        })
                    } else {
                        Value::Object(ObjectValue {
                            properties: Rc::new(RefCell::new(IndexMap::new())),
                        })
                    }
                }),
            ),
            (
                "freeze",
                native(|a| {
                    if let Some(Value::Object(obj)) = a.first() {
                        obj.borrow_mut()
                            .insert("__frozen__".to_string(), Value::Boolean(true));
                        Value::Object(obj.clone())
                    } else {
                        a.first().cloned().unwrap_or(Value::Undefined)
                    }
                }),
            ),
            (
                "seal",
                native(|a| a.first().cloned().unwrap_or(Value::Undefined)),
            ),
            (
                "isFrozen",
                native(|a| match a.first() {
                    Some(Value::Object(obj)) => Value::Boolean(
                        matches!(obj.borrow().get("__frozen__"), Some(Value::Boolean(true))),
                    ),
                    _ => Value::Boolean(false),
                }),
            ),
            ("isSealed", native(|_| Value::Boolean(false))),
            (
                "getOwnPropertyNames",
                native(|a| match a.first() {
                    Some(Value::Object(obj)) => {
                        let keys: Vec<Value> = obj
                            .borrow()
                            .keys()
                            .filter(|k| !is_internal_key(k))
                            .map(|k| Value::String(k.clone()))
                            .collect();
                        Value::Array(ArrayValue {
                            elements: Rc::new(RefCell::new(keys)),
                        })
                    }
                    _ => Value::Array(ArrayValue {
                        elements: Rc::new(RefCell::new(vec![])),
                    }),
                }),
            ),
            (
                "hasOwn",
                native(|a| match (a.first(), a.get(1)) {
                    (Some(Value::Object(obj)), Some(key)) => {
                        Value::Boolean(obj.borrow().contains_key(&key.to_string()))
                    }
                    _ => Value::Boolean(false),
                }),
            ),
            (
                "fromEntries",
                native(|a| {
                    let mut map = IndexMap::new();
                    if let Some(Value::Array(arr)) = a.first() {
                        for entry in arr.borrow().iter() {
                            if let Value::Array(pair) = entry {
                                let pair = pair.borrow();
                                if pair.len() >= 2 {
                                    map.insert(pair[0].to_string(), pair[1].clone());
                                }
                            }
                        }
                    }
                    Value::Object(ObjectValue {
                        properties: Rc::new(RefCell::new(map)),
                    })
                }),
            ),
            ("getPrototypeOf", native(|_| Value::Null)),
            (
                "defineProperty",
                native(|a| a.first().cloned().unwrap_or(Value::Undefined)),
            ),
            (
                "getOwnPropertyDescriptor",
                native(|a| {
                    if let (Some(Value::Object(obj)), Some(key)) = (a.first(), a.get(1))
                        && let Some(val) = obj.borrow().get(&key.to_string())
                    {
                        return make_obj(&[
                            ("value", val.clone()),
                            ("writable", Value::Boolean(true)),
                            ("enumerable", Value::Boolean(true)),
                            ("configurable", Value::Boolean(true)),
                        ]);
                    }
                    Value::Undefined
                }),
            ),
        ]),
    );
}
