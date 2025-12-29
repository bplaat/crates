/*
 * Copyright (c) 2023-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashMap;

use crate::value::Value;

pub(crate) fn env() -> HashMap<String, Value> {
    let mut env = HashMap::new();

    // Global variables
    env.insert("Infinity".to_string(), Value::Number(f64::INFINITY));
    env.insert("NaN".to_string(), Value::Number(f64::NAN));
    env.insert("undefined".to_string(), Value::Undefined);

    // Global functions
    env.insert(
        "isNaN".to_string(),
        Value::NativeFunction(|args: &[Value]| {
            if args.is_empty() {
                return Value::Boolean(true);
            }
            Value::Boolean(args[0].to_number().is_nan())
        }),
    );
    env.insert(
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
