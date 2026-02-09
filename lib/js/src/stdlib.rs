/*
 * Copyright (c) 2023-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::value::{ObjectValue, Value};

pub(crate) fn env() -> Rc<RefCell<IndexMap<String, Value>>> {
    let env = Rc::new(RefCell::new(IndexMap::new()));

    // MARK: Globals
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
    env.borrow_mut().insert(
        "isNaN".to_string(),
        Value::NativeFunction(|args: &[Value]| {
            if args.is_empty() {
                return Value::Boolean(true);
            }
            Value::Boolean(args[0].to_number().is_nan())
        }),
    );
    env.borrow_mut().insert(
        "isFinite".to_string(),
        Value::NativeFunction(|args: &[Value]| {
            if args.is_empty() {
                return Value::Boolean(false);
            }
            Value::Boolean(args[0].to_number().is_finite())
        }),
    );

    env
}
