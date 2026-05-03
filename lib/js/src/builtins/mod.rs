/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::value::{ObjectValue, Value};

pub(crate) mod array;
mod error;
mod globals;
mod json;
mod math;
pub(crate) mod number;
mod object;
pub(crate) mod string;

pub(super) fn make_obj(pairs: &[(&str, Value)]) -> Value {
    let mut map = IndexMap::new();
    for (k, v) in pairs {
        map.insert(k.to_string(), v.clone());
    }
    Value::Object(ObjectValue {
        properties: Rc::new(RefCell::new(map)),
    })
}

pub(super) fn native(f: fn(&[Value]) -> Value) -> Value {
    Value::NativeFunction(f)
}

/// Returns true for internal keys (`__foo__`) that should be hidden from JS code.
pub(super) fn is_internal_key(k: &str) -> bool {
    k.starts_with("__") && k.ends_with("__")
}

pub(crate) fn env() -> Rc<RefCell<IndexMap<String, Value>>> {
    let env = Rc::new(RefCell::new(IndexMap::new()));
    globals::register(&env);
    math::register(&env);
    number::register(&env);
    string::register(&env);
    array::register(&env);
    object::register(&env);
    json::register(&env);
    error::register(&env);
    env
}
