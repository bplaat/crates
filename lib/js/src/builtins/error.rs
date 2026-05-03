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

fn make_error(name: &str, args: &[Value]) -> Value {
    let message = args.first().map(|v| v.to_string()).unwrap_or_default();
    let mut map = IndexMap::new();
    map.insert("name".to_string(), Value::String(name.to_string()));
    map.insert("message".to_string(), Value::String(message.clone()));
    map.insert(
        "stack".to_string(),
        Value::String(format!("{name}: {message}")),
    );
    Value::Object(ObjectValue {
        properties: Rc::new(RefCell::new(map)),
    })
}

pub(super) fn register(env: &Rc<RefCell<IndexMap<String, Value>>>) {
    env.borrow_mut()
        .insert("Error".to_string(), native(|a| make_error("Error", a)));
    env.borrow_mut().insert(
        "TypeError".to_string(),
        native(|a| make_error("TypeError", a)),
    );
    env.borrow_mut().insert(
        "RangeError".to_string(),
        native(|a| make_error("RangeError", a)),
    );
    env.borrow_mut().insert(
        "SyntaxError".to_string(),
        native(|a| make_error("SyntaxError", a)),
    );
    env.borrow_mut().insert(
        "ReferenceError".to_string(),
        native(|a| make_error("ReferenceError", a)),
    );
    env.borrow_mut().insert(
        "EvalError".to_string(),
        native(|a| make_error("EvalError", a)),
    );
    env.borrow_mut().insert(
        "URIError".to_string(),
        native(|a| make_error("URIError", a)),
    );
}
